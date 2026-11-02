# LRM live test — remote appearance (`1.0.8-beta+lrm.1`)

**TEST REQUIRED** (dual-machine). Unit tests cannot prove glow.

## Setup

1. Both players: same LRM build (`build-public.cmd` or `build-dist.cmd`).
2. Ini: set the same password; under `[DISPLAY]` set:

   ```ini
   remote_player_appearance = normal
   ```

3. Optional A/B later: `phantom` on one session to confirm stock look returns.

## Steps

1. Host Kindred Bell, joiner Kindred Charm; meet in an open area (e.g. Dilapidated Temple courtyard).
2. Both press **F11**; save `%LOCALAPPDATA%\sekiro-coop\log.txt`.
3. Confirm visually: friend opaque, **no white glow**, equipment matches, lock-on and hits work.
4. Warp / load to another idol together; look again; F11 again.
5. One player dies and resurrects (or rejoins); look again.

## Log expectations

- Session header contains `1.0.8-beta+lrm.1` / `LRM_VERSION`.
- `REMOTE_LOOK` / `REMOTE_LOOK_PROBE` with `source=new` `mode=Normal`.
- Prefer `vptr_class=stock->solid` or solid; `other` → attach logs, do not force-fix.
- If still glowing with solid vtable active → follow `docs/RE_REQUIRED-remote-appearance.md` (write BP), do not ship more field guesses.

## Pass / fail

| Check | Pass |
|---|---|
| No white glow under `normal` | |
| Equipment unchanged | |
| Damage / lock-on / sync OK | |
| Survives map change | |
| Survives death / rejoin | |

If any visual fail with solid installed → **RE_REQUIRED**, not a silent “fixed” release.
