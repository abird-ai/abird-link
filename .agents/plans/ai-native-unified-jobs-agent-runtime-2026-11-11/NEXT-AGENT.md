# Next agent — unified Jobs + Agent runtime is finalized

Use `dotlink---l5` only for source work.

Canonical repos:

- AW: `/home/pvl/spaces/abird/src/aw`
- dotlink: `/home/pvl/spaces/abird/src/dotlink`

Do not use A1. Do not create another worktree. Never inspect or touch AW
`.agents/plans/ux/**`. Preserve unrelated dotlink `docs/CHATGPT_PLUGIN.md` and
the separate `dotlink-016-fixes` release worktree.

## Authority / exact pin

Normative design authority:

`890e9a383fe2edc3e0bd0aa34a2b217f2724475c`

Final shared implementation pin consumed by dotlink:

`0a6c6a0bea07a3886c4dc8be4f423eec3ea3c134`

`aw-tools`, `aw-tools-fs-unix`, and `aw-tools-fs-win` all pin that exact Git
revision. No sibling path or local digest copy remains.

## Status

Runtime Checkpoints 0/A/B/C/D/E/F/G and dotlink Checkpoint 3 are complete.

Final dotlink qualification against the exact pin:

- 651 active tests passed / 1 intentional ignored fixture;
- integrations 1/1, 8/8, 1/1, 12/12, 4/4;
- strict all-target/all-feature Clippy green;
- all-target/all-feature locked check green;
- fmt/diff green.

Final replay-first review guarantees exact Start redelivery reaches replay claim
before mutable Graph preparation, shell-policy checks, or cancel revision checks.

Live model-selection qualification used x2 inference-only ROCm Ollama
`qwen3.8:27b` with the exact current AW 23-tool catalog: 30/30 first-call
selection/schema/task validity, plus Bash Jobs continuation 3/3 and Agent
synchronous-result continuation 3/3.

## Next work

There is no remaining unified-runtime implementation task. Revalidate live Git
state before any future change. Release/tag/deploy or platform certification are
separate scopes and require explicit authorization.
