# Couch potato pack — notes

`pack_couchPotato.py` builds the six P3 couch potato models into `web/public/assets/models/pack_couchPotato/` and
their entries in `packs/pack_couchPotato/manifest.json`, an override pack listed after
`packs/couchPotato/manifest.json`. Their primitive-part placeholders (`tools/art/pack_couchPotato.py`, in
`packs/couchPotato`) stay as the fallback. Built 2026-10-14; they sit in Validate › Couch Potato until checked in play.

Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`).
As on the placeholders, the screens the residents watch (the cinema's, the console's) stand at the front edge facing
the seats, their oak backs to the room: their curtains and tray show from behind and from above.

| Model | Moving parts and clip | Played by |
|---|---|---|
| `cinemaWall` | the left and right curtains over the picture, each with its origin at its outer end: `curtains` gathers both to their sides (scaled to 22 % across, 1.2 s) | `marathon`, `channelSurf` |
| `gameConsole` | the disc tray with a disc on it: `tray` slides it 9 cm out of the console towards the players (0.6 s), `hold` 1.5 s, so it goes back in | `play`, `speedrun` |
| `recliner` | the back (with its pillows), hinged at the seat's back: `recline` tips it 11° back; the footrest, folded under the seat's front, rises 80° in the same clip (1.2 s) | `recline`, `doze` |
| `snackFridge` | the door with its takeout screen and handle, hinged on its right edge: `fridge_open` swings it 100° (0.9 s); cans, a takeout box and a pizza inside | `snack`, `takeout` |
| `fitnessGame` | the balance board on its rocker, on the mat: `board` tilts it 10° to one side (0.8 s) | `workout` |
| `lavaLamp` | the wax in the glass: `blobs` lifts three blobs 14 cm (2.5 s, slow); `drift` turns two more round the glass once in 8 s, all the time near the camera (`idle`) | `zoneOut`; ambient |

**Draw calls: 3 each, the lava lamp 4.**
- The cinema has two moving parts (the curtains, `fabric`), so its still body is all plain: the recliners are satin
  leather, the screen's oak back plain satin.
- The console's loveseat is the body's `fabric`, the tray plain.
- The recliner has two moving parts (back, footrest): all plain satin (microsuede), no textured finish.
- The fridge's padded lid is the body's `fabric`, the door plain.
- The workout game's mat and towel are the body's `fabric`, the board plain.
- The lava lamp: the still body plain plus its `glass` (the lamp's vessel: the blobs show through it), the rising
  wax one part, the drifting wax another: 4 draws, the extra one for the ambient drift (E2) the plan asks for; the
  pouf is plain satin to keep it at 4. `check_glb` warns "4 draw calls, aim for 3 at most": that is the ambient part.

Other warnings left, by design:
- the recliner's depth (0.76 m, placeholder 0.89 m): the placeholder has its footrest out at rest; here it is
  folded and rises in `recline`. The recliner stands 3 cm further from the wall than the placeholder's back, so the
  back tips without going into it (its top comes to about the wall's face);
- the snack fridge's width (0.58 m, placeholder 0.69 m): the placeholder's bowl of crisps hangs half off the lid;
  here it sits on it.

The plan's "A drifting blobs (E2)" is built (`drift`). The lamp does not glow at night (no `glow`, not asked); a glow
on the wax or glass would be one more draw at night.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.couchPotato.cinemaWall` | 1.88 × 1.52 × 0.91 m | 4,960 | 3 | Body, Upholstery, Curtains |
| `model.couchPotato.gameConsole` | 0.89 × 1.34 × 0.90 m | 3,556 | 3 | Body, Upholstery, Cushion |
| `model.couchPotato.recliner` | 0.94 × 0.95 × 0.76 m | 2,324 | 3 | Body, Upholstery, Throw |
| `model.couchPotato.snackFridge` | 0.58 × 0.54 × 0.54 m | 2,800 | 3 | Body, Upholstery |
| `model.couchPotato.fitnessGame` | 0.87 × 1.15 × 0.88 m | 2,732 | 3 | Body, Mat, Board |
| `model.couchPotato.lavaLamp` | 0.85 × 1.15 × 0.89 m | 2,896 | 4 | Body, Upholstery, Glass |

The painted materials: the cinema `cinema_frame` (the console between the seats), `leather`, `curtain`; the console
`console`, `gamer`, `mustard`; the recliner `plinth`, `plush`, `throw`; the fridge `mint`, `lid`; the workout game
`fit_stand`, `fit_mat`, `board`; the lamp `lamp_metal`, `pouf`, `vessel`.

`check_glb.py`: 6 models, 0 failed, the 3 warnings above. Khronos validator: 0 errors, 0 warnings each. Checked: the
gallery next to the placeholders; the clips bench at rest and open on WebGPU (the cinema and console from behind),
colours on WebGPU and WebGL; in play (`wants-asking`, WebGPU by day): every use, by the household's resident in
turn: the cinema (seated, the curtains drawn apart and closed after), the recliner (footrest up, back tipped), the
fridge (fetched), the console (seated, playing), the workout game (standing on the mat) and the lava lamp (seated,
the wax up, back down after); the recliner turned to four facings and a turned cinema in the yard; `office-night`
on WebGL (the lava lamp used at night, its glass and wax drawn).

## Rebuild

```
python3 -B tools/art/blender/build.py pack_couchPotato            # the six .glb, the pack, check_glb
python3 -B tools/art/blender/build.py pack_couchPotato --preview  # previews/<id>.png, <id>@<clip>.png
```
