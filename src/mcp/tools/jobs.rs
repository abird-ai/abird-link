//! Compatibility job tools for long-running shell commands.
//!
//! Native MCP Tasks and these tools address the same JobManager entries and use
//! the same job id. Clients without the Tasks extension can therefore start,
//! poll, page output and cancel without holding one request open.

use rmcp::{
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Tool},
    schemars,
    service::{RequestContext, RoleServer},
    tool, tool_router,
};
use serde::Deserialize;

use super::{LocalMachine, dot, ok_text, resolve_output_page, tool_error};
use crate::jobs::JobStream;
use std::time::Duration;
use tokio::time::Instant as TokioInstant;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct JobStartArgs {
    /// Command string to execute in the background.
    command: String,
    /// Working directory. Relative paths resolve from dotlink's launch directory.
    #[serde(default = "dot")]
    dir: String,
    /// Optional stdin text passed to the process.
    #[serde(default)]
    stdin: Option<String>,
    /// Execution timeout in seconds. Defaults to the profile shell ceiling and is capped there.
    #[serde(default)]
    timeout_secs: Option<u64>,
}

impl From<JobStartArgs> for super::shell::ShellArgs {
    fn from(args: JobStartArgs) -> Self {
        Self {
            command: args.command,
            dir: args.dir,
            stdin: args.stdin,
            timeout_secs: args.timeout_secs,
            max_output_bytes: None,
        }
    }
}

pub(crate) fn decode_job_start(
    arguments: serde_json::Map<String, serde_json::Value>,
) -> Result<super::shell::ShellArgs, serde_json::Error> {
    serde_json::from_value::<JobStartArgs>(serde_json::Value::Object(arguments)).map(Into::into)
}

