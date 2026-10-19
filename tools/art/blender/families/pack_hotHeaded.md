# Hot-headed pack — notes

`pack_hotHeaded.py` builds the six P8 hot-headed models into `web/public/assets/models/pack_hotHeaded/` and their
entries in `packs/pack_hotHeaded/manifest.json`, an override pack listed in the base manifest. Built
2026-10-14; they sit in Validate › Hot Headed until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`).
Seats stay where the renderer puts residents: the drum throne's top at 0.47 (default seat), the zafu pressed to 0.40
(`SEATS` `hotHeaded.coolDownCushion`), the plunge's bather at 0.35 in the middle of its tub (`SEATS`), the desk chair
at 0.45 in the middle of its tile.

| Model | Moving part and clip | Played by |
|---|---|---|
| `heavyBag` | the bag, straps, logo, chains and swivel, origin on the swivel under the boom: `swing` rocks it 8° back and forth (1.4 s, a loop) | `pound` (loop) |
| `smashBin` | the crate's board lid, hinged on its back top edge: `lid_open` flips it up 80° (0.6 s), `hold` 1 s | `smash` |
| `thunderDrums` | the crash and the ride, each on its stand's tip: `cymbals` rocks them (9° and −6°, 0.8 s, a loop) | `drum` (loop) |
| `coolDownCushion` | the zafu, origin on its underside: `sink_in` presses it from 0.45 to the 0.40 seat (0.6 s) | `countToTen`, `breathe` |
| `grudgeJournal` | the drawer under the desk top, facing the writer: `drawer_open` slides it 17 cm out towards them (0.6 s) | `writeItOut` |
| `coldPlunge` | the slatted cover, origin at its roller housing on the back rim: `cover_off` rolls it back into the housing (0.9 s) | `plunge` |

Where the plan's table says "Loop (E1) … Until then U / static" (the bag, the drums), the loop itself is built.

**Deviations.**
- `cover_off`: the cover is a roll-up slatted cover that retracts into a housing on the tub's back rim (a squash
  along its depth), rather than a lid sliding off sideways: sliding it off whole would take it out of the footprint
  or through the chiller behind the tub.
- The crate in the old part was open-topped; it gets a board lid for `lid_open`.
- The drums' cymbals are two moving parts on one clip (each rocks on its own stand), the hi-hat stays still.

**Draw calls: 3 each (the bag 2).** The still body takes its plain finishes and at most one textured finish, the
moving part one more. So that it stays at 3:
- the floor mats and the drum rug are plain satin (the old parts' were `fabric`);
- the drum kit's still body is all plain (shells gloss, hoops and stands chrome, heads satin, sticks satin), so its
  two cymbals take the other two draws;
- the cushion's plant is satin leaves (no `foliage`), its candle glass is left out;
- the desk's chair and top are `wood`, the drawer plain satin;
- the plunge tub's cladding is `wood`, its cover plain satin; no glass.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.hotHeaded.heavyBag` | 0.86 × 2.01 × 0.86 m | 3,112 | 2 | Body (the bag) |
| `model.hotHeaded.smashBin` | 0.85 × 0.64 × 0.88 m | 3,300 | 3 | Body (the crate) |
| `model.hotHeaded.thunderDrums` | 1.89 × 1.28 × 0.90 m | 5,672 | 3 | Body (the shells) |
| `model.hotHeaded.coolDownCushion` | 0.90 × 0.45 × 0.90 m | 2,748 | 3 | Body (the zafu) |
| `model.hotHeaded.grudgeJournal` | 0.84 × 1.02 × 0.68 m | 2,844 | 3 | Body (the desk's wood) |
| `model.hotHeaded.coldPlunge` | 0.86 × 0.86 × 1.39 m | 7,140 | 3 | Body (the cedar cladding) |

The desk's lamp is a little lower than the old part's (1.02 m against 1.11 m), the only size that differs by more
than 2 %.

`check-glb`: 6 models, 0 failed, no warnings (the crate's slats are unbevelled to keep it under 4,000 triangles).
Khronos validator: 0 errors, 0 warnings each. Clips bench: every clip at rest and open on WebGPU and WebGL (the bag
from the side, the drawer from behind). In play (`wants-asking`, WebGPU): the drums played, the bag pounded, the
plunge, the desk, the crate and the cushion used, each clip playing (the crate's lid shut again after its hold);
all six turned to rot 2 in the yard, the plunge and drums to rot 0, 1 and 3; colours on the clips bench; at night
(`office-night`, WebGL) in the house.

## Rebuild

```
cargo art pack_hotHeaded            # the six .glb, the pack, check-glb
cargo art pack_hotHeaded --preview  # previews/<id>.png, <id>@<clip>.png
```
