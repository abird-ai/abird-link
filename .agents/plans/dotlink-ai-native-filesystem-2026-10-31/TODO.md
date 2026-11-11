# DL-150 — AI-native tool surface + canonical Job graph runtime

Base: `84cd9b7`

Status: **model-facing design sealed; shared-engine cutover complete, with platform release gates still open**. Native filesystem and Graph/Job execution use the shared implementation in production, all three shared crates pin reviewed remote AW `4a23e92a8f516971bb232b98e4fc000554bcc4ba`, and the remote-pinned offline/locked gate is green. Remaining blockers are the explicitly documented native Bubblewrap and Windows/macOS platform checks.

## Current status override — 2026-11-09

The current source/checkpoints supersede unchecked implementation boxes in older historical Phase 7–9 and validation subsections below. Those boxes document the original roadmap and are **not instructions to reimplement completed Graph/Job/shared-engine work**. The canonical current handoff is [FULL-COMPREHENSIVE-HANDOFF-2026-11-09.md](FULL-COMPREHENSIVE-HANDOFF-2026-11-09.md).

Current reviewed state:

- [x] AW canonical shared runtime is remotely reachable at exact revision `4a23e92a8f516971bb232b98e4fc000554bcc4ba`.
- [x] dotlink cutover is committed at `b31683c79a8ffd7c5540ac9388e607d8af6c7ffa`; all shared crates use the exact remote Git pin and no sibling/path substitute remains.
- [x] Production `batch`, Graph-shaped `job_start`, `job_restart`, native nodes and ShellNodes use the shared `aw-tools::JobManager`; duplicate native filesystem engines are retired.
- [x] Remote-pinned Linux/conformance/integration/check/Clippy/format gates recorded in this plan are green.
- [!] dotlink `b31683c` is still one commit ahead of configured `origin/main`; pushing requires explicit user authorization.
- [!] Native Windows/macOS and successful Bubblewrap execution remain platform/environment acceptance gates.
- [ ] Native Windows UNC/network-share execution remains to be verified on a real Windows runner.

## Committed

### `c6f0e1c` — initial AI-native filesystem specification

- [x] Recorded the approved core tool split, token-efficiency rules, lifecycle
  primitives and implementation roadmap.

### `9b7d4b4` — read limits and intermediate singular-edit design

- [x] Kept read/write byte ceilings as runtime policy rather than routine model
  parameters.
- [x] Specified long-line read continuation and the then-intermediate singular
  edit shape, later superseded by `f09790d`.
- [x] Specified pre/post-publication mutation outcome handling.

### `6c1cb16` — status, paging, progress and initial batch grammar

- [x] Added one canonical outcome grammar and known/unknown paging conventions.
- [x] Added cumulative known-total/open-ended progress semantics.
- [x] Recorded the initial shallow batch orchestration design.

### `f28ce2c` — compact `ops` execution graph

- [x] Replaced shallow stages with positional graph nodes, backward-only status
  dependencies, joins, parallel roots and decision branches.
- [x] Kept nested tool arguments structured while limiting text syntax to the
  small dependency expression.

### `60b348d` — simplified fresh model-facing surface

- [x] Made query modifiers orthogonal to primary outcomes and made bare batch
  dependencies require complete success.
- [x] Removed advertised patch best-effort, cp recursive and asymmetric grep
  context knobs; unified paging and path composability.

### `f09790d` — single-file multi-replacement edit hot path

- [x] Made `edit(path,edits)` batch independent exact substitutions against one
  original snapshot with overlap rejection and one atomic publication.
- [x] Kept one advertised shape for one or many edits.

### `ebb5b84` — sealed v1 AI-native filesystem design

- [x] Sealed the v1 public surface, token/result grammar, concurrency boundary
  and deferred-feature list before implementation.

### `5942393` — shell write hints and durable shell-node v2

- [x] Added advisory shell `writes` scopes and specified graph-owned Bash/
  PowerShell nodes with process-terminal branching; later design converged on
  the canonical shared Job/Graph/Node runtime.

### `2a255f3` — durable batch controls/checkpoints

- [x] Added automatic resume, explicit restart generations, compact polling,
  idempotent cancellation and fail-closed node checkpoint semantics.

### `8a3e7bb` — unified durable lifecycle under `job_*`

- [x] Replaced parallel batch control concepts with the shared `job_*` namespace,
  one durable ID space, generic restart, and node-aware output/ack semantics.

### `a156142` — canonical Job/Graph/Node runtime

- [x] Made one Job owning one Graph the canonical execution architecture and
  defined shell calls as one-ShellNode Jobs rather than nested child Jobs.
- [x] Established `pwsh` as the modern PowerShell name; the final sealed design
  below further makes Windows `powershell` a distinct first-class interpreter.

### `bec528f` — finalized canonical Job graph design

- [x] Unified `batch(ops)` and `job_start(ops)` on one Graph grammar/runtime,
  sealed the three exact shell interpreter capabilities, and centralized shared
  Job lifecycle/replay/output semantics before the final correctness review.

### `81a54ce` — sealed reviewed AI-native Job graph design

- [x] Re-ran the complete design for correctness/regressions/minimality, removed
  stale status/runtime assumptions, and sealed the implementation-facing v1/v2
  contract before this edge-case review.

## Implementation roadmap

### Phase 0 — shared implementation convergence

