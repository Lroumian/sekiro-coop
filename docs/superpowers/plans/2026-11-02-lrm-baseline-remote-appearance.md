# LRM Baseline + Remote Appearance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the LRM fork baseline (`1.0.8-beta+lrm.1`) and ship a safe `remote_player_appearance` control that completes Native/Ghost de-glow without guessing RVA/offsets or breaking online ChrType semantics.

**Architecture:** Keep upstream `MOD_VERSION`; add `LRM_VERSION`. Product look is controlled by a new ini key when present; otherwise legacy `native_solid`/`ghost_solid` stay unchanged. Native solid vtable apply follows a classify-then-maybe-write state machine; Native shader clear is gated on LIVE-verified layout dump vs Local/Ghost. No Phase 2 work.

**Tech Stack:** Rust cdylib (`sekiro_coop`), existing me3 load path, `test.cmd` + `cargo test -p sekiro_coop`, `build-public.cmd`.

**Spec:** `docs/superpowers/specs/2026-11-02-lrm-baseline-and-remote-appearance-design.md` (rev2)

## Global Constraints

- No guessed RVA, offsets, vtable slots, or function signatures
- Do not change NetworkManipulator / network ChrType / TeamType in LRM.1
- No runtime writes to `chr+0x1F88`
- No user-facing `subtle`; modes are `normal` | `phantom` only
- Version form: internal `1.0.8-beta+lrm.1`; zip `sekiro-together-lrm-1.0.8-beta-lrm.1.zip`
- If `remote_player_appearance` absent → preserve legacy `native_solid`/`ghost_solid` exactly
- Periodic vptr **check** OK; **write** only `stock PlayerIns → our solid`; `other` → log+skip
- Write vptr only if remote still in player set, PGD/peer identity matches, not stale/delete
- Native shader clear only after read-only layout verification; else slot-130 only + `RE_REQUIRED`
- No protocol VERSION change; no Phase 2
- Prefer small commits; one concern per commit

## Review Focus

1. **Ini without new key** — must keep old glow/solid behaviour; unit-test legacy path unchanged.
2. **Ini with `remote_player_appearance=normal` and conflicting `native_solid=0`** — new key wins; log `appearance=new`.
3. **Native vptr class `other`** — never write; diagnostic only.
4. **Native shader before LIVE verify** — must not call Ghost offsets as writes; only read dumps.
5. **Public build** — `testsync` feature off; zip name includes `beta-lrm`.

## File map

| Path | Role |
|---|---|
| `docs/LRM.md` | Remotes, branches, version, sync/release |
| `docs/LRM-TEST-appearance.md` | Dual-machine checklist |
| `docs/RE_REQUIRED-remote-appearance.md` | Created only if Native shader unverified or glow remains |
| `crates/coop-dll/src/appearance.rs` | Enum, resolve from ini source, vptr classify helpers (pure + thin) |
| `crates/coop-dll/src/tuning.rs` | Atomically store appearance control source + mode |
| `crates/coop-dll/src/settings.rs` | Parse new key / legacy; default ini comment; unit tests |
| `crates/coop-dll/src/ui.rs` | Show LRM version with upstream beta |
| `crates/coop-dll/src/log.rs` | Session header includes `LRM_VERSION` |
| `crates/coop-dll/src/native.rs` | Safe apply / probe / diagnostics hooks |
| `crates/coop-dll/src/ghost.rs` | Product-mode gate; shared shader helper only after gate |
| `crates/coop-dll/src/puppet.rs` | Wire appearance ticks / F11 dump if needed |
| `crates/coop-dll/src/lib.rs` | `mod appearance;` |
| `build-public.cmd` / `ship-public/HOW-TO-PLAY.txt` | Zip naming + LRM note |

---

### Task 1: Git LRM baseline + `docs/LRM.md`

**Files:**
- Create: `docs/LRM.md`
- Modify: git remotes/branches only (no Rust yet)

**Interfaces:**
- Produces: documented `upstream`, `develop`, `fix/remote-appearance` workflow

