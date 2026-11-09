# Shared `aw-tools` implementation architecture

Author: Prasanna Loganathar
Co-authors: ChatGPT GPT-5.6 Sol
Owner: @pvl
Status: implementation architecture active; shared filesystem and JobManager cutover complete, platform release gates still open

Reviewed against:

- dotlink `c34ea5a` — canonical DL-150 model-facing semantics;
- AW `e337f0d` — shared core + secured backend consolidation;
- AW `6f7845d` — latest parity audit/handoff and validation record.
- AW `4a23e92` — current reviewed shared implementation, including host-limit/config convergence, canonical status projection and pre-effect native Graph footprint revalidation.

This document is the canonical **cross-repository implementation architecture**
for DL-150. The model-facing semantics remain defined by `README.md`. The goal
here is to implement those semantics once and consume them from both AW and
dotlink rather than preserve two implementations that must be manually kept in
parity.

## Goals

1. One Rust implementation of canonical tool semantics.
2. One Unix and one Windows secured filesystem backend.
3. One canonical Graph/Job/Task runtime and `JobManager`.
4. Thin host adapters for AW and dotlink authority, journaling, transport and
   process custody.
5. No weakening of either product's existing security boundary to achieve reuse.
6. No committed `../aw` path dependency and no vendored/copied implementation.
7. Keep dotlink a simple single-binary package unless a separate adapter crate
   later provides concrete value.

## Final package naming

The shared implementation has outgrown the name `aw-tools-core`. The target is:

```text
AW repository

aw-tools                 canonical reusable tool/runtime implementation
aw-tools-fs-unix         secured descriptor-rooted Unix backend
aw-tools-fs-win          secured handle-pinned Windows backend
aw-tools-adapter         AW host integration (current aw-tools)
```

Conceptually dotlink has:

```text
dotlink
  tools_adapter/          dotlink-specific thin integration
      -> aw-tools
      -> aw-tools-fs-{unix,win}
```

The dotlink adapter may be called `dotlink-tools-adapter` in architecture/docs,
but it should initially remain an internal module. Do not convert dotlink into a
workspace solely to mirror AW's package layout.

If the shared crates eventually deserve an independent repository, move the
same crates/API rather than copying them; consumers should only change the
source location.

## Dependency direction

The dependency graph must remain acyclic and explicit:

```text
aw-tools-fs-unix  ----->  aw-tools  <-----  aw-tools-fs-win
                              ^
                              |
                  canonical semantic/runtime API
                         /             \
                        /               \
          aw-tools-adapter         dotlink tools_adapter
                 |                         |
            AW authority              dotlink authority
            TurnJournal               MCP/OpenAI replay
            Process host              sandbox/shell host
```

More precisely, the filesystem crates implement traits declared by `aw-tools`,
so their Cargo dependency points **to** `aw-tools`; `aw-tools` must never depend
on either concrete backend. The adapter selects/composes the backend.

## Ownership rule

Use this rule for every move:

- If behavior should be identical when the same canonical tool/Graph is run
  through AW or dotlink, it belongs in `aw-tools`.
- If behavior is a native filesystem guarantee independent of AW/dotlink policy,
  it belongs in `aw-tools-fs-unix` or `aw-tools-fs-win`.
- If behavior decides who may perform an effect, how a product journals it, how
  a process is launched/sandboxed, or how a result is exposed on a product
  transport, it belongs in that product's adapter.

## Canonical `aw-tools` ownership

`aw-tools` should own all reusable semantics and state machines:

- canonical schemas and compact descriptions;
- argument parsing/validation after compatibility translation;
- status grammar (`ok|unchanged|fail|mixed|unknown|skip`);
- `more` / `incomplete` semantics and compact result fitting;
- reusable path-label grammar and ordering;
- metadata/selectors/traversal/search semantics;
- `ls`, `find`, `grep`, `read`, `bread` algorithms;
- write/edit/patch/binary planning and receipts;
- `cp`, `mv`, `rm` planning/semantic outcomes;
- mutation planning limits and private-artifact protocol;
- Graph grammar/compiler and canonical node registry;
- immutable prepared node plans;
- conservative native read/write footprints;
- captured node work/result ceilings;
- deterministic lower-coordinate native conflict gate;
- canonical Job/Graph/Node state;
- **one shared `JobManager`**;
- process-wide runner admission and per-Job width;
- cancellation/revocation coordination;
- output retention, raw-byte paging and ACK semantics;
- Task projection;
- restart semantics;
- replay state-machine semantics and bounded replay/control admission;
- common progress snapshots.

