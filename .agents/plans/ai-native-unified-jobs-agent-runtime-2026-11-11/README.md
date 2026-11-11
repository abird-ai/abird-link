# Unified Jobs + Agent runtime — dotlink integration

Author: Prasanna Loganathar
Co-authors: ChatGPT GPT-5.6 Sol
Owner: @pvl

Status: **FINAL INTEGRATION DESIGN SEALED** (2026-11-12); implementation not performed by this design pass.

Full semantic design lives in AW:

    /home/pvl/spaces/abird/src/aw/.agents/plans/
    ai-native-unified-jobs-agent-runtime-2026-11-11/DESIGN.md

This companion defines only dotlink's integration boundary. It does not rewrite sealed DL-150 or the two 2026-11-10 successor plans.

## New standalone dotlink model surface

Keep flat native filesystem and execution constructors:

    ls find grep read write edit patch cp mv rm bread bwrite bedit
    bash pwsh powershell batch

Standalone unified Job controls:

    jobs_list
    jobs_get
    jobs_read
    jobs_ack
    jobs_send
    jobs_cancel

Standalone dotlink does not advertise agent or jobs_subscribe because durable Agent ownership and wake recipients require a real AW Session/branch/Actor host. It also omits constructor wake fields.

When dotlink is used through an authenticated AW bridge, AW may additionally expose:

    agent
    jobs_subscribe

and wake-capable constructor fields, then route Agent refs through the same jobs_* control surface. AW owns contextual/canonical namespace resolution and wake-recipient authority.

## Canonical semantic IDs

Wire projection maps immediately to:

    read        -> fs.read
    write       -> fs.write
    ...
    jobs_get    -> jobs.get
    jobs_read   -> jobs.read

Model-facing Graph reuses the same flat leaf names as direct tools:

    ["read", {...}]
    ["bash", {...}]
    ["agent", {...}]       # AW-host only

Graph admission maps the flat name through the same below-wire semantic resolver (read -> fs.read, etc.) before policy/preparation/journaling.

The original provider wire ToolCall remains immutable evidence.

## Job ownership boundary

dotlink standalone owns/exposes:
- Process Jobs from bash/pwsh/powershell;
- Graph Jobs from batch;
- Graph embedded deterministic n:<1-based Graph coordinate> node streams/results beneath Graph Job refs, with no extra opaque node mapping;
- finite Job retention/ACK;
- MCP Task correlation;
- transport replay;
- process/sandbox custody.

Control/read refs use the shared AW semantics. Standalone has no implicit self, so get/read/ack/send/cancel require explicit finite ref; jobs_list has no ref.

rev is exposed only where it adds control value:
- send/cancel use supplied rev as exact compare-and-act;
- get uses rev as the conditional observation/change token for lost-wakeup-safe wait_ms polling;
- read and ack have no rev: exact ref + offset/cursor/ACK watermark preserve replayable DL-150 paging and idempotent release.
Calls that permit omitted rev remain valid against current state.

Standalone dotlink does not invent domain/principal/project/Agent identity. Its finite refs resolve inside its **authenticated live runtime session** as j:<handle> or j:<handle>/n:<1-based coordinate>. Because it has no current Agent, get/read/ack/send/cancel require an explicit finite ref; no omitted/"." AW self or structural default d:0/i:0/p:0/a:0 can mint dotlink authority. Under an AW bridge those :0 values are fixed AW canonical objects, never caller-relative aliases, and only AW resolves/authorizes them. jobs.subscribe exists only with a real authenticated AW Agent host.

When an authenticated AW bridge is present:
- AW owns fully resolved canonical /d:<h>/i:<h>/p:<h>/a:<h>[/t:<h>][/j:<h>...][/n:<h>...] identity;
- canonical handles are already direct immutable identities; no friendly-name alias lookup layer is part of the execution ref;
- AW owns contextual overlay resolution for d:/i:/p:/a:, omitted/".", "^" and t:/j:/n: paths relative to the current Agent;
- AW applies inherit-ancestors/reset-descendants-to-:0 **only to existing provisioned authenticated identities**; missing default principals/projects/root Agents fail closed and are never created by reference lookup;
- AW canonicalizes the request before routing a Process/Graph suffix into dotlink when dotlink owns that execution;
- dotlink never interprets "^", structural defaults, parent/coordinator state, Agent origin or Agent Session semantics.

dotlink does not implement AW's host ExecutionStrategy/OperationRouter or transparent DelegateAgent routing. Standalone Process/Graph operations execute through their normal local owners; under an AW bridge, AW chooses any Local/DelegateAgent strategy before delegating only the resulting concrete Process/Graph work to dotlink.

dotlink does not own/copy:
- AW principal/project/Agent namespace state;
- AW Session/branch;
- Agent Turn generation;
- Agent Step/model journal;
- Agent transcript/history;
- Agent parent/coordinator or wake recipient authority.

The dotlink registry/router may carry only exact resolved routing identity needed to reach an owner, never a shadow Agent state store.

The generic wire/API must not grow node/turn/step fields. Shared handles use the same delimiter-safe canonical lexical grammar as AW. Execution suffixes are owner-typed; Graph nodes use directly derivable n:<1-based coordinate> segments (n:0/leading zeros invalid), and any n:<coordinate>/j:<handle> continuation exists only for genuinely independent durable child work. Agent t:<handle>/Step semantics remain AW-owned, and Step has no public path segment.

## jobs.read

Replace old model-facing job_output with jobs_read.

Standalone valid views:

Process:
- stdout
- stderr
- result

Graph:
- result
- valid n:<1-based coordinate> result/stdout/stderr

