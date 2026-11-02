# RE REQUIRED — Remote appearance (LRM.1)

Status: **open** after LRM.1 code. Native solid vtable (slot 130) + Ghost `make_solid` / `solid_shader_tick` are in tree. Native **shader clear is read-only probed only** — not LIVE VERIFIED as safe to write.

## Why this file exists

If dual-machine play with `remote_player_appearance = normal` still shows white glow / phantom shading on the **Native** remote after slot-130 solid vtable is active (`REMOTE_LOOK` / `REMOTE_LOOK_PROBE` in the log), **do not guess more fields**. Especially do **not** write `chr+0x1F88` in product code.

## Already STATIC VERIFIED (do not re-litigate)

| Item | Where |
|---|---|
| PlayerIns slot 130 → PhantomParam apply | `docs/RE-FINDINGS-2.md` § Solid ghost (2); `native::solid_vtable` |
| ReplayGhost slot 248 + slot 130 + `ChrAsmModel+0xD8` | same; `ghost::make_solid` / `solid_shader_tick` |
| `chr+0x2150` / `+0xD8` / dirty `+0x108` on Ghost path | `ghost.rs` |

## LIVE REQUIRED before Native shader write

1. Log pairs already emit `REMOTE_LOOK_PROBE` with Native vs Local `asm` / `ghost_shader` — confirm both sides show readable asm and whether Native `+0xD8` is non-zero while Local is 0.
2. If Native `ghost_shader` is non-zero while solid vtable is installed: set a **write breakpoint** on that remote’s `ChrAsmModel+0xD8` (or the material value that actually differs).
3. Capture **RIP + call stack** of the writer.
4. Diff Local Player vs Native Remote: PhantomParam row (slot 130), ChrType (`+0x70`), TeamType (`+0x74`), draw group (`model+0x23`), material/shader state.
5. Identify the real appearance update path; patch only with version/prologue guards.
6. Only then share a shader-clear helper with Ghost.

## Explicitly out of LRM.1 runtime

- Forced PhantomParam row via `chr+0x1F88`
- ChrType / TeamType changes for “look”
- Periodic overwrite of **other** (non-stock, non-solid) vtables

## Log strings to collect

- `REMOTE_LOOK:`
- `REMOTE_LOOK_PROBE:`
- `native: solid look` / `REMOTE_LOOK: ... vptr_class=stock->solid`
- F11 dumps both machines at the same wall time
