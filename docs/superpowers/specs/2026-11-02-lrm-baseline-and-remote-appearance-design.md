# Sekiro Together LRM — Baseline + Remote Appearance Design (rev2)

Date: 2026-11-02  
Revision: **rev2** (supersedes rev1 in this file)  
Status: **rev2 approved** (partner 2026-11-02) — implementation plan next  
Scope: Milestone 0 (engineering baseline) + Milestone 1 (remote player appearance) **only**  
Repo: GitHub `Lroumian/sekiro-coop` (in-place LRM; approach A)  
Do **not** start Phase 2 in this workstream.

### Partner answers / review (2026-11-02)

- Glow still broken with current defaults → M1 is a completeness fix, not an ini rename.
- Role split: partner owns architecture / RE direction / acceptance; Codex implements under strict RE rules (no guessed RVA/offset/semantics).
- Working style: **verify → complete → converge**; reuse `stuck`, `world`, `ghost`/`native`, `netai` — do not rewrite.
- Rev1 risk feedback incorporated below (vtable write policy, Native shader verification gate, no `+0x1F88` belt in LRM.1, beta-preserving version, no user-facing `subtle`, legacy config priority, diagnostics, Verification Matrix).

### Codex / implementer hard rules (every task)

**Forbidden**

- Guessing RVA, offsets, vtable slots, or function signatures
- Clearing flags / random state to “fix” symptoms
- Large refactors of unverified RE code
- One commit that mixes independent problems
- Sleep/delay to hide races
- Client forcing host-authoritative game state to paper over desync

**Required**

- Read existing RE docs first; reuse hooks/abstractions
- Mark unverified facts `INFERRED` / `UNVERIFIED` / `RE REQUIRED`
- New RVA/offset only with version + prologue/signature guard
- Diagnose before changing behaviour
- Pure logic → unit tests; live-only items → explicit `TEST REQUIRED`
- Report: files changed, assumptions, RE gaps, live test steps

---

## Intent

Long-running maintenance (LRM) line for Sekiro Together. First ship: repeatable fork/build/release, proven by making **Native Remote** look like a normal player (no white glow), as a real display setting — without breaking NetworkManipulator / ChrType / TeamType online semantics.

Upstream: `Tihvert/sekiro-coop`. Upstream-worthy fixes → PR back; LRM-only stays on the fork.

## Goals

### Milestone 0 — LRM baseline

- `upstream` remote → `Tihvert/sekiro-coop`
- Branches: `main` (tagged LRM releases), `develop` (integration), `fix/*` / `perf/*`
- Version identity keeps **upstream beta** + LRM patch (see Versioning)
- Short `docs/LRM.md`: sync-from-upstream + cut-release checklists
- `test.cmd` + `build-public.cmd` work; public zip has **no** `testsync`
- Session log: upstream-base identity, LRM version, build time / git SHA as available
- No protocol VERSION change for M0/M1

### Milestone 1 — Remote appearance

- Product setting (user-facing modes **only** these two):

  ```ini
  [DISPLAY]
  remote_player_appearance = normal
  ```

  Allowed values: `normal` | `phantom`  
  Default when key present / new installs: `normal`

- Primary path: Native Remote `PlayerIns`  
- Secondary: ReplayGhost fallback must follow the same product mode when under new-key control
- Keep NetworkManipulator; **do not** change network ChrType / TeamType unless later LIVE-VERIFIED safe (out of LRM.1 default path)
- Acceptance (both machines): opaque, no white glow; equipment unchanged; damage / lock-on / pos / anim unchanged; survives map change; survives death + resurrect / remote rejoin

## Non-goals

- Phase 2+ (death/spectate, enemy rewrite, world objects, story, perf program, 3–6, campaign gate)
- Forgejo move / repo rename
- Shipping regulation.bin as primary fix
- Protocol bump for appearance
- User-facing `subtle` in LRM.1
- Runtime writes to `chr+0x1F88` in LRM.1

## Anti-goals

- Drive-by patches on `main`
- Unconditional periodic vptr restore onto unknown vtables
- Guessing Native `ChrAsmModel` layout from Ghost alone
- Masking remaining glow with extra field writes instead of RE
- Silently changing look for users who never set the new key

## Constraints

- Steam Sekiro **1.06** + me3; Rust stable MSVC workspace
- Build on existing: `native::solid_vtable`, `ghost::make_solid` / `solid_shader_tick`, `native_solid` / `ghost_solid`
- MIT fork code; `vendor/sekiro` NON-AI unchanged
- Same-LRM-build wire compat; cross-play with stock upstream best-effort

## Approaches

| | Summary | Result |
|---|---|---|
| Repo A | In-place on `Lroumian/sekiro-coop` | **Chosen** |
| Appearance A1 | Runtime slot-130 (+ VERIFIED shader) | **Chosen** |
| A2 regulation / A3 ChrType rewrite | — | Rejected as primary |

---

## Versioning (rev2)

Keep upstream beta identity visible.

