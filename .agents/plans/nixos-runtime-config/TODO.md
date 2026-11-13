# NixOS declarative runtime config

Base: `c34ea5ab115e2841384de349eda5ca9cea661b48`

## Committed

### `1bbbe57` — Stage declarative config outside shell runtime mounts

- [x] Reproduce store-backed config startup failure with shell enabled: exit 78,
  runtime `/nix/store` mount overlaps protected config.
- [x] Stage declarative config in a private per-unit runtime directory.
- [x] Add default/named unit assertions and unchanged external/saved-config checks.
- [x] Add a NixOS VM test for real shell execution, protected config/credentials,
  restart refresh, and stop cleanup.
- [x] Document lifecycle and remaining explicit `configFile` behavior.

## Uncommitted

None.

## Tests and verification

- [x] Baseline `nix flake check -L` — `c34ea5a`.
- [x] `nix flake check -L` on x86_64-linux — `1bbbe57`: build, Rust tests,
  clippy, formatting, Markdown, module evaluation, and pinned-QEMU VM regression.
- [x] `DOTLINK_TEST_BWRAP=1 cargo test --locked --all-features` — `1bbbe57`:
  781 passed.
- [x] `git diff --check` — `1bbbe57`.

## Review follow-up — 2026-11-13

- [x] Separate `nixosTests.dotlink-service` from aggregate checks; dedicated CI step.
- [x] Stage explicit store-backed base/default and named sibling config files.
- [x] Preserve non-store paths and use systemd-native argument escaping.
- [x] Correct authorship and co-author metadata.
- [x] Expand VM coverage to generated/store-backed default/named units.
- [x] Aggregate checks and two-node VM regression passed on x86_64-linux.
- [ ] Mark ready after upstream CI is green.