- [ ] **Step 1: Add upstream remote if missing**

```bash
git remote add upstream https://github.com/Tihvert/sekiro-coop.git
git fetch upstream
git remote -v
```

Expected: `origin` = `Lroumian/sekiro-coop`, `upstream` = `Tihvert/sekiro-coop`.

- [ ] **Step 2: Create `develop` from current `main` and topic branch**

```bash
git checkout main
git checkout -b develop
git checkout -b fix/remote-appearance
```

Do not force-push `main`. Push when partner asks.

- [ ] **Step 3: Write `docs/LRM.md`** covering: remotes, branch roles, version `1.0.8-beta+lrm.N`, sync-from-upstream checklist, cut-release checklist (`test.cmd`, `cargo test -p sekiro_coop`, `build-public.cmd`), note that public builds use `--no-default-features --features dev-native`.

- [ ] **Step 4: Commit**

```bash
git add docs/LRM.md
git commit -m "docs: add LRM fork workflow and version policy"
```

---

### Task 2: `LRM_VERSION` identity in UI + session log

**Files:**
- Modify: `crates/coop-dll/src/ui.rs` (near `MOD_VERSION`)
- Modify: `crates/coop-dll/src/log.rs` (`session_header`)
- Modify: `ship-public/HOW-TO-PLAY.txt` first line
- Modify: `build-public.cmd` zip naming if it only reads `MOD_VERSION`

**Interfaces:**
- Produces: `pub const LRM_VERSION: &str = "1.0.8-beta+lrm.1";` (or `lrm.1` patch component + format helper)
- Display: keep `MOD_VERSION = "1.0.8-beta"`; overlay/log show combined identity including LRM

- [ ] **Step 1: Add `LRM_VERSION` and a small pure formatter**

```rust
pub const LRM_VERSION: &str = "1.0.8-beta+lrm.1";
pub fn version_label() -> String { format!("sekiro_coop v{MOD_VERSION} ({LRM_VERSION})") }
```

Prefer keeping `MOD_VERSION` exactly `"1.0.8-beta"`.

- [ ] **Step 2: Unit test** `version_label` contains both `1.0.8-beta` and `lrm.1` (put test in `ui.rs` `#[cfg(test)]` or `appearance` later — here in `ui.rs`).

- [ ] **Step 3: Run** `cargo test -p sekiro_coop version_label -- --nocapture`  
  Expected: PASS

- [ ] **Step 4: Wire overlay status line + `session_header` to include `LRM_VERSION` / `version_label()`** without removing `COOP_GIT_HASH` / build epoch.

- [ ] **Step 5: Update `build-public.cmd` zip base name** to `sekiro-together-lrm-1.0.8-beta-lrm.1.zip` (parse `LRM_VERSION` or hardcode from constants — must not produce `1.0.8-lrm.1` without beta). Update `HOW-TO-PLAY.txt` header accordingly.

- [ ] **Step 6: Commit**

```bash
git commit -m "chore: add LRM_VERSION 1.0.8-beta+lrm.1 to UI, log, and public zip name"
```

---

### Task 3: Appearance enum + settings resolve (TDD)

**Files:**
- Create: `crates/coop-dll/src/appearance.rs`
- Modify: `crates/coop-dll/src/lib.rs` (`mod appearance;`)
- Modify: `crates/coop-dll/src/tuning.rs`
- Modify: `crates/coop-dll/src/settings.rs` (parse + default template + tests)

**Interfaces:**
- Produces:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppearanceMode { Normal, Phantom }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppearanceSource { New, Legacy }

pub struct AppearanceControl {
    pub source: AppearanceSource,
    pub mode: AppearanceMode, // meaningful when source == New
}

