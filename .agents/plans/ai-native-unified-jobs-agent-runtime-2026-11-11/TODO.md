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
- [ ] Implement typed SendInput={text}|{eof:true} and jobs.send mode auto|queue|steer|interrupt; Process j:<handle> live stdin needs retained custody **and current exact Control + independent process.input launcher/source/epoch/resource authority compatible with its original retained sandbox**. Revoked/narrowed rights reject new bytes before write; Process sandbox is not dynamically narrowed. Graph node delegates to owner. **Host-envelope invocation ID** stable across provider/SDK/bridge retries before input effect; changed duplicate rejects, ambiguous bytes never replay.

### Process/Graph Job behavior
- [ ] Reserve worst-case replay/exposure recoverability capacity before first Process/Graph effect; then preserve same-Job optimistic completion/handoff and DL-150 lost-receipt rule: replay owner retains ambiguous hidden work; without one, promote/expose the same execution for jobs.list recovery rather than discard/re-execute.
- [ ] Background/automatic handoff receipt returns compact ref + kind/state + current rev (+ useful progress), no argument echo, enabling immediate jobs_get({ref,rev,wait_ms}).
- [ ] Preserve writer quiescence before factual settlement.
- [ ] Preserve process-tree cancellation/unknown stop truthfulness.
- [ ] Preserve immutable Graph topology, preflight, conflict ordering and captured semantic result ceilings.
- [ ] Expose existing Graph nodes as deterministic n:<1-based coordinate> under Job ref, not a second opaque map; no redundant child Jobs or generic node field.
- [ ] Enable typed bread(media) Graph retention only through the shared exact representation.

### Unified scoped Policy and AW bridge boundary
- [ ] AW resolves one trusted effective Policy for actor, target workspace owner, domain, verified team role, Session and Graph/Node: exact ToolPolicy/Envelope/source/site/resource grants, mandatory denies, guardian Allow/Deny/Ask/Narrow, permitted Local/DelegateAgent(Graph|Node), provider/site/payer and limits. Dotlink **does not** resolve team memberships or AW domain/workspace/Session preferences independently; no second PolicyStore, schema or model-facing policy tool.
- [ ] Authenticated bridge passes only the final canonical original operation/ref, source/actor/site/epoch/generation proof and bounded route/resource envelope. Dotlink concrete owner rechecks current local sandbox/profile/launcher/source/site/resource deny/grant. Effective permission is intersection of upstream AW and local dotlink authority; an AW guardian `Allow`/route preference or expired membership proof never grants filesystem/shell/Job access in dotlink.
- [ ] Cross-domain/workspace owner checks remain AW responsibility: caller identity and target d/i/p owner are distinct, verified team role/active context is never inferred from `i:0` or `t:`, source and target domain restrictions both apply. Remote source/policy generation unavailable => refuse new effect rather than trusting cache. No cross-process live finite Job resurrection.
- [ ] Local execution may run at an authorized remote dotlink site; DelegateAgent is AW coordinating model inference and does not move site or payer implicitly. Preserve current `process.input`, ToolPolicy/Control, replay and original Process/Graph native Job custody on either route.
- [ ] Test one multi-team actor accessing a team-owned workspace through AW, with a second team denied: upstream positive grant plus downstream dotlink sandbox mandatory deny must fail. Verify stale team/source/site epoch, unauthorized target domain, Session-only attempted grant, required disallowed model/site, explicit Ask/Narrow and correct no-double-charge/replay behavior. Standalone dotlink must remain safe without AW PolicyContext.

### Crate ownership and local-first bootstrap
- [ ] Consume only the reviewed version of shared `aw-tools` Graph/Job/filesystem primitives. **AW** owns new `aw-policy` (pure resolver), `aw-session` (backend-neutral Session access) and `aw-host` (one Agent/Policy/Session/adapter composer); dotlink adds no dependencies on these crates, no duplicate `aw-actor`/`aw-tools-core`, and no SQLite/Agent Session runtime.
- [ ] Preserve standalone/local dotlink native Process/Graph correctness and no extra provider/policy/Session calls on its hot path. The AW host alone resolves authenticated multiuser Session policy and passes exact canonical intent/effect/epoch evidence to dotlink; each native tool still checks dotlink's current local authority. Include a check that adding the AW crate skeleton does not widen dotlink's dependency/build surface.

### Shared-session coordination: AW-hosted, core extension hooks only

