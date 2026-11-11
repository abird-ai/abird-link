# Full cross-repo shared-tools handoff — 2026-11-09

Author: pvl
Co-authors: ChatGPT / prior review agents
Owner: @pvl

This is the canonical continuation handoff for the dotlink ↔ AW shared-tools
convergence. It is a **map of the authoritative design, implementation history,
current source state, validation evidence and remaining work**. It does not
replace the sealed filesystem/Graph specification; it tells a new agent exactly
which files are authoritative and how the many historical phases fit together.

## 1. Read this first

Two repositories participate:

- AW: `/home/pvl/spaces/abird/src/aw`
- dotlink: `/home/pvl/spaces/abird/src/dotlink`

Exact reviewed checkpoints at this handoff:

- AW `main`: `4a23e92a8f516971bb232b98e4fc000554bcc4ba`
  (`fix(tools): revalidate graph topology before effects`)
- GitHub `abird-ai/aw` `main`: the same `4a23e92...` revision; this was verified
  with `git ls-remote`. The local `origin/main` tracking ref may be stale because
  the push used an explicit remote URL.
- dotlink `main`: `b31683c79a8ffd7c5540ac9388e607d8af6c7ffa`
  (`refactor(tools): cut over to shared aw-tools runtime`)
- dotlink `origin/main`: `c34ea5ab115e2841384de349eda5ca9cea661b48`.
  Therefore dotlink is one reviewed commit ahead of its configured tracking ref.

The dotlink Cargo dependency is no longer a sibling/path dependency. It is
pinned to the exact AW Git revision:

```text
https://github.com/abird-ai/aw
rev = 4a23e92a8f516971bb232b98e4fc000554bcc4ba
```

The same pin is used for `aw-tools`, `aw-tools-fs-unix` and
`aw-tools-fs-win`, and `Cargo.lock` records the exact Git source/hash.

### Authority order

When documents disagree, use this order:

1. current source + tests at the exact commits above;
2. dotlink sealed canonical spec:
   `.agents/plans/dotlink-ai-native-filesystem-2026-10-31/README.md`;
3. current dotlink shared implementation contract:
   `.agents/plans/dotlink-ai-native-filesystem-2026-10-31/SHARED-IMPLEMENTATION.md`;
4. AW convergence architecture:
   `.agents/plans/dotlink-tool-port/SHARED-AW-TOOLS-CONVERGENCE-2026-10-31.md`;
5. current plan/TODO status blocks;
6. older dated handoffs/review files only as historical provenance.

Do **not** resurrect an older handoff, phase checklist or local-only pin when the
current source/spec above has superseded it.

## 2. Original problem and design goal

The work began with dotlink's model-facing filesystem tools becoming large,
duplicated and difficult to reason about across:

- `ls`, `grep`, `read`, `write`, `edit`, `patch`;
- binary `bread`, `bwrite`, `bedit`;
- lifecycle `cp`, `mv`, `rm`;
- trusted shell execution;
- replay, cancellation and retained output;
- platform-specific secure filesystem implementations.

At the same time AW had its own native tools, Agent authority, Process custody,
Session lifecycle and durable journal. Keeping two independent implementations
would create two semantic/security surfaces that inevitably drift.

The target therefore became:

> **One canonical reusable Rust tool/runtime implementation in AW's `aw-tools`,
> with product-specific thin adapters in AW and dotlink.**

The design priorities repeatedly requested throughout the review were:

- correctness and fail-closed security first;
- clean, intuitive, composable semantics;
- shared lowest-layer fixes rather than per-tool patches;
- scalable/future-proof but maintenance-light design;
- Rust zero-cost/simple high-performance paths where clean;
- no compatibility baggage in canonical internal types;
- token-efficient model-facing schemas/results;
- no second AW Agent, ToolPolicy, TurnJournal, Process service or workflow DB.

## 3. Canonical final package architecture

### `aw-tools`

This is the canonical reusable implementation. It owns transport-neutral:

- canonical filesystem tool schemas and validation;
- path/model spelling and status grammar;
- bounded discovery/search/read rendering;
- exact edit, strict patch and binary splice semantics;
- lifecycle planning/execution for `cp`, `mv`, `rm`;
- mutation planning/budgets;
- Graph parser and dependency language;
- Job/Node lifecycle and the canonical `JobManager`;
- immutable `NodePlan`;
- conflict footprints and deterministic native scheduling;
- retained output, raw-byte paging/ACK/owners;
- replay state machine and restart semantics;
- task/spool/fence/retention primitives;
- host-neutral conformance fixtures;
- optional generic sandbox policy interfaces.

It must not depend on AW Agent/ToolPolicy/TurnJournal or dotlink transport types.

### `aw-tools-fs-unix`

Reusable Unix secured filesystem backend. It owns descriptor-rooted,
no-follow/no-cross-mount access, multi-root capability sets, exclusions,
mutation publication and private-artifact integration.

### `aw-tools-fs-win`