Preserve exact DL-150 retained-byte semantics: Process/Process-backed Graph-node default read is **stdout**; native/Graph default is result. Raw offsets are encoding-independent; EOF/truncation remains immutable; read never ACKs. ACK is monotonic/idempotent exact ref/view/offset with no rev, cannot exceed retained bytes delivered/available to that exact owner's lease, and cleanup waits for every replay/Task/protocol owner.

jobs.list is flat deterministic exposed-Job-root discovery with immutable exposure ordering and an opaque strictly-older cursor; later activity never reorders pages and it is not subtree traversal. jobs.get/read/ack/send/cancel use explicit standalone refs. Process Job refs are j:<handle>; Graph nodes are directly derivable j:<handle>/n:<1-based coordinate>.

Shared schemas stay strongly typed: ReadView=result|stdout|stderr for standalone resolved kinds; SendInput is exactly {text}|{eof:true}. jobs.send mode is auto|queue|steer|interrupt. Process j:<handle> accepts live input only when interactive custody was retained; Graph interactive n:<handle> delegates to the underlying node owner. Every effectful SDK/transport/bridge send carries one **stable host-envelope invocation ID** created before admission and reused on retry; an uncertain stdin effect is never blindly written twice. Dotlink standalone does not interpret AW causal budgets, subscriptions or Agent transcript filters; AW owns them. Unsupported inputs/modes are rejected rather than reinterpreted.

Public finite lifecycle uses only the shared vocabulary subset running|waiting|cancelling|cancelled|settled, with DL-150 outcomes reported separately.

jobs.read never acknowledges.

jobs.ack remains the only explicit retained-byte release watermark operation.

Agent views result/messages/tool_calls/tool_outputs/trace and the optional combined **Agent-only `filter`** are implemented/authorized by AW only. AW supports omitted/`"default"` result, `"none"` all currently authorized public categories, or custom booleans/tools with `exclude:true`; `filter` and `view` cannot be combined. Standalone dotlink does **not advertise** Agent filtering. If authenticated AW bridge transports such a call, it forwards the typed request/filter/cursor/result unchanged after AW's own authority check; dotlink never stores/reconstructs Agent transcript or implements its own disclosure evaluator.

## No compatibility surface

The new **advertised model API** does not preserve:

    job_start
    job_list
    job_get
    job_output
    job_ack
    job_cancel
    job_restart

or grouped jobs({op}) / agents({op}) proposals.

No model-facing restart exists.

DL-150 Job functionality is simplified rather than weakened:
- job_start(ops) -> batch({ops,background:true});
- shell-shaped job_start -> original shell constructor with background:true;
- job_output -> jobs.read;
- job_restart -> no equivalent retained-definition capability; a re-run is a fresh explicit replay-protected constructor under current schema/authority.

Retained Job definitions remain evidence only; a re-run is a fresh explicit constructor, never a retained-Graph execution capability. This remains a **hard pre-1.0 source/API cutover** with obsolete decoders/routes/state adapters removed, no migration/drain/dual-schema layer. However, resetting old transient Job/replay/Task state is allowed **only for an explicitly disposable isolated dev/test runtime after verified quiescence**: ingress closed, original runtime/supervised children stopped, and no live or ambiguously settled external effects, clients, paid attempts, Task/replay responses or output/ACK owner obligations. If this cannot be proven, abort that reset and require operator resolution or a separately isolated new deployment. Never erase unrelated durable Agent/Session data or reinterpret an old request under a new operation.

For the **new** API, finite constructors still reserve replay/exposure recovery capacity before the first effect and preserve DL-150 lost-receipt behavior: ambiguous synchronous delivery stays attached to its current replay owner when available; if no such owner remains within the same running implementation, expose the same execution for jobs.list recovery rather than discarding or rerunning it. Background/automatic handoff returns compact ref + state + rev so the caller can immediately conditional-poll jobs_get without an extra discovery round. Keep new-schema startup/replay/private-artifact/MCP Task/ACK correctness; removing job_restart does not weaken current recovery invariants.

## Security invariants retained

Do not regress:
- protected path/deny precedence;
- descriptor-rooted native filesystem authority;
- Bubblewrap guard semantics;
- explicit shell network/hardware grants;
- current launcher/interpreter identity;
- fixed transport body/time ceilings;
- replay claim correctness;
- exact output owner/ACK leases;
- MCP Task ownership;
- protocol-clean stdout;
- no cross-process live Job resurrection.

## Shared-code boundary

Consume exact reviewed aw-tools / secured backend revisions.

Shared low layer may provide **only concretely needed host-neutral primitives**:
- DomainId/PrincipalId/ProjectId/AgentId/TurnId/JobId/NodeId/Revision where genuinely host-neutral;
- typed immutable-handle ExecutionRef/ResolvedExecutionRef parsing and bounded contextual-path representation;
- typed Job snapshot/read/control request/result types;
- generic Job-kind specialization contracts for Meta/Input/View/Event/default+supported send modes;
- Graph/finite Job runtime;
- typed Text/ExactRead/Bytes/Media values;
- common owner/router traits.

AW-specific current-Agent contextual overlays, d:0/i:0/p:0/a:0 structural defaults, "^", parent/coordinator and Agent namespace resolution stay in AW rather than being copied into dotlink.

Do not copy AW Agent/Session code into dotlink.

## Cutover

This is intentionally a breaking vNext API. Do not implement adapters merely to keep old model tool names alive.

Before public cutover, measure actual schema bytes/tokens, first-valid-call rate, full model/tool rounds and result tokens against DL-150, including known n:<coordinate> output without a discovery call and deliberate long-Graph re-execution without job_restart. Preserve original native FS/Graph, Task/replay, sandbox, result/ACK and compact progress contracts.