**AW/shared implementation checkpoint (2026-11-09):** AW A1-A7 plus post-pin Job/output/config/topology hardening are freshly reviewed and pushed at commit `4a23e92a8f516971bb232b98e4fc000554bcc4ba`. The latest fix revalidates each canonical native filesystem footprint while its Graph node is still `Claimed`; any final-symlink/topology drift from the captured `NodePlan` settles unlaunched `skip` before the shared `Start` checkpoint. `aw-tools` remains the canonical semantic/Job crate, `aw-tools-adapter` remains the AW host adapter, and secured Unix/Windows backends depend only on the shared layer. The refactor remains consistent with AW's larger goals: one Agent/ToolPolicy/journal, narrowing-only host runtime limits, one JobManager, full/minimal/stateless/no-std boundaries, child-limit monotonicity and atomic scheduler-policy updates. Fresh gates: `aw-tools` 279/279, Unix secured backend 41/41, AW adapter 203/203, Agent core 78 active + 1 intentional ignore, AW CLI library 229/229 active, AW CLI binary 384/384 active + 1 intentional ignore, workspace all-target compile, strict affected Clippy, stateless/no-default, `minimal`, no-std+alloc, rustfmt/diff. Fresh Windows cross verification is environment-blocked before AW source compilation by a dangling cached Nix Windows-std symlink. `origin/main` now contains `4a23e92a8f516971bb232b98e4fc000554bcc4ba`, and dotlink pins all three shared crates to that exact remote Cargo Git revision. Sibling review paths are retired; never substitute a committed sibling path, local `file://` URL or vendored copy.

**Dotlink convergence checkpoint:** production native filesystem dispatch uses the shared canonical parser/executor/backend for all 13 tools and the duplicate native filesystem modules are removed. The model-visible Graph surface is live: `batch(ops)`, Graph-shaped `job_start(ops)`, `job_restart(id)`, native filesystem nodes and ShellNodes use the shared `JobManager` for canonical lifecycle, captured budgets/footprints, deterministic admission, output ownership/ACK, exposure and cancellation/revocation; dotlink retains transport-delivery replay, process custody/sandbox, TTL and MCP projection. Against remote-pinned AW `4a23e92`, linked conformance and a separately regenerated external fixture (13 schemas / 13 requests) pass; the explicit ignored fixture gate executes and passes. The main suite is 628/628 active unit tests plus 1 intentional external-fixture ignore; all five integration binaries pass; strict all-target Clippy/check, rustfmt and diff checks pass. The fresh review adds one pre-effect topology invariant: before a native node crosses from `Claimed` to `Running`, dotlink recomputes the shared canonical request footprint and requires exact equality with the captured `NodePlan`, so a retargeted final symlink cannot invalidate deterministic conflict scheduling. The tracked dependencies use exact remote `4a23e92`; Cargo regenerated `Cargo.lock` with `git+https://github.com/abird-ai/aw?...#4a23e92`, and the full no-override validation is green.

- [x] Treat `SHARED-IMPLEMENTATION.md` as the canonical cross-repo implementation architecture: one shared `aw-tools` semantic/runtime crate, one secured backend per platform, and thin AW/dotlink adapters.
- [x] Fold the three review clarifications into the canonical model/runtime spec: paged `job_list(offset=0,limit=64)`, exact crash-private mutation artifact ownership/recovery, and lexical-prefix-preserving output for an authorized symlinked discovery root.
- [x] Fresh reviewed AW implementation `4a23e92a8f516971bb232b98e4fc000554bcc4ba` is pushed and remotely reachable; dotlink uses that exact Cargo Git pin. No committed local-path substitute.
- [x] In AW, mechanically rename `aw-tools-core -> aw-tools` and current `aw-tools -> aw-tools-adapter`; the rename and subsequent convergence phases are complete and validated in AW.
- [x] Keep dotlink a single package; the thin shared-filesystem/rmcp adapter is internal (`mcp/shared_fs.rs`, `mcp/tools/shared_native.rs`) rather than a symmetry-only workspace crate.
- [x] All three AW dependencies point together at reviewed remote `4a23e92...`; Cargo regenerated the lockfile from that Git source and the complete offline/locked no-override gate is green. Never commit a `../aw` path, local `file://` dependency or vendored implementation.
- [x] Fresh cross-repo correctness review at AW `55d93db...`: AW shared/config/runtime architecture remained clean; dotlink retained its earlier pre-effect admission hardening and additionally fixed the pre-admission exposure race so shared visibility cannot lag an already-committed local durable surface. Full local-AW override main suite is 619/619 active unit tests plus 1 intentional ignore; production-binary strict Clippy, the exact external conformance fixture and all integration binaries pass. Strict all-target Clippy, rustfmt and diff checks pass. Added an explicit shutdown regression proving terminal shared file-backed output may remain unacknowledged while host shutdown safely expires shared retention, closes spool handles and removes the private session directory.
- [x] Fresh topology/cutover review at AW `4a23e92`: confirmed the prior AW configuration/runtime refactor still satisfies off-mode, child inheritance, selected-tool, dynamic-policy, direct-vs-Graph and private-state boundaries; fixed final-symlink retarget drift by revalidating the canonical footprint before native `Start` in both AW and dotlink. Dotlink regression proves both possible targets stay unchanged and the node settles `skip`. Historical local-AW override at that checkpoint was 620/620 active + 1 intentional external-fixture ignore; the separately regenerated AW fixture's explicit ignored gate passes; all integrations, strict all-target Clippy/check, rustfmt and diff checks are green. A disposable exact-Git consumer resolves `aw-tools` at `4a23e92` successfully, then is blocked only by the environment's known crates.io CA-chain failure when fetching registry metadata.
- [x] Fresh lifecycle/replay ownership review at AW `4a23e92`: standalone ShellNode execution now commits canonical shared `Start` before `Command::spawn`; a definitive spawn refusal settles known `fail`, while post-spawn custody ambiguity remains `unknown`. Any host entry that owns a shared Job now treats missing canonical state as split-brain rather than silently falling back to legacy lifecycle semantics. Shutdown cleans in place and retains host metadata/spool state if shared/output cleanup cannot be proven safe. Hosted replay revalidates the exact committed inner `Synchronous`/`Durable` representation immediately before delivering a cached terminal shell response; a stale body returns to the same inner execution's recoverable path instead of being delivered first and retired afterward. Full local-AW suite is now 628/628 active + 1 intentional external-fixture ignore; all five integration binaries, strict all-target Clippy/check, rustfmt and diff checks are green.
- [x] Remote-pin cutover verification: pushed AW `4a23e92a8f516971bb232b98e4fc000554bcc4ba`, replaced all three sibling review dependencies with exact HTTPS Git revisions, regenerated `Cargo.lock` through Cargo, and reran dotlink without dependency overrides. `cargo test --offline --locked --all-features` passes 628/628 active unit tests + 1 intentional external-fixture ignore and all five integrations (1/1, 8/8, 1/1, 12/12, 4/4). The separately regenerated AW fixture passes the ignored cross-repo gate (13 schemas / 13 requests). `cargo check --offline --locked --all-targets --all-features`, strict all-target/all-feature Clippy, rustfmt and `git diff --check` are green. No `../aw`, `file://` or vendored shared dependency remains.
- [x] Final integration lint pass removes an unused backend alias from production and replaces over-wide Job output page argument lists with one typed request/render path. Native Graph preparation/execution is production-live through `batch`, `job_start` and `job_restart`; current-AW all-target check and production-binary strict `-D warnings` are green.
- [!] Native Bubblewrap smoke reaches the real launcher but this host rejects `proc` mounting with `Operation not permitted`; successful namespace execution remains an environment gate. Dotlink Windows cross-build is separately blocked by target-cache/Cargo CA constraints. A fresh AW Windows cross-run is also environment-blocked before source compilation because the cached Windows std sysroot symlink points to a removed Nix store path; earlier Windows cross results are historical evidence only.
- [x] Make the shared `aw-tools` JobManager the sole canonical Graph/Job/Task lifecycle implementation; AW uses it through the original journal/authority/Process adapter and dotlink now uses it for production native/ShellNode Graph execution while retaining product transport/process responsibilities.

