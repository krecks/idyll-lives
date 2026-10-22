# Hot-headed pack — notes

`pack_hotHeaded.py` builds the six P8 hot-headed models into `web/public/assets/models/pack_hotHeaded/` and their
entries in `packs/pack_hotHeaded/manifest.json`, an override pack listed in the base manifest. Built
2026-10-14; they sat in Validate › Hot Headed until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`).
Seats stay where the renderer puts residents: the drum throne's top at 0.47 (default seat), the zafu pressed to 0.17, sat on cross-legged
(`furniture.json` `hotHeaded.coolDownCushion`, a seat under 0.2 m: the floor clips), the plunge's bather at 0.35 in the middle of its tub (`SEATS`), the desk chair
at 0.45 in the middle of its tile.

| Model | Moving part and clip | Played by |
|---|---|---|
| `heavyBag` | the bag, straps, logo, chains and swivel, origin on the swivel under the boom: `swing` rocks it 8° back and forth (1.4 s, a loop) | `pound` (loop) |
| `smashBin` | the crate's board lid, hinged on its back top edge: `lid_open` flips it up 80° (0.6 s), held open while plates are thrown in, shut when the use ends (2026-10-21: `hold` 1 s, every plate landing on the shut lid: I-packs-20) | `smash` |
| `thunderDrums` | the crash and the ride, each on its stand's tip: `cymbals` rocks them (9° and −6°, 0.8 s, a loop) | `drum` (loop) |
| `coolDownCushion` | the zafu, origin on its underside: `sink_in` presses it from 0.18 to the 0.17 seat (0.6 s). A real zafu's size, 0.38 across and 10 cm high on the 8 cm zabuton (2026-10-21: a 0.45 m pouf, M-packs-12) | `countToTen`, `breathe` |
| `grudgeJournal` | the drawer under the desk top, facing the writer: `drawer_open` slides it 17 cm out towards them (0.6 s). The top is at 0.745 (2026-10-21: 0.76, the journal on it at 0.77; M-packs-08) | `writeItOut` |
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
| `model.hotHeaded.heavyBag` | 0.86 × 2.01 × 0.86 m | 5,564 | 2 | Body (the bag) |
| `model.hotHeaded.smashBin` | 0.85 × 0.64 × 0.88 m | 2,860 | 3 | Body (the crate) |
| `model.hotHeaded.thunderDrums` | 1.89 × 1.29 × 0.90 m | 7,914 | 3 | Body (the shells) |
| `model.hotHeaded.coolDownCushion` | 0.90 × 0.24 × 0.90 m | 3,918 | 3 | Body (the zafu) |
| `model.hotHeaded.grudgeJournal` | 0.84 × 1.00 × 0.68 m | 3,540 | 3 | Body (the desk's wood) |
| `model.hotHeaded.coldPlunge` | 0.87 × 0.86 × 1.39 m | 5,182 | 3 | Body (the cedar cladding) |

The desk's lamp is a little lower than the old part's (1.02 m against 1.11 m), the only size that differs by more
than 2 %.

`check-glb`: 6 models, 0 failed, no warnings (the crate's slats are unbevelled to keep it under 4,000 triangles).
Khronos validator: 0 errors, 0 warnings each. Clips bench: every clip at rest and open on WebGPU and WebGL (the bag
from the side, the drawer from behind). In play (`wants-asking`, WebGPU): the drums played, the bag pounded, the
plunge, the desk, the crate and the cushion used, each clip playing (the crate's lid shut again after its hold);
all six turned to rot 2 in the yard, the plunge and drums to rot 0, 1 and 3; colours on the clips bench; at night
(`office-night`, WebGL) in the house.

## Detail pass (2026-10-22)

Rebuilt with the forms and the kit, keeping every key, size, centre, part, hinge, clip, `motions` and `paint`
(triangles before → after: bag 3,112 → 5,564, crate 3,300 → 2,860, drums 5,672 → 7,914, cushion 2,748 → 3,918,
desk 2,844 → 3,540, plunge 7,140 → 5,182; draws unchanged). The crate and the plunge came down: their boxes' bevels
were the default two segments (108 triangles a board); one segment is enough at this size.

- `heavyBag`: the upright and boom bent from one square tube (`kit.fillet`), gussets, a round brace with flanges, a
  T foot with rubber caps; weight plates turned with a raised rim on L-shaped pegs; gloves with swept thumbs, turned
  cuffs and straps, hung by their laces. The bag: a rounded leather body, raised straps and seams, a flame logo (an
  outline), a turned swivel, four D-rings and four chains of oval links.
- `smashBin`: slats set a hair out of true, steel corner brackets, a stencilled arrow; jagged shards (outlines); a
  jittered straw bed; a turned stool on splayed legs with a foot ring; soft goggles with lenses and a strap round
  the post.
- `thunderDrums`: drums with hoops and lugs (the toms' hoops now sit on their shells' ends: the old offsets put them
  on the wrong side of the tilt), a bass drum with a port, spurs, a pedal and beater; tripod stands with braces,
  clamps and rubber feet; a hi-hat with its pedal; cymbals with bells, felts and wingnuts; a padded throne; a rug with
  a border; sticks on the floor tom; the amp with a grille panel, a recessed speaker cone, knobs, a light, a handle
  and corner caps.
- `coolDownCushion`: a soft tufted zabuton; a singing bowl on a ring cushion and turned stand, a mallet; a candle on a
  dish; a potted plant of swept leaves; a leaf-shaped incense holder, the stick with its ash. The zafu: lofted with
  pleats round its side gathering into its buttoned top, piping and a carrying loop.
- `grudgeJournal`: hairpin legs (one swept rod each), the journal with curved pages and a ribbon, a pen with its
  clip, jittered crumples, a waste bin with a crumple in it, a ribbed cactus (fluted loft) with a flower, a jointed
  lamp with a spring and a deep shade; the chair with swept legs, a curved upholstered back and rail. The drawer: an
  inset front and a bar pull towards the writer, letters with flaps.
- `coldPlunge`: cedar boards a hair proud or shy, corner posts, two steel bands swept round, a moulded rim in one
  piece; bevelled ice; the housing with end caps; the chiller with a slotted grille (an outline with holes), buttons,
  feet and hoses arching into the tub (`kit.fillet`), a soft towel with a band, a lofted bottle.

`check-glb`: 6 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each.

## Rebuild

```
cargo art pack_hotHeaded            # the six .glb, the pack, check-glb
cargo art pack_hotHeaded --preview  # previews/pack_hotHeaded/<id>.png, <id>@<clip>.png
```