Reusable Windows secured filesystem backend. It owns the corresponding Windows
path/reparse/DACL/rename semantics and canonical UNC model paths.

### `aw-tools-adapter`

AW-specific thin adapter. It may own:

- AW `Tool` / `ToolSpec` projection;
- `LocalToolOptions`, `LocalToolLimits`, `LocalToolConcurrency`;
- original AW `ToolPolicy` grant issuance;
- `ToolBindingAuthority` integration;
- AW Session/branch lifecycle binding;
- AW `PipeProcess` custody and trusted shell preparation;
- AW provider/media compatibility projection;
- old AW wire/input compatibility where still intentionally accepted.

It must not contain another implementation of canonical filesystem/Graph/Job
semantics.

### dotlink adapter

Dotlink now consumes the pinned shared crates and keeps only product-specific:

- MCP schemas/projection and transport presentation;
- dotlink profile/grant mapping into prepared capability sets;
- shell/process integration specific to dotlink transport lifecycle;
- OpenAI tunnel replay/delivery ownership;
- dotlink private-state and session ownership integration;
- compatibility at the wire/deserialization boundary.

The large old duplicate native filesystem implementations were removed during
`b31683c`.

## 4. Package rename decision

The naming was deliberately changed to make the shared implementation the
obvious primary crate:

```text
old aw-tools-core  -> aw-tools
old aw-tools       -> aw-tools-adapter
```

This was chosen so both AW and dotlink consume the same `aw-tools`, while each
product can have a thin adapter. Do not reintroduce `aw-tools-core` as a second
canonical implementation.

The shared `JobManager` belongs in `aw-tools` too. AW-specific Job execution
binding lives in `aw-tools-adapter`, but the lifecycle/state machine itself is
shared.

## 5. Canonical model-facing tool surface

The sealed canonical filesystem surface is:

```text
ls
find
grep
read
write
edit
patch
cp
mv
rm
bread
bwrite
bedit
```

Canonical Graph/Job control includes:

```text
batch
job_start
job_list
job_get
job_output
job_ack
job_cancel
job_restart
```

Trusted interpreters are distinct capabilities when actually available:

```text
bash
pwsh
powershell
```

`pwsh` and Windows PowerShell are never aliases or silent fallbacks for each
other.

Deliberate omissions remain deliberate unless a new design explicitly changes
them: no separate model-facing `sed`, `tree`, `stat`, `mkdir`, `touch`, `chmod`,
`chown`, arbitrary symlink creation or hardlink creation just to mirror a shell.

## 6. Canonical status grammar

Primary outcomes are:

```text
ok
unchanged
fail
mixed
unknown
skip
```

Modifiers are:

```text
more
incomplete
```

Important settled decisions:

- use `fail`, not `err`;
- use `mixed` for mixed known outcomes;
- `unknown` means effect/result state cannot be proven;
- do not invent `done`, `end`, `.err`, `.done` or another terminal class;
- `unchanged` requires proof under existing read authority;
- write-only operations do not perform an unauthorized equality read merely to
  report `unchanged`;
- shell `fail` describes process semantics, not rollback of shell side effects;
- a genuinely unknown launched effect stays `unknown` even after later
  cancel/revoke intent;
- `more` means recoverable continuation; `incomplete` means coverage is not
  guaranteed. They are not synonyms.

Consumers should use the shared status helpers (`Snapshot::status_label()` etc.)
rather than reimplementing precedence.

## 7. Path and metadata contract

The canonical model path language is UTF-8 with `/` separators on every
platform.

Key rules:

- safe path labels are emitted bare;
- ambiguous/control labels use reversible JSON quoting;
- invalid-UTF-8 Unix names are not addressable through a hidden byte-path DSL;
  discovery reports `incomplete path_encoding` rather than inventing a second
  path language;
- `find` paths are directly reusable as tool input;
- `ls` intentionally emits basenames for its one directory;
- path sorting is deterministic, case-sensitive unsigned UTF-8 byte order of the
  canonical `/` spelling;
- unavailable selected metadata sorts last;
- timestamp range endpoints are RFC3339 and exclusive;
- do not synthesize unsupported timestamp precision;
- regular-file size is byte length; directory/other inode allocation trivia is
  not presented as portable size;
- POSIX mode means canonical permission/special bits where the platform has that
  concept; do not overload it with unrelated Windows attributes.

Windows UNC/network-share model paths use:

```text
//server/share/...
```

UNC is a distinct absolute namespace. Grant selection remains exact while
Windows namespace safety may use invariant folding. Native Windows execution is
still a platform gate.

## 8. Hidden names and `.gitignore`

Settled rules:

- `hidden=false` does not stop the engine from consulting a hidden `.gitignore`;
- the explicitly named root is admitted even when hidden;
- gitignore scope begins at the requested root only;
- do not silently load ancestor `.gitignore`, `.git/info/exclude` or global
  excludes;
- nested rules, anchored rules, directory rules and negation follow normal
  ordering semantics;