- [x] Final cross-repo status review: shared `unknown` dominates cancel/revoke intent; dotlink preserves its live `working` / terminal `completed|failed|cancelled|unknown` compatibility vocabulary while exposing canonical `skip` and shared Graph lifecycle/summary detail. Explicit external conformance fixture from AW `4a23e92...` passes.
- [x] Shared canonical file semantics no longer impose the old user-file single-hard-link restriction; private control/spool artifacts retain their stricter integrity checks.
- [x] Replace magic-prefix private staging/tombstone hiding with bounded exact identity-owned records supplied by protected host stores; dotlink prepares durable per-write-root registries and reconciles them before serving shared filesystem operations.
- [x] Retired duplicate dotlink native filesystem implementations only after shared canonical/security/failure and conformance tests passed; production dispatch now has one native filesystem engine.
- [x] Add a test-only dotlink differential boundary that can consume the versioned shared conformance fixture without affecting production tool dispatch or standalone builds.
- [x] Capture the first strict legacy-vs-canonical schema/tool/request drift report; do not normalize it away or weaken the shared expectations.
- [x] Use the linked `aw_tools::conformance` surface as the primary in-repo parity gate; retain external fixture-path injection only as an ignored cross-repository proof that a separately generated reviewed fixture matches the linked revision.
- [x] Build the thin dotlink filesystem adapter on the platform `CapabilitySet`: admitted grants/exclusions become `PreparedGrant`/`ExclusionSpec`, protected crash-durable Registry stores are reconciled before serving, final-symlink target namespaces are reauthorized, and recursive discovery/lifecycle semantics remain shared rather than reimplemented.
- [x] Dotlink direct/native, ShellNode and Graph limits are mapped without semantic drift: read/write ceilings narrow shared safety maxima, encoded binary schema capacity is distinct from decoded writes, direct rendering consumes the authenticated result budget, Graph `NodePlan`s capture semantic work/result ceilings, retention calls shared `expire_terminal`, large shell logs attach through shared file-backed output ownership/quota, and `--serialize` captures Graph width 1 without a second scheduler.
- [ ] Verify canonical Windows UNC/network-share paths (`//server/share/...`) on native Windows CI during cutover; the shared grammar/router already cross-compiles them, and they must never be silently routed through the launch drive.
- [x] Complete model-visible Graph surface: `batch`, Graph-shaped `job_start`, `job_restart`, native+ShellNode execution through one shared JobManager, current-authority restart, and `--serialize` width-1 capture. Direct shell calls retain their one-ShellNode compatibility path without a second Graph parser or manager.

### Phase 1 — shared discovery/metadata architecture

- [x] Introduce one shared metadata model with portable file/symlink `size`,
  POSIX `mode`, `mtime`, `ctime`, `btime` and precise unavailable semantics.
- [x] Refactor descriptor-rooted traversal so `hidden`, `gitignore`, `depth`,
  budgets, active mutation-private-name exclusion and incomplete reasons are
  single-sourced.
