//! Standalone finite Job control surface over the canonical dotlink JobManager.
//!
//! Process/Graph execution, retention, replay, output ownership and ACK remain in
//! `crate::jobs`. This module is only the flat model-facing projection.

use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Tool},
    schemars,
    service::{RequestContext, RoleServer},
    tool, tool_router,
};
use serde::Deserialize;

use aw_tools::execution_ref::{ExecutionRef, ExecutionSegment};

use super::{LocalMachine, graph, ok_text, resolve_capped, tool_error};
use crate::jobs::{JobHandoff, JobOutputStream, JobWait, ProcessInput};

const MAX_JOB_LIST_LIMIT: usize = 256;
const MAX_WAIT_MS: u64 = 30_000;

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
    if let Some(route) = router.map.get_mut("batch") {
        route.attr.input_schema = schema;
    }
}

pub(crate) fn with_job_limits(tool: &mut Tool, _max_secs: u64, max_output_bytes: usize) {
    let name = tool.name.as_ref();
    if !matches!(name, "jobs_read" | "jobs_list") {
        return;
    }
    let mut schema = (*tool.input_schema).clone();
    let Some(properties) = schema
        .get_mut("properties")
        .and_then(serde_json::Value::as_object_mut)
    else {
        return;
    };
    if name == "jobs_read"
        && let Some(property) = properties
            .get_mut("limit")
            .and_then(serde_json::Value::as_object_mut)
    {
        property.insert(
            "description".to_owned(),
            serde_json::Value::String(format!(
                "Maximum retained raw bytes for this page. Defaults to the profile output ceiling, must be at least {} and is capped at {max_output_bytes}.",
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
    if name == "jobs_list"
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct StandaloneRef {
    id: String,
    node: Option<usize>,
}

fn parse_ref(reference: &str) -> Result<StandaloneRef, String> {
    if !reference.starts_with("j:") {
        return Err("standalone Job ref must start with j:".into());
    }
    let resolved = ExecutionRef::parse(reference)
        .map_err(|_| "invalid standalone Job ref".to_owned())?
        .resolve(None, None)
        .map_err(|_| "invalid standalone Job ref".to_owned())?;
    match resolved.suffix() {
        [ExecutionSegment::Job(job)] => Ok(StandaloneRef {
            id: job.as_str().to_owned(),
            node: None,
        }),
        [ExecutionSegment::Job(job), ExecutionSegment::Node(node)] => Ok(StandaloneRef {
            id: job.as_str().to_owned(),
            node: Some(node.get() as usize),
        }),
        _ => Err("standalone Job ref supports only j:<handle>[/n:<coordinate>]".into()),
    }
}

#[derive(Debug, Clone, Copy, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
enum ReadView {
    Result,
    Stdout,
    Stderr,
}

impl From<ReadView> for JobOutputStream {
    fn from(value: ReadView) -> Self {
        match value {
            ReadView::Result => Self::Result,
            ReadView::Stdout => Self::Stdout,
            ReadView::Stderr => Self::Stderr,
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
struct JobsListArgs {
    #[serde(default)]
    cursor: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct JobsGetArgs {
    #[serde(rename = "ref")]
    reference: String,
    #[serde(default)]
    rev: Option<u64>,
    #[serde(default)]
    wait_ms: Option<u64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct JobsReadArgs {
    #[serde(rename = "ref")]
    reference: String,
    #[serde(default)]
    view: Option<ReadView>,
    #[serde(default)]
    offset: Option<u64>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    encoding: OutputEncoding,
    #[serde(default)]
    wait_ms: Option<u64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct JobsAckArgs {
    #[serde(rename = "ref")]
    reference: String,
    #[serde(default)]
    view: Option<ReadView>,
    offset: u64,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
enum SendMode {
    #[default]
    Auto,
    Queue,
    Steer,
    Interrupt,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct SendInput {
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    eof: Option<bool>,
}

impl SendInput {
    fn into_process(self) -> Result<ProcessInput, String> {
        match (self.text, self.eof) {
            (Some(text), None) if text.len() <= 64 * 1024 => {
                Ok(ProcessInput::Text(text.into_bytes()))
            }
            (None, Some(true)) => Ok(ProcessInput::Eof),
            _ => Err("jobs_send input must be exactly {text:string} or {eof:true}".into()),
        }
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct JobsSendArgs {
    #[serde(rename = "ref")]
    reference: String,
    #[serde(default)]
    rev: Option<u64>,
    #[serde(default)]
    mode: SendMode,
    input: SendInput,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct JobsCancelArgs {
    #[serde(rename = "ref")]
    reference: String,
    #[serde(default)]
    rev: Option<u64>,
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
                Ok(_) => match self.jobs.standalone_status(id, None) {
                    Ok((_, status)) => ok_text(status),
                    Err(error) => tool_error(error),
                },
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
                            let _ = self.jobs.expose_compatibility(id);
                            return match self.jobs.standalone_status(id, None) {
                                Ok((_, status)) => ok_text(status),
                                Err(error) => tool_error(error),
                            };
                        }
                        match self.jobs.mark_synchronous_result(id) {
                            Ok(_) => result,
                            Err(error) => tool_error(error),
                        }
                    }
                    Ok(None) => {
                        let _ = self.jobs.expose_compatibility(id);
                        match self.jobs.standalone_status(id, None) {
                            Ok((_, status)) => ok_text(status),
                            Err(error) => tool_error(error),
                        }
                    }
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
            Ok(Some(JobHandoff::Compatibility(_))) => Some(
                self.jobs
                    .standalone_status(id, None)
                    .map(|(_, status)| ok_text(status))
                    .unwrap_or_else(tool_error),
            ),
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
        description = "Execute a bounded canonical Graph of already-granted native filesystem and shell nodes. Set background=true to return the admitted durable Graph Job immediately; otherwise dotlink returns the exact compact result when it completes within this request and hands off the same Job if the synchronous window closes.",
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
        let background = args.background;
        let _display_title = args.title;
        let prepared = match graph::prepare(self, args.ops).await {
            Ok(prepared) => prepared,
            Err(error) => return tool_error(error),
        };
        let started = match graph::start(self, prepared, None, request_key, background).await {
            Ok(started) => started,
            Err(error) => return tool_error(error),
        };
        if !started.is_new
            && let Some(response) = self.replay_graph_response(&started.id, result_budget).await
        {
            return response;
        }
        if background {
            return match self.jobs.standalone_status(&started.id, None) {
                Ok((_, status)) => ok_text(status),
                Err(error) => tool_error(error),
            };
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
        description = "List recent exposed standalone Process/Graph Jobs newest first. Paging uses a runtime-generation-bound opaque cursor; no Agent, Session or subtree state is listed.",
        annotations(
            title = "List Jobs",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn jobs_list(
        &self,
        Parameters(args): Parameters<JobsListArgs>,
        context: RequestContext<RoleServer>,
    ) -> CallToolResult {
        let limit = args.limit.unwrap_or(20);
        if !(1..=MAX_JOB_LIST_LIMIT).contains(&limit) {
            return tool_error(format!(
                "jobs_list limit must be within 1..={MAX_JOB_LIST_LIMIT}"
            ));
        }
        let ceiling = self
            .request_result_budget(&context)
            .unwrap_or(self.config.max_output_bytes)
            .min(self.config.max_output_bytes);
        match self
            .jobs
            .standalone_list_fitted(args.cursor.as_deref(), limit, |page| {
                let result = ok_text(page.to_owned());
                crate::mcp::serialized_json_len(&result).is_some_and(|size| size <= ceiling)
            }) {
            Ok(page) => ok_text(page),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Observe one explicit standalone Process/Graph Job or Graph node. rev is a conditional observation token; wait_ms may wait for a different revision without changing execution timeout.",
        annotations(
            title = "Get Job status",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn jobs_get(&self, Parameters(args): Parameters<JobsGetArgs>) -> CallToolResult {
        let reference = match parse_ref(&args.reference) {
            Ok(reference) => reference,
            Err(error) => return tool_error(error),
        };
        let wait_ms = args.wait_ms.unwrap_or(0);
        if wait_ms > MAX_WAIT_MS {
            return tool_error(format!("jobs_get wait_ms exceeds {MAX_WAIT_MS}"));
        }
        match self
            .jobs
            .standalone_get(&reference.id, reference.node, args.rev, wait_ms)
            .await
        {
            Ok(status) => ok_text(status),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Read one replayable retained standalone Job view by raw offset. Process and Process-backed Graph nodes default to stdout; native/Graph results default to result. Reading never acknowledges or releases bytes.",
        annotations(
            title = "Read Job output",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn jobs_read(
        &self,
        Parameters(args): Parameters<JobsReadArgs>,
        context: RequestContext<RoleServer>,
    ) -> CallToolResult {
        if args.wait_ms.unwrap_or(0) > MAX_WAIT_MS {
            return tool_error(format!("jobs_read wait_ms exceeds {MAX_WAIT_MS}"));
        }
        let reference = match parse_ref(&args.reference) {
            Ok(reference) => reference,
            Err(error) => return tool_error(error),
        };
        let ceiling = resolve_capped(args.limit, self.config.max_output_bytes);
        let response_ceiling = self.request_result_budget(&context).unwrap_or(ceiling);
        let offset = args.offset.unwrap_or(0);
        let base64 = matches!(args.encoding, OutputEncoding::Base64);
        let is_graph = match self.jobs.standalone_is_graph(&reference.id) {
            Ok(value) => value,
            Err(error) => return tool_error(error),
        };
        if is_graph && reference.node.is_none() {
            if args
                .view
                .is_some_and(|view| !matches!(view, ReadView::Result))
            {
                return tool_error(
                    "Graph root exposes only result; use j:<handle>/n:<coordinate> for node streams",
                );
            }
            return match self
                .jobs
                .standalone_graph_result_page_wait(
                    &reference.id,
                    offset,
                    ceiling,
                    response_ceiling,
                    base64,
                    args.wait_ms.unwrap_or(0),
                )
                .await
            {
                Ok(page) => ok_text(page),
                Err(error) => tool_error(error),
            };
        }
        match self
            .jobs
            .output_node_page_wait(
                &reference.id,
                crate::jobs::OutputPageRequest {
                    node: reference.node,
                    stream: args.view.map(Into::into),
                    offset,
                    requested_bytes: ceiling,
                    response_ceiling,
                    base64,
                },
                args.wait_ms.unwrap_or(0),
            )
            .await
        {
            Ok(page) => ok_text(page),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Explicitly acknowledge one exact retained Job ref/view through a raw byte offset after safe receipt. ACK is monotonic/idempotent and is never implied by jobs_read.",
        annotations(
            title = "Acknowledge Job output",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn jobs_ack(&self, Parameters(args): Parameters<JobsAckArgs>) -> CallToolResult {
        let reference = match parse_ref(&args.reference) {
            Ok(reference) => reference,
            Err(error) => return tool_error(error),
        };
        match self.jobs.acknowledge_node_output(
            &reference.id,
            reference.node,
            args.view.map(Into::into),
            args.offset,
        ) {
            Ok(_) => match self.jobs.standalone_status(&reference.id, reference.node) {
                Ok((_, status)) => ok_text(status),
                Err(error) => tool_error(error),
            },
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Send typed live stdin to one explicit interactive Process Job/node. Process supports mode=auto only; input is exactly {text} or {eof:true}. Current launcher/sandbox/tool authority is rechecked before delivery.",
        annotations(
            title = "Send Job input",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn jobs_send(&self, Parameters(args): Parameters<JobsSendArgs>) -> CallToolResult {
        if !matches!(args.mode, SendMode::Auto) {
            return tool_error("standalone Process jobs_send supports only mode=auto");
        }
        let reference = match parse_ref(&args.reference) {
            Ok(reference) => reference,
            Err(error) => return tool_error(error),
        };
        let input = match args.input.into_process() {
            Ok(input) => input,
            Err(error) => return tool_error(error),
        };
        match self
            .send_process_input_checked(&reference.id, reference.node, args.rev, input)
            .await
        {
            Ok(revision) => ok_text(format!(
                "{} rev={revision} ok",
                crate::jobs::JobManager::standalone_ref(&reference.id, reference.node)
            )),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Idempotently request cancellation of one explicit standalone Process/Graph Job. rev is an optional compare-and-act precondition; Graph-node cancellation is not exposed initially.",
        annotations(
            title = "Cancel Job",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn jobs_cancel(&self, Parameters(args): Parameters<JobsCancelArgs>) -> CallToolResult {
        let reference = match parse_ref(&args.reference) {
            Ok(reference) => reference,
            Err(error) => return tool_error(error),
        };
        if reference.node.is_some() {
            return tool_error("standalone jobs_cancel targets a Job root, not a Graph node");
        }
        if let Some(expected) = args.rev {
            let (current, _) = match self.jobs.standalone_status(&reference.id, None) {
                Ok(snapshot) => snapshot,
                Err(error) => return tool_error(error),
            };
            if expected != current {
                return tool_error(format!(
                    "stale Job revision: expected {expected}, current {current}"
                ));
            }
        }
        match self.jobs.cancel(&reference.id) {
            Ok(_) => match self.jobs.standalone_status(&reference.id, None) {
                Ok((_, status)) => ok_text(status),
                Err(error) => tool_error(error),
            },
            Err(error) => tool_error(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standalone_refs_are_explicit_and_node_coordinates_are_canonical() {
        assert_eq!(
            parse_ref("j:abc-123").unwrap(),
            StandaloneRef {
                id: "abc-123".into(),
                node: None
            }
        );
        assert_eq!(
            parse_ref("j:abc-123/n:4").unwrap(),
            StandaloneRef {
                id: "abc-123".into(),
                node: Some(4)
            }
        );
        for invalid in ["", ".", "^", "abc", "j:", "j:x/n:0", "j:x/n:01", "j:x/x:1"] {
            assert!(parse_ref(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn jobs_list_schema_rejects_zero_and_caps_pages() {
        let mut list = LocalMachine::all_tools().get("jobs_list").cloned().unwrap();
        with_job_limits(&mut list, 3600, 8 * 1024 * 1024);
        let limit = &list.input_schema["properties"]["limit"];
        assert_eq!(limit["minimum"], 1);
        assert_eq!(limit["maximum"], MAX_JOB_LIST_LIMIT);
    }

    #[test]
    fn standalone_constructors_expose_background_without_wake_or_legacy_start() {
        let router = LocalMachine::all_tools();
        let batch = router.get("batch").expect("batch");
        let batch_props = batch.input_schema["properties"].as_object().unwrap();
        assert!(batch_props.contains_key("background"));
        assert!(!batch_props.contains_key("wake"));
        assert!(router.get("job_start").is_none());
        assert!(router.get("job_restart").is_none());

        if let Some(shell) = router.get(crate::mcp::shell_tool_name()) {
            let shell_props = shell.input_schema["properties"].as_object().unwrap();
            assert!(shell_props.contains_key("background"));
            assert!(!shell_props.contains_key("wake"));
        }
    }

    #[test]
    fn jobs_read_schema_uses_profile_result_ceiling_and_has_no_filter() {
        let mut read = LocalMachine::all_tools().get("jobs_read").cloned().unwrap();
        with_job_limits(&mut read, 3600, 8 * 1024 * 1024);
        let output_limit = &read.input_schema["properties"]["limit"];
        assert_eq!(output_limit["maximum"], 8 * 1024 * 1024);
        assert!(
            !read.input_schema["properties"]
                .as_object()
                .unwrap()
                .contains_key("filter")
        );
    }
}