- never parse/use a truncated prefix of an ignore file;
- unreadable/malformed/over-budget ignore state yields safe results plus
  `incomplete gitignore`; it never widens authority;
- private mutation artifacts are hidden by **exact active identity-owned
  records**, not by a magic `.dotlink-*` prefix.

## 9. Direct result/paging contract

Compact discovery/search rows are atomic fitting units. Never split a path or
metadata row merely to fill a response.

For grep content, one primary hit plus its coalesced context group is atomic.

If the first selected atomic unit cannot fit the semantic result ceiling, return
an actionable error. Never emit an empty/non-advancing cursor.

Direct query cursors are positions in the snapshot observed by that call. They
are not hidden cross-call filesystem snapshot/revision tokens. External mutation
may change later pages.

Retained Job output is different: retained bytes are immutable and raw-byte
offsets remain stable until acknowledgement/expiry.

## 10. `read` semantics

Important sealed behavior:

- logical line numbering is 1-based;
- `byte` is an offset within logical line content, excluding LF/CRLF terminator;
- byte cursor must be on a valid scalar boundary and may equal line-content
  length;
- if byte equals content length, the line terminator is still continuation
  before the next line;
- preserve source line endings;
- `numbered=true` prefixes `line<TAB>` and repeats the same source line number for
  long-line continuation;
- `numbered=false` returns exact source slice data plus separate status, not a
  marker embedded into the bytes;
- useful data followed by an I/O fault must be fault-marked/error/mixed as
  appropriate, never presented as complete success;
- binary/invalid UTF-8 direct read fails and points to `bread`.

## 11. Final-symlink operation classes

This was a major correctness/security correction and must not regress.

### Content operations

`read`, `bread`, `write`, `bwrite`, `edit`, `bedit`, and patch Update File may
follow an existing final symlink only after:

1. lexical deny/protected checks;
2. resolved-target authorization;
3. regular-file target validation.

Mutation replaces the resolved target entry and preserves the symlink object.

### Create-only operations

Missing-target `write`/`bwrite` and patch Add File create the named entry only.
A dangling final symlink is not "missing" and is never followed. A concurrent
link insertion must lose the no-replace publication race.

### Lifecycle operations

`cp`, `mv`, `rm` operate on the final symlink object, never its target.
Discovery may report symlinks but never traverses child symlinks.

## 12. Write/edit/patch semantics

### `write` / `bwrite`

- atomic publication;
- missing parent suffix is part of the same publication unit; a failed operation
  leaves no visible partial parents;
- existing target must resolve to an authorized regular file;
- write-only authority never performs an equality read;
- preserve target mode/DACL access protection;
- staging must never be broader than the target;
- atomic replacement may break hard-link topology; preserving atomicity and
  authority is more important than preserving inode alias topology.

### `edit`

- non-empty bounded edit list;
- canonical tuples match one original snapshot;
- independent replacements are planned before publication;
- overlaps/ambiguity are rejected unless explicit `all` semantics apply;
- old AW compatibility input stays in the adapter only, not canonical shared
  representation.

### `patch`

Canonical patch is strict:

- full preflight/stage before effect where required;
- untouched bytes remain exact;
- line-ending inheritance is local to the insertion/update site;
- Add File preserves actual grammar payload rather than whole-file normalization;
- duplicate/alias targets and file/dir ancestor conflicts fail pre-effect;
- Add File sections sharing one missing parent suffix coalesce into one atomic
  namespace publication unit;
- exact contributing input coordinates are retained so mixed/uncertain commit
  ledgers can be truthful after faults;
- legacy `best_effort` may be accepted only in a compatibility adapter and is
  not canonical semantics.

## 13. Lifecycle semantics

### `cp(from,to)`

- destination is exact; no implicit basename/directory inference;
- destination must not exist;
- missing parents publish atomically with the copy;
- files, complete bounded directory trees and symlink objects are supported;
- symlink payload is copied verbatim/no-follow;
- destination-inside-source fails;
- unsupported/special descendants fail; never silently skip;
- basic mode/access protection is retained where supported;
- no ownership/xattr/timestamp administrative promise;
- hard-link topology is not promised.

### `mv(from,to)`

- destination parent must already exist;
- one same-filesystem rename or no namespace change;
- no implicit copy+delete fallback;
- self/descendant moves rejected by namespace identity, not string prefix;
- an existing hard-link alias at destination is still destination-exists failure;
- case-only rename is a real rename where supported, not `unchanged`;
- cross-device move fails and should suggest explicit `cp` + `rm` if desired.

### `rm(paths,recursive)`

- input non-empty, exact names, no globs;
- reject duplicate roots, aliases and ancestor/descendant overlaps before effect;
- nonrecursive removes file/symlink/empty directory only;
- recursive removal never falls back to visible child-by-child deletion;
- all cleanup records are durably reserved before the first root detach;
- each root is revalidated immediately before detach;
- commit is atomic rename to an exact identity-owned tombstone, then background/
  bounded cleanup;
