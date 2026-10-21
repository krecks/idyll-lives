# Neat pack — notes

`pack_neat.py` builds the six P13 neat models into `web/public/assets/models/pack_neat/` and their entries in
`packs/pack_neat/manifest.json`, an override pack listed in the base manifest. Built 2026-10-14; they sat in
Validate › Neat until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`),
keeping its footprint, its wall side (backs on the wall face at depth −0.43; the old parts' were at −0.455) and
the desk chair's seat at 0.45 m. The entry console and the rain shower tower stood 2 cm into the wall; they are
centred on their `Model` `centre` now (`G.settle()`), backs on the face, as the wall-standing ones' old parts
stood, so the model and the wall agree within a centimetre.

| Model | Moving part and clip | Played by |
|---|---|---|
| `steamMopDock` | the mop (pole, tank, head, pad), its origin at its head's foot: `mop_off` hides it (0.2 s) while the game draws it in the resident's hands (content `held`, pushed over the floor) | `mop` |
| `laundryCenter` | the washer's porthole door, hinged on its left: `door_open` swings it 100° (0.8 s), `hold` 2.5 s: open for the washing to go in or come out, shut for the wash and the folding (2026-10-21) | `laundry`, `fold` |
| `rainShowerTower` | streaks of rain from the square head, built full length, origin at the head: `water_on` grows them down (0.6 s); the glass door across the middle of the front, hinged on its right: `door_open` swings it 95° out (0.8 s), `hold` 1.5 s and `out` 1.5 s: it lets the bather in, shuts behind them and opens again to let them out (2026-10-21: it was part of the still body, walked through) | `steamShower` |
| `organizerDesk` | the middle bin of the hutch's lower row, with cards in it: `drawer_open` slides it 13 cm out towards the sitter (0.6 s). Since 2026-10-21 the desk stands with its back on the wall, the hutch on the desk's back open to the sitter and the room, the chair in front (seat 0.45 at depth 0.25, the hips at 0.21: `furniture.json` `back`): it stood the other way round, the hutch's back and the modesty panel to the room (M-packs-05) | `sort`, `ledger` |
| `robotVacuumDock` | the robot, origin at its middle: `vacuum_round` (a loop, `G.path`, 5 s a round at 0.25 m/s) drives it 18 cm out of the dock, once round a 14 cm circle in front of it (to the left, clear of the plant), facing the way it goes, and back in, round after round while it is programmed (2026-10-21: it peeked 60 cm out and back) | `program` (loop) |
| `entryConsole` | the key tray drawer under the top (front, knob, tray, keys): `tray` slides it 20 cm out (0.6 s). The mirror and the hook board hang on a birch back panel rising from the top (posts and a cap), so they stand on something free of a wall too (2026-10-21, M-packs-01; Q-06) | `tidy` |

The mop is not hidden while in use: the `clean` animation holds no prop, so it stays out of its dock, leaning, and
goes back when the use ends. The robot's round trip is the clip played open and back: out and turned, held 2 s,
and home.

**Draw calls: 3 each, the shower 4.**
- Plain finishes everywhere and at most one textured finish per still body: the mop dock's spare pads on the
  floor are `fabric` (its refill shelf and the mop's own pad plain); the laundry centre's towels and shirts are
  `fabric` (its counter top plain birch); the desk and the console are textured `birch`; the robot's rug is
  `fabric`. Each moving part is plain.
- The shower: plain plus its glass on the still body, the rain plain, the door glass: 4 (Q-05 allows a fourth draw for a shower door; the door has no pull, to stay one draw). Its bench is plain teak and its towels
  satin, so no textured finish.
- The portholes, the robot dock's bin window and the mop dock's panel are plain gloss (no glass draw); the
  console's mirror is `chrome`, as on the old part (not live mirror glass, which would be a draw of its own).

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.neat.steamMopDock` | 0.79 × 1.25 × 0.71 m | 2,152 | 3 | Body, Mop |
| `model.neat.laundryCenter` | 1.82 × 1.74 × 0.62 m | 6,286 | 3 | Body, Washer & dryer |
| `model.neat.rainShowerTower` | 0.90 × 2.18 × 0.90 m | 2,528 | 4 | Body, Bench |
| `model.neat.organizerDesk` | 1.60 × 1.25 × 0.90 m | 4,064 | 3 | Body, Wood, Chair |
| `model.neat.robotVacuumDock` | 0.86 × 0.52 × 0.86 m | 2,100 | 3 | Body, Trim, Rug |
| `model.neat.entryConsole` | 0.87 × 1.81 × 0.32 m | 4,188 | 3 | Body, Wood |

The plan's colour part is "Body" for all six: the white frames and docks (`body`), the robot's white dock and
robot (`appliance`) and the shower's tiles (`tile`).

`check-glb`: 6 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each.
On the clips bench (WebGPU and WebGL): every clip at rest and open, the desk's bin from behind (the sitter's side).
Gallery: each next to its old part. In play (`wants-asking`, WebGPU, each bought in the living room and used by
the resident): the mop comes out of its dock (`mop`), the washer's door opens (`laundry`), the rain runs over the
resident (`steamShower`), the resident sits at the desk (`sort`), the robot rolls out and home (`program`) and the
key tray drawer opens and shuts (`tidy`). At night (`office-night`, WebGL): the desk at all four facings in the
yard, the laundry centre, mop dock, robot, shower and console coloured.

## Rebuild

```
cargo art pack_neat            # the six .glb, the pack, check-glb
cargo art pack_neat --preview  # previews/pack_neat/<id>.png, <id>@<clip>.png
```
