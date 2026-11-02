# Sekiro Together — LRM fork notes

This repository (`Lroumian/sekiro-coop` on GitHub) is the **LRM** (long-running maintenance) line of [Tihvert/sekiro-coop](https://github.com/Tihvert/sekiro-coop) (Sekiro Together).

Design (rev2): `docs/superpowers/specs/2026-11-02-lrm-baseline-and-remote-appearance-design.md`  
Plan: `docs/superpowers/plans/2026-11-02-lrm-baseline-remote-appearance.md`

## Remotes

| Remote | URL |
|---|---|
| `origin` | `https://github.com/Lroumian/sekiro-coop.git` (this fork) |
| `upstream` | `https://github.com/Tihvert/sekiro-coop.git` |

```bat
git remote add upstream https://github.com/Tihvert/sekiro-coop.git
git fetch upstream
```

## Branches

| Branch | Role |
|---|---|
| `main` | Tagged LRM releases only |
| `develop` | Integration |
| `fix/*` / `perf/*` | Topic work (e.g. `fix/remote-appearance`) |
| `upstream-sync/*` | Optional short-lived merge branches |

Do not drive-by patch `main`. Land on `develop`, smoke, then release-cut to `main`.

## Version identity

- Upstream string stays a **beta**, e.g. `MOD_VERSION = "1.0.8-beta"`.
- LRM patch: `LRM_VERSION = "1.0.8-beta+lrm.N"` (UI + session log).
- Public zip (no `+` in the filename): `sekiro-together-lrm-1.0.8-beta-lrm.N.zip`.
- After upstream `1.0.9-beta`: rebase/merge, then `1.0.9-beta+lrm.1`.

Never ship a bare `1.0.8-lrm.1` label (reads as stable).

## Sync from upstream

1. `git fetch upstream`
2. Branch `upstream-sync/…` from `develop` (or merge `upstream/main` into `develop`).
3. Resolve conflicts; re-run LRM patches / tests.
4. Merge to `develop`. Do not invent a second root history.

## Cut an LRM release

1. On `develop` (or release branch): version bump `LRM_VERSION` / docs as needed.
2. `test.cmd` (coop-net + coop-audit).
3. `cargo test -p sekiro_coop --lib` (DLL unit tests; needs MSVC env like the `.cmd` files).
4. `build-public.cmd` — uses `--no-default-features --features dev-native` (no `testsync` in the public DLL).
5. Dual-machine smoke for the milestone checklist (e.g. `docs/LRM-TEST-appearance.md`).
6. Merge to `main`, tag, attach the zip. Prefer PR-back to upstream when the fix is not LRM-specific.

## Implementer rules (short)

No guessed RVA/offsets. Diagnose before behaviour change. One concern per commit. Phase work stays phased — do not mix death/Boss/world Flag into an appearance patch.
