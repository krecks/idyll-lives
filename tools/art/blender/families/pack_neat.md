# Neat pack — notes

`pack_neat.py` builds the six P13 neat models into `web/public/assets/models/pack_neat/` and their entries in
`packs/pack_neat/manifest.json`, an override pack listed after `packs/neat/manifest.json`. Their primitive-part
placeholders (`tools/art/pack_neat.py`, in `packs/neat`) stay as the fallback. Built 2026-10-14; they sit in
Validate › Neat until checked in play.

Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`),
keeping its footprint, its wall side (backs on the wall face at depth −0.43; the placeholders' were at −0.455) and
the desk chair's seat at 0.45 m. The entry console and the rain shower tower stood 2 cm into the wall; they are
centred on their `Model` `centre` now (`G.settle()`), backs on the face. The placeholders of the wall-standing ones (`tools/art/pack_neat.py`, `pack_shapes.to_wall`) stand with their backs
on the wall face too, so the fallback, the model and the wall agree within a centimetre.

| Model | Moving part and clip | Played by |
|---|---|---|
| `steamMopDock` | the mop (pole, tank, head, pad), its origin at its head's foot: `mop_off` hides it (0.2 s) while the game draws it in the resident's hands (content `held`, pushed over the floor) | `mop` |
| `laundryCenter` | the washer's porthole door, hinged on its left: `door_open` swings it 100° (0.8 s) | `laundry`, `fold` |
| `rainShowerTower` | streaks of rain from the square head, built full length, origin at the head: `water_on` grows them down (0.6 s) | `steamShower` |
| `organizerDesk` | the middle bin of the hutch's lower row, with cards in it: `drawer_open` slides it 13 cm out towards the sitter (0.6 s) | `sort`, `ledger` |
| `robotVacuumDock` | the robot, origin at its middle: `vacuum_out` rolls it 60 cm out of the dock while turning it round (1.4 s), `hold` 2 s, then rolls it home | `program` |
| `entryConsole` | the key tray drawer under the top (front, knob, tray, keys): `tray` slides it 20 cm out (0.6 s) | `tidy` |

The mop is not hidden while in use: the `clean` animation holds no prop, so it stays out of its dock, leaning, and
goes back when the use ends. The robot's round trip is the clip played open and back: out and turned, held 2 s,
and home.

**Draw calls: 3 each.**
- Plain finishes everywhere and at most one textured finish per still body: the mop dock's spare pads on the
  floor are `fabric` (its refill shelf and the mop's own pad plain); the laundry centre's towels and shirts are
  `fabric` (its counter top plain birch); the desk and the console are textured `birch`; the robot's rug is
  `fabric`. Each moving part is plain.
- The shower: plain plus its glass on the still body, the rain plain: 3. Its bench is plain teak and its towels
  satin, so no textured finish.
- The portholes, the robot dock's bin window and the mop dock's panel are plain gloss (no glass draw); the
  console's mirror is `chrome`, as on the placeholder (not live mirror glass, which would be a draw of its own).

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.neat.steamMopDock` | 0.79 × 1.25 × 0.71 m | 2,152 | 3 | Body, Mop |
| `model.neat.laundryCenter` | 1.82 × 1.74 × 0.62 m | 6,286 | 3 | Body, Washer & dryer |
| `model.neat.rainShowerTower` | 0.90 × 2.18 × 0.90 m | 2,408 | 3 | Body, Bench |
| `model.neat.organizerDesk` | 1.60 × 1.25 × 0.83 m | 4,064 | 3 | Body, Wood, Chair |
| `model.neat.robotVacuumDock` | 0.86 × 0.52 × 0.86 m | 2,100 | 3 | Body, Trim, Rug |
| `model.neat.entryConsole` | 0.87 × 1.77 × 0.34 m | 3,756 | 3 | Body, Wood |

The plan's colour part is "Body" for all six: the white frames and docks (`body`), the robot's white dock and
robot (`appliance`) and the shower's tiles (`tile`).

`check_glb.py`: 6 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each.
On the clips bench (WebGPU and WebGL): every clip at rest and open, the desk's bin from behind (the sitter's side).
Gallery: each next to its placeholder. In play (`wants-asking`, WebGPU, each bought in the living room and used by
the resident): the mop comes out of its dock (`mop`), the washer's door opens (`laundry`), the rain runs over the
resident (`steamShower`), the resident sits at the desk (`sort`), the robot rolls out and home (`program`) and the
key tray drawer opens and shuts (`tidy`). At night (`office-night`, WebGL): the desk at all four facings in the
yard, the laundry centre, mop dock, robot, shower and console coloured.

## Rebuild

```
python3 -B tools/art/blender/build.py pack_neat            # the six .glb, the pack, check_glb
python3 -B tools/art/blender/build.py pack_neat --preview  # previews/<id>.png, <id>@<clip>.png
```
