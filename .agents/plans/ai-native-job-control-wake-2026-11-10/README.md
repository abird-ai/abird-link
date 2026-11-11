# dotlink companion — AI-native operations / Jobs / Agents vNext

Author: Prasanna Loganathar
Co-authors: ChatGPT GPT-6, ChatGPT GPT-5.6 Sol
Owner: @pvl

Status: **DESIGN FINAL companion / implementation not yet authorized** (2026-11-10).
Base: dotlink main b31683c79a8ffd7c5540ac9388e607d8af6c7ffa; AW main 4a23e92a8f516971bb232b98e4fc000554bcc4ba.

The single canonical full design lives in AW:

- /home/pvl/spaces/abird/src/aw/.agents/plans/ai-native-job-control-wake-2026-11-10/README.md
- /home/pvl/spaces/abird/src/aw/.agents/plans/ai-native-job-control-wake-2026-11-10/DESIGN.md
- /home/pvl/spaces/abird/src/aw/.agents/plans/ai-native-job-control-wake-2026-11-10/TODO.md
- /home/pvl/spaces/abird/src/aw/.agents/plans/ai-native-job-control-wake-2026-11-10/NEXT-AGENT.md

This file is integration scope only, not a second semantics specification.

## Final vNext shape relevant to dotlink

~~~text
canonical IR   [operation,args]
Graph node     [operation,args,when?]

constructors   bash / pwsh / batch
AW-only Agent  agents.run when a real authenticated AW host supplies it

Jobs ops       jobs.list|get|output|ack|send|cancel|subscribe
wire fallback  jobs({op:"...", ...})

effects        Read | Write | Process | Control | Unknown
~~~

`background` is direct-call response disposition only; it never changes the Job
execution/replay identity and is absent from Graph nodes. `title` is bounded
non-authoritative metadata. Wake is a separate parent-owned subscription sidecar.

dotlink remains a thin adapter over exact-pinned shared aw-tools and platform
backends. It must not duplicate filesystem/Graph/Job semantics or invent an AW
Agent, Session writer, parent wake authority, durable workflow DB, event bus or
alternate Process supervisor.

## Adapter boundaries

- Keep existing Profile permissions, deny precedence, protected paths, Bubblewrap,
  Windows/macOS guards, tunnel replay and MCP Task ownership.
- Normalize grouped object-wire `jobs({op,...})` immediately to canonical semantic
  operations `jobs.*`; exact internal capability names are kernel-safe `jobs_*`.
  The public grouped name itself grants nothing.
- Preserve old `job_*` only at the intentional compatibility boundary during
  migration; do not make them permanent duplicate semantic grant identities.
- Control grants for cancel/ACK/subscribe do not imply filesystem Write, shell
  spawn, arbitrary stdin, network, or broad Job enumeration.
- `jobs.send` shell input additionally requires exact original Process/input and
  sandbox authority. `jobs.output` remains the outbound stdout/stderr/result path.
- Read-only filesystem profiles may independently grant safe owned Job Control;
  this must not weaken existing deny/write/process semantics.
- Completion events derive from the same JobManager settlement and writer
  quiescence. No second lifecycle/event store.
- Only expose wake when a real authenticated current AW parent Agent/owner/writer
  binding exists. Standalone dotlink remains passive.
- Only expose `agents.run` or Agent Graph nodes when dotlink is actually embedded
  behind/connected to an authenticated AW Agent host; never infer it from client
  strings.
- Typed `bread`/Media Graph results land in shared aw-tools first; dotlink supplies
  only transport/media disclosure projection.
- Preserve retained-output raw-byte ACK ownership, OpenAI replay, native MCP Task
  result claims, Job-session scope and unknown-stop semantics.
- Sealed DL-150 history remains unchanged.

## Integration sequence

1. Wait for explicit implementation authorization and AW shared implementation.
2. Consume reviewed shared AW crates through one exact Git revision.
3. Add the thinnest object-wire jobs adapter -> canonical `jobs.*` resolver.
4. Preserve old/new replay/Task/output/cancel parity before schema cutover.
5. Integrate factual Job settlement event projection; AW wake only with real parent.
6. Integrate typed Graph media through shared result representation.
7. Validate compact schema/token budget and existing product/platform gates.
8. Intentional cutover only with explicit owner approval.

No commit, push, version, tag, release or deployment is authorized by design
finalization.