- recovery reconciles exact original/tombstone identity before serving filesystem
  discovery;
- stale/absent/identity-mismatched records are retired/diagnosed, never used to
  hide/delete a user replacement.

A volatile private-artifact store is enough for transient staging/backup names,
but **not** for namespace-detaching recursive `rm` tombstones.

## 14. Hard-link, mount and reparse decisions

User regular files may legitimately have `nlink > 1`; do not reject them merely
because they are hard-linked.

Single-link integrity checks remain appropriate for private spool/control state
where aliasing would violate ownership.

Recursive operations preserve explicit no-cross-mount/reparse safety. Symlink
no-follow alone is not a sufficient boundary.

## 15. Graph language

One `Job` owns one immutable `Graph`. A Graph owns Nodes. Nodes are not nested
Jobs.

Each op is compact:

```text
[tool, args, when?]
```

Coordinates are stable 1-based positions.

Dependency language rules:

- omitted `when` means previous **complete** `ok|unchanged` with no
  `more|incomplete`;
- `*` means an independent root;
- backward references only;
- `&` binds tighter than `|`;
- parentheses allowed;
- whitespace insignificant;
- exact outcome/modifier predicates plus `N.*`;
- no negation, variables, loops, output substitution, forward refs, hidden
  comparison language or nested `batch/job_start/job_*`.

False branches become `skip condition`. Skip is expected terminal state, not an
implicit failure.

## 16. Deterministic Graph execution

Native nodes carry conservative read/write footprints from validated prepared
operations.

If two native scopes overlap and at least one writes, lower Graph coordinate has
priority. A higher conflicting node is not claimed while the lower conflicting
node is nonterminal and still may execute.

This is a machine scheduling rule, **not** a hidden model-visible dependency.
`when` remains the explicit dependency language.

Disjoint native scopes remain parallel.

Shell `writes` hints are advisory scheduling information only:

- omitted = unknown writes;
- `[]` = caller expects no writes;
- relative to effective shell `dir`;
- no glob/shell/env expansion;
- never authority, sandbox policy, replay identity or correctness proof;
- inaccurate hints can allow shell-level contention;
- explicit Graph `when` is the only model-visible happens-before contract.

## 17. JobManager / NodePlan ownership

`aw-tools::JobManager` is the one canonical manager.

It owns:

- immutable Graph definition and `NodePlan`s;
- pending/claimed/running/terminal node lifecycle;
- active-node/retained-job/replay/spool accounting;
- deterministic conflict admission;
- retained result/stdout/stderr streams;
- raw-byte paging and ACK owners;
- cancellation/revocation state;
- replay decisions;
- restart of terminal retained Graphs after revalidation;
- compact status/listing projections.

A `NodePlan` captures before execution:

- canonical tool + arguments;
- semantic work/result ceilings;
- conservative footprint;
- registry generation;
- authority binding generation/fingerprint;
- prepared runtime/interpreter identity;
- Task-result eligibility;
- deterministic replay identity.

Outer transport/result budgets are presentation only. They may cause the same
Job to be handed off, but must never rerun/repage a child in a way that changes
node modifiers or downstream effects.

## 18. Process and shell correctness

Canonical ordering for a shell node is:

```text
pending -> claimed -> running -> terminal
```

The canonical Start/running checkpoint is committed **before** OS process spawn.

Consequences:

- definitive pre-spawn refusal can settle as known `fail`;
- once spawn may have happened, ambiguity is `unknown`;
- cancellation/revocation must use the same owned process tree;
- inability to prove stop/reattach is `unknown`, never fabricated
  `fail cancelled`;
- shell stdout/stderr truncation is retained-output observability only and does
  not add Graph `incomplete` or change process success semantics;
- direct shell calls use the same one-ShellNode Job path rather than a parallel
  durability engine.

## 19. Job output and ACK

`job_output` offsets and ACK values are **raw retained byte positions** even when
text projection is UTF-8 lossy or output is base64 encoded.

ACK is monotonic and idempotent and cannot exceed retained stream extent.

Multiple owners matter: Job presentation, Task projection and replay/delivery
may independently own bytes. One owner's ACK must not physically discard bytes
while another exact owner still needs them.

Compact process terminal metadata remains available even when bulk output is
ACKed/released.

Authority revocation closes old output/native Task payloads. Compact status and
cancel/settle control may remain sufficient to finish safe shutdown. Regranting
similar current authority does **not** reauthorize old payload bytes.

## 20. Replay and restart boundaries

### AW

AW keeps its original single Agent/ToolPolicy/TurnJournal. The outer journaled
tool invocation is the durable replay fence for a Graph-producing call.

Per-node Graph state is intentionally live-session state; AW does not create a
second persistent Graph database. Process loss during an unresolved invocation
remains uncertain and is not automatically reconstructed/replayed.

### dotlink

Dotlink transport replay and response ownership remain product-specific, but
shared-owned Jobs must never silently fall back to the old lifecycle.

Once an entry is shared-owned:

- missing shared canonical state is split-brain/fail-closed;
- cancellation/exposure/completion/cleanup must not interpret shared NotFound as
  a legacy object;
- shutdown retains evidence when cleanup cannot be proven safe;
- hosted replay revalidates the exact inner synchronous/durable representation
  immediately before delivering a cached terminal response;
- if representation changed but the same execution still exists, discard the
  stale cached body and return to recoverable-shell handling rather than deliver
  stale output then repair ownership afterward.

`job_restart(id)` is a new execution. It recompiles/revalidates the retained
immutable Graph against **current** schema/tool visibility, authority, prepared
runtime/interpreter and current host limits, then captures new NodePlan ceilings.
The source Job remains immutable history.

## 21. AW authority and larger architecture alignment

The refactor was explicitly checked against AW's broader goals:

- one original Agent;
- one ToolRegistry/ToolPolicy;
- one immutable TurnJournal authority path;
- one Process service/custody path;
- Domain → Workspace → Session remains the user model;
- no credential/provider authority restored from old history;
- exact source/site/effect grants and revocation preserved;
- journal intent before effect preserved;
- unknown effects remain inspectable/uncertain, never silently replayed;
- no new persistent Graph database;
- no raw WAL replication or lock bypass;
- no_std/alloc kernel boundary remains intact;
- minimal/stateless build boundaries remain intact.

Persisted `FileConfig` still does **not** become tool authority. Runtime tool
configuration is host-owned.

## 22. AW configuration readiness refactor

A fresh cross-repo review found an important policy gap: the top-level Agent
parallel settings and Graph runtime originally had independent fixed limits, so
one `batch/job_start` could have run nodes in parallel even when the operator had
selected serial tool execution.

The final AW refactor fixed this with:

- `LocalToolLimits`: narrowing-only host ceilings shared by direct and Graph
  execution;
- `LocalToolConcurrency`: one host-owned concurrency snapshot for top-level Agent
  scheduling and nested Graph scheduling;
- generic Agent lifecycle notification for dynamic `Agent::set_*` updates;
- transactional validation before applying a policy update;
- live Jobs may block a policy change rather than silently outlive a newly
  tightened ceiling;
- subagents inherit ceilings while retaining deliberately narrower tool grants;
- model/Session rebuilds preserve exact `LocalToolOptions`;
- tool-off mode does not overwrite an embedding host's unrelated scheduler
  policy;
- hard safety maxima remain compile-time shared invariants; runtime config may
  narrow, never widen them.

This design is intentional. Do not convert every safety constant into a user
knob.

## 23. Shared engine maxima versus product limits

The shared engine keeps maxima high enough for both products while each host may
narrow them. The exact current source is authoritative, but the design includes:

- shared large bounded read/render/per-stream ceilings;
- shared aggregate session spool maximum sized for dotlink profiles;
- AW product defaults narrowed through `LocalToolLimits`;
- dotlink product/profile ceilings mapped through the same shared APIs;
- direct result/presentation budgets distinct from immutable Graph semantic
  node budgets.

A configuration change must never create this failure mode:

```text
schema permits X
but direct execution enforces Y
and Graph execution captures Z
```

All three must derive from the same validated host limits plus hard shared
maxima.

## 24. Historical implementation chronology

This section gives the new agent the evolution without requiring every old chat.

### 2026-10-26 to 2026-10-29 — dotlink hardening/parity reviews

Repeated security/correctness passes addressed:

- replay-ledger lifetime/capacity and uncertain outcomes;
- policy revocation vs cached shell responses;
- protected config/private state;
- secure path handling and path composability;
- bounded discovery/grep/read/write/edit/patch;
- staged publication and crash behavior;
- Windows process containment and ACL-aware staging;
- exact source/tool authority checks;
- result-budget correctness and JSON-safe truncation.

These reviews established the expectation that fixes should live in shared
lowest layers and that stale cached authority/result data must fail closed.

### 2026-10-30 — first shared toolset extraction in AW

AW introduced `aw-tools-core` as a reusable semantics layer and extracted:

- matchers/text classification;
- read window logic;
- mutation budgets;
- patch grammar;
- binary splice/digest planning;
- grep work accounting;
- task/spool/fence concepts;
- optional sandbox policy interface.

This was transitional. It proved reuse, but left too much canonical execution in
the old AW tool crate.

### 2026-10-31 — rename and convergence plan

The architecture was re-sealed:

- `aw-tools-core -> aw-tools`;
- old `aw-tools -> aw-tools-adapter`;
- JobManager also belongs in shared `aw-tools`;
- AW adapter keeps authority/process/projection only;
- dotlink should consume the same shared crates instead of carrying copies.

Canonical plan:
`aw/.agents/plans/dotlink-tool-port/SHARED-AW-TOOLS-CONVERGENCE-2026-10-31.md`.

### A1/A2 — rename + adapter thinning

Mechanical package rename landed first, followed by movement of canonical search,
patch, binary and filesystem execution semantics into `aw-tools`.

