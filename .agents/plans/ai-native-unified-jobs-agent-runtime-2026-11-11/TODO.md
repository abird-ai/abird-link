# Unified Jobs + Agent runtime — dotlink integration TODO

Full semantic authority: AW `ai-native-unified-jobs-agent-runtime-2026-11-11/DESIGN.md`
AW implementation order: companion AW `TODO.md`

This file contains only dotlink-owned integration work. Do not duplicate AW Policy,
Session, Agent, guardian or routing semantics here.

## Integration discipline

- Start after AW has a reviewed implementation revision through Checkpoint E.
- Pin/consume one exact shared AW revision; no sibling path/copy.
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
- [ ] Keep standalone/model Graph on the conservative DL-150 leaf surface; advanced host/SDK
      leaves do not expand provider schema.

### Dotlink checkpoint 1 — standalone
- [ ] Run together: flat catalog, shell sync/handoff/background, Graph whole/node reads,
      ACK ownership, authorized→revoked stdin, cancel/unknown, duplicate control replay,
      protocol-clean stdio, and representative protected-path/sandbox denial.

## D2 — authenticated AW bridge

- [ ] Accept only AW-resolved canonical refs/operations and bounded upstream actor/source/site/
      epoch/grant/writer evidence. AW Policy/Session/team semantics remain opaque upstream facts.
- [ ] Require both upstream AW authority and current local dotlink launcher/sandbox/source/site/
      resource authority for every native effect.
- [ ] Preserve WriterFence requirements for durable/multi-writer remote ownership.
- [ ] Keep Agent jobs/read/filter/history/parent state owned by AW. Standalone dotlink never stores it.
- [ ] Reject constructor wake and later subscribe on dotlink-owned live Process/Graph sources until
      a separately reviewed atomic source-owner event protocol exists.

### Dotlink checkpoint 2 — bridge
- [ ] Run together: AW→remote Process/Graph execution/control/read, stale writer/source/site denial,
      missing local sandbox grant despite upstream allow, revoked stdin before bytes, duplicate
      request idempotence, no Agent state leakage, unsupported remote wake rejection.

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
thin and authority-intersecting, no old route is reachable, no Agent/Session/Policy engine exists
in dotlink, and all applicable grouped checkpoints pass.

## Execution guard

Implementation sequencing only. Preserve unrelated work. No push/version/tag/release/deploy
without separate explicit authorization.