| Context | Form |
|---|---|
| Internal / UI / log | `1.0.8-beta+lrm.1`, then `…+lrm.2` |
| After upstream `1.0.9-beta` | `1.0.9-beta+lrm.1` |
| Zip filename (no `+`) | `sekiro-together-lrm-1.0.8-beta-lrm.1.zip` |

Prefer **`LRM_VERSION`** (or equivalent) alongside upstream `MOD_VERSION`, so upstream string stays honest if still `"1.0.8-beta"`. Combined display: `Co-op v{MOD_VERSION}+lrm.N` or explicit `LRM_VERSION` in overlay + session header.  
**Do not** ship bare `1.0.8-lrm.1` (reads as stable).

---

## Config compatibility (rev2)

| Situation | Behaviour |
|---|---|
| `remote_player_appearance` **present** (any valid value) | New key is **sole product** control for remote look. Log `appearance=new mode=…`. Legacy `native_solid` / `ghost_solid` do **not** override product mode; they may remain as DEBUG/diagnostics only if still useful, but must not fight the new enum. |
| Key **absent** | **Preserve** pre-LRM behaviour of `native_solid` / `ghost_solid` exactly. Log `appearance=legacy`. No silent default to “always normal”. |
| Invalid new value | Fallback documented (prefer `normal` + warn, or refuse and stay legacy — pick one in impl and unit-test; recommend: warn + `normal` when key present but bad). |

Do **not** try to losslessly compress every old `ghost_solid` 0/1/2 × `native_solid` bool combo into one enum.

`subtle`: may exist as an internal/reserved Rust variant **only** if needed for future; **not** in default ini, UI copy, or user docs for LRM.1.

---

## Code diagnosis (unchanged facts)

| Today | Gap |
|---|---|
| Native `CHR_TYPE_REMOTE = 1` | Keep for net semantics; strip look without type rewrite |
| `finish_spawn` solid vtable once, only if stock PlayerIns | Stock restore / race → skipped forever; needs **safe** re-apply policy (below), not blind rewrite |
| Ghost: make_solid + per-frame shader tick | Native has no shader path yet |
| Public build includes `dev-native` | Not a feature-flag miss |

---

## Native appearance safety state machine (rev2)

```text
NativeAppearance::apply(remote)   // also used by periodic CHECK

confirm remote ptr readable
    ↓
confirm still in WorldChrMan player set
    ↓
confirm PGD / peer identity matches creation
    ↓
confirm not stale / delete / despawn lifecycle
    ↓
classify vptr:
  stock PlayerIns  → may replace with our solid copy
  our solid copy   → OK, no write
  any other        → LOG + SKIP  (never write)
    ↓
read-only model / asm probe
    ↓
if ChrAsmModel+ghost-shader layout is LIVE VERIFIED or STATIC VERIFIED for Native:
    shader clear (shared helper only after verification)
else:
    skip shader write; emit / update RE_REQUIRED
```

### Periodic check vs periodic force

- Periodic **classification + diagnostics**: allowed (low frequency / on change).
- Periodic **write**: only when classification is **stock PlayerIns** and lifecycle checks pass.
- Never overwrite **other** vtables to “fix” appearance.

### `chr+0x1F88` forced-row

**Not in LRM.1 runtime code.** Document only under RE_REQUIRED if slot-130 + verified shader still leave glow.

### Native shader clear gate

Before sharing `solid_shader_tick` with Native:

1. Read-only compare Local Player, Native Remote, ReplayGhost: model ptr, asm ptr, `+0xD8` (or documented field).
2. Confirm same `ChrAsmModel` shape / offset as Ghost path (`chr+0x2150`, `+0xD8`, dirty `+0x108`) — STATIC from RE-FINDINGS-2 for Ghost/PlayerIns draw path is necessary but **not sufficient** until Native live dump matches.
3. Only then extract shared helper.
4. If unconfirmed: LRM.1 ships **slot-130 / solid vtable only** for Native + RE_REQUIRED; **no guessed offsets**.

Ghost path may keep existing make_solid + shader tick (already in tree for ReplayGhost).

---

## Diagnostics (rev2)

Emit on state change, F11, or low-frequency probe — **never every frame**.

Suggested fields:

```text
REMOTE_LOOK:
  source=new|legacy
  mode=normal|phantom|legacy-native_solid|…
  peer=
  kind=native|ghost
  chr=
  pgd_slot=
  vptr_class=stock|solid|other
  chr_type=
  team=
  phantom_row=
  local_phantom_row=
  model=
  asm=                    # when probed
  ghost_shader=           # only after VERIFIED for that kind
  lifecycle=
```

---

## Verification Matrix (rev2)

Classify every address/field used in M1:

| Item | Class | Notes |
|---|---|---|
| PlayerIns stock vtable / clone + slot 130 override | STATIC VERIFIED (in-tree) | `native::solid_vtable`, RE-FINDINGS-2 § Solid ghost (2) |
| `ghost_phantom_row` → local player slot 130 | STATIC VERIFIED (in-tree) | |
| ReplayGhost slot 248 → PlayerIns false | STATIC VERIFIED | `ghost::make_solid` |
| ReplayGhost `ChrAsmModel` at `chr+0x2150`, shader `+0xD8`, dirty `+0x108` | STATIC VERIFIED (RE + ghost tick) | |
| Native Remote same asm/shader offsets | **RE REQUIRED → LIVE VERIFIED** before write | Read-only dump first |
| `CHR_TYPE_REMOTE = 1` kept | LIVE practice / product constraint | Do not change in LRM.1 |
| TeamType left alone | Product constraint | |
| Glow gone under `normal` after M1 | **LIVE VERIFIED** (acceptance) | Dual-machine |
| `chr+0x1F88` forced row | INFERRED / RE REQUIRED | No LRM.1 code |
| Who rewrites Native vptr back to stock | INFERRED | Diagnose via vptr_class transitions |
| True ApplyPhantomAppearance writer if still glowing | RE REQUIRED | Write BP on shader/material |

If Normal still glows after slot-130 + VERIFIED shader:

1. Write breakpoint on the relevant shader/material value  
2. Capture RIP + call stack  
3. Diff Local vs Native (PhantomParam row, ChrType, TeamType, draw group, material)  
4. Patch only after that path is known + guarded  

---

## Branch model

```text
Tihvert/sekiro-coop
        │
Lroumian/sekiro-coop
  ├─ main
  ├─ develop
  ├─ upstream-sync/*
  ├─ fix/remote-appearance
  └─ …
```

## Milestone deliverables

**M0:** remotes/branches doc, version policy, sync/release checklist, tests + public build green, testsync out of public zip.

**M1:** new ini key (`normal`/`phantom` only), safe Native apply state machine, Ghost under same product mode when `appearance=new`, diagnostics, `docs/LRM-TEST-appearance.md`, `docs/RE_REQUIRED-remote-appearance.md` if gaps remain, merge to `develop` after checklist; `main` + tag after dual-machine smoke.

## Out-of-scope roadmap (later designs only)

| Phase | Focus |
|---|---|
| 2 | Death / resurrect / black screen — **diagnose first**, no blind fix |
| 3 | Boss/enemy audit + authority |
| 3.5 | Difficulty presets; Dynamic Aggro only after target-setter RE |
| 4 | WorldObject (lifts/levers) beyond flag sync |
| 5 | WorldPhase / NPC classification |
| 6 | Module profiling on existing stall hooks |
| 7–8 | Multi-client / campaign gate |

## Success criteria

- Contributor can follow `docs/LRM.md` on this fork
- Build shows `1.0.8-beta+lrm.1` (or equivalent) without looking “stable 1.0.8”
- Dual-machine `normal`: no white glow, acceptance list, or honest RE_REQUIRED with dumps — not silent “fixed”
- Upstream beta can still merge without a second history

## Files expected for implementation (after rev2 sign-off)

| File | Likely change |
|---|---|
| `docs/LRM.md` | new — remotes, version, sync/release |
| `docs/LRM-TEST-appearance.md` | new — live checklist |
| `docs/RE_REQUIRED-remote-appearance.md` | new if Native shader / residual glow |
| `docs/superpowers/specs/2026-11-02-…-design.md` | this rev2 (done) |
| `crates/coop-dll/src/ui.rs` | LRM version display |
| `crates/coop-dll/src/log.rs` (or session header site) | version / SHA / build time |
| `crates/coop-dll/src/settings.rs` | new key + legacy priority + tests |
| `crates/coop-dll/src/tuning.rs` | appearance mode storage |
| `crates/coop-dll/src/native.rs` | safe apply / classify / diagnostics |
| `crates/coop-dll/src/ghost.rs` | shared helper **only after** verification gate; Ghost path wiring |
| `crates/coop-dll/src/puppet.rs` | call sites / F11 dump hooks if needed |
| `build-public.cmd` / `ship-public/HOW-TO-PLAY.txt` | zip name + LRM wording |
| Possibly small new `appearance.rs` | enum + classify + apply (optional, prefer small) |

## Rev1 → rev2 diff (summary)

1. Vtable: periodic **check** OK; **write** only stock→solid; other→log/skip; lifecycle guards.  
2. Native shader: read-only verify vs Local/Ghost before shared tick; else slot-130 only + RE_REQUIRED.  
3. No `+0x1F88` runtime belt in LRM.1.  
4. Version: `1.0.8-beta+lrm.1` / zip `…-1.0.8-beta-lrm.1.zip`.  
5. User modes: `normal` \| `phantom` only (no user `subtle`).  
6. Config: new key present → product control; absent → legacy unchanged; log source.  
7. Rich `REMOTE_LOOK` diagnostics on change/F11/low-freq.  
8. Verification Matrix + Codex hard rules; Phase 2 explicitly out.

## Risks remaining

- Slot-130 alone may not kill glow → must not invent fields; stop at RE_REQUIRED.  
- Mis-classifying vptr during delete → mitigated by player-set/PGD/lifecycle checks.  
- Legacy users without new key keep old look (including glow) until they set `normal` — intentional.  
- Partner still owns RE write-BP campaign if RE_REQUIRED fires.