- [x] Introduce shared selector, ordering and compact-rendering primitives with
  canonical `/` path separators, reversible UTF-8 labels, and fail-closed
  `path_encoding` coverage for unaddressable invalid-UTF-8 Unix names.
- [x] Preserve current deny/protected-path and symlink non-traversal invariants.
- [x] Preserve file/path-level mutation coordination as the correctness baseline;
  do not add model-facing range/revision coordination knobs in v1.

### Phase 2 — `ls` + `find`

- [x] Shrink `ls` to immediate-directory inspection only.
- [x] Add `find` for recursive glob/regex/kind/depth/time discovery.
- [x] Add compact path output and opt-in ordered metadata columns.
- [x] Add `mtime`/`ctime`/`btime` filtering and sorting.
- [x] Add `hidden` and `gitignore` consistently to both.
- [x] Preserve bounded globally correct selection/paging.

### Phase 3 — `grep` + read-side schema cleanup

- [x] Align `grep` with `depth`, `hidden`, `gitignore`.
- [x] Keep `glob` as the content-search path prefilter.
- [x] Replace advertised `before`/`after` with one bounded symmetric
  `context`; use follow-up `read` for asymmetric inspection.
- [x] Make content results always line-numbered and remove advertised
  `numbered`.
- [x] Convert files/count results to compact deterministic text; define a grep
  hit/count as one matching logical line and order content by path then line.
- [x] Remove the legacy fixed per-file match-count cap; bounded file bytes/global
  scan work plus logical paging keep work finite while making later matches
  reachable.
- [x] Make `offset`/`limit` count primary hits in content mode and matching files
  in files/count mode; context never consumes page slots or leaks hits across
  page boundaries.
- [x] Stop advertising `read.max_read_bytes`; retain it as runtime/profile
  policy and surface the effective limit only in actionable errors.
- [x] Replace advertised read `offset`/`limit` with
  `line`/`lines`/`byte` and support continuation inside oversized UTF-8
  lines.
- [x] Use the same continuation cursor when a direct-call profile/transport
  ceiling ends a page; Graph-node modifiers instead come only from the semantic
  node ceilings captured at Job creation.
- [x] Define mid-read I/O faults as error-marked/fault-marked results with any
  safely obtained prefix and cursor, never as a complete successful read.

### Phase 4 — text mutation schema/output cleanup

- [x] Make `edit(path,edits)` the single-file hot-path primitive, with compact
  tuples `[old,new,all?]`.
- [x] Resolve every tuple against one original snapshot; `all` uses
  left-to-right non-overlapping matches; reject ambiguous non-`all` matches and
  overlaps between selected ranges before staging.
- [x] Build/verify/publish the file once so several independent replacements
  remain one atomic visible mutation.
- [x] Keep contextual, dependent, overlapping and multi-file editing
  responsibility in strict `patch`.
- [x] Keep one advertised edit shape even for a single replacement; do not add a
  top-level singular shorthand.
- [x] Keep legacy multi-file edit objects and the temporary singular edit shape
  only as compatibility deserialization if needed.
- [x] Replace verbose successful mutation receipts with compact outcomes where
  no additional information is needed.
- [x] Define one shared mutation outcome model: not committed, committed,
  unknown, and explicit mixed multi-object commit state.
- [x] Ensure write/edit/binary staging failures never expose partial target
  contents; over-limit writes fail before staging/publication.
- [x] Keep `write`/`bwrite` genuinely write-authority-only: successful publish is
  `ok` and must not perform a hidden equality read to manufacture `unchanged`.
- [x] Preserve existing target access protection during atomic replacement; an
  in-flight temporary must never be broader than the target it replaces.
- [x] Single-source final-component symlink semantics: content tools follow only
  after lexical + resolved authorization and preserve the link entry; create-only
  paths reject dangling links; lifecycle tools act on the link itself/no-follow.
- [x] Keep Codex patch grammar, advertise strict-only `patch(patch)`, and
  leave move/delete to dedicated tools; coalesce shared missing-parent Add File
  sections into deterministic atomic namespace publication units.
- [x] Accept legacy `best_effort` only at the compatibility boundary if a
  migration window is required.

### Phase 5 — lifecycle primitives

- [x] Design/implement `cp(from,to)` through shared mutation/staging: files copy
  as files, symlinks as exact no-follow links, directories as complete bounded
  trees; missing parent suffixes publish atomically; special descendants and
  destination-inside-source fail closed; hard-link topology is not promised.
- [x] Design/implement same-filesystem `mv(from,to)`; never implicit copy+delete,
  require the destination parent to exist, reject self/descendant moves by
  namespace identity, and handle hard-link aliases/case-only renames explicitly.
- [x] Design/implement `rm(paths,recursive=false)` with full-request preflight;
  recursive roots commit by atomic detach to exact private tombstones before
  cleanup, never caller-visible child-by-child deletion. Durably reserve **all**
  bounded identity-bound cleanup records before the first detach, revalidate each
  source identity immediately before rename, reconcile them before serving
  filesystem discovery after restart, and reject duplicate/alias/ancestor-
  descendant overlaps before any mutation.
- [x] Auto-create missing destination parents only for `cp`, inside the same
  staged publication unit; `mv` is strictly one rename or no namespace change.
- [x] Do not add glob expansion, destination inference or implicit overwrite.
- [x] Add replay protection/capability visibility for all lifecycle mutations.

### Phase 6 — canonical status, paging and progress

- [x] Implement primary outcomes `ok`, `unchanged`, `fail`, `mixed`,
  `unknown`, `skip` plus orthogonal query modifiers `more` and
  `incomplete`; keep `unknown` as the sole fail-closed ambiguity name (no
  `maybe` alias).
