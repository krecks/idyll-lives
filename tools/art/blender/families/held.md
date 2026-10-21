# Held (`families/held.py`)

The 29 things residents carry and cook with. One pack, `packs/held/manifest.json`, models in
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
| `item.prepBowl` | a wide glazed bowl of diced potato with carrot and green beans | 1 | 288 |
| `item.mealPot` | the dark iron stew pot, rolled rim, loop handles, a pale inside, stew with chunks | 1 | 360 |
| `item.potDirty` | the same pot scraped empty: a tidemark, a brown film and burnt bits on the bottom | 1 | 408 |
| `item.tastingBoard` | a wooden board with a tab handle: a wedge of cheese, salami, crackers, grapes (also the cheese board); empties to `item.boardDirty` | 2 | 436 |
| `item.grillPlatter` | an oval platter: two sausages with grill marks, a marked burger, a cob of corn | 1 | 476 |
| `item.platterDirty` | the platter with a smear of grease and sauce, crumbs and a gnawed cob | 1 | 336 |
| `item.pizza` | a round wooden board, a golden raised crust, tomato, cheese, pepperoni and basil; empties to `item.pizzaBoardDirty` | 2 | 508 |
| `item.bakingTray` | a steel tray with a rim and six bread rolls, golden tops and paler feet | 1 | 420 |
| `item.trayDirty` | the tray with a grease stain, burnt patches and crumbs | 1 | 236 |
| `item.pieDish` | a cherry pie in a glazed dish: a crimped pastry rim and a lattice over red filling; empties to `item.pieDishDirty` | 1 | 400 |
| `item.mealPlate` | a plate of stew with mashed potato and green beans (also leftovers) | 1 | 392 |
| `item.pizzaSlice` | a plate with a slice: base, tomato at the cut sides, cheese, pepperoni, crust at the back | 1 | 420 |
| `item.treatPlate` | chocolate-chip cookies, one leaning on the others, and an iced cupcake with a cherry (also the snack) | 1 | 476 |
| `item.pieSlice` | a slice of the cherry pie: pastry under and over the filling, its crimped edge, a spoon of cream | 1 | 444 |
| `item.sandwich` | a sandwich cut corner to corner: bread, lettuce, cheese, tomato; crust outside, crumb at the cut | 1 | 240 |
| `item.plateDirty` | a plate with a smear of gravy, a streak of sauce, crumbs and the fork | 1 | 348 |
| `tool.board` | the chopping board: a carrot half cut into rounds, chopped greens, the knife | 2 | 290 |
| `tool.pan` | the frying pan, its handle to the back, a heap of stir-fry | 1 | 356 |
| `tool.grillFood` | two sausages with grill marks and a marked burger | 1 | 400 |
| `tool.mixingBowl` | a deep glazed bowl of batter, a wooden spoon leaning on its far rim (plain satin, one draw) | 1 | 312 |
| `item.iceCream` | a glazed dessert bowl, three scoops (vanilla, strawberry, chocolate) and a spoon; empties to `item.bowlDirty` | 1 | 464 |
| `item.bowlDirty` | the dessert bowl with a film of melted ice cream and the spoon left in it | 1 | 288 |
| `item.toast` | a plate of toast cut corner to corner, toasted tops, dark crusts, a pat of butter on each half | 1 | 264 |
| `item.smoothie` | a 14 cm tumbler of berry smoothie (opaque glass, the smoothie shown through it) and a paper straw | 1 | 212 |
| `item.noodles` | a white card noodle pail with a red band, noodles and spring onion, chopsticks standing in it | 1 | 268 |
| `item.takeout` | an open kraft takeaway box, its lid up at the back: a burger and fries | 1 | 424 |
| `item.boardDirty` | the tasting board cleared: a cheese rind, a bare grape stalk, cracker crumbs, a smear | 2 | 216 |
| `item.pizzaBoardDirty` | the pizza's round board after it: crust ends at the edge, a grease smear, crumbs | 2 | 344 |
| `item.pieDishDirty` | the pie dish emptied: a smear of filling, pastry crumbs against its sides | 1 | 304 |

- **Their own models** (2026-10-21, review M-held-01): ice cream, toast, the smoothie, noodles and takeout borrowed the
  treat plate, the sandwich, the prep bowl and the meal plate; each now has its own (`base.json` `items`), sized as the
  real thing (a 14 cm tumbler, a 10 cm noodle pail, a 12.5 cm dessert bowl), with a smaller `radius` on a top.
- **The dish that went in comes out** (2026-10-21, review M-held-02): the tasting and cheese boards empty to
  `boardDirty`, the pizza to its own round board (`pizzaBoardDirty`), the pie to its dish (`pieDishDirty`), the ice
  cream to its bowl (`bowlDirty`), not to the grill's oval platter or the baking tray. Each is a content `items` entry
  (`dirty`, washed as the others are).
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