A Task is a projection of a Job, not another execution state machine.

## Native backend ownership

`aw-tools-fs-unix` and `aw-tools-fs-win` own native correctness only:

- descriptor/handle-rooted path traversal;
- no-follow intermediate components;
- existing mount/reparse safety rules;
- no-follow entry metadata;
- stable object/parent identity;
- safe final-link resolution primitives;
- native regular-file handles;
- mutation coordination and cross-process locks;
- staged temporary creation;
- mode/DACL protection;
- snapshot verification;
- atomic create/replace/rename/detach;
- exact symlink payload copy;
- bounded tree traversal primitives;
- platform-specific no-replace/case-only/cross-volume behavior.

The backends do **not** decide AW Actor grants or dotlink AccessPolicy.

## AW adapter ownership

`aw-tools-adapter` owns AW-specific integration:

- `ToolSpec` / ToolRegistry projection;
- `ToolEffect` and exact Actor/source/site/epoch/Envelope checks;
- original TurnJournal integration;
- AW Session ownership;
- existing Process/Actor custody;
- `PipeProcess`, PTY and AW platform process launch integration;
- AW-specific compatibility request shapes;
- `ToolOutput` / provider projection;
- AW UI/browser helpers that are not canonical model-tool semantics;
- AW configuration/CLI wiring.

“Thin” means no duplicate canonical tool/Job semantics. It does not require
AW-specific process custody to be artificially moved into the shared crate.

## Dotlink adapter ownership

Dotlink retains only product-specific integration around the shared engine:

- `AccessPolicy` lexical/protected/resolved authorization;
- configured read/write grants and deny rules;
- dotlink protected state location;
- MCP/rmcp Tool and ContentBlock projection;
- OpenAI transport request identity and delivery acknowledgement;
- exact interpreter preparation and Bubblewrap/shell host integration;
- runtime/session authority generation;
- dotlink logging and transport limits;
- MCP Tasks protocol projection.

The adapter must not retain a second implementation of grep, read paging,
mutation planning, lifecycle semantics, Graph scheduling or Job output/ACK.

## Authority and prepared capabilities

Do not make `aw-tools` depend directly on AW's ToolPolicy or dotlink's
AccessPolicy. Authorization must be host-supplied while execution uses pinned
native capabilities.

The target flow is:

```text
model path
  -> lexical host policy/protected check
  -> backend secure resolution
  -> resolved host policy reauthorization when required
  -> prepared capability / immutable operation plan
  -> aw-tools canonical semantic operation
  -> aw-tools-fs-* native effect
```

A successful boolean pathname check followed by an ambient reopen is not an
acceptable abstraction. Prepared plans must retain enough pinned identity to
make the later effect refer to the object/namespace that was authorized.

### Operation classes

Use one shared operation-class model:

- content-existing: `read`, `bread`, existing `write`/`bwrite`, `edit`, `bedit`,
  patch Update; may resolve an existing final symlink only after lexical and
  resolved authorization;
- content-create: missing `write`/`bwrite`, patch Add; creates the named entry;
  a dangling final symlink is existing namespace state and fails closed;
- lifecycle-entry: `cp`, `mv`, `rm`; final symlink is the object itself;
- discovery-root: explicit `ls/find/grep` root may resolve after dual checks;
  child symlinks are reported but never traversed.

A resolved discovery root must preserve the caller's lexical model-facing prefix
in emitted paths, while its native capability remains bound to the resolved
object.

## Hard-link semantics

The current AW single-link restriction is not part of the final shared design.
The shared implementation must support ordinary multiply-linked regular files:

- read/search treats them as normal regular files;
- atomic replacement may break the replaced pathname's hard-link relationship;
- other hard links continue to reference the old inode;
- `cp` need not preserve hard-link topology;
- `rm` of one exact hard-link pathname is valid;
- a single multi-root `rm` request naming aliases of the same removable object is
  rejected during preflight;
- `mv` to an already-existing hard-link alias is `fail destination exists`, not
  `unchanged`.

Never weaken atomic replacement into in-place mutation merely to preserve link
count/topology.

## Path representation

Retain the DL-150 UTF-8 public path contract:

- `/` model separators on every platform;
- valid UTF-8 Unix names, including whitespace/control characters legal to the
  host filesystem, are addressable and rendered with reversible JSON quoting
  where needed;
- NUL and structurally invalid components remain rejected;
- invalid-UTF8 Unix names remain unaddressable in v1 and cause
  `incomplete path_encoding` rather than lossy output;
- Windows keeps its native invalid-name restrictions.

Backend validation must not reject a valid UTF-8 Unix name merely because its
rendering needs quoting.

## Exact private mutation artifacts

Prefix-based hiding such as `.dotlink-*` or `.aw-stage-*` is not sufficient:
it can hide user-created names and cannot prove crash ownership.

Introduce one shared private-artifact protocol in `aw-tools`, backed by a
host-provided bounded protected-state store.

### Required record

A private record binds at minimum:

- artifact kind (`stage`, `backup`, `rm_tombstone`, ...);
- exact parent namespace identity;
- exact private basename;
- expected/source object identity where applicable;
- current record state;
- bounded creation/generation metadata needed for safe recovery.

### Named staging sequence

For platforms/filesystems that require a named staging file:

1. choose a collision-resistant exact name;
2. create the empty object with restrictive/no-broader protection via O_EXCL or
   equivalent;
3. capture its stable no-follow identity;
4. durably register that exact identity/name before writing user bytes;
5. write/sync/stage;
6. publish atomically;
7. remove/retire the exact record after the private name is gone.

A crash before step 4 may leave only an empty/restrictive ordinary artifact; it
must not be magically hidden or deleted without ownership proof. Once a durable
record exists, discovery may hide only the exact matching identity/name.

Use unnamed native temporaries where they cleanly preserve the same atomic and
security properties, but never require a weaker portable fallback.

### `rm` and Windows backup recovery

`rm` keeps the sealed pre-detach identity record. A volatile Registry is not
sufficient: the Store must explicitly guarantee that a successful reservation is
recoverable after process loss. For a multi-root request, reserve **every** exact
identity-bound tombstone record before the first namespace detach; if any
reservation fails, retire earlier unpublished reservations and mutate nothing.
Immediately before each rename, revalidate that the source still has the identity
bound into its record. Then rename the already-identified root to its tombstone.
A published tombstone whose cleanup is uncertain retains its record for restart
reconciliation; unused later reservations are retired.

A Windows replacement backup record is reserved before replacement and binds the
future backup name to the original target identity. Recovery hides/deletes it
only when the current object identity matches that proof.

This replaces broad magic-prefix hiding in both products.

## Canonical Graph node plan

At Job creation, compile every node into an immutable `NodePlan` containing:

- canonical tool identity + graph eligibility/version;
- validated canonical arguments;
- captured host authority generation/fingerprint needed for later revalidation;
- captured semantic work ceiling;
- captured semantic result ceiling;
- conservative native read/write footprint;
- interpreter/runtime binding for ShellNodes;
- deterministic internal replay/node identity;
- output/Task eligibility metadata.

Outer transport result budgets are not part of this plan and therefore cannot
change node terminal modifiers or downstream side effects. Direct tool execution
is different: the product adapter passes the current authenticated presentation
budget into canonical direct rendering/paging before returning the result. The
transport's final response fitter remains a defensive postcondition, not the
normal pagination mechanism and never a substitute for pre-effect receipt fit.

## Native conflict footprints

Footprints come from the same prepared path planning used by the operation, not
from a second string parser.

