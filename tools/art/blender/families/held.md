# Held (`families/held.py`)

The 82 things residents carry, cook with and hold while they use a thing. One pack, `packs/held/manifest.json`, models
in `models/held/`. Built 2026-10-16. They aren't buyable: no content entry, no Validate group, no paint.

- **Items** (`item.*`, content `items[].model`): drawn on the palms while carried (characters.ts `writeCarried`:
  one hand or both, content `hold`, level and facing the way the resident faces, base 2 cm under the palms), and set
  down on tops, tables and the floor (items.ts, one thin-instanced template per model, turned by the item's `yaw`).
- **Tools** (actions.ts `STATION_PROPS`): the chopping board (`chop`), the stew pot (`stir`: `item.mealPot` itself, the
  pot they then carry away; it was the frying pan, `tool.pan`, still built but no longer drawn), the mixing bowl (`mix`,
  2026-10-21) and the food on the grill (`flip`), on the station's top while a resident works there (at its
  `STATION_SPOTS`, else 15 cm towards the cook), turned to face the way they face.
- **The cook's tool in hand** (2026-11-03; actions.ts `PROPS.knife`, `woodenSpoon`, `spatula`, `whisk`, `ACTIONS[tag].prop`):
  a chef's knife while chopping (the board's own knife is gone), a wooden spoon while stirring, a turner at the grill and a
  whisk while mixing (the mixing bowl's wooden spoon rests on its rim, not doubled in the hand). Each is built lying, its
  front (+depth) the working end, its origin at the middle of its grip, and turned into the fist by `turn` (grip `hand`).
- **Bites** (2026-11-03; actions.ts `BITES`, `biteOf`): what is eaten by hand at the table goes to the mouth as the fork
  does (`eatFork`, grip `bite`): half the sandwich, half a slice of toast, the pizza slice, a cookie (the treats), the
  takeaway's burger, the hot dog out of its boat, a cracker with cheese (the cheese board), the croissant, a few crisps,
  and the apple itself (`item.apple`, already the size of one). Each lies face up, the end bitten to the front, held at
  its back end, 4 cm along the bite's line from the palm. The plate stays on the table, or, carried (seated without a
  table, or standing with no free seat), on the left palm while the right hand takes the fork or the bite to the mouth
  (residents.ts: standing, `eat` is played as `eatFork`, and between bites the plate is held in front, `holdPlate`, not
  at the hip); a drink served as a plate (the smoothie: `SIPPED`) is drunk from its glass, `item.smoothie` itself in
  the hand (`drink`, held between sips: `holdCup`); what is itself the food (the hot dog, the apple, the pastry,
  the crisps) carried in the hand goes to the mouth as it is, nothing more in the hand.
- **Props in hand** (`tool.*`, 2026-10-26, review G-70; actions.ts `PROPS`, residents.ts `writeHand`): what a resident
  holds while they use a thing, chosen by the use's animation tag and the object (`ACTIONS[tag].prop`) and placed by
  the hands as posed, by its grip (props.ts `placeHand`): a book open across both palms, its pages to the eyes
  (`read`); a phone at the ear (`phone`) or in both hands before the chest (`phoneScreen`); a mug, a tumbler, a wine
  glass or a can upright beside the palm, tipped to the lips as the hand comes to the mouth (`drink`); a fork, a spoon
  or chopsticks from the palm to the mouth while a plate that wants them is eaten at the table (`eat`); the shaker
  between both hands (`shake`); a dart, a ping-pong bat and a pool cue (`play`); a coin for the fountain or the giving
  jar; a bag of bird seed on the left palm (`feed` on the park bench). Each is built as it lies or stands on a table,
  its origin where the hand holds it: the middle of its base for the cups and the bags, the butt of the cue, the
  middle of a grip for the rest; what is held along its length points to the front (+depth).

So each keeps the old part's frame: origin at the middle of its base, front +depth (away from the one holding it).
Each is built in place in the game's axes (`F`, as its parts were), stood on the floor without moving it
sideways; the pan and the grill food, which stand off their middle, take their measured `centre`.

