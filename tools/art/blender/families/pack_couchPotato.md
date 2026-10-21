# Couch potato pack — notes

`pack_couchPotato.py` builds the six P3 couch potato models into `web/public/assets/models/pack_couchPotato/` and
their entries in `packs/pack_couchPotato/manifest.json`, a pack listed in the base manifest's `packs` (the
trait's only model manifest). Built 2026-10-14; they sat in Validate › Couch Potato until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`).
The cinema wall and the game console stood 2–3 cm into the wall; they are centred on their `Model` `centre` now
(`G.settle()`), their backs on the wall face at −0.43, as the wall-standing ones' old parts stood, so the model
and the wall agree within a centimetre.
The screens the residents watch (the cinema's, the console's) stand on the wall at the back of the footprint, the
seats in front of them, the sitters facing the object's back (content `"seat": {"height": 0.45, "forward": 0.05,
"facing": "back"}` for the cinema, `"forward": 0.12` for the console, as the piano and the writer's bureau). Each is
built as the old part first stood (the seats at the back, the screen at the front edge facing them, its oak back to
the room) and turned round (`_turn_round`), so its oak back is on the wall and the console's tray slides out the other
way (+Z game, towards the players); the old parts were turned the same way. The cinema's hips stay where they were on its seats (0.05 is the old 0 turned round);
the console's sit 8 cm further back than before, against the loveseat's back, so the knees stay under the shelf.

| Model | Moving parts and clip | Played by |
|---|---|---|
| `cinemaWall` | the left and right curtains over the picture, each with its origin at its outer end: `curtains` gathers both to their sides (scaled to 22 % across, 1.2 s) | `marathon`, `channelSurf` |
| `gameConsole` | the disc tray with a disc on it: `tray` slides it 9 cm out of the console towards the players, away from the wall (0.6 s), `hold` 1.5 s, so it goes back in | `play`, `speedrun` |
| `recliner` | the back (with its pillows), hinged at the seat's back: `recline` tips it 11° back; the footrest, folded under the seat's front, rises 80° in the same clip (1.2 s); the sitter rides the back (`ride: recline`) | `recline`, `doze` |
| `snackFridge` | the door with its takeout screen and handle, hinged on its right edge: `fridge_open` swings it 100° (0.9 s); cans, a takeout box and a pizza inside | `snack`, `takeout` |
| `fitnessGame` | the balance board on its rocker, on the mat: `board` tilts it 10° to one side (0.8 s) | `workout` |
| `lavaLamp` | the wax in the glass: `blobs` lifts three blobs 14 cm (2.5 s, slow); `drift` turns two more round the glass once in 8 s, all the time near the camera (`idle`). The glass and the wax glow at night with the content's `light` (2 m, at 0.85 m over the side table), switched as a lamp (M-packs-07). The side table stands at the pouf's front right and the pouf is turned 50° to face it, its sitter too (content `places`), coming from its left (2026-10-21: the lamp stood behind the sitter's shoulder, I-packs-11) | `zoneOut`; ambient |

**Draw calls: 3 each, the lava lamp 4.**
- The cinema has two moving parts (the curtains, `fabric`), so its still body is all plain: the recliners are satin
  leather, the screen's oak back plain satin.
- The console's loveseat is the body's `fabric`, the tray plain.
- The recliner has two moving parts (back, footrest): all plain satin (microsuede), no textured finish.
- The fridge's padded lid is the body's `fabric`, the door plain.
- The workout game's mat and towel are the body's `fabric`, the board plain.
- The lava lamp: the still body plain plus its `glass` (the lamp's vessel: the blobs show through it), the rising
  wax one part, the drifting wax another: 4 draws, the extra one for the ambient drift (E2) the plan asks for; the
  pouf is plain satin to keep it at 4. `check-glb` warns "4 draw calls, aim for 3 at most": that is the ambient part.

Other warnings left, by design:
- the recliner's depth (0.76 m, old part 0.89 m): the old part had its footrest out at rest; here it is
  folded and rises in `recline`. The recliner stands 3 cm further from the wall than the old part's back, so the
  back tips without going into it (its top comes to about the wall's face);
- the snack fridge's width (0.58 m, old part 0.69 m): the old part's bowl of crisps hung half off the lid;
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
| `model.couchPotato.lavaLamp` | 0.90 × 1.15 × 0.89 m | 2,896 | 4 | Body, Upholstery, Glass |

The painted materials: the cinema `cinema_frame` (the console between the seats), `leather`, `curtain`; the console
`console`, `gamer`, `mustard`; the recliner `plinth`, `plush`, `throw`; the fridge `mint`, `lid`; the workout game
`fit_stand`, `fit_mat`, `board`; the lamp `lamp_metal`, `pouf`, `vessel`.

`check-glb`: 6 models, 0 failed, the 3 warnings above. Khronos validator: 0 errors, 0 warnings each. Checked: the
gallery next to the old parts; the clips bench at rest and open on WebGPU (the cinema and console from behind),
colours on WebGPU and WebGL; in play (`wants-asking`, WebGPU by day): every use, by the household's resident in
turn: the cinema (seated, the curtains drawn apart and closed after), the recliner (footrest up, back tipped), the
fridge (fetched), the console (seated, playing), the workout game (standing on the mat) and the lava lamp (seated,
the wax up, back down after); the recliner turned to four facings and a turned cinema in the yard; `office-night`
on WebGL (the lava lamp used at night, its glass and wax drawn).

## Rebuild

```
cargo art pack_couchPotato            # the six .glb, the pack, check-glb
cargo art pack_couchPotato --preview  # previews/pack_couchPotato/<id>.png, <id>@<clip>.png
```
