//! Compatibility projections for the canonical live Job/Graph runtime.
//!
//! Native MCP Tasks and these tools address the same JobManager entries and use
//! the same job id. Clients without the Tasks extension can therefore start,
//! poll, page output and cancel without holding one request open.

use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Tool},
    schemars,
    service::{RequestContext, RoleServer},
    tool, tool_router,
};
use serde::Deserialize;

use super::{LocalMachine, graph, ok_text, resolve_capped, tool_error};
use crate::jobs::{JobHandoff, JobOutputStream, JobWait};

const MAX_JOB_LIST_LIMIT: usize = 256;

pub(crate) fn canonicalize_graph_router(
    router: &mut rmcp::handler::server::router::tool::ToolRouter<LocalMachine>,
    capabilities: crate::mcp::Capabilities,
    shell_tool: Option<aw_tools::filesystem::batch::NodeTool>,
) {
    let tools = graph::allowed_node_tools(capabilities, shell_tool);
    let schema = graph::input_schema(&tools)
        .as_object()
        .cloned()
        .expect("canonical Graph schema is an object");
    let schema = std::sync::Arc::new(schema);
    for name in ["batch", "job_start"] {
        if let Some(route) = router.map.get_mut(name) {
            route.attr.input_schema = schema.clone();
        }
    }
}