### A3 — native path correctness

Resolved operation-class target semantics, dual authorization for final content
symlinks, hardlink policy, Unix quoted/control UTF-8 path support, lexical
reusable discovery roots, mount/reparse rules and alias/race behavior.

### A4 — private mutation/lifecycle

Replaced broad-prefix temporary hiding with exact identity-owned private-artifact
records, durable `rm` tombstone recovery, restart reconciliation, and complete
`cp/mv/rm` semantics.

### A5 — canonical JobManager

Consolidated Job/task/spool/retention into one shared manager and introduced
immutable NodePlan, footprints, deterministic scheduling, output ownership, ACK,
replay and restart.

### A6 — AW host binding

Bound the shared manager to the original AW ToolPolicy/TurnJournal/Session and
PipeProcess custody. Added mixed native/ShellNode Graphs, direct shell unification,
revocation/cancellation, unknown fencing and live-session lifecycle.

### A7 — external consumer readiness

Cleaned public API/dependency boundaries, added `aw-tools/conformance`, reusable
CapabilitySet backends and standalone consumer fixtures. The shared filesystem
execution duplication was removed, including strict patch and streamed binary
edit orchestration.

Several intermediate pins were reviewed (`75f6f00`, `c0d8ee3`, `3cb3cea`,
`55d93db` etc.). They are historical checkpoints. The final pin for dotlink is
**`4a23e92...`**.

### Final AW review/config series

The last review series fixed/refined:

- hidden/exposed Job capacity separation;
- output quiescence/owners/compaction;
- atomic process-output attachment;
- binary/grep bounds;
- optimistic exposure;
- global vs local execution policy;
- exact read projection;
- shared native footprints;
- enabled-surface-scoped runtime limits;
- unified Agent/Graph parallelism policy;
- cancellation unknown precedence;
- final Graph topology revalidation before effects.

The final reviewed AW commit series ends at `4a23e92`.

### Dotlink cutover

Dotlink then replaced its duplicate implementations with the remote-pinned shared
engine. The cutover commit is `b31683c`.

The diff from the pre-cutover plan commit is intentionally deletion-heavy:
roughly 10.6k additions / 15.9k deletions, including removal of old tool-specific
`binary`, `edit`, `grep`, `ls`, `mutation`, `patch`, `text`, staging and much of
secure-fs duplication.

During final dotlink review four important correctness fixes were made:

1. shared Start checkpoint precedes OS spawn;
2. shared ownership never falls back to legacy lifecycle on missing state;
3. shutdown retains evidence if safe cleanup cannot be proven;
4. hosted replay revalidates the exact inner response representation before
   delivery.

## 25. Key AW commits in the convergence series

For source archaeology, the main sequence from the first consolidation to the
final pin includes:

```text
e337f0d consolidate AI-native filesystem core and secure backends
75f6f00 converge shared tool runtime
c0d8ee3 reserve complete shell graph output
3cb3cea share canonical patch and binary execution
bbd60b0 harden shared runtime admission
5b6e098 harden shared scheduler namespaces
2b6a2d1 unify canonical request validation
8178e56 decouple direct config from job limits
105cff2 share canonical direct filesystem execution
0b27885 harden shared runtime policy and recovery
a189402 split mutation read and write ceilings
1c57075 seal graph and grant-root semantics
05141b9 separate direct result ceilings
d0adcfc make media ceiling host-selected
53afc4d align shared binary and base-root limits
59a34b7 expose canonical request path intent
82e1f5d share canonical read projection
5280bf0 fence cross-process stage recovery
2ddc4b4 share graph result semantics
593d14a preserve graph commit outcomes
f3fc6e2 separate hidden and exposed job capacity
d837e91 publish failed output quiescence
a1ec219 compact retained output by owner
e8a50a6 bound retained file output
59bdc3a attach process output atomically
ec9359d preserve output owner acknowledgements
f9abb96 bound binary inputs and grep paging
630f807 seal shared job admission and projection
fd20330 reserve optimistic job exposure
25a87a7 expose output owner state
4d1215e separate global and local execution policy
2fe8fd8 preserve exact read data projection
ac97e7e share native graph footprints
ee5e991 align job capacity with exposed graph surface
b28bb7a scope runtime limits to enabled surfaces
f9a12cc unify tool parallelism updates
a8a8b5d stabilize final integration review
55d93db preserve unknown cancellation outcomes
4a23e92 revalidate graph topology before effects
```

Use Git/source, not the commit message alone, for exact behavior.

## 26. Current validation evidence

### AW final reviewed state

Recorded final gates at the `4a23e92` series:

- `aw-tools`: 279/279;
- `aw-tools-adapter`: 203/203;
- `aw-agent-core`: 78 passed + 1 intentional ignore;
- strict affected Clippy: clean;
- `--no-default-features`: clean;
- `--no-default-features --features minimal`: clean;
- `aw-agent-kernel --no-default-features --features alloc`: clean;
- format/diff checks: clean.