- [x] Make `mixed` represent multiple material terminal outcome families,
  including mixtures that contain unknown components; use `unknown` only when
  all material components are unknown, and treat `skip` as structurally
  neutral when another material outcome exists.
- [x] Use one paging grammar based on actual follow-up argument names
  (`more offset=200`, `more line=42 byte=...`), with optional known page
  fractions only when already cheap.
- [x] Keep `more` (recoverable continuation) distinct from `incomplete`
  (coverage not guaranteed), including combined states.
- [x] Separate exact data and status into distinct MCP content blocks when status
  text would corrupt returned bytes/text.
- [x] Standardize progress as cumulative snapshots: fixed total when known,
  omitted total when open-ended, stable units and terse phase messages.
- [x] Ensure progress never carries the only copy of result data.

### Phase 7 — shared Job/Graph/Node runtime + native `batch`

- [ ] Introduce the canonical execution model: one `Job` owns one immutable
  `Graph`; a Graph owns one or more `Node`s; nodes are never nested Jobs.
- [ ] Generalize the existing JobManager around a lightweight shared Graph/Node
  runtime without duplicating replay, cancellation, retention or Task state.
- [ ] Add the two shared Graph constructors together: `batch(ops)`
  optimistically waits, while `job_start(ops)` durably exposes the same Graph
  immediately.
- [ ] Represent each operation as compact `[tool,args,when?]` with stable
  1-based coordinates derived from position.
- [ ] Default omitted `when` to previous **complete** success
  (`ok|unchanged` with no `more`/`incomplete`); use `*` for an independent
  root.
- [ ] Implement bounded backward-only dependency expressions with insignificant
  whitespace, `&` binding tighter than `|`, parentheses, exact status/modifier
  predicates and `N.*` for any terminal outcome; reject empty graphs and add no
  negation/implicit adjacency/status classes such as `.err` or `.done`.
- [ ] Reject forward references/cycles by construction and evaluate conditions
  only after all referenced operations are terminal.
- [ ] Support sequence, fan-out, joins and status-based decision branches without
  variables, loops, output substitution or nested batches.
- [ ] Mark false branches as `skip condition`; expected skips do not make an
  otherwise successful Job an error.
- [ ] Runtime-validate Graph-node args against the already registered tool schema
  instead of duplicating every schema into `batch` or `job_start`.
- [ ] Add one central Graph-node eligibility/version property; initially restrict
  v1 Graph nodes to exact-result native filesystem tools and reuse it for both
  `batch` and `job_start` validation; keep typed-content `bread` direct-only.
- [ ] Give every native node adapter a conservative machine-only read/write
  footprint derived from the same normalized/resolved planning as the tool. Gate
  conflicting native nodes deterministically by lower Graph coordinate before
  claim; widen uncertain scopes instead of guessing disjointness.
- [ ] Capture deterministic per-node semantic work/result ceilings when the Job
  is created; outer transport/result budgets may hand off/page the Job result but
  must never re-page a child or change `more`/`incomplete` and thereby alter
  dependent side effects.
- [ ] Aggregate `more`/`incomplete` modifiers explicitly and never duplicate
  exceptional nodes in both detail blocks and summaries.
- [ ] Add compact fixed-total Graph/Job progress (`3/12 2 ok`) and forward known
  (`412/1000 copy`) or unknown (`918 scan`) node progress snapshots.
- [ ] Replay-protect every `batch`/`job_start` Job creation regardless of node
  mix, and derive deterministic per-node replay identities so redelivery cannot
  duplicate Graph work or repeat committed mutations.
- [ ] For `job_start`, commit durable Job identity/exposure/replay state after
  complete Graph preflight and before any root node starts; lost receipts must be
  recoverable through `job_list` without duplicate creation.
- [ ] Enforce hard operation-count, parallel-width, aggregate byte/work and
  retained-result ceilings; reserve bounded immutable Graph/replay/control state
  before any root node is claimed.
- [ ] Add one process-wide Graph node-runner permit budget shared by every Job;
  acquire a permit before `pending -> claimed`, keep a per-Job parallel-width
  ceiling, and keep active-Job/runner/retained-Job/spool budgets distinct.
- [ ] Decouple Job controls from shell permission: any runtime that can expose a
  Job must expose the controls needed to observe/cancel/consume it, including
  native-only read/write profiles.
- [ ] Generalize abandoned-response safety: durable-request transports retain
  hidden replay ownership; transports without durable request identity expose
  the same Job through `job_list` when the last undecided response disappears.
- [ ] Project a Job as native MCP Task only when its exact eventual ordinary
  tool result is guaranteed to fit the Task-result budget; otherwise use the
  same Job through `job_*` without truncation or semantic rewriting.

### Phase 8 — shell naming + write hints

- [ ] Advertise three distinct interpreter capabilities when actually available:
  `bash`, modern PowerShell `pwsh`, and Windows PowerShell `powershell`.
- [ ] Bind each tool/node to its exact prepared interpreter; never alias or
  silently fall back between `pwsh` and `powershell`.
- [ ] Add optional `writes` path scopes identically to all advertised shell
  arguments; omission means unknown, `[]` means expected no writes.
- [ ] Resolve relative hint paths from effective shell `dir`; use no globs,
  shell expansion or environment interpolation.
- [ ] Treat scopes as advisory scheduling metadata only; never as authorization,
  sandbox policy, replay identity, correctness proof or happens-before edge;
  explicit Graph dependencies are the ordering contract.
