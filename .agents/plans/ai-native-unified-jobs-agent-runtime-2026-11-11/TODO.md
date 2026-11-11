# Unified Jobs + Agent runtime — dotlink integration TODO

Base: b31683c79a8ffd7c5540ac9388e607d8af6c7ffa

Full semantic authority: AW ai-native-unified-jobs-agent-runtime-2026-11-11/DESIGN.md

## Committed

None. Documentation only.

## Uncommitted

### Shared revision and surface cutover
- [ ] Wait for/review the AW implementation revision; consume one exact remote Git revision with no sibling path/copy.
- [ ] Map flat provider and model-facing Graph filesystem names to canonical fs.* operation IDs below the wire through one resolver; do not make the model learn both read and fs.read.
- [ ] Replace old job_* model routes with standalone jobs_list/get/read/ack/send/cancel; advertise jobs_subscribe only through an authenticated AW Agent host.
- [ ] Hard pre-1.0 source/API cutover: remove job_start/job_restart/old aliases, obsolete decoders/adapters/tests in the same source change. Typed shell/batch constructor + background=true replaces job_start; fresh constructors replace restart. No migration, automated drain, dual-schema or legacy replay support. Reset only verified isolated disposable dev/test state per explicit quiescence gate.
- [ ] Keep batch as the single Graph constructor; background disposition exposes the same Graph Job.
- [ ] Do not add grouped jobs({op}) or a generic model-visible call tool.

### Typed hierarchical refs / jobs.read
- [ ] Adopt shared typed-handle ExecutionRef/ResolvedExecutionRef and Revision types plus canonical handle lexical grammar; send/cancel use compare-and-act rev, get uses conditional observation rev, and read/ack expose no rev.
- [ ] Standalone dotlink resolves finite j:<handle>[/n:<1-based coordinate>] refs only in its authenticated live session, with no AW d/i/p/a/default Agent authority. get/read/ack/send/cancel require explicit ref; jobs_list has no ref. Known Graph coordinates need no scope discovery.
- [ ] With AW bridge, accept only AW-resolved fully canonical identity; AW alone handles fixed provisioned canonical d/i/p/a :0 defaults, overlays, ".", "^" and Turn/Node paths. :0 is never remapped to the authenticated caller; missing/unauthorized defaults fail and never create principals/Agents. dotlink never interprets those structural semantics or widens access.
- [ ] Project Process **default stdout** and explicit stderr/result through jobs.read; Graph native defaults result, Process-backed Graph node stdout. Preserve exact raw-byte offset/EOF/truncation and typed ReadView.
- [ ] Project Graph whole/n:<1-based coordinate> typed result streams through jobs.read; validate owner-typed n/j suffix transitions before dispatch.
- [ ] Keep jobs.read pure/read-only; never ACK or advance retention.
- [ ] Keep jobs.ack exact/idempotent on ref/view/raw offset, no rev; watermark cannot exceed retained bytes delivered/available to that owner's output lease, and lost-response retries/other Task/replay owners remain safe.
- [ ] Preserve MCP Task/replay ownership over retained bytes.
- [ ] Use shared typed JobReadPage/Content representation rather than a dotlink-only parallel format.
- [ ] Implement typed SendInput={text}|{eof:true} and jobs.send mode auto|queue|steer|interrupt; Process j:<handle> accepts live stdin only with retained interactive custody; Graph node delegates to owner. **Host-envelope invocation ID** is stable for provider/SDK/bridge retries, established before stdin/EOF effect. Changed duplicate rejects; ambiguous bytes are not blindly replayed; unsupported inputs/modes reject.

### Process/Graph Job behavior
- [ ] Reserve worst-case replay/exposure recoverability capacity before first Process/Graph effect; then preserve same-Job optimistic completion/handoff and DL-150 lost-receipt rule: replay owner retains ambiguous hidden work; without one, promote/expose the same execution for jobs.list recovery rather than discard/re-execute.
- [ ] Background/automatic handoff receipt returns compact ref + kind/state + current rev (+ useful progress), no argument echo, enabling immediate jobs_get({ref,rev,wait_ms}).
- [ ] Preserve writer quiescence before factual settlement.
- [ ] Preserve process-tree cancellation/unknown stop truthfulness.
- [ ] Preserve immutable Graph topology, preflight, conflict ordering and captured semantic result ceilings.
- [ ] Expose existing Graph nodes as deterministic n:<1-based coordinate> under Job ref, not a second opaque map; no redundant child Jobs or generic node field.
- [ ] Enable typed bread(media) Graph retention only through the shared exact representation.

