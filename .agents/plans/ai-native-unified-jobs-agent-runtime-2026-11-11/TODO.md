# Unified Jobs + Agent runtime — dotlink integration TODO

Full semantic authority: AW `ai-native-unified-jobs-agent-runtime-2026-11-11/DESIGN.md`
AW implementation order: companion AW `TODO.md`

This file contains only dotlink-owned integration work. Do not duplicate AW Policy,
Session, Agent, guardian or routing semantics here.

## Integration discipline

- **D1 standalone finite Jobs** may start after AW **Checkpoints A/B** expose reviewed shared
  ref/Job contracts. It does not depend on AW Session sharing or delegated Agents.
- **D2 AW-authenticated bridge** waits for AW **Checkpoint E** and policy/Start admission
  contract. D3 follows D2; AW final G validates the product together.
- Pin/consume one exact reviewed shared AW revision at each integration boundary; no sibling path/copy.
- Develop with targeted checks. Run the grouped dotlink checkpoint once per vertical slice,
  not after every edit.
- Carry forward unchanged sandbox/native/replay results; re-run them only when their owner or
  dependency changed, or at the final capable-host gate.
- Preserve standalone/local hot paths: no AW Session/Policy/provider dependency.

## D1 — shared revision + standalone jobs cutover

- [x] Consume exact reviewed shared `aw-tools`/ref/Job types and mechanical flat-name →
      semantic-ID mapping.
      - 2026-11-13: pinned `aw-tools`, `aw-tools-fs-unix` and `aw-tools-fs-win` to reviewed AW
        implementation checkpoint `9e9ab043410241c3f797f760d93a1cfd9126fdd2`. Standalone
        `j:<handle>[/n:<coordinate>]` parsing now uses shared `ExecutionRef`/typed suffixes and
        shell-name resolution uses shared `OperationId`; no AW Agent/Session/Policy host layer is
        imported.
- [x] Replace old standalone job_* routes with `jobs_list/get/read/ack/send/cancel`;
      shell/`batch background:true` replaces start and fresh constructors replace restart.
- [x] Preserve Process natural stdout, Graph natural result/node coordinate reads, retention/ACK,
      same-Job handoff, stdin current-authority checks, cancellation/unknown settlement,
      replay/MCP Task ownership and protected-path/sandbox invariants. Every startup
      generates a fresh authenticated runtime generation: old finite `j:` refs are never
      reused, old cursors/proofs reject, and live Process/Graph Jobs cannot resurrect.
- [x] Keep standalone/model Graph on reviewed DL-150 native/shell leaves. AW-hosted model
      Graphs additionally allow `agent` with original Agent/Turn/child-effect owner; dotlink
      standalone never exposes it. **Nested `batch` is rejected on model/SDK/host Graphs**.
      Qualified typed SDK/host `jobs.*` leaves do not expand the standalone provider schema.

### Dotlink checkpoint 1 — standalone
- [x] Run together: flat catalog, shell sync/handoff/background, Graph whole/node reads,
      ACK ownership, authorized→revoked stdin, cancel/unknown, duplicate control replay,
      **runtime restart rejects stale Job refs/cursors without ID reuse or resurrection**,
      protocol-clean stdio, and representative protected-path/sandbox denial.
      - 2026-11-13: focused changed-path gates plus owner-sensitive MCP Task/replay/stdio/
        protected-path carry-forward are green on L5. Graph root `result` is synthesized and
        non-ACKable; exact node ACK is fenced to the highest offset actually returned by
        `jobs_read`.
      - 2026-11-14 repin qualification against AW `9e9ab043410241c3f797f760d93a1cfd9126fdd2`:
        `cargo fmt --all -- --check`, locked all-feature tests and strict all-target Clippy are
        green; the main binary suite is 637 passed / 1 intentionally ignored and all integration
        suites, including canonical stdio Jobs flow, pass.

## D2 — authenticated AW bridge

- [ ] Accept only AW-resolved canonical refs/operations and authenticated original replay ID,
      actor/source/site/resource/grant/membership/writer provenance. AW Policy/Session/team
      semantics remain opaque upstream facts.
- [ ] Define one **per-attempt AW source-fenced Start authorization**: AW atomically
      rechecks current grants/epochs and records exact original invocation, effect-attempt ID,
      operation/ref/args digest, target dotlink host/profile **and live runtime generation**,
      bounded expiry and source/site/writer fence, serialized with revocation. dotlink
      consumes it once through the **existing** replay owner and rechecks current local
      launcher/sandbox/source/site/resource authority. Proof expiry must not outlive
      dedupe retention; no old-generation proof can restart an ambiguous effect, and AW
      may not issue a replacement Start for that same unknown attempt. AW offline blocks
      **new** authorization, not an already committed same-generation dispatch.
- [ ] Preserve WriterFence requirements for durable/multi-writer remote ownership.
- [ ] Keep Agent jobs/read/filter/history/parent state owned by AW. Standalone dotlink never stores it.
- [ ] Reject constructor wake and later subscribe on dotlink-owned live Process/Graph sources until
      a separately reviewed atomic source-owner event protocol exists.

### Dotlink checkpoint 2 — bridge
- [ ] Run together: AW→remote Process/Graph execution/control/read, stale writer/source/site denial,
      missing local sandbox grant despite upstream allow, revoked stdin before bytes, duplicate
      request idempotence, no Agent state leakage, unsupported remote wake rejection. Race
      AW revoke against remote Start: revoke-first denies; Start-first permits at most one
      accepted/unknown attempt. Test duplicate proof/response loss, expiry versus dedupe
      retention, target generation restart, offline **before new authorization** versus offline
      **after accepted Start**. Old proof never executes in a new generation; no double launch.

## D3 — cutover cleanup + final integration gate

- [ ] Delete old job_start/job_restart/job_* compatibility decoders/adapters/tests and dead bridge
      helpers; no migration/legacy replay service.
- [ ] Update dotlink live docs/examples to the implemented standalone/bridge surface.
- [ ] Record final standalone schema bytes/tokens and confirm advanced shared host capabilities did
      not grow the model Graph/tool surface.

### Dotlink checkpoint 3 — final
- [ ] Run only the affected dotlink package/integration lanes plus:
  - standalone Job flows from checkpoint 1;
  - bridge authority/replay flows from checkpoint 2;
  - MCP Task/OpenAI replay/retention lanes if their owner changed;
  - native Bubblewrap/Windows/macOS gates only when touched or on a capable final host.
- [ ] Do not requalify unchanged native filesystem/sandbox primitives merely because old routes were removed.

## Completion

dotlink integration is complete when the standalone surface is canonical, the AW bridge remains
thin, current-authority-intersecting, runtime-generation-fenced and replay-safe at remote Start,
no old route is reachable, no Agent/Session/Policy engine exists in dotlink, and all
applicable grouped checkpoints pass.

## Execution guard

Implementation sequencing only. Preserve unrelated work. No push/version/tag/release/deploy
without separate explicit authorization.