- [ ] Preserve each shell node's `writes` arguments unchanged inside
  `batch(ops)` and `job_start(ops)` so durable/graph execution does not discard
  scheduler information.
- [ ] Document create/modify/delete/rename semantics and conservative subtree
  scope behavior.

### Phase 9 — one-node shell Jobs + v2 shell Graph nodes

- [ ] Route ordinary `bash`, `pwsh` and `powershell` through the canonical
  one-node Job Graph path rather than separate shell execution/durability
  machinery.
- [ ] Keep the one-node path lightweight; architectural unification must not add
  unnecessary persistence/allocation to short synchronous shell commands.
- [ ] Add `bash`, `pwsh` and `powershell` as v2 ShellNodes; keep `job_start`,
  job/task controls and nested `batch` excluded from Graph node dispatch.
- [ ] Preserve the already-shared `batch(ops)` / `job_start(ops)` Graph
  constructors when adding ShellNodes; do not fork shell-specific graph parsing.
- [ ] Map ShellNode process outcomes canonically everywhere: successful exit ->
  `ok`; known non-zero/timeout/launch-failure/cancellation -> `fail`; ambiguous
  state -> `unknown`.
- [ ] Keep Graph outcome separate from the direct MCP envelope: a normally
  launched process that exits non-zero is Graph-`fail` but preserves the
  existing direct-shell MCP-success process result with `success=false`;
  pre-execution/timeout/internal failures remain MCP errors as appropriate.
- [ ] Keep compact ShellNode exit/success/timeout/error metadata plus separate
  bounded stdout/stderr streams; shell failure must never imply rollback of
  shell side effects.
- [ ] Make the outer Job the sole durable owner automatically when execution
  outlives a synchronous transport window; no per-node durability flag and no
  ShellNode escapes as an independent user-visible child Job/Task.
- [ ] Keep the graph scheduler running after handoff so dependents continue when
  long shell nodes terminate even without active client polling.
- [ ] Persist/retain per-node `pending/claimed/running/terminal` state and
  deterministic replay identity.
- [ ] Persist `claimed` before shell launch and bind the child to a deterministic
  graph+node runner identity; ambiguous launch state must fail closed, never
  relaunch.
- [ ] Checkpoint every terminal node before unlocking dependents; later failures
  or lost responses must never re-execute terminal predecessors.
- [ ] Redelivery reattaches to the same running child or marks it `unknown`
  rather than restarting it.
- [ ] Keep Job/Graph durability scoped to one live dotlink Job session; do not
  resurrect Job IDs or Graphs after process exit/crash/session replacement.
- [ ] Preserve cross-process **request replay safety** separately: either prove a
  retired-session request cannot be redelivered to a replacement process or use
  a bounded protected replay tombstone committed crash-atomically before root
  execution; bind request identity + semantic hash, retain through the maximum
  redelivery horizon, fail closed as `unknown`, and never persist the full Graph
  merely for deduplication.
- [ ] Reuse bounded shell spooling for child stdout/stderr and deterministic
  retained-result rendering/paging without feeding outer delivery budgets back
  into node terminal modifiers.
- [ ] Keep one `job_*` lifecycle and one ID namespace for all exposed Jobs;
  derive compact status labels from Graph shape/tool identity rather than
  durability kinds, and do not add parallel `batch_get/cancel/output/ack` tools.
- [ ] Prefer concise advertised `id` in new job control schemas while accepting
  legacy `job_id` at the compatibility boundary.
- [ ] Make `job_get(id)` a cheap Graph-status snapshot only; label one-node Jobs
  by node tool/interpreter and multi-node Jobs as `graph`, without repeatedly
  dumping retained bulk output.
- [ ] Define resume as automatic reattachment to the same job/checkpoint ledger;
  do not add a redundant `job_resume` tool.
- [ ] Extend `job_output`/`job_ack` with optional Graph `node`: one-node Jobs may
  omit it, multi-node Jobs require it; NativeToolNodes expose `result`, while
  ShellNodes expose stdout/stderr/result with natural stream inference.
- [ ] Add generic replay-protected `job_restart(id)`: any terminal exposed Job
  creates a fresh Job from its retained immutable Graph while retaining the
  source Job as history.
- [ ] Revalidate the entire retained Graph against current tool visibility,
  authority and exact prepared interpreter availability before a restart can
  begin; old Job authority must never authorize new work.
- [ ] Make Jobs runtime-session scoped rather than per-client; callers sharing
  one authorized LocalMachine/runtime share one exposed Job namespace.
- [ ] On incompatible execution-authority change, revoke the old Job session as
  a whole: stop new starts, mark pending nodes `skip revoked`, terminate
  ShellNode trees, let active native mutations settle safely, fail closed all old
  `job_output`/Task-result payload reads, and settle cancelled/revoked before
  replacing the session.
- [ ] Job cancellation records intent before action, prevents pending node
  starts, marks them `skip cancelled`, terminates running ShellNode trees, and
  preserves safe native-mutation terminal semantics.
- [ ] Repeated cancellation is idempotent; cancelling a terminal run is a no-op
  returning its terminal state. If process termination/reattachment cannot prove
  a running ShellNode stopped, settle that node `unknown`, never fake `fail`.
- [ ] Treat shell spool truncation as retained-output observability only; it must
  not add Graph `incomplete` or alter process-success dependencies.
- [ ] Keep `job_output` offsets/acks in raw retained bytes across UTF-8-lossy text
  and exact base64 projections; ack is monotonic/idempotent and cannot release
  bytes still owned by an exact native Task result/replay owner.
