# Foodie pack — notes

`pack_foodie.py` builds the six P6 foodie models into `web/public/assets/models/pack_foodie/` and their entries in
`packs/pack_foodie/manifest.json`, an override pack listed in the base manifest. Built 2026-10-14; they sat in
Validate › Foodie until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`),
backs on the wall's face (depth −0.43). The island's top is at 0.9, the content's `surface.height` (the
old part's was 0.83), so what is put on it sits on it; its two counter stools' seats are at 0.65, 25 cm under the top (`furniture.json`
`foodie.chefsIsland` 0.65, the hips at depth −0.235, the sitters at x ±0.5 facing the top, the knees under its back overhang; foot rings at 0.3; 2026-10-21: they were 0.45, chair height, the top at the chest: M-packs-04).

| Model | Moving part and clip | Played by |
|---|---|---|
| `herbGarden` | a basil sprig in the pot on the lower shelf, built full size and scaled to nothing on the soil: `sprig` grows it (0.9 s) | `tendHerbs` |
| `deliPantry` | the two lower doors, hinged on their outer edges: `doors_open` swings them out 100° (0.8 s), `hold` 2.5 s | `nibble`, `cheeseBoard` |
| `espressoBar` | the coffee from the left group's spout (grown down, 0.4 s) and the steam wand on its ball joint (swung out 40°, 0.6 s): `brew`, `hold` 4 s | `espresso`, `latteArt` |
| `cookbookLectern` | the right-hand page, origin on the spine and tilted onto the desk (`rotation_mode` YXZ): `page` turns it 172° about the spine (0.9 s), until the read ends | `studyRecipes`, `foodMagazines` |
| `pizzaOven` | the iron grille door and the peel with a pizza: `door_off` (1.4 s) lifts the door off the mouth and sets it on the ground before the left pillar, then carries the peel from the counter round in front of the oven and in through the mouth | `bakePizza` |
| `pizzaOven` | flat flames in the mouth behind the grille, built full height and flattened onto the sill: `flames` grows them (0.8 s) | `warmUp`, `bakePizza` (and any `bake` step) |
| `chefsIsland` | the middle drawer on the cook's side: `drawer_open` pulls it out 12 cm (0.6 s), `hold` 2.5 s; the ring under the copper pan, scaled to nothing at rest: `burner_on` grows it, glowing red, while it cooks (0.5 s). Keyed by the recipe's step tags (A-systems-03), so each step at it plays its own | the `chop` (prep) and `stir` (cook) steps of `tastingMenu` |

The espresso's clip keys two parts; the pizza oven's `door_off` keys two parts with their own timing (`_clip`
with keyframes: the peel waits for the door, then goes up, forward, along and in, so it never passes through the
dome or the door).

**Detail pass (2026-10-22).** Rebuilt with the forms and the kit (README "The toolkit"); keys, sizes, centres,
parts, clips, motions, paint and draws as before. Triangles in the table below (was: before the pass).
- `herbGarden`: legs with rounded edges; a moulded rim round the trough (`kit.moulding`, `plane='top'`); bumpy
  soil; basil of extruded leaves in tiers up a stem, rosemary of tapering stems, thyme as lumpy mounds; a saucer
  under the sprig's pot, rolled pot rims; a shaped trowel; a woven basket with an arched handle; the sprig's leaves
  extruded.
- `deliPantry`: raised-panel doors (`kit.shaker`) with hinges and upright bar pulls; a crown moulding; shelf lips;
  a rounded marble ledge; lofted jars with label bands and turned lids, a crock, oil and wine bottles; cheese
  wheels with their rind painted, one cut; loaves wobbled with three slashes; lofted salami tied on strings; a
  board with a hanging hole and a wedge of cheese; tomatoes with their calyx.
- `espressoBar`: swept posts, castors in forks; shelf lips; a draped towel on its rail; the machine's body and
  base rounded, the boiler's foot banded in brass, an eagle with wings, a gauge needle; a drip tray with a grate;
  turned group heads and portafilters with tapered handles; levers bent; cups with crema and handles; a milk jug
  with a handle; a syrup pump; lumpy sacks tied at the top.
- `cookbookLectern`: three swept feet with pads; a turned column with rings; a rounded desk with a scalloped apron
  and bracket; the page blocks swept from a curved section, rising from the spine; a curled turning page; a
  ribbon hanging over the edge; brass page clips; cookbooks with page edges (`_pages`); a turned stool; a
  recipe-card box with its lid propped open.
- `pizzaOven`: logs crooked (`G.jitter`) with pale cut ends; a rounded hearth; the dome turned and wobbled; the
  arch one extruded ring; bevelled voussoirs; a lofted flue on a collar and a turned cap; a shaker door with an
  iron bar pull on the counter; a lumpy dough ball, a turned sauce bowl, a lofted oil bottle; the door's rim one
  bent bar, a rail and an oak bar pull; the peel's blade a shaped outline, a tapered handle; the pizza with a
  turned crust rim and basil leaves; lofted, leaning flames.
- `chefsIsland`: a rounded top; shaker drawers and doors with brass bar pulls; the foot rail bent round to the
  ends; a rounded hob with rings; a turned pan with a tapered handle, a stock pot with its lid, knob and handles; a
  carrot with leaves, a tomato's calyx, a knife with its grip; cheese wedges on the plates; a menu stand; the
  stools' legs tapered sweeps, a curved back rail.

**Draw calls: 3 each, the pizza oven 4.** So that they stay at 3:
- herbs, pots and soil are plain (satin, matte), the herb garden's cedar its one `wood`;
- the pantry and the espresso cart are all plain (their oak-coloured shelves and tops satin), as their two moving
  parts take the other draws;
- the lectern keeps `wood` (walnut) beside its plain finishes and one moving part; the island's stools are plain satin oak now (2026-10-21), so the hot ring is its third draw.

The pizza oven has three moving parts (door, peel, fire: the plan's door and peel for `bakePizza`, flames for
`warmUp`), so its body is all plain (the logs satin, not bark): 4 draws, `check-glb`'s one warning. The fire
shows through the grille door (a solid door would hide it for `warmUp`); `bakePizza` also lights it, so the door
comes off a burning oven (the plan names `flames` for `warmUp` only).

| Model | Size (w × h × d) | Triangles (2026-10-22) | Draws | Colour parts |
|---|---|---|---|---|
| `model.foodie.herbGarden` | 0.86 × 1.29 × 0.76 m | 5,528 (was 6,708) | 3 | Body, Pots |
| `model.foodie.deliPantry` | 0.88 × 1.99 × 0.57 m | 5,532 (was 4,392) | 3 | Body, Worktop, Shelves |
| `model.foodie.espressoBar` | 0.76 × 1.51 × 0.51 m | 6,324 (was 5,364) | 3 | Body, Worktop, Machine |
| `model.foodie.cookbookLectern` | 0.84 × 1.15 × 0.59 m | 2,644 (was 2,276) | 3 | Body, Stool & ledge, Cookbook |
| `model.foodie.pizzaOven` | 1.89 × 1.90 × 0.92 m | 7,616 (was 5,056) | 4 | Body, Worktop, Brick |
| `model.foodie.chefsIsland` | 1.86 × 1.13 × 0.89 m | 7,176 (was 6,380) | 3 | Body, Worktop, Stools |

The herb garden has no worktop; its second part is the pots. The rosemary stands 7 cm over the old part's.

`check-glb`: 6 models, 0 failed; the pizza oven's 4-draw warning above. Khronos validator: 0 errors, 0 warnings
each. Gallery next to the old parts; clips bench at rest and open on WebGPU and WebGL, coloured (pantry, oven).
In play (`wants-asking`, WebGPU): the pizza oven baked in the yard (door down, peel in, fire, all back after),
the island sat at for a bite, the pantry opened for a snack, the herbs tended, the cookbook read (the page over),
the espresso bar used (the resident hides the stream from the camera; seen on the bench). At night
(`office-night`, WebGL): the oven in four facings, the island, the herb garden, the pantry and the lectern turned.

## Rebuild

```
cargo art pack_foodie            # the six .glb, the pack, check-glb
cargo art pack_foodie --preview  # previews/pack_foodie/<id>.png, <id>@<clip>.png
```