A footprint consists of conservative read/write namespace scopes. Conflict when
scopes overlap and at least one writes. When the backend cannot prove a narrow
scope—case-folding ambiguity, unsupported alias resolution, etc.—widen/mark it
unknown and serialize conservatively.

For conflicting runnable native nodes, lower Graph coordinate has claim
priority. This is a machine scheduling gate only; it does not become a Graph
status dependency.

## Shared `JobManager`

The `JobManager` belongs in `aw-tools` because its semantics must be identical in
AW and dotlink.

It owns:

- runtime Job registry and one ID namespace;
- Job/Graph/Node lifecycle;
- immutable Graph and `NodePlan`s;
- active/retained/control/output quotas;
- process-wide runner permits;
- per-Job width;
- deterministic native conflict admission;
- claim/start/terminal ordering;
- cancellation/revocation intent;
- output ownership, paging, ACK and retention;
- exposure/handoff state;
- Task projection;
- explicit restart orchestration;
- replay-state transitions.

It does **not** own a product database, grant system, MCP server or shell
sandbox.

### Host interfaces

Prefer several narrow host capabilities over one giant product trait. The exact
Rust API is implementation-time work, but responsibilities are:

- `JobJournal`: durable transition CAS/checkpoint supplied by the host;
- `NodeAuthority`: prepare/revalidate exact node authority;
- `ShellRunner`: launch/inspect/stop a prepared process tree;
- `OutputStore`: bounded immutable/raw-byte retained output;
- `ReplayStore`: bounded atomic cross-process claim/tombstone primitives where
  the transport can redeliver across process lifetime;
- `JobIdSource`: host/session-safe Job identity allocation if the shared default
  is not sufficient;
- `PrivateMutationStore`: exact durable private-artifact records.

A host may implement several capabilities with one protected store internally,
but the shared API should not conflate their semantics.

### Journal/effect ordering

The manager proposes transitions; the host commits them through its original
journal/authority owner:

```text
Job admitted durably
  -> node claim committed
  -> authority/runner revalidated
  -> running/launch checkpoint committed where required
  -> effect starts
  -> terminal outcome committed
  -> dependent nodes may become runnable
```

A journal outcome that may already have committed fences the node/Job as unknown
rather than replaying an effect.

AW therefore keeps its one TurnJournal. Dotlink keeps its existing runtime and
transport replay ownership. Neither gets a second database/executor.

## Direct calls versus Jobs

Do not allocate a durable Job for every ordinary direct filesystem read/write
merely for architectural purity.

- direct v1 native calls use the same canonical `aw-tools` operation engine;
- `batch(ops)` and `job_start(ops)` always create the shared Job;
- ShellNodes use the shared Job runtime;
- direct shell calls may use a one-ShellNode Job and optimistically wait so
  timeout/handoff/replay share one lifecycle;
- native MCP/host Tasks use a one-NativeToolNode Job only when Task projection is
  requested/eligible;
- a direct native call that is not handed off as a Task need not pay durable Job
  allocation/retention overhead.

This keeps the architecture unified without adding unnecessary hot-path state.

## `batch` and `job_start`

Both are views over one constructor:

```text
compile Graph + NodePlans
reserve Job/control/result/replay state
create Job

batch:     optimistically wait; expose same Job only if handoff is needed
job_start: expose same Job immediately, then allow roots to start
```

No node is restarted during handoff.

## Job output and ACK

Retained output positions are absolute raw-byte positions independent of text or
base64 projection. The shared manager owns:

- raw offset/next-offset semantics;
- monotonic/idempotent ACK;
- immutable truncation metadata;
- output owner/reference accounting;
- reader leases;
- terminal writer sealing/quiescence;
- cleanup only after every exact owner is safe to release.

The host chooses the protected storage location and transport projection.

## Job listing correction before implementation

The current sealed `job_list(limit=64)` remedy cannot guarantee access to every
retained Job: the retained cap may exceed what one legal result budget can hold.
Before implementation, fold this small schema correction into the canonical
DL-150 README:

```text
job_list(offset=0, limit=64)
```

