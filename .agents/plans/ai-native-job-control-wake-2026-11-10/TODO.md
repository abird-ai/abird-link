# dotlink vNext operations / Job Control / wake integration — TODO

Author: Prasanna Loganathar
Co-authors: ChatGPT GPT-6, ChatGPT GPT-5.6 Sol
Owner: @pvl

Base: b31683c79a8ffd7c5540ac9388e607d8af6c7ffa
Status: **design finalized; implementation not yet authorized**.
Canonical semantics/TODO live in the AW companion plan.

## Committed

None.

## Uncommitted

- [x] Finalize dotlink integration vocabulary: canonical `[operation,args]`,
  grouped jobs object-wire `op` fallback, semantic `jobs.*` / kernel-safe
  `jobs_*`, no permanent duplicate job_* identities.
- [x] Define background/title/wake as non-Graph execution side semantics:
  background=response disposition, title=metadata, wake=parent subscription.
- [x] Define jobs.send inbound vs jobs.output outbound and exact extra shell
  Process/input authority.
- [x] Define standalone dotlink as passive: no invented AW Agent/wake/agents.run.
- [ ] At implementation start audit current profile/runtime/session/capability
  compatibility for Control and read-only Job-management grants.
- [ ] Integrate only a reviewed exact AW Git pin; no sibling/vendored shared fork.
- [ ] Add strictly authorized grouped jobs model/MCP projection and normalize
  immediately into the shared canonical operation resolver.
- [ ] Preserve legacy job_* compatibility, OpenAI replay, MCP Task status/result
  ownership, output EOF/writer quiescence and raw-byte ACK during migration.
- [ ] Integrate shared factual shell Job settlement event projection. Reject/hide
  wake when no verified AW parent binding exists.
- [ ] Support typed Graph bread/media only through shared aw-tools and dotlink
  media-disclosure/transport policy.
- [ ] Validate read-only native Graph + explicit Control without write/process
  widening; validate interactive stdin only where actual Process custody supports it.
- [ ] Run current product tests/lint/compile and available native platform gates;
  leave unavailable macOS/Windows/Bubblewrap runtime checks explicitly pending.
- [ ] Review grouped schema bytes/token cost and representative model calls
  against legacy job_* advertising.
- [ ] Preserve sealed DL-150, global plan index and unrelated dirty work.
- [ ] Separate explicit authorization required for commit/push/version/tag/release.

## Tests and verification

- [ ] canonical tuple <-> jobs({op,...}) equivalence.
- [ ] exact jobs_* op/effect/source/owner/site grants.
- [ ] ReadOnly FS + permitted Job Control without shell/file Write.
- [ ] background/foreground same-execution handoff/replay identity.
- [ ] shell jobs.send authority + jobs.output stdout/stderr/result directionality.
- [ ] fast completion/writer-quiescence/wake dedup; standalone wake unavailable.
- [ ] wait_ms status/output race/EOF semantics.
- [ ] typed Graph media/raw-byte ACK/replay owner preservation.
- [ ] existing dotlink Job/Graph/MCP/OpenAI replay/stdio suites.
- [ ] supported native Windows/macOS/Bubblewrap checks or explicit pending status.
- [ ] no duplicate event/Job/Graph/Process runtime.
