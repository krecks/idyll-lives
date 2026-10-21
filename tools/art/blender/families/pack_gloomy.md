# Gloomy pack — notes

`pack_gloomy.py` builds the six P7 gloomy models into `web/public/assets/models/pack_gloomy/` and their entries in
`packs/pack_gloomy/manifest.json`, an override pack listed in the base manifest. Built 2026-10-14; they sat in
Validate › Gloomy until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`),
backs on the wall's face (−0.43; the tub's ledge −0.93). Seats keep the default 0.45 (the cello's stool, the desk's
chair, the window seat's cushion top); the tub keeps `SEATS`/`BEDS` 0.25 (its inside floor is at 0.095).

| Model | Moving part and clip | Played by |
|---|---|---|
| `recordCabinet` | the lid over the turntable's well, hinged along its back: `lid_open` lifts it 80° (0.9 s) | `listen`, `newAlbum` |
| `recordCabinet` | the platter and record, origin on the spindle: `platter_spin`, one turn clockwise in 1.8 s (33 rpm), a loop | `listen`, `newAlbum` (loop) |
| `cello` | the cello, built upright on its stand, origin on the end pin's foot: `off_stand` (0.9 s) moves it across to between the player's knees and leans it back 14° | `practise`, `lament` |
| `journalDesk` | the tall candle's flame, built full size on its wick and scaled to nothing: `candle` lights it (0.6 s); its material glows at night | `journal`, `poetry` |
| `rainLamp` | the rain, 18 lines with drops from the canopy to the pool, squashed to nothing under the canopy: `rain` runs them down (0.6 s) | `watchRain` |
| `rainLamp` | the ripples on the pool: `ripple` swells them 12 % and back (2 s), ambient (`idle`) | always (not on Low) |
| `windowSeat` | the seat cushion with its throw and a book, origin on the underside: `sink_in` presses them 2 cm (0.6 s) | `brood`, `doze` |
| `soakingTub` | the water up to 0.52 with a little foam, origin under the tub's floor, flat there at rest: `fill` raises it (1.5 s) | `soak` |

The music animation poses the player round the model's own cello (characters.ts, `def.includes('cello')`), so the
cello is a part of the model and `off_stand` brings it to where the pose expects it (the old part's spot, end pin
at depth 0.3). The old part's music stand is gone (it stood where the cello is played); an open book of music
lies on the floor and the bow hangs on the cello's stand.

The rain lamp is an open frame (a canopy on three posts over a pool, a little figure under an umbrella on a rock),
not a glass column: no `glass` draw. The plan's "A rain (E2)" is the ambient `ripple` on the pool; the rain itself
runs only while watched, as the table's `rain` clip says.

The window seat's pane is opaque gloss with raindrops and runs on it, not see-through glass: it stands against the
house wall, which glass would show.

**Draw calls: 3 each, the record cabinet 4.** The cabinet's walnut (`wood`) body, its lid and the platter loop (the
plan's extra "L the platter turns (E1)"): 4, `check-glb`'s one warning. Its lid pull is wood, not brass, so the
lid stays one draw. Elsewhere: the cello's body and stand `wood`, the cello itself one plain part; the desk `wood`,
the chair's upholstery plain satin; the window seat all plain but its cushion part (`fabric` throw and cushion, a
satin book); the tub's ledge `wood`, the rest plain.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.gloomy.recordCabinet` | 0.90 × 0.92 × 0.81 m | 2,552 | 4 | Body, Trim |
| `model.gloomy.cello` | 0.78 × 1.43 × 0.74 m | 3,924 | 3 | Body, Trim, Cello |
| `model.gloomy.journalDesk` | 1.50 × 1.20 × 0.83 m | 2,768 | 3 | Body, Trim |
| `model.gloomy.rainLamp` | 0.80 × 1.50 × 0.62 m | 5,334 | 3 | Body, Trim |
| `model.gloomy.windowSeat` | 1.84 × 1.97 × 0.71 m | 3,668 | 3 | Body, Cushion, Trim |
| `model.gloomy.soakingTub` | 1.86 × 0.92 × 1.85 m | 4,120 | 3 | Body, Trim |

The desk's apron runs only along its far edge and its ends (the old part's deep apron took the writer's hands).

`check-glb`: 6 models, 0 failed; the record cabinet's 4-draw warning above. Khronos validator: 0 errors, 0
warnings each. Gallery next to the old parts; clips bench at rest, open and mid-loop on WebGPU and WebGL,
coloured (cabinet, window seat). In play (`wants-asking`, WebGPU): the cello played (in the player's arms), the
record played (lid up, record turning), brooding on the window seat, writing at the desk by the lit candle, watching
the rain, soaking in the tub. At night (`office-night`, WebGL): the window seat in four facings, the tub, the
cabinet, the desk and the lamp. Not seen: the cello in the night yard (no free spot for its facing), `doze` on the
window seat.

## Rebuild

```
cargo art pack_gloomy            # the six .glb, the pack, check-glb
cargo art pack_gloomy --preview  # previews/<id>.png, <id>@<clip>.png
```
