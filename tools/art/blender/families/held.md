# Held (`families/held.py`)

The 30 things residents carry and cook with. One pack, `packs/held/manifest.json`, models in
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
| `item.ingredients` | a kraft bag lofted with its gussets folded in, a turned-over cuff; a slashed baguette, a lettuce, a tapering carrot with three swept fronds, a lobed tomato with a star calyx | 1 | 718 |
| `item.prepBowl` | a wide glazed bowl, its rim rolled, heaped diced potato with carrot, swept green beans and peas | 1 | 632 |
| `item.mealPot` | the iron stew pot: a rolled rim, swept loop handles riveted on, stew with beef, carrot coins, potato, peas and parsley | 1 | 1,376 |
| `item.potDirty` | the same pot scraped empty: a tidemark, a brown film and burnt bits on the bottom | 1 | 940 |
| `item.tastingBoard` | a paddle board (one outline, a tab handle with a hole): a cheese wedge and cubes, a fan of salami, crackers, a ramekin of fig jam, grapes on a swept stalk; empties to `item.boardDirty` | 2 | 1,496 |
| `item.grillPlatter` | an oval platter, rim rolled: two bent sausages with grill marks, a rough-edged marked burger, a cob of corn in kernel rows with butter, parsley | 1 | 1,348 |
| `item.platterDirty` | the platter with a smear of grease and sauce, crumbs, a gnawed cob and a crumpled napkin | 1 | 976 |
| `item.pizza` | a round board, a puffed crimped crust, tomato inside it, cheese browned in spots, curled pepperoni, basil leaves, eight slices cut; empties to `item.pizzaBoardDirty` | 2 | 1,436 |
| `item.bakingTray` | a pressed steel tray, its edge rolled, six rolls slashed across their tops | 1 | 1,436 |
| `item.trayDirty` | the tray with a grease stain, burnt patches, crumbs and a burnt bit | 1 | 724 |
| `item.pieDish` | a cherry pie in a fluted dish: a crimped pastry rim and a lattice woven over and under; empties to `item.pieDishDirty` | 1 | 736 |
| `item.mealPlate` | a plate of stew: beef and carrot coins in gravy, a ridged dollop of mash, bent green beans, parsley (also leftovers) | 1 | 708 |
| `item.pizzaSlice` | a plate with a slice: base, tomato at the cut sides, cheese browned in a spot, pepperoni, basil, a swept puffed crust | 1 | 548 |
| `item.treatPlate` | chocolate-chip cookies, one leaning on the others, a cupcake in a pleated case, icing piped in a swirl, a cherry with its stalk (also the snack) | 1 | 740 |
| `item.pieSlice` | a slice of the cherry pie with a crimped swept edge, filling run out at its tip, a piped rosette of cream with mint | 1 | 688 |
| `item.sandwich` | a sandwich cut corner to corner, loaf corners rounded: crumb on top and at the cut, crusts round, lettuce frilled out, cheese, tomato | 1 | 500 |
| `item.plateDirty` | a plate with a smear of gravy, a streak of sauce, crumbs, a pea, the knife and fork laid side by side (outlines) | 1 | 548 |
| `tool.board` | the chopping board: rounded, a juice groove, a hanging slot; a tapering carrot, carrot coins, parsley, a chef's knife (outlined blade, bolster, riveted handle) | 2 | 720 |
| `tool.pan` | the frying pan, its handle to the back: a heavy rim, a riveted bracket, a tapering grip; stir-fry of beef strips, broccoli, peppers and carrot | 1 | 1,288 |
| `tool.grillFood` | two bent sausages with grill marks and a rough-edged marked burger | 1 | 628 |
| `tool.mixingBowl` | a deep glazed bowl, its rim rolled, batter with a stirred ridge, a tapering wooden spoon leaning on its far rim (plain satin, one draw) | 1 | 576 |
| `item.iceCream` | a dessert bowl, rim rolled, three scoops with ragged lips, chocolate sauce trickled over, a fan wafer, a spoon; empties to `item.bowlDirty` | 1 | 676 |
| `item.bowlDirty` | the dessert bowl with a film of melted ice cream, a streak and drip, the spoon left in it | 1 | 496 |
| `item.toast` | toast cut corner to corner, toasted tops, dark crusts, a scorch, a pat of butter melting into a pool on each half | 1 | 532 |
| `item.smoothie` | a 14 cm tumbler (thick foot, frothy top) of berry smoothie, a striped bendy straw, a raspberry and mint | 1 | 600 |
| `item.noodles` | a white card noodle pail with a red band, front and back flaps up, a wire handle, noodles and spring onion, tapered chopsticks | 1 | 616 |
| `item.takeout` | an open kraft clamshell, sides flared, its shallow lid up at the back: a sesame burger (patty, cheese, lettuce, tomato) and fries inside | 1 | 700 |
| `item.boardDirty` | the tasting board cleared: a cheese rind, a bare branching stalk, half a cracker, crumbs, smears of grease and jam | 2 | 640 |
| `item.pizzaBoardDirty` | the pizza's round board after it: bent crust ends, a grease smear, sauce, crumbs, the pizza wheel | 2 | 712 |
| `item.pieDishDirty` | the fluted pie dish emptied: a smear of filling, pastry crumbs, the pie server | 1 | 456 |

