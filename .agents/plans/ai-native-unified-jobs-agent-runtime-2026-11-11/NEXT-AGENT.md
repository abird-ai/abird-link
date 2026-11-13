# Next agent — unified Jobs + Agent runtime, Phase 6 D2

Use `dotlink---l5` only.

Work only in:

- AW: `/home/pvl/spaces/abird/src/aw`
- dotlink: `/home/pvl/spaces/abird/src/dotlink`

Do not use A1. Do not create another worktree. Never inspect or touch AW
`.agents/plans/ux/**`. Preserve unrelated work, especially dotlink
`docs/CHATGPT_PLUGIN.md` and the separate `dotlink-016-fixes` release worktree.
Do not commit, push, tag, release, or deploy unless the user separately authorizes
that future checkpoint.

## Authority and pushed checkpoints

Normative design authority remains AW:

`890e9a383fe2edc3e0bd0aa34a2b217f2724475c`

Reviewed/pushed implementation checkpoints:

- AW code: `9e9ab043410241c3f797f760d93a1cfd9126fdd2`
  (`Implement unified Jobs and Agent runtime checkpoint`)
- dotlink D1: `3f59bb16b0262f6157351fd28b13167cc1b3e5db`
  (`Cut over standalone Jobs to shared AW runtime`)
- dotlink pins `aw-tools`, `aw-tools-fs-unix`, and `aw-tools-fs-win` exactly to
  AW `9e9ab043410241c3f797f760d93a1cfd9126fdd2`.

Read in order:

1. both repos' `AGENTS.md`, `.agents/plans/README.md`, `.agents/plans/TODO.md`;
2. AW `.agents/docs/CURRENT_HANDOFF.md` in full;
3. AW unified plan README, DESIGN.md, TODO.md;
4. dotlink companion README, TODO.md, CURRENT_HANDOFF.md;
5. live source/tests/current Git status. Source and executed tests outrank prose.

## What is complete

AW Phases 1–5 implementation is committed and pushed. Its shared layer exports
`OperationId`, typed `ExecutionRef`, coordination/event primitives and the
host-neutral remote-Start contract.

AW-side D2 authorization is implemented:

- one durable source-fenced Start authorization per original effect attempt;
- replay lookup occurs before current reauthorization;
- exact operation/ref/args/request/target-generation binding;
- actor/source/site/resource/grant/policy/Session/writer/membership provenance;
- bounded target-clock lease;
- ToolBinding read fence serializes Start commit against source/grant/site
  revocation;
- revoke-before-Start denies;
- Start-before-revoke preserves exactly the committed attempt;
- changed intent under the same attempt cannot mint a replacement Start.

dotlink D1 is complete and pushed:

- exact reviewed shared AW pin;
- shared `OperationId` and `ExecutionRef` consumed mechanically;
- model surface is flat `jobs_list/get/read/ack/send/cancel`;
- shell and `batch(..., background:true)` are constructors;
- no `job_start`, `job_restart`, old `job_*`, Agent, subscribe/wake or nested
  batch;
- standalone refs remain only `j:<handle>[/n:<coordinate>]`;
- fresh live runtime generation per startup; stale refs/cursors cannot resurrect;
- same-Job handoff/replay, ACK ownership, stdin current-authority checks,
  cancel/unknown and sandbox/protected-path behavior are preserved.

Latest dotlink qualification against AW `9e9ab04`:

- `cargo fmt --all -- --check` green;
- `cargo test --offline --locked --all-features` green:
  main suite 637 passed / 1 intentionally ignored, all integration suites green;
- `cargo clippy --offline --locked --all-targets --all-features -- -D warnings`
  green.

## Exact next boundary: D2 remote consume/dispatch

Do not redo D1 or the shared pin.

Implement the dotlink half of the authenticated AW→dotlink Start bridge.

For each original remote effect or Process stdin delivery, dotlink must receive
one AW-committed authorization bound to:

- AW origin + original authenticated invocation/replay identity;
- exact effect-attempt ID;
- canonical `OperationId` / resolved finite ref;
- exact arguments/request digest;
- target dotlink host/profile;
- authenticated dotlink live runtime generation;
- actor/source/site/resource/grant/policy/writer/membership provenance;
- bounded target-issued lease.

Required ordering:

1. authenticate/validate the Start envelope and exact target generation;
2. claim the exact attempt through dotlink's existing replay/owner ledger before
   any external effect — do not add a second replay database;
3. independently recheck current dotlink launcher/sandbox/source/site/resource
   authority;
4. execute at most once;
5. return/replay the same factual Process/Graph receipt or pending/unknown state.

Same-generation exact duplicate delivery returns the same receipt and never a
second effect. Expired/old-generation proofs never launch. After restart,
ambiguous old effects stay unknown; AW must not issue a replacement Start for
the same original attempt. A deliberate fresh invocation is new intent.

AW offline before authorization means no new Start. An already committed,
same-generation, unexpired Start may finish after AW disconnects.

Initial D2 must reject wake/subscription for dotlink-owned remote Process/Graph
sources. Agent/Session/Policy/history remain entirely AW-owned. Standalone
dotlink remains finite/native only.

The existing `JobManager::runtime_generation_fingerprint()` is intentionally
retained for D2. Reuse current request-key/replay ownership, shell claim,
generation fencing and local authority paths rather than building parallel
machinery.

## Validation/checkpoint

Use targeted tests while implementing. Once the vertical D2 slice is coherent,
run dotlink Checkpoint 2 / AW Checkpoint F as one grouped matrix, covering at
least:

- remote Process and Graph Start/read/control;
- exact duplicate Start and lost reply;
- source/site/writer/membership revocation ordering;
- proof expiry and live-generation restart;
- ambiguous original effect/no replacement launch;
- local launcher/sandbox denial;
- Process stdin per-attempt Start;
- explicit remote wake/subscribe rejection;
- standalone D1 surface unchanged.

Record concise evidence in the live TODOs and pause before D3. Keep one model
turn near ~30 minutes and checkpoint at the next coherent boundary.
