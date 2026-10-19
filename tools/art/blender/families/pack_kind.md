# Kind pack — notes

`pack_kind.py` builds the six P9 kind models into `web/public/assets/models/pack_kind/` and their entries in
`packs/pack_kind/manifest.json`, an override pack listed in the base manifest. Built 2026-10-14; they sit in
Validate › Kind until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`).
Seats stay where the renderer puts residents: the rocker's cushion and the poufs at 0.45–0.49 (default seat), the
settee's seat cushions at 0.45 at x = ±0.5, its sleepers on the default 0.5 bed height.

| Model | Moving part and clip | Played by |
|---|---|---|
| `neighbourlyOven` | the range's oven door, hinged on its foot: `oven_open` drops it 85° forward (0.8 s) | `familyPie`, `treatsToShare` |
| `givingJar` | a coin standing in the cork lid's slot: `coin` drops it 10 cm through into the jar (0.6 s) | `donate` |
| `knittingRocker` | the whole chair with its cushion and throw, turning about its rockers' centre (so they roll on the floor): `rock` rocks it ±5° (1.6 s, a loop); `ride` carries the knitter | `knitGift`, `rock` (loop) |
| `welcomeTeaTable` | the teapot, hinged on its foot under the spout: `pour` tips it 30° (0.8 s), `hold` 1.5 s | `shareTea` |
| `songbirdFeeder` | the feeder's roof, hinged on its back eave: `lid_open` lifts it 70° (0.7 s); three songbirds, built where they perch (tray and bath) and shrunk into the feeder house: `bird` grows them out (1 s) | `refill`; `watchBirds` |
| `hearthSettee` | the two seat cushions and the quilt over the right one, origin on their underside: `sink_in` presses them 2 cm (0.6 s) | `curlUp`, `quiltNap` |

Where the plan's table says "Loop (E1) … Until then static" (the rocker), the loop itself is built.

**Deviations.**
- `pour` has `hold` 1.5 s (the plan names none): `shareTea` lasts 30 minutes, and a teapot tipped all that time looks
  wrong. The bookworm trolley's `pour` does the same.
- `coin`: with no `hold`, the coin stays in the jar while the resident is at it and comes back up into the slot (the
  next coin ready) when the use ends.
- The tea table's two poufs are two materials (sage `pouf` "Fabric", rose `pouf_rose` "Second pouf"), as in the
  old part; the settee's cushions take its `body` colour with the frame.

**Draw calls: 3 each.** The still body takes its plain finishes and at most one textured or glass finish, the moving
parts the rest. So that it stays at 3:
- the counter's top and shelf are plain satin (`butcher`), the range's tea towel takes the `fabric` draw; the door
  is all plain;
- the giving jar's pedestal is plain satin, its jar takes the `glass` draw, the coin the third; the ribbon is satin;
- the rocker's chair is `rocker` (wood) and `cushion`/`throw` (fabric) on its moving part, so the yarn basket beside
  it is all plain;
- the tea table is plain satin (`table`), the poufs and rug take the `fabric` draw, the teapot is plain;
- the feeder station's still body is all plain (the tray satin, no glass, the shrubs satin leaves), so its roof and
  its birds take the other two draws;
- the settee's frame and cushions are `body` (fabric) and the quilt `fabric`: one textured draw on each part, the
  legs and lantern plain.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.kind.neighbourlyOven` | 1.83 × 1.72 × 0.63 m | 5,292 | 3 | Body (the range's enamel), Fabric (the towel), Cupboards |
| `model.kind.givingJar` | 0.64 × 1.32 × 0.66 m | 3,068 | 3 | Body (the pedestal), Fabric (the ribbon) |
| `model.kind.knittingRocker` | 0.72 × 1.05 × 0.87 m | 2,840 | 3 | Body (the chair), Fabric (the cushion) |
| `model.kind.welcomeTeaTable` | 1.86 × 0.56 × 0.86 m | 4,616 | 3 | Body (the table), Fabric, Second pouf, Rug |
| `model.kind.songbirdFeeder` | 0.90 × 1.70 × 0.88 m | 5,004 | 3 | Body (the feeder house), Fabric (the seed sack), Roof |
| `model.kind.hearthSettee` | 1.90 × 0.95 × 0.83 m | 3,188 | 3 | Body (the velvet), Fabric (the quilt) |

The range sits on the wall's face (−0.43), 2.5 cm forward of the old part's; the jar is 3.5 cm taller with the
coin standing in its slot; the rocker's yarn basket stands 2 cm further out.

`check-glb`: 6 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each. Clips bench: every clip
at rest and open on WebGPU and WebGL (the rocker from the side, the coin close up). In play (`wants-asking`, WebGPU):
each object used by a resident: the oven door open while the pie bakes, the coin, the knitter rocking, the tea
poured, the feeder's roof up for the refill and the songbirds out while watched, the settee's cushions; all six
turned in the yard (the settee to all four facings, the range to 1, 2 and 3); colours on the clips bench; at night
(`office-night`, WebGL) in the house.

## Rebuild

```
cargo art pack_kind            # the six .glb, the pack, check-glb
cargo art pack_kind --preview  # previews/<id>.png, <id>@<clip>.png
```