Ordering remains newest exposed Job first with Job ID as tie-breaker. `offset`
counts Jobs in that live ordering and continuation is `more offset=N`. Like
other direct live queries, it is not a hidden snapshot; newly exposed/expired
Jobs between calls may shift later offsets. Lost-receipt recovery remains good
because new Jobs appear at the front.

## Replay

Keep replay semantics shared, identity extraction host-specific.

`aw-tools` owns the replay claim/state machine and semantic-hash invariants. The
host supplies:

- replay namespace/key material;
- protected bounded atomic storage;
- durable create-if-absent/compare primitives;
- delivery retirement acknowledgement.

For dotlink, the key originates from the validated OpenAI/MCP transport request.
For AW, the identity originates from the original Actor/source/turn/call journal
scope. Same identity + different semantic hash fails closed.

Do not persist the complete Graph solely for cross-process replay; runtime Jobs
remain live-session objects as specified by DL-150.

## AW Process and dotlink shell boundaries

Do not force the two products to share process-launch policy.

AW may keep its current `PipeProcess`/PTY/Process Actor custody in
`aw-tools-adapter`. Dotlink may keep its exact interpreter preparation,
Bubblewrap, hardware/network grants and contained process launcher in its
adapter.

Both implement the shared runner contract so the common JobManager owns the
lifecycle semantics after launch: cancellation intent, stop proof, unknown stop,
exit classification, output sealing and terminal settlement.

## Compatibility boundaries

Compatibility belongs at adapters only:

AW may temporarily accept its old:

- read `offset/limit`;
- conditional `create_only` / `expected_sha256` writes;
- legacy edit objects;
- patch `best_effort`;
- asymmetric grep context;
- historical shell selection/config names.

Dotlink may accept its documented legacy job/shell argument shapes during the
migration window.

The canonical schemas produced by `aw-tools` contain only the final DL-150
surface.

## Feature layout

Do not mix feature renaming with the mechanical package rename. First preserve
current behavior/features exactly.

After the rename is green, simplify compiled features if useful. Target concepts
are:

- `read-only` default;
- `rw`;
- `graph` for pure Graph compilation;
- `jobs` for JobManager/Task/output/replay runtime;
- `sandbox` for genuinely reusable sandbox planning only;
- optional async support required by the manager/host adapter.

The current `batch`/`durable` feature names may migrate in a separate mechanical
commit. Cargo features never grant runtime authority.

## Dotlink dependency source

Do not commit a dependency such as `path = "../aw/crates/aw-tools"`; that makes a
dotlink checkout non-standalone.

Initial cutover should use an exact pinned Git revision of the AW repository for
`aw-tools` and the platform backend crate(s), recorded in `Cargo.lock` and the
Nix build inputs as needed. Differential development may temporarily use local
Cargo patching outside committed configuration.

If cross-product release coupling becomes operationally painful, move the same
shared crates to a dedicated repository. Do not vendor them back into dotlink.

## Migration phases

### Phase 0 — seal cross-repo implementation contract

- record this architecture in both repos;
- fold `job_list(offset,limit)`, exact crash-private artifact ownership and
  lexical-prefix-preserving symlinked discovery roots into the canonical DL-150
  spec before Rust implementation;
- refresh AW's canonical spec snapshot after that dotlink plan commit;
- build one parity matrix from the current AW and dotlink implementations;
- no semantic/code changes in this phase.

### Phase 1 — mechanical AW package rename

In one reviewable semantic-no-op change:

- `aw-tools-core` -> `aw-tools`;
- current `aw-tools` -> `aw-tools-adapter`;
- update manifests/imports/reexports/tests;
- preserve features and behavior exactly;
- update AW root/surface/UI consumers to the adapter where they require AW
  integration;
- update FS backends to depend on the new canonical `aw-tools` package;
- get the pre-rename test/lint/cross-build matrix green before proceeding.

Do not combine semantic fixes with this rename.

### Phase 2 — make the adapter genuinely semantic-free

Move remaining canonical behavior out of `aw-tools-adapter`, especially:

- complete grep scanner/context/paging;
- binary range normalization/planning;
- patch orchestration that is not AW journal policy;
- canonical result fitting/status formatting;
- shared schema argument validation;
- common mutation/lifecycle planning.