pub fn resolve_appearance(
    remote_player_appearance: Option<&str>, // None = key absent; Some("") treat as present-empty → warn+Normal
    native_solid: bool,
    ghost_solid: u8,
) -> AppearanceControl;
```

- When `source == Legacy`, callers must keep using `native_solid` / `ghost_solid` as today (do not invent a collapsed enum).
- Tuning stores: `APPEARANCE_SOURCE` (`0=legacy,1=new`), `APPEARANCE_MODE` (`0=normal,1=phantom`), leave existing `NATIVE_SOLID`/`GHOST_SOLID` updated as today for legacy/debug.

- [ ] **Step 1: Write failing tests in `appearance.rs`**

Names/assertions:
- `absent_key_is_legacy` — `resolve_appearance(None, true, 1).source == Legacy`
- `normal_key_is_new_normal` — `Some("normal")` → `New` + `Normal`
- `phantom_key` — `Some("phantom")` → `New` + `Phantom`
- `invalid_key_warns_to_normal` — `Some("subtle")` or `"nope"` → `New` + `Normal` (and settings warn path covered in settings tests)
- `new_key_ignores_native_solid_false` — `Some("normal")` with `native_solid=false` still `New`+`Normal` (product mode; runtime apply uses mode, not legacy flag)

- [ ] **Step 2: Run** `cargo test -p sekiro_coop resolve_appearance -- --nocapture`  
  Expected: FAIL (module/fn missing)

- [ ] **Step 3: Implement `resolve_appearance` + tuning stores + `settings` parse**

Use existing `Ini::get` / `choice`. Rules from spec:
- key present → `AppearanceSource::New`; valid `normal`/`phantom`; invalid → warn + `Normal`
- key absent → `Legacy`
- Default **new** ini template: include `remote_player_appearance = normal` under `[DISPLAY]` with comment that omitting the key keeps old `native_solid`/`ghost_solid` behaviour for existing files — **do not** rewrite existing user inis to insert the key automatically on upgrade (avoids silent look change). Only brand-new generated defaults get the key.

- [ ] **Step 4: Settings unit tests** for parse of sample ini strings (present/absent/invalid) and assert tuning atomics / control source log string helper `appearance_source_label() -> &'static str` (`"new"` / `"legacy"`).

- [ ] **Step 5: Run** `cargo test -p sekiro_coop --lib`  
  Expected: PASS for appearance/settings tests (full lib OK preferred)

- [ ] **Step 6: Commit**

```bash
git commit -m "feat: add remote_player_appearance normal|phantom with legacy fallback"
```

---

### Task 4: Native safe solid apply + diagnostics (no new offsets)

**Files:**
- Modify: `crates/coop-dll/src/native.rs` (`finish_spawn`, tick path)
- Modify: `crates/coop-dll/src/appearance.rs` (optional pure `classify_vptr(vptr, stock, solid) -> VptrClass`)
- Modify: `crates/coop-dll/src/puppet.rs` only if F11 dump is centralized there

**Interfaces:**
- Consumes: `AppearanceControl` / tuning source+mode; existing `solid_vtable()`, player-set helpers, PGD fields on `Remote`
- Produces: `fn classify_player_ins_vptr(vptr: usize, stock: usize, solid: usize) -> VptrClass` with `Stock | Solid | Other`
- Produces: apply that writes **only** on `Stock` when lifecycle OK; logs `REMOTE_LOOK` on change / F11 / ≤1/10s

- [ ] **Step 1: Unit test** `classify_player_ins_vptr` for stock/solid/other.

- [ ] **Step 2: Implement safe apply used from `finish_spawn` and a low-frequency recheck**

When `AppearanceSource::New` + `Normal`: attempt solid apply under state machine.  
When `New` + `Phantom`: do **not** install solid vtable (leave stock).  
When `Legacy`: keep today’s `native_solid()` gate exactly.

Lifecycle gates before write (use existing APIs where possible):
- chr readable
- in player set
- peer/PGD identity still matches `Remote`
- not in delete/stale path you already use for remotes

- [ ] **Step 3: Read-only probe** logging `chr_type`, `team` (`+0x74` already used in `log_checks`), phantom row via existing slot-130 call pattern, draw group via existing look/model reads — **no new offsets**. Include `vptr_class`, `mode`, `source`.