- **Detail pass** (2026-10-22): rebuilt with the forms (sweeps, lofts, outlines extruded) at the same sizes, centres,
  origins and draws: crusts, rims, handles, straws, beans and piped icing are sweeps (the rods-and-spheres crusts
  are gone); boards, leaves, cutlery and smears are outlines; turned things `lathe`s, now fluted, lobed or bent where
  the thing is (`wave`, `deform`). At most 750 triangles under 35 cm, 2,000 for the few 35-40 cm across (half the
  budgets: they are drawn many times). Was 212-508 each.

- **Their own models** (2026-10-21, review M-held-01): ice cream, toast, the smoothie, noodles and takeout borrowed the
  treat plate, the sandwich, the prep bowl and the meal plate; each now has its own (`base.json` `items`), sized as the
  real thing (a 14 cm tumbler, a 10 cm noodle pail, a 12.5 cm dessert bowl), with a smaller `radius` on a top.
- **The dish that went in comes out** (2026-10-21, review M-held-02): the tasting and cheese boards empty to
  `boardDirty`, the pizza to its own round board (`pizzaBoardDirty`), the pie to its dish (`pieDishDirty`), the ice
  cream to its bowl (`bowlDirty`), not to the grill's oval platter or the baking tray. Each is a content `items` entry
  (`dirty`, washed as the others are).
- **Bought in town** (2026-10-22, review M-held-03): the hot dog stand's hot dog is carried in the takeaway box
  (`item.takeout`) and the ice cream cart's in the dessert bowl (`item.iceCream`), not on the treat plate or as a
  sandwich. Still to be made (nothing here fits): a cone or paper cup of ice cream, a hot dog in its bun, a packet of
  crisps (the snack machine's `crisps`), an apple (the market's `fruit`) and a pastry in a paper bag (the bakery
  case's `pastry`); those three keep the treat plate meanwhile, the grocery's `shopSandwich` the sandwich.
- **Finishes.** Dishes glazed `gloss`, pans and trays `metal` (the pan's handle `satin`), food `satin` or `matte`.
  The wooden boards (chopping, tasting, pizza) take `wood` as the old parts did: their second draw call.
- **The plate** (`plate()`) is shared: a foot, a rim up to 1.7 cm and a paler well at 0.9 cm (`WELL`), where the food
  stands. The platter is the same, oval.
- **Shapes.** Round things are `lathe`s (a profile turned about the up axis, one pole at each end, a material per
  band, edges over 40° kept sharp; `wave` flutes a dish, pleats a case, lobes a tomato; `deform` bends a sausage):
  the plate, bowls, pots, pan, pizza, rolls, scoops, cookies, corn and the sausages (laid down, their grill marks as
  bands). Wedges and the sandwich halves are `prism`s over their outline. Crusts and pastry edges are `crust` sweeps
  (crimped by a wave in their thickness), icing and cream `swirl`s, smears `blob`s, leaves `leaf` outlines. Small
  lumpy food takes `lump(rough=)`: `G.boulder`'s own noise is too broad to show on a thing a few centimetres across.
- **Sizes** within 10 % of the old part on every axis (as the gate checked while the old parts were there). The
  pan's handle runs to the back as the old part's (whose crossbar was across, likely by mistake), 2 cm longer.
- **The old `item.pizzaSlice`** was a three-sided cone stood on its edge, through the plate; it is now a flat wedge
  on the plate.