- [ ] Forward compact Job/Graph progress while ShellNodes run without inventing
  a shell percentage.
- [ ] Allow `writes` hints to reduce known scheduling conflicts inside the graph;
  a shell write scope conflicts with native reads as well as writes it may
  invalidate, while unknown hints remain conservative and never become authority.
- [ ] Do not enable ShellNodes by a registry bit alone; exact interpreter
  binding, runtime-session Job control, durability, replay, cancellation and
  output retention ship as one coherent v2 feature.

### Phase 10 — compatibility, measurements and docs

- [ ] Keep compatibility strictly at wire/deserialization boundaries.
- [ ] Do not publish an intermediate `job_start` regression: advertise the new
  `job_start(ops)` shape only when ShellNodes are ready; temporarily accept the
  old shell-shaped `job_start(command,...)` request only at deserialization and
  translate it to one ShellNode.
- [ ] Measure exact serialized advertised-tool schema bytes before/after,
  including the explicit `edits` vs abbreviation tradeoff.
- [ ] Record a representative tokenizer estimate for the same serialized surface.
- [ ] Add a repeatable baseline/regression gate for permanent schema cost.
- [ ] Measure representative `ls`/`find`/`grep` result token cost
  before/after.
- [ ] Update Architecture, Security and user-facing documentation only where the
  final behavior requires it.
- [ ] Remove stale duplicated tool-contract prose from older live docs rather
  than maintaining conflicting specifications.

## Tests and verification

- [ ] Unit tests for shared metadata semantics on supported platforms: portable
  file/symlink size, unavailable directory/other size, POSIX-mode availability,
  timestamp precision and exclusive RFC3339 range endpoints.
- [ ] Unit tests for `hidden` and `gitignore` parity across discovery tools,
  including hidden ignore control files and bounded/unreadable ignore coverage.
- [ ] Unit tests for glob/regex/depth/kind/time selectors and rejection of invalid
  ranges/regex/globs/zero page sizes.
- [ ] Unit tests for compact escaping/rendering and result-budget fitting:
  canonical `/` separators, reusable quoted UTF-8 paths, invalid-UTF-8 Unix
  `path_encoding` incompleteness, exact active private-name exclusion even with
  `hidden=true`, indivisible discovery/grep units, and an actionable error (never
  a non-advancing cursor) when the first selected unit cannot fit.
- [ ] Read continuation tests for a single UTF-8 line larger than
  `max_read_bytes` and larger than the direct transport result budget, including
  CRLF preservation, EOF/empty-file behavior, UTF-8 scalar-boundary validation,
  and `byte == line-content-length` terminator handling.
- [ ] Read I/O fault tests proving useful prefixes are fault-marked/tool-errors
  and never presented as complete successful data.
- [ ] Write failure-injection tests before staging, during staging, before
  publication and after publication, asserting the reported commit state;
  separately prove write-only `write`/`bwrite` never read target contents for
  equality and staged replacement never broadens target mode/DACL protection.
- [ ] Final-symlink matrix tests: read/write/edit/binary/patch Update follow only
  after lexical + resolved authorization and preserve the link; dangling/create-
  only links fail closed; `cp`/`mv`/`rm` operate on the link object without
  touching its target.
- [ ] Binary tests for original-snapshot offsets, same-offset ordered inserts,
  insert-before-replace ordering, positive-range overlap rejection and SHA
  precondition revalidation.
- [ ] Batch mutation failure-injection tests proving mixed commit ledgers are
  exact and never hidden behind a generic error.
- [ ] Unit tests for globally correct sorting/paging under bounded traversal.
- [ ] Path-composability tests: every `find` result is directly reusable as a
  tool path; `ls` intentionally returns basenames.
- [ ] Status grammar tests for primary success/failure/mixed/unknown/skip
  outcomes plus orthogonal `more`/`incomplete` modifiers.
- [ ] Paging tests for one canonical cursor grammar, known total, unknown total,
  final page and `more + incomplete` combined state.
- [ ] Progress tests for fixed-total, unknown-total and phase-only updates,
  asserting cumulative/monotonic semantics.
- [ ] Shell `writes` schema/normalization tests: omitted vs empty, relative
  resolution, subtree scopes and no glob/shell expansion.
- [ ] Shell capability tests proving `bash`, `pwsh` and `powershell` are exposed
  independently only when their exact prepared interpreters are available, with
  no alias/fallback between PowerShell generations.
- [ ] Tests proving shell `writes` does not change authorization/sandboxing,
  inaccurate hints cannot grant access, and only Graph dependencies—not advisory
  hints—provide model-visible execution ordering.
- [ ] Shared Graph-parser tests proving `batch(ops)` and `job_start(ops)` accept
  the same valid graphs/reject the same invalid graphs and never permit nested
  `batch`, `job_start`, `job_*` or Task-control nodes.
- [ ] Graph execution tests for complete-success default sequencing, multiple
  roots, fan-out, joins, modifier/status branches, condition skips and
  backward-reference validation.
- [ ] Native Graph conflict tests proving overlapping read/write scopes are
  admitted in stable lower-coordinate order independent of runner timing, skip/
  failure does not become an implicit status dependency, uncertain scopes widen
  conservatively, and disjoint scopes still run in parallel.
- [ ] Assert paged/incomplete child results do not satisfy a bare dependency.
- [ ] Batch replay tests proving committed mutation nodes are not re-executed
  after response loss/redelivery.
- [ ] Graph budget tests proving node `more`/`incomplete` and downstream side
  effects are identical across outer transport/result budgets and completion
  order; an oversized aggregate must hand off the same Job rather than re-page a
  child.
