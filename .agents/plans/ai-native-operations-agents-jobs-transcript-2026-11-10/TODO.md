# dotlink vNext operations / Jobs / Agent transcript integration — TODO

Author: Prasanna Loganathar
Co-authors: ChatGPT GPT-6, ChatGPT GPT-5.6 Sol
Owner: @pvl

Status: **design reviewed/resolved; implementation not authorized**.
Canonical semantics: AW `.agents/plans/ai-native-operations-agents-jobs-transcript-2026-11-10/DESIGN.md`. Dotlink companion `README.md` is integration-only. Existing DL-150 and older plans remain unmodified.

## Committed

None. No implementation or test was run for this design update.

## Design decisions

- [x] Keep original DL-150 small native leaf schemas and immutable Graph node/`when` grammar.
- [x] Canonical tuple `[operation,args]` is internal/SDK/Graph, not a second model tool. Provider sees filtered `jobs({op,...})`, never duplicate aliases.
- [x] Original provider call is immutable replay/journal provenance; separate trusted resolved `jobs.*` op.
- [x] Batch outer Control admission never grants nested Write/Process; preflight every conditional node.
- [x] Standalone dotlink owns Jobs only, not durable AW Agents/Session transcript or parent wake.
- [x] ReadOnly FS with independently permitted Job Control remains supported.
- [x] Distinct `bash`, `pwsh`, `powershell` launcher identities and original sandbox permissions preserved.
- [x] `jobs.output` is retained raw execution output and original ACK owner; `agents.read` is an AW-only current-authorized durable history projection.
- [x] AW `agents.read.filter` replaces `include`/top-level `tools`: `default`, `none` (all currently authorized sanitized fields), or custom category booleans/tool-name restriction with optional `exclude:true` to subtract matching records; no dotlink transcript filter authority.
- [x] Legacy `job_*` including replay-protected retained-Graph restart supported through explicit compatibility; no default vNext `jobs.restart`.
- [x] Graph `bread` typed media, captured node result ceilings and immutable dependency outcomes come from shared AW tools.
- [x] Actual schema/token/workflow measurements are mandatory for model-first vNext cutover.

## Implementation

- [ ] Obtain separate implementation approval, inspect live commits and existing unrelated dirty work; preserve original plans.
- [ ] Consume only a reviewed exact AW shared Git pin; no duplicate Graph/Job/native filesystem source.
- [ ] Implement filtered grouped `jobs` model-wire projection, exact per-op resolver, current epoch/site/actor grants and original ToolCall journal evidence.
- [ ] Support new Effect::Control without widening old ReadOnly/Profile permissions or bypassing authority in native-only Graphs.
- [ ] Implement bounded `get` status and `output` bytes/EOF `wait_ms` with no global lock held, replay or output ACK on read.
- [ ] Preserve monotonic output-owner ACK, native MCP Task and OpenAI tunnel replay even under changed invocation metadata/disposition.
- [ ] Maintain separate actual `pwsh` and `powershell` interpreter capability and correct original shell stdin authority.
- [ ] Project typed bread/media Graph results from shared aw-tools; outer transport result fitting must never modify node `more`/`incomplete` or `when`.
- [ ] Preserve original `job_restart` host/legacy operation until intentional migration; advanced vNext projection only if approved/needed.
- [ ] Hide/reject parent wake/Agent run/read in standalone dotlink; any AW bridge retains AW Agent/Session/lineage/transcript authorization.
- [ ] If a real AW bridge transports `agents.read`, preserve the exact `filter` value, including `exclude:true`, and current permission/byte-paging semantics end to end; dotlink never interprets transcript filtering, selects fields or creates disclosure grants.
- [ ] Preserve Profile deny, protected paths, sandbox/mount/native platform semantics and current runtime Job session boundary.
- [ ] Test existing product/transport/Task/Graph suites and meaningful native platform gates; keep unavailable native smoke explicitly pending.
- [ ] Measure actually generated schema bytes/tokenizer estimate and representative first-valid-tool/round/result tokens against DL-150 before cutover.
- [ ] Do not commit, push, version, tag, release, deploy or edit old plans/global indexes without separate authorization.

## Verification

- [ ] Per-op grouped schema vs canonical internal tuple, exact source/effect/epoch grants and immutable provider ToolCall.
- [ ] ReadOnly FS plus granted Graph Control/all-Read leaves; conditional mutation denied pre-start.
- [ ] Distinct Shell interpreters, input authority, timeouts and Job status/output/wait/EOF.
- [ ] Typed bread/media preserves captured node semantic modifiers and raw-byte output ACK/Task/replay owners.
- [ ] No standalone Agent/transcript/wake authority; authenticated bridge keeps actual AW disclosure/safe wake owner.
- [ ] Current OpenAI/stdio/MCP/Graph/Job and available native platform suite results.
- [ ] Real model-schema and workflow token/selection baseline recorded and acceptable.
- [ ] No earlier plan, implementation source, coordinator queue or unrelated working-tree content modified by this draft.
