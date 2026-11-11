# NixOS declarative runtime config

Base: `c34ea5ab115e2841384de349eda5ca9cea661b48`

## Uncommitted

- [x] Reproduce store-backed config startup failure with shell enabled: exit 78,
  runtime `/nix/store` mount overlaps protected config.
- [x] Stage declarative config in a private per-unit runtime directory.
- [x] Add default/named unit assertions and unchanged external/saved-config checks.
- [x] Add a NixOS VM test for real shell execution, protected config/credentials,
  restart refresh, and stop cleanup.
- [x] Document lifecycle and remaining explicit `configFile` behavior.

## Tests and verification

- [x] Baseline `nix flake check -L` on the base revision.
- [ ] Final `nix flake check -L` (build, tests, clippy, formatting, Markdown,
  module evaluation, and the new VM check).
- [x] `DOTLINK_TEST_BWRAP=1 cargo test --locked --all-features`: 781 passed.
- [x] `git diff --check`.
- [x] NixOS VM service regression using locally available QEMU 11.0.1;
  the exact flake QEMU 11.1.1 check remains pending its binary-cache download.