pub(crate) fn with_job_limits(tool: &mut Tool, _max_secs: u64, max_output_bytes: usize) {
    let name = tool.name.as_ref();
    if !matches!(name, "job_output" | "job_list") {
        return;
    }
    let mut schema = (*tool.input_schema).clone();
    let Some(properties) = schema
        .get_mut("properties")
        .and_then(serde_json::Value::as_object_mut)
    else {
        return;
    };
    if name == "job_output"
        && let Some(property) = properties
            .get_mut("max_output_bytes")
            .and_then(serde_json::Value::as_object_mut)
    {
        property.insert(
            "description".to_owned(),
            serde_json::Value::String(format!(
                "Serialized MCP result budget for this page. Defaults to {max_output_bytes}, must be at least {} and is capped at {max_output_bytes}.",
                crate::mcp::MIN_BYTE_LIMIT,
            )),
        );
        property.insert(
            "minimum".to_owned(),
            serde_json::Value::from(crate::mcp::MIN_BYTE_LIMIT),
        );
        property.insert(
            "maximum".to_owned(),
            serde_json::Value::from(max_output_bytes),
        );
    }
    if name == "job_list"
        && let Some(property) = properties
            .get_mut("limit")
            .and_then(serde_json::Value::as_object_mut)
    {
        property.insert("minimum".to_owned(), serde_json::Value::from(1));
        property.insert(
            "maximum".to_owned(),
            serde_json::Value::from(MAX_JOB_LIST_LIMIT),
        );
    }
    tool.input_schema = std::sync::Arc::new(schema);
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct JobIdArgs {
    /// Job/task identifier returned by a direct tool handoff or job_start.
    #[serde(alias = "job_id")]
    id: String,
}

#[derive(Debug, Clone, Copy, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
enum OutputStream {
    Result,
    Stdout,
    Stderr,
}

impl From<OutputStream> for JobOutputStream {
    fn from(value: OutputStream) -> Self {
        match value {
            OutputStream::Result => Self::Result,
            OutputStream::Stdout => Self::Stdout,
            OutputStream::Stderr => Self::Stderr,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
enum OutputEncoding {
    #[default]
    Text,
    Base64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct JobOutputArgs {
    /// Job/task identifier.
    #[serde(alias = "job_id")]
    id: String,
    /// Graph node coordinate. May be omitted only for a one-node Job.
    #[serde(default)]
    node: Option<usize>,
    /// result/stdout/stderr. Omit to use the node's natural primary stream.
    #[serde(default)]
    stream: Option<OutputStream>,
    /// Raw byte offset in the selected retained stream. Defaults to zero.
    #[serde(default)]
    offset: Option<u64>,
    /// Maximum response bytes for this page. Defaults to the profile output ceiling and is capped there.
    #[serde(default)]
    max_output_bytes: Option<usize>,
    /// text returns UTF-8-lossy text for model readability; base64 preserves arbitrary bytes exactly.
    #[serde(default)]
    encoding: OutputEncoding,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct JobAckArgs {
    /// Job/task identifier.
    #[serde(alias = "job_id")]
    id: String,
    /// Graph node coordinate. May be omitted only for a one-node Job.
    #[serde(default)]
    node: Option<usize>,
    /// result/stdout/stderr. Omit to use the node's natural primary stream.
    #[serde(default)]
    stream: Option<OutputStream>,
    /// Highest raw byte offset safely received by the caller.
    offset: u64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct JobListArgs {
    /// Zero-based offset into the current newest-first exposed-job snapshot.
    #[serde(default)]
    offset: usize,
    /// Maximum recent exposed jobs to return. Defaults to 64, capped at 256.
    #[serde(default)]
    limit: Option<usize>,
}

#[tool_router(router = "jobs_router", vis = "pub(crate)")]
impl LocalMachine {
    async fn finish_graph_call(
        &self,
        id: &str,
        deadline: Option<tokio::time::Instant>,
        result_budget: Option<usize>,
    ) -> CallToolResult {
        match self.jobs.wait_until(id, deadline).await {
            Ok(JobWait::Pending) => match self.jobs.expose_compatibility(id) {
                Ok(receipt) => receipt,
                Err(error) => tool_error(error),
            },
            Err(error) => tool_error(error),
            Ok(JobWait::Terminal) => {
                let render_budget = result_budget
                    .unwrap_or(self.config.max_output_bytes)
                    .min(self.config.max_output_bytes)
                    .max(1);
                match self.jobs.render_graph_result(id, render_budget) {
                    Ok(Some(text)) => {
                        let result = ok_text(text);
                        if result_budget.is_some_and(|limit| {
                            crate::mcp::serialized_json_len(&result).is_none_or(|size| size > limit)
                        }) {
                            return match self.jobs.expose_compatibility(id) {
                                Ok(receipt) => receipt,
                                Err(error) => tool_error(error),
                            };
                        }
                        match self.jobs.mark_synchronous_result(id) {
                            Ok(_) => result,
                            Err(error) => tool_error(error),
                        }
                    }
                    Ok(None) => match self.jobs.expose_compatibility(id) {
                        Ok(receipt) => receipt,
                        Err(error) => tool_error(error),
                    },
                    Err(error) => tool_error(error),
                }
            }
        }
    }

    async fn replay_graph_response(
        &self,
        id: &str,
        result_budget: Option<usize>,
    ) -> Option<CallToolResult> {
        match self.jobs.replay_handoff(id) {
            Ok(None) => None,
            Err(error) => Some(tool_error(error)),
            Ok(Some(JobHandoff::Compatibility(receipt))) => Some(receipt),
            Ok(Some(JobHandoff::Synchronous)) => {
                let render_budget = result_budget
                    .unwrap_or(self.config.max_output_bytes)
                    .min(self.config.max_output_bytes)
                    .max(1);
                Some(match self.jobs.render_graph_result(id, render_budget) {
                    Ok(Some(text)) => ok_text(text),
                    Ok(None) => tool_error(
                        "committed synchronous Graph result no longer fits its response budget",
                    ),
                    Err(error) => tool_error(error),
                })
            }
            Ok(Some(JobHandoff::NativeTask(_))) => Some(tool_error(
                "Graph response was unexpectedly committed as a native Task",
            )),
        }
    }

    #[tool(
        description = "Execute a bounded canonical Graph of already-granted native filesystem and shell nodes. Independent nodes may run concurrently; conditions and conservative path footprints determine deterministic ordering. Returns the exact compact result when it completes within this request, otherwise hands off the same durable Job for job_get/job_output/job_ack/job_cancel.",
        annotations(
            title = "Run tool Graph",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn batch(
        &self,
        Parameters(args): Parameters<graph::GraphArgs>,
        context: RequestContext<RoleServer>,
    ) -> CallToolResult {
        let result_budget = self.request_result_budget(&context);
        let request_key = self.request_job_key(&context);
        let prepared = match graph::prepare(self, args.ops).await {
            Ok(prepared) => prepared,
            Err(error) => return tool_error(error),
        };
        let started = match graph::start(self, prepared, None, request_key, false).await {
            Ok(started) => started,
            Err(error) => return tool_error(error),
        };
        if !started.is_new
            && let Some(response) = self.replay_graph_response(&started.id, result_budget).await
        {
            return response;
        }
        let deadline = self
            .request_sync_budget_millis(&context)
            .and_then(|millis| {
                tokio::time::Instant::now().checked_add(std::time::Duration::from_millis(millis))
            });
        self.finish_graph_call(&started.id, deadline, result_budget)
            .await
    }

    #[tool(
        description = "Start a bounded canonical Graph as a durable Job. The Job is exposed before any root node may execute. Poll with job_get, read node result/stdout/stderr with job_output, acknowledge with job_ack, cancel with job_cancel, or create a fresh execution with job_restart.",
        annotations(
            title = "Start tool Graph job",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn job_start(
        &self,
        Parameters(args): Parameters<graph::GraphArgs>,
        context: RequestContext<RoleServer>,
    ) -> CallToolResult {
        let prepared = match graph::prepare(self, args.ops).await {
            Ok(prepared) => prepared,
            Err(error) => return tool_error(error),
        };
        match graph::start(self, prepared, None, self.request_job_key(&context), true).await {
            Ok(started) => match self.jobs.expose_compatibility(&started.id) {
                Ok(receipt) => receipt,
                Err(error) => tool_error(error),
            },
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Create a new durable execution from a terminal exposed Job's immutable Graph. Every node is revalidated and re-prepared under current grants, limits, filesystem authority and shell identity before the new Job is admitted. The source Job is never replayed in place.",
        annotations(
            title = "Restart tool Graph job",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn job_restart(
        &self,
        Parameters(args): Parameters<graph::RestartArgs>,
        context: RequestContext<RoleServer>,
    ) -> CallToolResult {
        let source = match self.jobs.shared_snapshot(&args.id) {
            Ok(snapshot)
                if snapshot.lifecycle == aw_tools::filesystem::job::Lifecycle::Terminal =>
            {
                match self.jobs.shared_graph(&args.id) {
                    Ok(graph) => graph,
                    Err(error) => return tool_error(error),
                }
            }
            Ok(_) => return tool_error("job_restart requires a terminal source Job"),
            Err(error) => return tool_error(error),
        };
        let prepared = match graph::prepare_existing(self, source).await {
            Ok(prepared) => prepared,
            Err(error) => return tool_error(error),
        };
        match graph::start(
            self,
            prepared,
            Some(args.id),
            self.request_job_key(&context),
            true,
        )
        .await
        {
            Ok(started) => match self.jobs.expose_compatibility(&started.id) {
                Ok(receipt) => receipt,
                Err(error) => tool_error(error),
            },
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "List recent exposed Jobs. Useful after a lost creation response; returns stable job ids, aggregate status/progress and retained-output usage.",
        annotations(
            title = "List Jobs",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn job_list(
        &self,
        Parameters(args): Parameters<JobListArgs>,
        context: RequestContext<RoleServer>,
    ) -> CallToolResult {
        let limit = args.limit.unwrap_or(64);
        if !(1..=MAX_JOB_LIST_LIMIT).contains(&limit) {
            return tool_error(format!(
                "job_list limit must be within 1..={MAX_JOB_LIST_LIMIT}"
            ));
        }
        let ceiling = self
            .request_result_budget(&context)
            .unwrap_or(self.config.max_output_bytes)
            .min(self.config.max_output_bytes);
        match self.jobs.list_fitted(args.offset, limit, |page| {
            let result = ok_text(page.to_owned());
            crate::mcp::serialized_json_len(&result).is_some_and(|size| size <= ceiling)
        }) {
            Ok(page) => ok_text(page),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Get compact status for an exposed Job. Multi-node Graphs include aggregate progress and node outcome ranges; one-node shell Jobs retain process detail. The same opaque id is used by native MCP Tasks when eligible.",
        annotations(
            title = "Get Job status",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn job_get(&self, Parameters(args): Parameters<JobIdArgs>) -> CallToolResult {
        match self.jobs.status(&args.id) {
            Ok(status) => ok_text(status),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Read one pure/replayable retained Job node page by raw byte offset. One-node Jobs may omit node; multi-node Jobs require it. Native nodes expose result; shell nodes expose result/stdout/stderr and default to stdout. job_output never acknowledges or deletes bytes; continue from next_offset and then call job_ack.",
        annotations(
            title = "Read Job output",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn job_output(
        &self,
        Parameters(args): Parameters<JobOutputArgs>,
        context: RequestContext<RoleServer>,
    ) -> CallToolResult {
        let ceiling = resolve_capped(args.max_output_bytes, self.config.max_output_bytes);
        let response_ceiling = self.request_result_budget(&context).unwrap_or(ceiling);
        let offset = args.offset.unwrap_or(0);
        let base64 = matches!(args.encoding, OutputEncoding::Base64);
        match crate::blocking::run(|| {
            self.jobs.output_node_page(
                &args.id,
                crate::jobs::OutputPageRequest {
                    node: args.node,
                    stream: args.stream.map(Into::into),
                    offset,
                    requested_bytes: ceiling,
                    response_ceiling,
                    base64,
                },
            )
        }) {
            Ok(page) => ok_text(page),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Acknowledge one Job/node/stream raw-byte offset after safe receipt. One-node Jobs may omit node; multi-node Jobs require it. This is the only job-output operation that advances cleanup; physical release waits for every exact output owner.",
        annotations(
            title = "Acknowledge Job output",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn job_ack(&self, Parameters(args): Parameters<JobAckArgs>) -> CallToolResult {
        match self.jobs.acknowledge_node_output(
            &args.id,
            args.node,
            args.stream.map(Into::into),
            args.offset,
        ) {
            Ok(status) => ok_text(status),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Request cancellation of a running Job. Pending native nodes are skipped; running shell nodes stop their contained command tree. Terminal Jobs are left unchanged and retained output remains pageable until acknowledged or expired.",
        annotations(
            title = "Cancel Job",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn job_cancel(&self, Parameters(args): Parameters<JobIdArgs>) -> CallToolResult {
        match self.jobs.cancel(&args.id) {
            Ok(status) => ok_text(status),
            Err(error) => tool_error(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_list_schema_rejects_zero_and_caps_pages() {
        let mut list = LocalMachine::all_tools().get("job_list").cloned().unwrap();
        with_job_limits(&mut list, 3600, 8 * 1024 * 1024);
        let limit = &list.input_schema["properties"]["limit"];
        assert_eq!(limit["minimum"], 1);
        assert_eq!(limit["maximum"], MAX_JOB_LIST_LIMIT);
    }

    #[test]
    fn job_output_schema_uses_profile_result_ceiling() {
        let mut output = LocalMachine::all_tools()
            .get("job_output")
            .cloned()
            .unwrap();
        with_job_limits(&mut output, 3600, 8 * 1024 * 1024);
        let output_limit = &output.input_schema["properties"]["max_output_bytes"];
        assert_eq!(output_limit["maximum"], 8 * 1024 * 1024);
        assert!(
            output_limit["description"]
                .as_str()
                .unwrap()
                .contains("8388608")
        );
    }
}
