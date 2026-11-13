# CURRENT HANDOFF — dotlink companion for unified Jobs + Agent runtime

Prepared: 2026-11-14
Owner: @pvl
Machine: **L5 only**
Repo: `/home/pvl/spaces/abird/src/dotlink`

> The canonical comprehensive cross-repo handoff is
> `/home/pvl/spaces/abird/src/aw/.agents/docs/CURRENT_HANDOFF.md`.
> This companion records dotlink's current checkpoint and exact next boundary.

## Current checkpoint

- Branch: `main`.
- Pushed D1 implementation checkpoint:
  `3f59bb16b0262f6157351fd28b13167cc1b3e5db`
  (`Cut over standalone Jobs to shared AW runtime`).
- Exact reviewed AW shared pin:
  `9e9ab043410241c3f797f760d93a1cfd9126fdd2`.
- `aw-tools`, `aw-tools-fs-unix`, and `aw-tools-fs-win` all use that exact pin.
- D1 standalone cutover is complete and qualified.
- D2 dotlink remote Start consume/dispatch is the next implementation boundary.
- `docs/CHATGPT_PLUGIN.md` remains a pre-existing unrelated dirty modification;
  preserve it unless the user explicitly asks to include it.
- Existing second worktree:
  `/home/pvl/spaces/abird/src/dotlink-016-fixes` on `release/0.16`.
  Do not touch it and do not create another worktree.

## Required reading order

1. `AGENTS.md`
2. `.agents/plans/README.md`
3. `.agents/plans/TODO.md`
4. this file
5. AW canonical handoff:
   `/home/pvl/spaces/abird/src/aw/.agents/docs/CURRENT_HANDOFF.md`
6. dotlink unified plan README/TODO/NEXT-AGENT
7. AW unified `DESIGN.md` at normative design commit `890e9a...`
8. live source/tests/Git state

Source and executed tests outrank prose.

## D1 completed behavior

Standalone model surface:

- `jobs_list`
- `jobs_get`
- `jobs_read`
- `jobs_ack`
- `jobs_send`
- `jobs_cancel`

Constructors:

- shell + `background:true`
- `batch(..., background:true)`

Removed/unreachable:

- `job_start`
- `job_restart`
- old `job_*`

Standalone intentionally does not expose Agent, Agent history/filter,
`jobs_subscribe`, wake, or nested `batch`.

The final reviewed pin cutover uses shared AW primitives without importing AW
host semantics:

- shell flat names resolve through shared `OperationId`;
- `j:<handle>[/n:<coordinate>]` grammar uses shared `ExecutionRef` typed suffixes;
- no `aw-host`, AW Session/Policy/Agent runtime dependency is introduced.

Correctness retained:

- Process natural stdout;
- Graph root synthesized result and exact node reads;
- Graph-root aggregate non-ACKable; exact node ACK delivery-fenced;
- current-authority interactive stdin;
- same-Job sync/handoff/background replay;
- cancellation/unknown settlement;
- fresh runtime generation per startup;
- stale refs/cursors cannot resurrect;
- no Job ID reuse across generations;
- existing native launcher/sandbox/protected-path checks remain authoritative.

Final D1 repin gates on L5:

- `cargo fmt --all -- --check` — green;
- `cargo test --offline --locked --all-features` — green:
  637 main tests passed, one reviewed-fixture test intentionally ignored, and all
  integration suites passed;
- `cargo clippy --offline --locked --all-targets --all-features -- -D warnings`
  — green.

## AW side already available for D2

AW pushed code checkpoint:
`9e9ab043410241c3f797f760d93a1cfd9126fdd2`.

It provides the host-neutral remote Start value contract and the AW-side durable
authorization owner. AW already implements replay-before-reauthorization,
source/grant/site fencing, writer/Session participation rechecks, exact
operation/ref/request binding, target host/profile/live generation and bounded
target-clock lease.

Therefore do not add another upstream policy/grant system in dotlink.

## D2 exact next boundary

For every remote effect or Process stdin delivery:

1. validate the authenticated AW Start and exact dotlink host/profile/generation;
2. atomically claim its exact effect-attempt through the existing native
   replay/owner ledger before effect;
3. reject changed/duplicate/expired/old-generation proof as appropriate;
4. independently recheck current local launcher/sandbox/source/site/resource
   authority;
5. execute at most once and return the factual Process/Graph receipt.

Duplicate same-generation delivery replays the same factual/pending/unknown
receipt. A restart never turns an ambiguous old attempt into a fresh launch.
AW must not issue a replacement Start for that same original attempt.

AW offline blocks new authorization but does not invalidate an already committed
same-generation unexpired Start. Initial D2 rejects remote wake/subscription.

Keep Agent/Session/Policy/history entirely AW-owned. Standalone dotlink remains
finite/native only and its D1 surface must not change.

## Operating constraints

- L5 only.
- Canonical main checkout only.
- Preserve `docs/CHATGPT_PLUGIN.md`.
- Do not touch `dotlink-016-fixes`.
- No new worktree.
- No further commit/push/tag/release/deploy without explicit authorization.
- Keep a turn near ~30 minutes and pause at a coherent checkpoint.
- Do not requalify unchanged primitives repeatedly.

Next-agent prompt:
`.agents/plans/ai-native-unified-jobs-agent-runtime-2026-11-11/NEXT-AGENT.md`