pub(crate) fn with_job_limits(tool: &mut Tool, max_secs: u64, max_output_bytes: usize) {
    let name = tool.name.as_ref();
    if !matches!(name, "job_start" | "job_get" | "job_output") {
        return;
    }
    let mut schema = (*tool.input_schema).clone();
    let Some(properties) = schema
        .get_mut("properties")
        .and_then(serde_json::Value::as_object_mut)
    else {
        return;
    };
    if name == "job_start"
        && let Some(property) = properties
            .get_mut("timeout_secs")
            .and_then(serde_json::Value::as_object_mut)
    {
        property.insert(
            "description".to_owned(),
            serde_json::Value::String(format!(
                "Background execution timeout in seconds. Defaults to {max_secs} and is capped at {max_secs}."
            )),
        );
        property.insert("maximum".to_owned(), serde_json::Value::from(max_secs));
    }
    if name == "job_get"
        && let Some(property) = properties
            .get_mut("tail_lines")
            .and_then(serde_json::Value::as_object_mut)
    {
        property.insert("minimum".to_owned(), serde_json::Value::from(0));
        property.insert(
            "maximum".to_owned(),
            serde_json::Value::from(crate::limits::MAX_JOB_TAIL_LINES),
        );
    }
    if name == "job_output"
        && let Some(property) = properties
            .get_mut("max_output_bytes")
            .and_then(serde_json::Value::as_object_mut)
    {
        property.insert(
            "description".to_owned(),
            serde_json::Value::String(format!(
                "Serialized MCP result budget for this page. Defaults to {} and is capped at {}; must be at least {}.",
                super::human_bytes(
                    (crate::limits::DEFAULT_OUTPUT_PAGE_BYTES.min(max_output_bytes)) as u64
                ),
                super::human_bytes(max_output_bytes as u64),
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
    tool.input_schema = std::sync::Arc::new(schema);
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct JobIdArgs {
    /// Job/task identifier returned by bash, powershell or job_start.
    job_id: String,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
enum OutputStream {
    #[default]
    Stdout,
    Stderr,
}

impl From<OutputStream> for JobStream {
    fn from(value: OutputStream) -> Self {
        match value {
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

/// Wait for one job poll.
///
/// The default applies only where a transport supplies a response window: that is
/// where waiting saves round trips. Without one (stdio, local HTTP, the test
/// harness) a read stays immediate unless the caller asks, and an explicit wait is
/// always clamped to the remaining window so it cannot outlive its response.
fn resolve_wait(wait_secs: Option<u64>, budget: Option<Duration>) -> Duration {
    let requested = wait_secs.map(Duration::from_secs);
    let ceiling = crate::limits::DEFAULT_JOB_WAIT;
    match (requested, budget) {
        (Some(wait), Some(budget)) => wait.min(budget),
        // No window (stdio, local HTTP): the wait is still bounded, because one
        // poll must never hold a request — or a serial transport — indefinitely.
        (Some(wait), None) => wait.min(ceiling),
        (None, Some(budget)) => ceiling.min(budget),
        (None, None) => Duration::ZERO,
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct JobGetArgs {
    /// Job/task identifier.
    job_id: String,
    /// Wait up to this many seconds for the job to finish before returning its
    /// status. Defaults to 90 where the transport supplies a response window and
    /// is clamped to what remains of it, so a wait cannot outlive the response.
    /// 0 returns immediately, which is what an interactive status check wants;
    /// 90 is also the ceiling where no response window exists.
    #[serde(default)]
    wait_secs: Option<u64>,
    /// Trailing lines of *each* stream (stdout and stderr) to include in this
    /// reply; a stream with nothing in it is omitted. Defaults to {job_tail}, at
    /// most {job_tail_max}; 0 returns status only. The end of a build answers
    /// "how is it going" and "why did it fail" without paging.
    #[serde(default)]
    tail_lines: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct JobOutputArgs {
    /// Job/task identifier returned by bash, powershell or job_start.
    job_id: String,
    /// stdout or stderr. Defaults to stdout.
    #[serde(default)]
    stream: OutputStream,
    /// Raw byte offset in the selected spool stream. Defaults to zero; `job_get` already carries the tail, so page here only for the full output.
    #[serde(default)]
    offset: Option<u64>,
    /// Maximum response bytes for this page. Defaults to the short-page size and is capped at the profile ceiling.
    #[serde(default)]
    max_output_bytes: Option<usize>,
    /// Read the last N bytes of the stream instead of paging from `offset`
    /// (0 reads one byte). Mutually exclusive with `offset`. Returns at once
    /// unless `wait_secs` is given, which waits for the job to finish so the tail
    /// is final. The page is the end of that window; acknowledging it frees the
    /// whole prefix, including output it did not show.
    #[serde(default)]
    tail_bytes: Option<usize>,
    /// text returns UTF-8-lossy text for model readability; base64 preserves arbitrary bytes exactly.
    #[serde(default)]
    encoding: OutputEncoding,
    /// Wait up to this many seconds for output past `offset` (or for the job to
    /// finish) before paging. Defaults to 90 where the transport supplies a
    /// response window and is clamped to what remains of it; 90 is also the
    /// ceiling where no window exists. 0 pages immediately.
    #[serde(default)]
    wait_secs: Option<u64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct JobAckArgs {
    /// Job/task identifier.
    job_id: String,
    /// stdout or stderr. Defaults to stdout.
    #[serde(default)]
    stream: OutputStream,
    /// Highest byte offset safely received by the caller.
    offset: u64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct JobListArgs {
    /// Maximum recent exposed jobs to return. Defaults to 64, capped at 256.
    #[serde(default)]
    limit: Option<usize>,
}

#[tool_router(router = "jobs_router", vis = "pub(crate)")]
impl LocalMachine {
    #[tool(
        description = "Start a shell command in the background; returns its durable job/task id immediately. Ordinary bash/powershell use the same execution layer but first wait through their response window. Spools retain <={spool} per stream, drained fully, with the retained prefix marked truncated. Follow up: job_get=status, job_output(offset)=output, job_ack(highest read)=free spool, job_cancel=stop.",
        annotations(
            title = "Start background shell job",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn job_start(
        &self,
        Parameters(args): Parameters<JobStartArgs>,
    ) -> Result<CallToolResult, McpError> {
        // The background half lives with the shell it starts, so the transport
        // dispatch and this handler cannot drift apart.
        self.background_receipt(args.into(), cfg!(windows), None)
    }

    #[tool(
        description = "List recent durable jobs exposed in this runtime, not scoped to a conversation, so entries may belong to another client of the same bridge. Useful after a lost creation response: stable ids, status/phase, elapsed, session spool usage.",
        annotations(
            title = "List shell jobs",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn job_list(&self, Parameters(args): Parameters<JobListArgs>) -> CallToolResult {
        ok_text(self.jobs.list(args.limit.unwrap_or(64).clamp(1, 256)))
    }

    #[tool(
        description = "Current state of a durable shell job: status, elapsed, exit state, stdout/stderr byte counts, and the last {job_tail} lines of each stream so one call shows how far it got or why it failed. The job id is the native task id when Tasks are supported. `wait_secs`: 90 by default where the transport supplies a response window, clamped to what remains, waiting for completion; 0 answers immediately. `tail_lines` defaults to {job_tail} and is capped at {job_tail_max}; 0 omits the lines.",
        annotations(
            title = "Get shell job status",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn job_get(
        &self,
        Parameters(args): Parameters<JobGetArgs>,
        context: RequestContext<RoleServer>,
    ) -> CallToolResult {
        let wait = resolve_wait(
            args.wait_secs,
            self.request_sync_budget_millis(&context)
                .map(Duration::from_millis),
        );
        if !wait.is_zero() {
            let deadline = TokioInstant::now() + wait;
            if let Err(error) = self.jobs.wait_until(&args.job_id, Some(deadline)).await {
                return tool_error(error);
            }
        }
        let tail_lines = args
            .tail_lines
            .unwrap_or(crate::limits::DEFAULT_JOB_TAIL_LINES)
            .min(crate::limits::MAX_JOB_TAIL_LINES);
        // The tail reads the spool, so it runs off the async worker; a status
        // probe with no tail does no I/O beyond the metadata `status` already
        // needs.
        let status = match tail_lines {
            0 => self.jobs.status(&args.job_id),
            _ => crate::blocking::run(|| self.jobs.status_with_tail(&args.job_id, tail_lines)),
        };
        match status {
            Ok(status) => ok_text(status),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Read content from a job's stdout or stderr: a page from `offset`, or the last `tail_bytes` of the stream. Never acknowledges or deletes, so it stays replayable; `wait_secs` (default and clamp as in job_get) waits for new bytes past `offset` or completion, 0 pages what is retained now. Pages default to the short-page size and are bounded by the profile ceiling; page from `next_offset` until `complete=true`, then `job_ack` the highest safely received offset. For a quick look at the end without reading the middle, `job_get` already returns the last lines of each stream. Streams retain <={spool} and report truncation.",
        annotations(
            title = "Read shell job output",
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
        let ceiling = resolve_output_page(args.max_output_bytes, self.config.max_output_bytes);
        let response_ceiling = self.request_result_budget(&context).unwrap_or(ceiling);
        let stream = args.stream.into();
        let offset = args.offset.unwrap_or(0);
        let base64 = matches!(args.encoding, OutputEncoding::Base64);
        let window = self
            .request_sync_budget_millis(&context)
            .map(Duration::from_millis);
        if args.tail_bytes.is_some() && args.offset.is_some_and(|value| value > 0) {
            return tool_error("pass either `offset` or `tail_bytes`, not both");
        }
        // A tail peek is "show me the end now". Only an explicit `wait_secs`
        // defers it; inheriting the default wait would silently turn every peek
        // into a wait for completion.
        let wait = match (args.tail_bytes.is_some(), args.wait_secs) {
            (true, None) => Duration::ZERO,
            _ => resolve_wait(args.wait_secs, window),
        };
        if !wait.is_zero() {
            let deadline = TokioInstant::now() + wait;
            // A tail read wants the end of the stream, so it waits for the job
            // itself; an offset read waits for new bytes past that offset.
            let waited = match args.tail_bytes.is_some() {
                true => self
                    .jobs
                    .wait_until(&args.job_id, Some(deadline))
                    .await
                    .map(|_| ()),
                false => {
                    self.jobs
                        .wait_for_output(&args.job_id, stream, offset, deadline)
                        .await
                }
            };
            if let Err(error) = waited {
                return tool_error(error);
            }
        }
        let request = crate::jobs::OutputPageRequest {
            offset,
            tail: args.tail_bytes,
            bytes: ceiling,
            response_ceiling,
            base64,
        };
        match crate::blocking::run(|| self.jobs.output_page(&args.job_id, stream, request)) {
            Ok(page) => ok_text(page),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Acknowledge `job_output`'s `next_offset` for the same `stream` once received. The only operation that advances cleanup: after both terminal streams are fully acknowledged, compatibility spools are released. It is a high-water mark, so acknowledging an offset releases everything before it — including output that was skipped by a tail read. Native tasks instead keep the exact-result prefix that `tasks/get` replays, until task retention expires. Unacked output holds the session spool budget until retention ({retention}).",
        annotations(
            title = "Acknowledge shell job output",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn job_ack(&self, Parameters(args): Parameters<JobAckArgs>) -> CallToolResult {
        match self
            .jobs
            .acknowledge_output(&args.job_id, args.stream.into(), args.offset)
        {
            Ok(status) => ok_text(status),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Cancel a running shell job; kills its command tree. Terminal jobs are unchanged. Output already spooled stays readable via job_output until acknowledged or expired.",
        annotations(
            title = "Cancel shell job",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn job_cancel(&self, Parameters(args): Parameters<JobIdArgs>) -> CallToolResult {
        match self.jobs.cancel(&args.job_id) {
            Ok(status) => ok_text(status),
            Err(error) => tool_error(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_schemas_expose_only_the_relevant_profile_limits() {
        let mut start = LocalMachine::all_tools().get("job_start").cloned().unwrap();
        with_job_limits(&mut start, 3600, 8 * 1024 * 1024);
        let start_props = start.input_schema["properties"].as_object().unwrap();
        assert_eq!(start_props["timeout_secs"]["maximum"], 3600);
        assert!(
            !start_props.contains_key("max_output_bytes"),
            "background output is paged separately and must not advertise a synchronous stream cap"
        );

        let mut output = LocalMachine::all_tools()
            .get("job_output")
            .cloned()
            .unwrap();
        with_job_limits(&mut output, 3600, 8 * 1024 * 1024);
        let output_limit = &output.input_schema["properties"]["max_output_bytes"];
        assert_eq!(output_limit["maximum"], 8 * 1024 * 1024);
        let output_description = output_limit["description"].as_str().unwrap();
        assert!(output_description.contains("8 MiB"), "{output_description}");
        assert!(
            output_description.contains("256 KiB"),
            "{output_description}"
        );

        // The tail peek advertises the same default and cap the handler clamps
        // to, in the tool description and in the argument itself.
        let mut get = LocalMachine::all_tools().get("job_get").cloned().unwrap();
        with_job_limits(&mut get, 3600, 8 * 1024 * 1024);
        let tail = &get.input_schema["properties"]["tail_lines"];
        assert_eq!(tail["maximum"], crate::limits::MAX_JOB_TAIL_LINES as u64);
        assert_eq!(tail["minimum"], 0);

        let tokens =
            crate::mcp::tools::description_tokens(crate::limits::DEFAULT_JOB_RETENTION_SECS);
        crate::mcp::tools::substitute_description_tokens(&mut get, &tokens);
        let get_description = get.description.as_deref().unwrap();
        assert!(
            get_description.contains(&format!(
                "the last {} lines",
                crate::limits::DEFAULT_JOB_TAIL_LINES
            )),
            "{get_description}"
        );
        assert!(
            get_description.contains(&format!("capped at {}", crate::limits::MAX_JOB_TAIL_LINES)),
            "{get_description}"
        );
        let tail_description = get.input_schema["properties"]["tail_lines"]["description"]
            .as_str()
            .unwrap()
            .to_owned();
        assert!(!tail_description.contains('{'), "{tail_description}");
        assert!(
            tail_description.contains(&crate::limits::DEFAULT_JOB_TAIL_LINES.to_string())
                && tail_description.contains(&crate::limits::MAX_JOB_TAIL_LINES.to_string()),
            "{tail_description}"
        );

        let tokens =
            crate::mcp::tools::description_tokens(crate::limits::DEFAULT_JOB_RETENTION_SECS);
        crate::mcp::tools::substitute_description_tokens(&mut start, &tokens);
        let start_description = start.description.as_deref().unwrap();
        assert!(
            start_description.contains(&crate::mcp::tools::human_bytes(
                crate::limits::MAX_JOB_SPOOL_BYTES_PER_STREAM
            )),
            "{start_description}"
        );
    }
}

#[cfg(test)]
mod wait_tests {
    use super::resolve_wait;
    use std::time::Duration;

    #[test]
    fn job_wait_defaults_only_where_a_response_window_exists() {
        let secs = Duration::from_secs;
        // Hosted transport: wait by default, capped by what remains.
        assert_eq!(resolve_wait(None, Some(secs(60))), secs(60));
        assert_eq!(resolve_wait(None, Some(secs(200))), secs(90));
        // No window (stdio, local HTTP, tests): stay immediate.
        assert_eq!(resolve_wait(None, None), Duration::ZERO);
        // An explicit wait is honoured and clamped, never lengthened.
        assert_eq!(resolve_wait(Some(10), Some(secs(60))), secs(10));
        assert_eq!(resolve_wait(Some(200), Some(secs(60))), secs(60));
        assert_eq!(resolve_wait(Some(5), None), secs(5));
        // A sub-second transport budget is respected rather than rounded up.
        assert_eq!(
            resolve_wait(None, Some(Duration::from_millis(1500))),
            Duration::from_millis(1500)
        );
        // 0 always means "answer now".
        assert_eq!(resolve_wait(Some(0), Some(secs(60))), Duration::ZERO);
        // Without a window an explicit wait is still bounded: one poll must not
        // hold a serial transport open for as long as the caller typed.
        assert_eq!(resolve_wait(Some(3_600), None), secs(90));
    }
}

#[cfg(test)]
mod wire_tests {
    use super::{JobGetArgs, JobOutputArgs};

    #[test]
    fn wait_is_accepted_on_both_waiting_tools_and_typos_still_fail() {
        let get: JobGetArgs = serde_json::from_value(serde_json::json!({
            "job_id": "job-1",
            "wait_secs": 30
        }))
        .expect("job_get accepts wait_secs");
        assert_eq!(get.wait_secs, Some(30));

        let output: JobOutputArgs = serde_json::from_value(serde_json::json!({
            "job_id": "job-1",
            "wait_secs": 5
        }))
        .expect("job_output accepts wait_secs");
        assert_eq!(output.wait_secs, Some(5));

        // Omission is the common case and must stay valid.
        let omitted: JobOutputArgs =
            serde_json::from_value(serde_json::json!({"job_id": "job-1"})).unwrap();
        assert_eq!(omitted.wait_secs, None);

        // The strict-field policy still names a typo instead of ignoring it.
        assert!(
            serde_json::from_value::<JobGetArgs>(serde_json::json!({
                "job_id": "job-1",
                "wait": 30
            }))
            .is_err()
        );
    }
}