- [ ] **Step 4: Read-only Native vs Local vs Ghost asm/`+0xD8` compare** using **only** offsets already used in `ghost::solid_shader_tick` (`ASM_MODEL=0x2150`, `ASM_GHOST_SHADER=0xD8`). Log values; do **not** write on Native yet. Set a tuning/atomic `NATIVE_SHADER_LAYOUT_LIVE_OK` only if partner later confirms; default false in LRM.1 code unless dump shows Local and Native both have readable asm and Ghost matches — still treat as UNVERIFIED until live session marks it; safest default: **never write Native shader in this task**.

- [ ] **Step 5: Run** `cargo test -p sekiro_coop --lib`  
  Expected: PASS

- [ ] **Step 6: Commit**

```bash
git commit -m "feat: safe Native solid vtable apply and REMOTE_LOOK diagnostics"
```

---

### Task 5: Ghost product-mode wiring + optional shared shader helper gate

**Files:**
- Modify: `crates/coop-dll/src/ghost.rs`
- Modify: `crates/coop-dll/src/puppet.rs` (ghost_solid / make_solid call sites)

**Interfaces:**
- Consumes: appearance source/mode
- When `New`+`Normal`: run existing `make_solid` + `solid_shader_tick` as today for solid look
- When `New`+`Phantom`: skip make_solid / shader tick (stock ghost look)
- When `Legacy`: unchanged `ghost_solid()` behaviour
- Shared Native shader helper: **only** if Task 4 left an explicit verified flag; otherwise skip and write `docs/RE_REQUIRED-remote-appearance.md`

- [ ] **Step 1: Gate puppet/ghost call sites** on appearance control (no new RE).

- [ ] **Step 2: If Native shader still UNVERIFIED**, create `docs/RE_REQUIRED-remote-appearance.md` with the spec’s write-BP checklist (ChrAsmModel+0xD8 writer, Local vs Native compare, no `+0x1F88` belt).

- [ ] **Step 3: Write `docs/LRM-TEST-appearance.md`** dual-machine steps (Bell/Charm, set `normal`, F11 both sides, map change, death/rejoin; expect no glow or attach RE_REQUIRED dumps).

- [ ] **Step 4: Run** `cargo test -p sekiro_coop --lib` and `test.cmd`  
  Expected: PASS

- [ ] **Step 5: Commit**

```bash
git commit -m "feat: wire ghost appearance modes and LRM appearance test/RE docs"
```

---

### Task 6: Public build verification + stop

**Files:** none required unless zip script tweaks from Task 2 failed

- [ ] **Step 1: Run** `test.cmd`  
  Expected: coop-net + coop-audit PASS

- [ ] **Step 2: Run** `cargo test -p sekiro_coop --lib`  
  Expected: PASS

- [ ] **Step 3: Run** `build-public.cmd`  
  Expected: zip under `dist/public/` named like `sekiro-together-lrm-1.0.8-beta-lrm.1.zip`; DLL built with `--no-default-features --features dev-native`

- [ ] **Step 4: Report only** (no Phase 2): files changed; test results; public build OK; VERIFIED/UNVERIFIED/RE_REQUIRED; live test steps from `LRM-TEST-appearance.md`; stop.

- [ ] **Step 5: Commit** any leftover doc/script fixes; do not tag `main` until dual-machine smoke signed off by partner.

---

## Spec coverage checklist

| Spec item | Task |
|---|---|
| upstream/develop/topic + LRM.md | 1 |
| `1.0.8-beta+lrm.1` + zip name | 2 |
| `remote_player_appearance` + legacy priority | 3 |
| Safe Native vtable state machine + REMOTE_LOOK | 4 |
| Native shader gate / RE_REQUIRED | 4–5 |
| Ghost under product mode | 5 |
| Live checklist | 5 |
| Public build / testsync off | 6 |
| No Phase 2 / no `+0x1F88` / no user `subtle` | all |
