# Held (`families/held.py`)

The 20 things residents carry and cook with. One pack, `packs/held/manifest.json`, models in
`models/held/`. Built 2026-10-16. They aren't buyable: no content entry, no Validate group, no paint.

- **Items** (`item.*`, content `items[].model`): drawn on the palms while carried (characters.ts `writeCarried`:
  one hand or both, content `hold`, level and facing the way the resident faces, base 2 cm under the palms), and set
  down on tops, tables and the floor (items.ts, one thin-instanced template per model, turned by the item's `yaw`).
- **Tools** (`tool.*`, characters.ts `STATION_PROPS`): the chopping board (`chop`), the pan (`stir`), the mixing bowl (`mix`, 2026-10-21) and the food on
  the grill (`flip`), on the station's top while a resident works there, 15 cm towards the cook, turned to face the
  way they face.

So each keeps the old part's frame: origin at the middle of its base, front +depth (away from the one holding it).
Each is built in place in the game's axes (`F`, as its parts were), stood on the floor without moving it
sideways; the pan and the grill food, which stand off their middle, take their measured `centre`.

| Key | What it shows | Draws | Tris |
|---|---|---|---|
| `item.ingredients` | a kraft paper bag with a folded cuff, a lettuce, a carrot with its tops and a tomato | 1 | 288 |
| `item.prepBowl` | a wide glazed bowl of diced potato with carrot and green beans (also the smoothie, the noodles) | 1 | 288 |
| `item.mealPot` | the dark iron stew pot, rolled rim, loop handles, a pale inside, stew with chunks | 1 | 360 |
| `item.potDirty` | the same pot scraped empty: a tidemark, a brown film and burnt bits on the bottom | 1 | 408 |
| `item.tastingBoard` | a wooden board with a tab handle: a wedge of cheese, salami, crackers, grapes (also the cheese board) | 2 | 436 |
| `item.grillPlatter` | an oval platter: two sausages with grill marks, a marked burger, a cob of corn | 1 | 476 |
| `item.platterDirty` | the platter with a smear of grease and sauce, crumbs and a gnawed cob | 1 | 336 |
| `item.pizza` | a round wooden board, a golden raised crust, tomato, cheese, pepperoni and basil | 2 | 508 |
| `item.bakingTray` | a steel tray with a rim and six bread rolls, golden tops and paler feet | 1 | 420 |
| `item.trayDirty` | the tray with a grease stain, burnt patches and crumbs | 1 | 236 |
| `item.pieDish` | a cherry pie in a glazed dish: a crimped pastry rim and a lattice over red filling | 1 | 400 |
| `item.mealPlate` | a plate of stew with mashed potato and green beans (also leftovers, takeout) | 1 | 392 |
| `item.pizzaSlice` | a plate with a slice: base, tomato at the cut sides, cheese, pepperoni, crust at the back | 1 | 420 |
| `item.treatPlate` | chocolate-chip cookies, one leaning on the others, and an iced cupcake with a cherry (also the snack) | 1 | 476 |
| `item.pieSlice` | a slice of the cherry pie: pastry under and over the filling, its crimped edge, a spoon of cream | 1 | 444 |
| `item.sandwich` | a sandwich cut corner to corner: bread, lettuce, cheese, tomato; crust outside, crumb at the cut (also toast) | 1 | 240 |
| `item.plateDirty` | a plate with a smear of gravy, a streak of sauce, crumbs and the fork | 1 | 348 |
| `tool.board` | the chopping board: a carrot half cut into rounds, chopped greens, the knife | 2 | 290 |
| `tool.pan` | the frying pan, its handle to the back, a heap of stir-fry | 1 | 356 |
| `tool.grillFood` | two sausages with grill marks and a marked burger | 1 | 400 |
| `tool.mixingBowl` | a deep glazed bowl of batter, a wooden spoon leaning on its far rim (plain satin, one draw) | 1 | 312 |

- **Finishes.** Dishes glazed `gloss`, pans and trays `metal` (the pan's handle `satin`), food `satin` or `matte`.
  The wooden boards (chopping, tasting, pizza) take `wood` as the old parts did: their second draw call.
- **The plate** (`plate()`) is shared: a foot, a rim up to 1.7 cm and a paler well at 0.9 cm (`WELL`), where the food
  stands. The platter is the same, oval.
- **Shapes.** Round things are `lathe`s (a profile turned about the up axis, one pole at each end, a material per
  band, edges over 40° kept sharp): the plate, bowl, pots, pan, pizza, rolls, the cupcake, and the sausages (laid
  down, their grill marks as bands). Wedges and the sandwich halves are `prism`s over their outline. Lumpy food is
  `G.boulder` at one or two subdivisions.
- **Sizes** within 10 % of the old part on every axis (as the gate checked while the old parts were there). The
  pan's handle runs to the back as the old part's (whose crossbar was across, likely by mistake), 2 cm longer.
- **The old `item.pizzaSlice`** was a three-sided cone stood on its edge, through the plate; it is now a flat wedge
  on the plate.