Keep only AW compatibility translation, authority, journal, process/provider/UI
projection and configuration.

### Phase 3 — path/capability convergence

- introduce the prepared operation-class path/capability boundary;
- implement final-component symlink semantics safely;
- preserve lexical + resolved authorization;
- remove single-hard-link restrictions from canonical regular-file semantics;
- preserve no-cross-mount/reparse hardening;
- admit valid quoted UTF-8 Unix control names;
- keep invalid-UTF8 `path_encoding` behavior;
- add native race/alias tests.

### Phase 4 — mutation/lifecycle/private-state convergence

- finish shared write/edit/patch/binary publication semantics;
- finish cp/mv/rm against the canonical contract;
- replace prefix hiding with exact private artifact records;
- add restart reconciliation for tombstones/backups/stages;
- add crash/failure injection around every reservation/stage/publication/cleanup
  boundary;
- verify hard-link semantics and case-only/cross-device behavior.

### Phase 5 — shared JobManager convergence

Do not add a third manager. Consolidate existing useful AW task/job primitives and
dotlink `src/jobs.rs` behavior into the one `aw-tools` manager:

- Graph + immutable NodePlan;
- journal transition proposals;
- Job registry/admission;
- runner permits/width;
- native conflict gate;
- captured semantic budgets;
- replay claims;
- output store/writer leases/raw paging/ACK;
- cancellation/revocation;
- Task projection;
- restart;
- paged `job_list`.

Delete/supersede overlapping Task/Job state machines as each behavior migrates;
do not layer a new manager over both old ones.

### Phase 6 — AW host binding

Bind the manager to AW's existing:

- Actor/source/site/epoch grants;
- TurnJournal CAS;
- Session owner;
- Process/Process Actor custody;
- existing secured filesystem root;
- provider result projection.

Only after claim-before-effect, terminal-before-dependent, reattachment,
revocation, retention/output and replay tests pass may AW advertise model-visible
`batch` / `job_*`.

### Phase 7 — dotlink adapter and differential consumer

**Current checkpoint (2026-11-09):** AW `4a23e92a8f516971bb232b98e4fc000554bcc4ba` is the reviewed, pushed shared implementation, and dotlink consumes that exact remote Git revision. The baseline includes shared output-owner/file-spool/exposure APIs, narrowing-only host runtime limits, atomic scheduler-policy publication and pre-effect native Graph footprint revalidation. A final-symlink/topology change after Graph preparation now fails before `Start` rather than executing under a stale deterministic conflict footprint. AW's host-neutral fixture, native/minimal/stateless/no-std gates and affected strict Clippy are green; fresh Windows cross validation is environment-blocked by a dangling cached Nix target sysroot before AW source compilation. Dotlink production native filesystem dispatch uses the shared `request`/`direct` engine and multi-root `CapabilitySet` for all 13 tools; duplicate native filesystem implementations are removed. The production Graph surface (`batch`, Graph-shaped `job_start`, `job_restart`) executes native and ShellNodes through the shared `JobManager`; dotlink retains transport delivery/replay, process/sandbox custody, wall-clock retention and MCP projection. The remote-pinned offline/locked build proves 628/628 active main unit tests plus 1 intentional external-fixture ignore, all five integration binaries, strict all-target/all-feature Clippy/check and rustfmt/diff green. The final lifecycle/replay ownership pass also commits shared `Start` before OS spawn, treats missing canonical state for shared-owned host entries as split-brain, retains shutdown metadata/spools when cleanup is not provably safe, and revalidates the exact committed inner response representation before cached terminal shell delivery. The independently generated 13-schema/13-request fixture passes the explicit cross-repository ignored gate. No sibling path is committable. AW `4a23e92` is pushed, all three dotlink shared dependencies point at that exact remote Git revision, Cargo regenerated the lockfile from Git, and full no-override validation is complete. Remaining gates are platform-specific Bubblewrap and native Windows/macOS execution checks.

The implemented thin dotlink filesystem adapter now:

- translates the already-authorized finite `AccessPolicy` snapshot into one
  `CapabilitySet`:
  - one `PreparedGrant` per canonical read/write grant root;
  - duplicate rights for one root share one host-owned Registry object;
  - one exact `private_artifact::Registry` per write-capable root; operations
    needing only transient stages/backups may use volatile custody, but `rm` is
    advertised only after a durable Store is prepared under dotlink's protected
    private state;
  - the dotlink `PrivateMutationStore` must use a stricter durability path than
    generic profile writes: a reservation succeeds only after the record file and
    containing directory are durably synced. A post-rename directory-sync failure
    is an error before filesystem detach even if it leaves a harmless orphan record;
  - place mutation records under a stable child of dotlink's existing protected
    `$XDG_STATE_HOME/abird/dotlink` control-plane root (for example
    `mutation-artifacts-v1/`). `protected_paths_for_config()` already protects that
    whole state root through `oauth::state_dir()`, including under broad writable-home
    grants, so do not add a model-visible workspace exception or reuse Job spool paths;
  - both lexical and resolved `deny_read` / `deny_write` / protected identities
    become read/write `ExclusionSpec` entries so recursive shared walkers cannot
    traverse authority holes;
- calls `reconcile_private_artifacts()` before exposing filesystem tools after
  startup/restart;
- preserves exact/case-sensitive Windows grant authority; invariant case folding
  is namespace lock/overlap safety only and never widens a grant;
- after `resolve_content_path()` follows an explicitly permitted final symlink,
  re-run dotlink lexical/resolved deny/protected authorization on the returned
  canonical absolute model path before payload IO;
- uses `/` for unrestricted Unix; for unrestricted Windows, prepares only the drive
  roots involved in the already-authorized request. Windows UNC/network-share
  paths use the shared canonical `//server/share/...` spelling; native Windows
  execution CI must verify that route on a real Windows filesystem before release;
- maps dotlink's existing runtime ceilings directly onto shared host-neutral policy:
  - `max_read_bytes` / `max_write_bytes` remain product ceilings inside the shared
    64 MiB read/mutation safety maxima;
  - `SchemaCaps.binary_data_bytes` is an **encoded JSON input-length** ceiling, so
    bwrite/bedit must still enforce dotlink's configured decoded `max_write_bytes`
    after base64/hex decode (choose a schema encoded ceiling large enough for the raw
    profile, e.g. hex may require 2x raw bytes);
  - direct presentation uses `min(max_output_bytes, authenticated request result
    budget when present)` before canonical rendering. The current OpenAI tunnel
    injects a fixed trusted result ceiling above the maximum 8 MiB profile ceiling,
    but the adapter must not rely on that incidental ordering for future transports;
    raw Job paging must fit the complete serialized response without changing
    raw-byte offsets;
  - dotlink's 64 MiB/stream and up-to-8 GiB aggregate Job spool policy fits the shared
    safety maxima; use `attach_shell_output_files(...)` with protected host-opened
    spool files rather than switching large logs to memory;
  - `job_retention_secs` remains a dotlink clock/timer policy and calls shared
    `JobManager::expire_terminal(id)` after expiry. Expiry must not recreate or bypass
    replay state; Busy/live/replay-owned Jobs remain fail-closed;
  - direct/native, one-node shell and multi-node Graph execution use dotlink's existing transport, retention and spool ceilings; Graph `NodePlan`s capture per-node semantic budgets and `--serialize` maps to width 1 without a second scheduler;
- projects shared semantic results into compact MCP output without reimplementing
  filesystem traversal, mutation, lifecycle, Job retention or paging semantics;
- keeps the legacy-vs-shared differential machinery test-only; canonical filesystem requests, security decisions, statuses, paging and mutation/fault regressions are green against the reviewed AW head;
- uses production native-Graph preparation/execution for `batch`, `job_start` and `job_restart`, and uses typed Job output page requests/render state rather than another wide compatibility API;
- on host shutdown, terminal shared file-backed output is expired through the shared manager before the private spool directory is removed; an explicit regression covers unacknowledged retained output and handle closure.