| Key | What it shows | Draws | Tris |
|---|---|---|---|
| `item.ingredients` | a kraft bag lofted with its gussets folded in, a turned-over cuff; a slashed baguette, a lettuce, a tapering carrot with three swept fronds, a lobed tomato with a star calyx | 718 | 718 |
| `item.prepBowl` | a wide glazed bowl, its rim rolled, a heap rising 3.5 cm over the rim thick with two dozen dice of potato, carrot and red pepper, swept green beans and peas | 716 | 716 |
| `item.mealPot` | the iron stew pot: a rolled rim, swept loop handles riveted on, stew with beef, carrot coins, potato, peas and parsley | 1,376 | 1,376 |
| `item.potDirty` | the same pot scraped empty: a tidemark, a brown film and burnt bits on the bottom | 940 | 940 |
| `item.tastingBoard` | a paddle board (one outline, a tab handle with a hole): a cheese wedge and cubes, a fan of salami, crackers, a ramekin of fig jam, grapes on a swept stalk; empties to `item.boardDirty` | 1,496 | 1,496 |
| `item.grillPlatter` | an oval platter, rim rolled: two bent sausages 2.8 cm thick with grill marks, a rough-edged marked burger, a cob of corn in kernel rows with butter, parsley | 1,348 | 1,348 |
| `item.platterDirty` | the platter with a smear of grease and sauce, crumbs, a gnawed cob and a crumpled napkin | 976 | 976 |
| `item.pizza` | a round board, a puffed crimped crust, tomato inside it, cheese browned in spots, curled pepperoni, basil leaves, eight slices cut; empties to `item.pizzaBoardDirty` | 1,436 | 1,436 |
| `item.bakingTray` | a pressed steel tray, its edge rolled, what it serves (`treatPlate`): six chocolate-chip cookies and three iced cupcakes with cherries | 1,760 | 1,760 |
| `item.trayDirty` | the tray with a grease stain, burnt patches, cookie crumbs, smudges of chocolate, a dab of icing and a burnt bit | 856 | 856 |
| `item.pieDish` | a cherry pie 26 cm across in a fluted dish 4.7 cm deep: a crimped pastry rim and a lattice woven over and under; empties to `item.pieDishDirty` | 716 | 716 |
| `item.mealPlate` | a plate of stew: beef and carrot coins in gravy, a ridged dollop of mash, bent green beans, parsley (also leftovers) | 708 | 708 |
| `item.pizzaSlice` | a plate with a slice: base, tomato at the cut sides, cheese browned in a spot, pepperoni, basil, a swept puffed crust | 548 | 548 |
| `item.treatPlate` | chocolate-chip cookies, one leaning on the others, a cupcake in a pleated case in the well, icing piped in a swirl, a cherry with its stalk (also the snack) | 740 | 740 |
| `item.pieSlice` | a sixth of the cherry pie (11 cm to its crust) with a crimped swept edge, filling run out at its tip, a piped rosette of cream with mint | 688 | 688 |
| `item.cakeSlice` | a slice of the café's layer cake on a plate: two sponges, jam and cream between, white icing over the top and round its back, the layers at the cut sides, a strawberry standing on a green calyx, its seeds showing | 472 | 472 |
| `item.sandwich` | a sandwich cut corner to corner, loaf corners rounded: crumb on top and at the cut, crusts round, lettuce frilled out, cheese, tomato | 500 | 500 |
| `item.plateDirty` | a plate with a smear of gravy, a streak of sauce, crumbs, a pea, the knife and fork laid side by side (outlines) | 548 | 548 |
| `item.bbqPlate` | a plate from the barbecue (`grillPlatter`'s `serving`): a sausage marked by the bars, a marked burger, a piece of corn with butter, parsley; eaten with a fork | 780 | 780 |
| `item.tastingPlate` | a plate from the tasting menu (`tastingBoard`'s `serving`): a wedge and a cube of cheese, folds of salami, two crackers, grapes, a dab of fig jam; eaten with a fork | 704 | 704 |
| `item.freshPlate` | something fresh from the garden: a salad on the plate, torn leaves in three greens heaped, tomato wedges on their skins, cucumber and radish slices; eaten with a fork | 708 | 708 |
| `item.plateStackDirty` | three dirty plates stacked, each a little askew, its foot in the well below (the lower two their wells flat, unseen): the top one's leftovers and fork, a drip of gravy down the middle one's rim, a scrap of lettuce over its edge. The sim shows it when two or more dirty plates are carried at once (both hands; content radius 0.156). Over the 750: three plates | 876 | 876 |
| `tool.board` | the chopping board: rounded, a juice groove, a hanging slot; a tapering carrot, carrot coins, parsley (the knife is in the hand: `tool.knife`) | 600 | 600 |
| `tool.pan` | the frying pan, its handle to the back: a heavy rim, a riveted bracket, a tapering grip; stir-fry of beef strips, broccoli, peppers and carrot (no longer drawn: the stew is stirred in its pot) | 1,288 | 1,288 |
| `tool.grillFood` | two bent sausages 2.8 cm thick with grill marks and a rough-edged marked burger | 628 | 628 |
| `tool.mixingBowl` | a deep glazed bowl, its rim rolled, flat batter with the whisk's swirl on it, a tapering wooden spoon, its bowl in the batter, its handle resting on the far rim (plain satin, one draw) | 616 | 616 |
| `item.iceCream` | a dessert bowl, rim rolled, three scoops with ragged lips, chocolate sauce trickled over two of them, a fan wafer (no spoon: the hand's) ; empties to `item.bowlDirty` | 644 | 644 |
| `item.bowlDirty` | the dessert bowl with a film of melted ice cream lying on its sloping inside, a streak of chocolate, a drip of strawberry from the rim, the spoon left in it | 438 | 438 |
| `item.toast` | toast cut corner to corner, toasted tops, dark crusts, a scorch, a pat of butter melting into a pool on each half | 532 | 532 |
| `item.smoothie` | a 14 cm tumbler (thick foot, frothy top) of berry smoothie, a striped bendy straw, a raspberry and mint | 600 | 600 |
| `item.noodles` | a white card noodle pail with a red band, front and back flaps up, a wire handle, noodles and spring onion (no chopsticks: the hand's) | 592 | 592 |
| `item.takeout` | an open kraft clamshell, sides flared, its shallow lid up at the back: a sesame burger (patty, cheese, lettuce, tomato) and fries inside | 700 | 700 |
| `item.boardDirty` | the tasting board cleared: a cheese rind, a bare branching stalk, half a cracker, crumbs, smears of grease and jam | 640 | 640 |
| `item.pizzaBoardDirty` | the pizza's round board after it: bent crust ends, a grease smear, sauce, crumbs, the pizza wheel | 712 | 712 |
| `item.pieDishDirty` | the fluted pie dish emptied: a smear of filling, pastry crumbs, the pie server | 456 | 456 |
| `item.iceCreamCup` | a paper cup 9 cm across in nine pink stripes, two ragged-lipped scoops (vanilla, strawberry) (no spoon: the hand's, `tool.paddle`) | 340 | 340 |
| `item.hotDog` | a hot dog on a paper boat 9 cm across: a bun 6.3 cm across in two lathed halves side by side, inside the boat, a bent sausage longer than it, a mustard zigzag and a line of ketchup | 624 | 624 |
| `item.crisps` | a packet of crisps lying on its back, puffed, blue with a yellow band, crimped seals, its top torn open on the silvered inside, five crisps spilt | 616 | 616 |
| `item.apple` | an apple 7.5 cm across, lobed a little, red with a yellow blush at the stalk and the eye, a stalk and a leaf | 250 | 250 |
| `item.pastryBag` | a white paper bag 12 cm wide, its top folded open, a croissant of five lobes standing out of it | 176 | 176 |
| `item.cutFlowers` | a bunch of cut flowers 37 cm tall standing out of the hand, the stems gathered at the grip and curving up to six heads (three daisies with an amber eye, two tulip buds, a small daisy) and four leaves, the same heads as the vase of flowers (`tabletop.py`) | 624 | 624 |
| `item.daisyChain` | a daisy chain 16 cm across, coiled flat in the hand: a ring of plaited green stems with eleven white daisies round it (an amber eye on each), heads up, and the two stem ends tucked back under where the chain was closed | 1,034 | 1034 |
| `item.parcel` | a parcel from the shop 42 x 30 x 34 cm (2026-11-03, sim-core `setup.rs`): a corrugated card box, its edges rounded a little, brown tape over the top seam and a hand's width down the front and back, a white address label with three red lines; at the front door, in both hands, and on the floor while what's in it is put together | 140 | 140 |
| `tool.book` | an open cloth-bound book, 30 x 22 cm: the boards, two page blocks rising from the spine and curving down to the fore-edges, a sheet over each with seven grey bands of type | 496 | 496 |
| `tool.phone` | a smartphone 7.2 x 14.8 cm, a rounded dark body, the glass and a lit screen, a camera bump | 288 | 288 |
| `tool.mug` | a glazed mug 8 cm across with a blue band, its handle to the right, coffee and crema | 480 | 480 |
| `tool.glass` | a tumbler 7 x 11 cm of an orange drink (seen through the glass), two ice cubes | 340 | 340 |
| `tool.wineGlass` | a stemmed glass 19 cm high, red wine to the bowl's widest | 468 | 468 |
| `tool.can` | a 330 ml can, red with a white band, its ends pressed in, a ring pull | 392 | 392 |
| `tool.fork` | a polished dinner fork 20 cm long, 3 mm thick, an outline extruded: a handle widening to 1.5 cm at its end, four tines 2.6 cm across (chrome, to catch the light from the game camera) | 92 | 92 |
| `tool.spoon` | a steel dessert spoon 17 cm long, an oval bowl | 124 | 124 |
| `tool.chopsticks` | a pair of tapering wooden chopsticks 23 cm long | 24 | 24 |
| `tool.coin` | a brass coin 2.3 cm across, a milled rim | 140 | 140 |
| `tool.seedBag` | a small kraft bag of bird seed, its top rolled open over the seed | 116 | 116 |
| `tool.dart` | a steel dart 15 cm long: a knurled barrel, a black shaft, three blue flights | 180 | 180 |
| `tool.cue` | a pool cue 1.47 m (the pool table's was on its cloth): a wrapped butt, a joint, a tapering shaft, a ferrule and a chalked tip | 188 | 188 |
| `tool.bat` | a ping-pong bat: a wooden blade 15 x 16 cm, red rubber over black, a flared handle (the ping-pong table's two were on its top) | 296 | 296 |
| `tool.shaker` | the bar cart's cocktail shaker, 22 cm, chrome (the cart's own is hidden while it is in the hands) | 320 | 320 |
| `tool.wateringCan` | a green plastic watering can, 5 litres: a round body, a hoop handle over it (held at its middle, 26 cm up), a spout to a rose 35 cm out | 336 | 336 |
| `tool.secateurs` | bypass secateurs, 17.5 cm, closed: red grips, a steel blade over an iron anvil, the pivot bolt | 220 | 220 |
| `tool.paintbrush` | an artist's brush, 25 cm: a lacquered wooden handle, a steel ferrule, bristles dipped in blue | 192 | 192 |
| `tool.chalk` | a stick of white chalk, 8 cm, its tip worn down | 44 | 36 |
| `tool.babyBottle` | a baby's bottle, 17 cm: a clear body two thirds full of milk, a blue screw ring, a teat | 380 | 380 |
| `tool.postcard` | a postcard, 14.8 x 10.5 cm, a seaside view in bands | 48 | 48 |
| `tool.basketball` | a basketball, 25 cm with its seams, orange leather and black seams (the hoops' own is hidden while it is in play: bounced, held, shot and back) | 672 | 672 |
| `tool.pen` | a ballpoint, 14 cm: a navy barrel, a chrome cone and point, a clip | 124 | 124 |
| `tool.pencil` | a yellow pencil, 17 cm, six-sided, sharpened: the bare wood cone and the lead | 60 | 60 |
| `tool.marker` | a whiteboard marker, 15.5 cm with its cap on its back end: a white body, a blue cap and felt tip | 148 | 148 |
| `tool.receiver` | the phone box's receiver, 21 cm: a black bar, the earpiece's and the mouthpiece's cups (drawn on the box's hook by code, `REST_PROPS`, and in the caller's hand) | 268 | 268 |
| `tool.paddle` | the ice cream cart's flat wooden spoon, 9.5 cm (`EATEN_WITH.iceCreamCone`) | 28 | 28 |
| `tool.knife` | a chef's knife 28 cm: a black riveted handle, a steel bolster, a blade curving up to its point, edge to the left | 156 | 156 |
| `tool.woodenSpoon` | a wooden spoon 31 cm, held 10 cm from its end | 144 | 144 |
| `tool.spatula` | a slotted turner 27 cm: a black grip, a steel shank bent down to a blade 8 x 10 cm with three slots | 280 | 280 |
| `tool.whisk` | a balloon whisk 33 cm: a steel handle, four loops of wire 6 cm across | 464 | 464 |
| `tool.biteSandwich` | half the sandwich (`item.sandwich`'s), its point to the front | 140 | 140 |
| `tool.biteToast` | half a slice of the buttered toast | 156 | 156 |
| `tool.bitePizza` | the pizza slice (`item.pizzaSlice`'s), held by its crust | 328 | 328 |
| `tool.biteCookie` | a chocolate-chip cookie | 84 | 84 |
| `tool.biteBurger` | the takeaway's sesame burger | 420 | 420 |
| `tool.biteHotDog` | the hot dog out of its boat, along the depth | 532 | 532 |
| `tool.biteCracker` | a cracker with a slice of cheese | 52 | 52 |
| `tool.biteCroissant` | the bakery's croissant, lying | 100 | 100 |
| `tool.biteCrisps` | three crisps, one on another | 96 | 96 |

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
- **Bought in town** (2026-10-22, review M-held-03; made 2026-10-26): each in what it is sold in. The ice cream cart's
  ice cream in a paper cup (`item.iceCreamCup`, radius 0.05), the hot dog stand's hot dog in its bun on a paper boat
  (`item.hotDog`, 0.1), the snack machine's crisps in their packet (`item.crisps`, 0.11, the item's label *Crisps*),
  the market stall's fruit as an apple (`item.apple`, 0.045) and the bakery case's pastry in a paper bag
  (`item.pastryBag`, 0.07); the grocery's `shopSandwich` keeps the sandwich, which fits. Carried on the right palm and
  eaten from it on a bench or a café chair (the eat clip takes it to the mouth), or set down on a table.
- **Thin in the gate:** the fork, the spoon, the chopsticks, the coin, the phone, the postcard, the pen, the pencil, the
  chalk, the paintbrush and the ice cream cart's paddle are under the 1.2 cm the gate wants of a thing on a floor (a house's floor is drawn 1 cm up):
  they are only ever in a hand, so its warning stands.
- **Fixes** (2026-11-03): the dirty bowl's melt lies on its sloping inside (it was flat under it); the ice cream's sauce
  over the scoops (it was inside them); the noodles and the ice cream lost their own chopsticks and spoon (the hand's are
  the ones that move); the baking tray shows what it serves; the hot dog's bun fits its boat; the ice cream cup is
  striped; sausages 2.8 cm thick (were 4.5-5); the pie 26 cm (was 33.6), its slice and dish with it; the prep bowl
  heaped; the treat plate's cupcake off the rim; content `radius` matched to the models (the plate 0.144, the prep
  bowl 0.162, the pie dish 0.13, the takeaway box 0.146, the crisps 0.12; the pots 0.195 by their handles, the trays
  0.23 and the tasting boards 0.227 by their corners, the dirty bowl 0.099 by its spoon).
- **Servings** (2026-11-03): the barbecue and the tasting menu served a plate of stew (`mealPlate`); each now serves
  its own plate, `bbqPlate` and `tastingPlate` (content `items`, empty to `plateDirty`, eaten with a fork).
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

## The soft forms pass (2026-11-13)

No change to the script: its soft boxes take the shared form's new shape (gamekit/forms.py: rounded edges turning in equal parts, so the same triangles stand closer to the curve; puffed boxes' corners pinched in; cords following the seams). Same sizes (to 1.2 mm), parts, clips, seat and mattress heights (`cargo review sizes` and `uses` unchanged). 1 model changed shape: `crisps`; the same triangles.