### AW bridge boundary
- [ ] Standalone dotlink never implements AW ExecutionStrategy/OperationRouter or transparent DelegateAgent routing; it executes its local Process/Graph semantics only. With an AW bridge, AW owns strategy selection and may send concrete Process/Graph work to dotlink after its own route/authority admission.
- [ ] Standalone dotlink never advertises agent/jobs_subscribe/wake fields or creates principal/project/Agent Session/Turn/parent/wake-recipient state.
- [ ] Define authenticated bridge routing for AW-owned canonical Agent/execution refs only if/when AW integration requires it.
- [ ] Forward Agent jobs.get/read/send/cancel/subscribe to the authenticated AW owner, preserving host-envelope invocation identity, Agent-only normalized filter/cursor and current AW disclosure/causal budget checks. Standalone dotlink omits Agent filter and subscription/wake fields; never cache transcript/Turn/parent state.
- [ ] Preserve current-Agent contextual-overlay resolution and recipient/authority checks at AW, not transport text.

### Removal without security regression
- [ ] Remove job_restart model/execution API while preserving startup/replay/private-artifact reconciliation; retained Job definitions are audit/replay evidence only and never executable authority.
- [ ] Delete obsolete compatibility decode/state paths and legacy-only tests outright. Stop old dev/test runtimes; before resetting old transient state, **explicitly prove isolated disposability/quiescence** (closed ingress, stopped supervised Process tree, no active/unknown external effect, client, paid provider attempt, outstanding Task/replay response or ACK lease). Unknown obligations block reset; no new compatibility/migration service. Retain new-schema replay/Task/ACK correctness and unrelated durable AW Session/Agent data.
- [ ] Preserve protected path, deny, sandbox, OAuth, launcher, network/hardware, request-limit and stdout cleanliness invariants.
- [ ] Preserve no cross-process live finite Job resurrection.

## Tests and verification

- [ ] New flat tool catalog contains no old job_* / job_start / job_restart names.
- [ ] No grouped jobs op schema or model-visible generic call; common jobs.* schemas use compact typed enums/oneOf rather than arbitrary strings/objects or huge specialization unions.
- [ ] jobs_list is deterministic flat exposed-root discovery with opaque older-than cursor; standalone get/read/ack/send/cancel reject omitted/"." refs rather than inventing self context.
- [ ] standalone advertised schema omits jobs_subscribe, constructor wake **and Agent history `filter`**; AW bridge projection adds those only with real current Agent/recipient authority.
- [ ] jobs_read Process stdout/stderr/result paging exact.
- [ ] jobs_read Graph deterministic n:<coordinate>/whole result exact, without a scope-lookup call.
- [ ] jobs_ack watermark never implied by read: exact ref/view/raw offset, no rev, monotonic/idempotent retry, cannot exceed bytes retained and delivered/available to this owner, immutable truncation/status retained, physical cleanup waits all owners.
- [ ] direct shell synchronous/handoff/replay behavior unchanged; background=true exposes the same execution, and interactive=false/true correctly closes or retains live stdin custody.
- [ ] shell cancel/timeout/unknown/writer-quiescence tests green.
- [ ] Graph deterministic conflict/status/more/incomplete/result-ceiling tests green.
- [ ] MCP Task + OpenAI replay + retained result ownership suites green.
- [ ] stdio/MCP protocol output clean.
- [ ] native filesystem shared conformance green.
- [ ] Bubblewrap and Windows/macOS native gates run on capable hosts or remain explicitly pending.
- [ ] exact provider schema byte/token comparison and representative model workflows recorded before cutover, including jobs.list recovery, get rev/wait long polling, typed send/read validation, no-rev vs rev controls and compact j:<handle>/n:<handle> Graph access.

## Execution guard

Do not modify the historical DL-150 or 2026-11-10 plan directories. Do not commit/push/version/tag/release/deploy without separate explicit authorization. Preserve unrelated dirty work.