### Phase 8 — dotlink Graph/Job cutover and duplicate deletion

Completed foundations:

- [x] register shared canonical filesystem schemas/operations;
- [x] bind the shared `JobManager` to dotlink shell replay/transport/process hosting for canonical lifecycle, output ownership/ACK, exposure and spool quota;
- [x] retire duplicate filesystem/search/read/edit/patch/binary/lifecycle implementations after parity;
- [x] keep product-specific policy/sandbox/process-custody/transport/TTL code outside the shared crate.

Completed model-facing Graph cutover:

- [x] Graph-shaped `job_start(ops,...)` is production-live while direct background shell requests retain the one-ShellNode compatibility path;
- [x] model-visible `batch` uses the same shared Graph preparation/execution path;
- [x] `job_restart` performs fresh current authority/config/schema/interpreter revalidation;
- [x] native filesystem nodes execute through the shared `request`/`direct` engine and ShellNodes through the existing dotlink process/sandbox host, with canonical shared `NodePlan`, footprint, condition and conflict semantics;
- [x] `--serialize` maps to Graph width 1 without a second scheduler;
- [x] retain transport-delivery replay, process custody, TTL and MCP projection as dotlink host responsibilities rather than moving them into the shared semantic manager;
- [ ] ensure release/Nix/static build remains standalone and reproducible after the exact Git pin is advanced.

### Phase 9 — portability and acceptance

Required before calling convergence complete:

- Linux native tests and adversarial races;
- native Windows NTFS ACL/reparse/hardlink/case-only/ReplaceFile recovery tests;
- native macOS ACL/xattr/no-follow/rename tests;
- mount/reparse boundary tests;
- Bubblewrap/runtime tests where namespace support exists;
- cross-process replay race/crash tests;
- private-artifact crash recovery tests;
- Graph outer-budget invariance tests;
- deterministic native conflict-order tests;
- cancellation-stop-unknown tests;
- raw output/ACK/multi-owner tests;
- strict Clippy/rustfmt;
- AW full workspace/CLI/Agent gates;
- dotlink full cargo/Nix/transport/replay gates.

## Reuse acceptance criteria

Convergence is complete only when:

1. AW and dotlink import the same `aw-tools` implementation revision.
2. No product adapter contains an independent implementation of a canonical tool
   or Job state machine.
3. Unix/Windows native filesystem semantics have one implementation each.
4. The shared JobManager is the only Graph/Job/Task execution-lifecycle state
   machine used by both products.
5. AW still has one original Agent/ToolPolicy/TurnJournal/Process authority.
6. Dotlink still has one AccessPolicy/transport replay/sandbox authority.
7. Direct calls and Graph nodes produce the same canonical semantic outcomes for
   the same prepared operation.
8. Cross-platform and failure-injection gates above are green.
9. The old dotlink implementation is deleted rather than retained as fallback.

## Explicit non-goals

- Do not move AW Agent policy into `aw-tools`.
- Do not move dotlink AccessPolicy/OAuth/tunnel code into `aw-tools`.
- Do not create a shared persistent workflow database.
- Do not make Cargo features permissions.
- Do not force every direct native call to allocate a retained Job.
- Do not share process-launch policy by weakening AW or dotlink behavior.
- Do not preserve two implementations as a permanent fallback.
- Do not convert dotlink to a Cargo workspace only for naming symmetry.

## Review findings incorporated from current AW

The latest AW implementation already resolved several gaps that older planning
must not re-open:

- requested-root `.gitignore` isolation;
- malformed/unavailable/over-budget ignore coverage reporting;
- source-glob slash/basename semantics;
- first-record result-ceiling progress;
- invalid-UTF8 `path_encoding` reporting;
- atomic grep hit+context result units;
- write-only canonical write not probing equality;
- extracted secured Unix/Windows backend crates;
- pure Graph/Job transition projection and exact-source task primitives.

Remaining high-value convergence work is therefore the final-link/authority
boundary, hard-link semantics, exact private artifact recovery, adapter thinning,
full shared JobManager, host bindings and dotlink cutover—not reimplementing the
already-correct shared pieces.
