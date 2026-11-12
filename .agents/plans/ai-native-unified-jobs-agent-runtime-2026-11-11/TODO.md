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

- [ ] Consume exact reviewed shared `aw-tools`/ref/Job types and mechanical flat-name →
      semantic-ID mapping.
- [ ] Replace old standalone job_* routes with `jobs_list/get/read/ack/send/cancel`;
      shell/`batch background:true` replaces start and fresh constructors replace restart.
- [ ] Preserve Process natural stdout, Graph natural result/node coordinate reads, retention/ACK,
      same-Job handoff, stdin current-authority checks, cancellation/unknown settlement,
      replay/MCP Task ownership and protected-path/sandbox invariants.
- [ ] Keep standalone/model Graph on reviewed DL-150 native/shell leaves. AW-hosted model
      Graphs additionally allow `agent` with original Agent/Turn/child-effect owner; dotlink
      standalone never exposes it. **Nested `batch` is rejected on model/SDK/host Graphs**.
      Qualified typed SDK/host `jobs.*` leaves do not expand the standalone provider schema.

### Dotlink checkpoint 1 — standalone
- [ ] Run together: flat catalog, shell sync/handoff/background, Graph whole/node reads,
      ACK ownership, authorized→revoked stdin, cancel/unknown, duplicate control replay,
      protocol-clean stdio, and representative protected-path/sandbox denial.

## D2 — authenticated AW bridge

- [ ] Accept only AW-resolved canonical refs/operations and authenticated original replay ID,
      actor/source/site/resource/grant/membership/writer provenance. AW Policy/Session/team
      semantics remain opaque upstream facts.
- [ ] Define one **per-attempt AW source-fenced Start authorization**: existing AW authority
      owner atomically rechecks current epochs/grants and records Start against the original
      invocation, serialized with revocation. dotlink checks a fresh exact single-use proof and
      current local launcher/sandbox/source/site/resource authority before effects. Old
      upstream envelopes cannot substitute for Start; AW offline/mismatched/unknown attempt
      blocks new work; already committed/unknown effects remain truthful.
- [ ] Preserve WriterFence requirements for durable/multi-writer remote ownership.
- [ ] Keep Agent jobs/read/filter/history/parent state owned by AW. Standalone dotlink never stores it.
- [ ] Reject constructor wake and later subscribe on dotlink-owned live Process/Graph sources until
      a separately reviewed atomic source-owner event protocol exists.

### Dotlink checkpoint 2 — bridge
- [ ] Run together: AW→remote Process/Graph execution/control/read, stale writer/source/site denial,
      missing local sandbox grant despite upstream allow, revoked stdin before bytes, duplicate
      request idempotence, no Agent state leakage, unsupported remote wake rejection. Race
      AW revoke against remote Start: revoke-first denies; Start-first is one accepted/unknown
      attempt; offline AW/stale proof cannot authorize another effect. No double launch.

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
thin, current-authority-intersecting and source-fenced at remote Start, no old route is reachable,
no Agent/Session/Policy engine exists in dotlink, and all applicable grouped checkpoints pass.

## Execution guard

Implementation sequencing only. Preserve unrelated work. No push/version/tag/release/deploy
without separate explicit authorization.