The last attempted fresh Windows cross verification was blocked before AW source
compilation because the cached Windows GNU std sysroot became a dangling Nix
store path. Earlier Windows cross-build evidence exists, but this is not a fresh
native Windows pass.

### dotlink remote-pinned final reviewed state

Recorded after replacing sibling dependencies with the exact remote AW pin:

- main active unit suite: 628/628 + 1 intentional external-fixture ignore;
- five integration binaries: 1/1, 8/8, 1/1, 12/12, 4/4;
- independently generated AW conformance fixture: 13 schemas / 13 request
  vectors, clean;
- all-target/all-feature Cargo check: clean;
- strict Clippy `-D warnings`: clean;
- rustfmt: clean;
- `git diff --check`: clean;
- Alejandra: clean;
- Markdownlint: clean.

Do not call a platform gate passed merely because Linux tests or cross compile
passed.

## 27. Remaining genuine work

The core shared-engine convergence is complete. Remaining work is narrower.

### Immediate repository action

Dotlink `b31683c` is committed but is still one commit ahead of its configured
`origin/main`. **Do not push without explicit user authorization.** If the user
authorizes a push, revalidate worktree/HEAD first and push the reviewed commit
without mixing unrelated changes.

### Platform acceptance

Still required before claiming full cross-platform release parity:

- native Windows NTFS tests: DACL preservation, reparse behavior, hardlinks,
  case-only rename, `ReplaceFileW`, Job Object/process cleanup;
- native Windows UNC/network share execution for `//server/share/...`;
- native macOS ACL/xattr/no-follow/rename tests;
- successful Bubblewrap runtime on a host that permits the required namespaces
  and `/proc` mount;
- Windows/macOS cross-process race/interoperability where applicable.

### Separate product capabilities, not blockers for shared filesystem cutover

These remain separate designs/features and should not be smuggled into the
shared-engine work:

- true provider/MCP typed native media for `bread(format=mcp)`;
- separately authorized sandbox launcher/runtime;
- broader live MCP wire/media interoperability work;
- any durable remote RPC/storage workflow above the intentionally live-session
  shared Graph runtime.

### Documentation/TODO cleanup

Both repos contain older historical TODO sections with unchecked boxes that were
later implemented by A5/A6/A7/A8 or the final cutover. The **current status block
and this handoff override those historical checkboxes**. Do not reimplement work
just because an old dated subsection was never mechanically checked off.

A future cleanup pass may archive/reconcile those historical checklists, but it
must preserve useful provenance.

## 28. Fresh-review checklist before any new implementation

A new agent should always revalidate current HEAD and then review these seams:

1. no canonical filesystem implementation has crept back into
   `aw-tools-adapter` or dotlink;
2. no AW Agent/ToolPolicy/TurnJournal type has leaked into `aw-tools` public
   semantic API;
3. direct tool and Graph node execution call the same canonical operation
   implementation;
4. schemas, direct execution and Graph NodePlan ceilings derive from the same
   host limits + shared hard maxima;
5. selected tool subsets cannot make unselected native tools available inside a
   Graph;
6. dynamic Agent parallel-policy narrowing cannot be bypassed by nested Graphs;
7. final symlink lexical + resolved authorization remains operation-class
   correct;
8. recursive walkers do not cross symlink/mount/reparse authority boundaries;
9. private names are exact identity-owned records, never broad prefixes;
10. `rm` reserves durable cleanup state before namespace detach;
11. Start/claimed/running ordering prevents duplicate or ambiguous shell launch;
12. unknown launched effects remain unknown;
13. shared-owned dotlink Jobs never fall back to legacy lifecycle;
14. retained output ACK respects all owners;
15. replay/delivery revalidates current representation and authority immediately
    before disclosure;
16. shutdown preserves evidence when safe cleanup cannot be proven;
17. no sibling/path dependency is committed for AW shared crates;
18. platform gates are reported accurately rather than inferred from Linux.

## 29. Source map for the next agent

### AW

Start with:

```text
AGENTS.md
.agents/plans/README.md
.agents/plans/TODO.md
.agents/plans/dotlink-tool-port/README.md
.agents/plans/dotlink-tool-port/TODO.md
.agents/plans/dotlink-tool-port/SHARED-AW-TOOLS-CONVERGENCE-2026-10-31.md
crates/aw-tools/
crates/aw-tools-fs-unix/
crates/aw-tools-fs-win/
crates/aw-tools-adapter/
crates/aw-agent-core/src/tool.rs
crates/aw-agent-core/src/agent.rs
```

Important shared modules include filesystem schema/path/access/backend,
discovery/search/read, operations/patch/binary, lifecycle/private artifacts,
footprints, Graph/batch/job/job_manager/runner, task/spool/fence/retention and
conformance.

### dotlink

Start with:

```text
AGENTS.md
.agents/plans/TODO.md
.agents/plans/dotlink-ai-native-filesystem-2026-10-31/README.md
.agents/plans/dotlink-ai-native-filesystem-2026-10-31/TODO.md
.agents/plans/dotlink-ai-native-filesystem-2026-10-31/SHARED-IMPLEMENTATION.md
Cargo.toml
src/mcp/shared_fs.rs
src/mcp/tools/shared_native.rs
src/mcp/tools/graph.rs
src/mcp/tools/jobs.rs
src/mcp/tools/shell.rs
src/jobs.rs
src/private_state.rs
src/transports/openai.rs
```

`b31683c` is the cutover diff to study if a regression appears.

## 30. Testing principles

The user explicitly does **not** want evidence/test infrastructure over-engineered.
Prefer:

```text
working implementation
-> compact targeted regression
-> real integration feedback
-> full relevant gate
```

Do not spend disproportionate time creating ever more textual fixtures, pinning
artifacts or evidence manifests when the implementation can be exercised
against the real runtime.

At the same time, do not weaken correctness gates. Tests should exist where they
protect a concrete contract/bug class, especially replay, authority, path,
mutation, cancellation and platform boundaries.

Use at least 8 Cargo build jobs where practical.

## 31. Git/working rules

Follow each repo's `AGENTS.md`.

In particular:

- preserve unrelated work;
- do not touch reserved AW plan/UX ownership outside the requested scope;
- review exact staged paths before commit;
- signed commits only;
- no push/tag/release/deploy without explicit user authorization;
- never disable TLS verification as a workaround;
- avoid committing generated build outputs, runtime DBs, real grants or secrets;
- dotlink shared dependencies must stay exact remote Git pins, not sibling paths.

## 32. Recommended next step

There is no need to redesign the shared filesystem/Graph runtime again merely to
continue. The next agent should:

1. verify both exact HEADs and clean/dirty status;
2. read this handoff plus the authority files listed above;
3. perform one fresh focused correctness/regression review of the current
   cross-repo boundary;
4. if no new issue appears, treat shared convergence as complete;
5. if the user authorizes it, push dotlink `b31683c`;
6. continue only with the next explicitly requested work or the remaining native
   platform gates;
7. if a bug is found, fix it at the lowest shared layer unless it is genuinely a
   product adapter/transport concern, then rerun the relevant AW + dotlink gates.

## 33. Next-agent starting prompt

Copy/paste this into a fresh agent:

```text
Work across these two repos using dotlink-l5:

  /home/pvl/spaces/abird/src/aw
  /home/pvl/spaces/abird/src/dotlink

First read both AGENTS.md files and then read the canonical cross-repo handoff:

  dotlink/.agents/plans/dotlink-ai-native-filesystem-2026-10-31/
  FULL-COMPREHENSIVE-HANDOFF-2026-11-09.md

Then read, in authority order:

  dotlink/.agents/plans/dotlink-ai-native-filesystem-2026-10-31/README.md
  dotlink/.agents/plans/dotlink-ai-native-filesystem-2026-10-31/SHARED-IMPLEMENTATION.md
  aw/.agents/plans/dotlink-tool-port/SHARED-AW-TOOLS-CONVERGENCE-2026-10-31.md
  the current TODO/status blocks in both plans

Expected reviewed checkpoints at handoff:

  AW:      4a23e92a8f516971bb232b98e4fc000554bcc4ba
  dotlink: b31683c79a8ffd7c5540ac9388e607d8af6c7ffa

AW's GitHub main is 4a23e92. Dotlink b31683c is committed locally and is one
commit ahead of its configured origin/main; do not push unless I explicitly ask.
Dotlink must keep its exact remote AW Git pin and must not fall back to ../aw path
dependencies.

Revalidate both repositories from source before acting. The target architecture
is already implemented: aw-tools is the canonical reusable filesystem/Graph/
Job runtime; aw-tools-fs-unix/win are reusable secured backends;
aw-tools-adapter is AW-specific authority/process/projection glue; dotlink is a
thin product/transport adapter. Do not recreate duplicate canonical tool or
JobManager implementations.

Do one fresh correctness/security/regression review of the current boundary,
with special attention to authority/revocation, final symlinks, mount/reparse
boundaries, exact private artifact recovery, rm detach/recovery, Graph conflict
ordering, Agent-vs-Graph concurrency ceilings, shell Start-before-spawn,
unknown cancellation outcomes, retained output ownership/ACK, shared-owned
split-brain handling, replay representation revalidation, and shutdown evidence
retention.

Fix any real issue cleanly at the lowest appropriate shared layer; keep product
transport/authority concerns in their adapters. Preserve AW's one
Agent/ToolPolicy/TurnJournal/Process architecture and minimal/no_std boundaries.
Do not over-engineer evidence fixtures; prefer real implementation + compact
regressions + full relevant gates. Use >=8 Cargo build jobs where practical.

Do not push, tag, release or deploy without explicit authorization. Report
platform gates honestly: native Windows/macOS and a successful Bubblewrap
runtime remain separate environment/platform acceptance work.
```