- [ ] Keep **`aw-tools` host-neutral** for optional opaque origin/request ID, owner/coordination/writer-generation fences and existing typed Job/Graph receipts/callbacks. No separate `aw-tools-core` crate, Session ACL, participant inbox, team identity service or collaboration engine in core/dotlink.
- [ ] AW's `aw-tools-adapter` maps those hooks to one original authorized Process/fs/Graph/Job execution owner; the AW SessionStore/Policy/Actor owns memberships, shared drafts/approved context, per-branch coordinator binding, one Input queue/Turn writer, initiator/payer/context disclosure and revocation. dotlink does not create team users or Agent coordinator IDs on share.
- [ ] On AW bridged requests, require exact authenticated canonical actor/source/owner/resource, Session/branch audience and Workspace writer epoch, original replay identity and bounded upstream effective grant evidence. Independently recheck dotlink profile/launcher/sandbox/resource/source/site authority; **both AW upstream and dotlink local grants are required**. A coworker's privileges or workspace label cannot authorize native effects.
- [ ] Concurrent shared participants on one remote original Process/Graph Job must not create new Jobs from duplicate admissions or stale Session/Workspace writer generations. Revocation prevents future new effects under an old audience epoch; already committed/unknown external effects remain factual. No remote shared Agent transcript/Session store or implicit cross-host wake protocol.
- [ ] Verify a one-user standalone/bridged Local request incurs no additional collaborator identity lookup/queue/Agent allocation or model Step. Integration tests: Alice shares personal Session with Bob while keeping same Agent; Bob's missing Process/site grant denies even if Alice has it; team-owned workspace submits through one AW coordinator; stale membership/Workspace writer and replay fail closed; curated-context Session does not expose private origin.

### AW bridge boundary
- [ ] Shared aw-tools primitives permit **Local or delegated callbacks for every prepared operation**, including FS, shell/Process, jobs.*, batch and agent constructors; no hard-coded Local-only tool classes. Dotlink standalone lacks AW Agent/runtime/policy and therefore executes exact native/Process/Graph operations locally. Under authenticated AW bridge, AW selects Local or DelegateAgent **for any operation** under independent grants, with original dotlink Process/Graph/native owner still committing the effect/receipt. For Graphs, AW defaults Local; when delegation chosen, default **Graph** = one Agent/Turn for all original immutable nodes; **Node** is advanced activation-time dynamic Local/Agent routing. Optional one-shot whole-Graph guardian can inform either mode. Capture union of candidate route footprints before roots, narrow per-node on activation without changing `when`/status. No dotlink Agent runner, new public tool, route engine or duplicate Graph JobManager. Graph-node eligibility is host-selected with exact prepared adapter/owner and bounded nested Graph depth/budget; no core class blacklist or self/ancestor Job-control deadlock.
- [ ] Standalone dotlink never advertises agent/jobs_subscribe/wake fields or creates principal/project/Agent Session/Turn/parent/wake-recipient state. Initial AW bridge rejects constructor wake for dotlink-owned Process/Graph **before launch**, and rejects `jobs_subscribe` on an already-live dotlink-owned source **before registration/wake Input**, unless reviewed source-fenced atomic registration/event publication is implemented. Passive read/status/control remains supported.
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
- [ ] standalone advertised schema omits jobs_subscribe, constructor wake **and Agent history `filter`**; AW bridge projects Agent-only capabilities with genuine recipient ownership **and source-owner wake support**. An AW recipient alone cannot enable remote dotlink-owned Process/Graph wake: constructor wakes reject before launch; later subscriptions on running sources reject before registration/notification.
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
- [ ] Record exact provider schema bytes/tokens and workflows before cutover. Compare **Local**, AW-hosted DelegateAgent for native filesystem/Process/Job controls/constructors, **Graph** one coordinator Turn (may use multiple model Steps), **Node** activation-time dynamic Local/Agent routing (skip branches pay zero routing), and optional single whole-Graph guardian pass. Verify original dotlink Process/Graph/native execution, replay/ACK/semantic results, union conflict envelope and exact per-node outcomes for all choices; attribute all parent/auxiliary provider tokens/cost/latency. Test revoked stdin, unsupported remote wake, no duplicate effects and Task/replay retention.

## Execution guard

Do not modify the historical DL-150 or 2026-11-10 plan directories. Do not commit/push/version/tag/release/deploy without separate explicit authorization. Preserve unrelated dirty work.