- [ ] Graph parser tests for empty-op rejection, precedence (`&` before `|`),
  whitespace/grouping, bounded expressions and rejection of hidden extra syntax.
- [ ] Graph admission tests proving immutable definition/result/control quotas and
  replay-tombstone capacity fail before any root node is claimed, with unexpired
  correctness tombstones never evicted merely to admit new work.
- [ ] V2 shell-node graph tests for exit-0 success, non-zero error branch,
  timeout, launch failure, cancellation, joins and downstream skips.
- [ ] V2 durability/replay tests proving transport handoff does not mark a shell
  node terminal, the scheduler continues after handoff, terminal predecessors
  never rerun, and redelivery never starts a duplicate process.
- [ ] Process/session-boundary tests proving controlled shutdown/reload revokes
  the old Job session, replacement runtimes do not resurrect old Job IDs, and
  stale spool/session directories are cleaned rather than treated as live Jobs.
- [ ] Cross-process replay tests proving retired-session redelivery is either
  protocol-impossible or rejected from a minimal replay claim/tombstone without
  re-executing the Graph; race two processes on one request and prove exactly one
  can atomically create the durable claim, semantic-hash mismatch fails closed,
  and crash-after-claim-before-root never permits speculative reclamation.
- [ ] Security tests proving retained live-session Graph definitions/spools stay
  in protected private state, do not leak commands/stdin/args into routine logs,
  and are cleaned with Job/session retention.
- [ ] Shared durable-control tests for compact shape-derived `job_list`/`job_get`,
  idempotent `job_cancel`, cancel races, automatic resume, generic `job_restart`
  generation/history retention, full current schema/authority/resource-policy
  recompilation with newly captured node ceilings, and replay-safe restart
  delivery.
- [ ] Lost `job_start` response tests proving its durable ID is discoverable and
  the Graph is never created/executed twice.
- [ ] Abandoned optimistic-`batch` tests proving local transports expose the
  same Job when the last response attempt disappears while durable-request
  transports retain replay ownership instead of duplicating execution.
- [ ] Job capability tests proving native-only runtimes expose the shared
  `job_*` controls without shell authority.
- [ ] Global runner-admission tests proving many Graph Jobs cannot exceed the
  process-wide running-node/process budget.
- [ ] Native Task projection tests proving Tasks are used only when the exact
  eventual ordinary result fits; otherwise the same Job uses `job_*`.
- [ ] Typed-result tests proving `bread(format=mcp)` remains direct-only until a
  content-aware retained-result representation exists.
- [ ] `job_output`/`job_ack` tests for one-node implicit node 1, required
  multi-node selection, ShellNode stdout/stderr/result, NativeToolNode result,
  raw-byte offsets across UTF-8-lossy text/base64, monotonic/idempotent ack,
  acknowledgement cleanup and multi-owner Task/replay retention.
- [ ] Authority-revocation tests proving old `job_output` and native Task result
  payloads fail closed while compact settle/cancel control remains sufficient for
  safe shutdown and replacement policy never re-authorizes old payloads.
- [ ] Cancellation ambiguity tests proving an unprovable process-stop state is
  `unknown`, not fabricated `fail cancelled`.
- [ ] V2 output-spool tests proving large ShellNode output remains
  bounded/pageable under one durable Job owner and spool truncation never adds a
  Graph `incomplete` modifier or changes process-success dependencies.
- [ ] Edit tests for multiple independent substitutions, multiline tuples,
  `all`, ambiguity, duplicate/overlapping ranges, order-independence and one
  atomic publication.
- [ ] Strict-patch tests proving legacy best-effort behavior is absent from the
  advertised schema and shared missing-parent Add File sections publish as one
  deterministic namespace unit with exact mixed ledgers after commit faults.
- [ ] `cp` tests proving directories copy as complete bounded trees without a
  recursive flag; symlink payloads are copied verbatim/no-follow, special
  descendants and destination-inside-source fail closed, and hard-link topology
  is not accidentally promised.
- [ ] Grep tests for one symmetric `context` field, path/line deterministic order,
  one-hit-per-matching-line count semantics, matches beyond the old per-file cap,
  coalesced context and page-boundary clipping; cover compatibility-only old
  asymmetric fields if retained.
- [ ] Mutation race/alias/replay tests for `cp`, `mv`, `rm`, including hard-link
  aliases and case-only rename behavior where supported.
- [ ] `mv` tests proving missing destination parents fail without creating any
  namespace entries.
- [ ] `cp` failure-injection tests proving auto-created parent suffixes publish
  atomically with the destination or remain invisible.
- [ ] `rm(paths)` tests rejecting empty input, duplicates, aliases and
  ancestor/descendant overlaps before any deletion; recursive deletion must
  atomically detach each root before cleanup, hide only identity-owned exact
  private names, and never fall back to visible child-by-child removal. Inject
  crashes after cleanup-record reservation and after detach; restart recovery
  must safely retire/clean the exact object while an absent or identity-mismatched
  name is never hidden/deleted.
- [ ] Directory-tree copy / recursive-delete protected-path and symlink tests.
- [ ] Cross-device `mv` rejection tests.
- [ ] Cross-platform native Windows/macOS filesystem tests.
- [ ] `cargo fmt --all -- --check`.
- [ ] `cargo clippy --locked --all-targets --all-features -- -D warnings`.
- [ ] `cargo test --locked --all-features` with at least 8 build jobs.
- [ ] `nix flake check -L`.
- [ ] Markdown lint.
- [ ] Full CI matrix green.
