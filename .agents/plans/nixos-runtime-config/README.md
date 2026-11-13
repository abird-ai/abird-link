# NixOS declarative runtime config

Author: @bioinformatist
Co-authors: OpenAI Codex
Owner: @bioinformatist

## Problem

- Declarative `settings` point `DOTLINK_CONFIG` into `/nix/store`.
- Shell preparation rejects the overlap between its store runtime mount and
  the protected active config (exit 78).

## Design

- Copy generated JSON and store-backed `configFile` into a per-unit systemd `RuntimeDirectory` at startup.
- Keep directories `0700`, config files `0400`, and named-profile filenames.
- Let systemd refresh on restart and remove on stop.
- Leave non-store `configFile`, saved profiles, credentials, and sandbox guards
  unchanged; store source JSON remains public and must not contain secrets.
- Keep real secrets on `LoadCredential`. Do not put JSON there: its aggregate
  1 MiB limit is below dotlink's 4 MiB config ceiling.

- Expose the x86_64-linux VM regression separately as `nixosTests.dotlink-service`;
  CI runs it explicitly, while `nix flake check` keeps the eval-only module check.

## References

- `nix/module.nix`
- `nix/tests/module.nix`
- `nix/tests/service.nix`
- `.agents/docs/nixos-service.md`
