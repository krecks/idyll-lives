# Outdoor

The outdoor things are built in four families. `families/outdoor.py` builds the eleven OU1 patio models, the five
OU2 loungers and swings and the three OU3 grills (notes: `outdoor.md`); `families/yard.py` the nine OU4 play and yard
things (notes: `yard.md`); `families/garden_things.py` the nine OU5 pond, small garden things and lights (notes:
`garden_things.md`); `families/garden_decor.py` the six garden decor models (gnome, bird bath, fountain, lantern,
wheelbarrow, boulders). Each model is built to the content's seat heights (`seat`) and table tops
(`surface`), where the game has them. Every item is `outdoors`: it can only
be bought outside.

Moving parts: the parasol and the string lights stir in the wind and the pond ripples (ambient, E2, off on Low); the
hammock, porch swing and swing set swing while used (E1); the sun lounger reclines, the hot tub's cover comes off and
stays off until it is put back (C1), the grills open, the fire pit burns; the yard things open, tip and grow (the
raised bed, E4); the washing stays out on the line until taken in (C1). Outdoor things are seen from far: the yard
things and most of the garden things have a far version (E7); the patio, lounger, grill and garden decor models have
none.

## OU1. Patio seating and tables (11)

- **Family:** `tools/art/blender/families/outdoor.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `bench` Park bench | 2x1 · floor | sit:sit | outdoor / seating | Frame, Slats | Static | Static. |
| `patioSet` Folding bistro set | 1x2 · floor | coffee:drink sit:sit | outdoor / seating | Frame, Slats | Static | Static. |
| `picnicTable` Picnic table | 3x2 · floor | picnic:eat coffee:drink | outdoor / seating | Frame, Planks, Cloth | Static | Static. |
| `gardenChair` Garden chair | 1x1 · floor | sit:sit | outdoor / seating | Chair | Static | Static. |
| `patioTableRound` Round patio table | 1x1 · floor | — | outdoor / seating | Frame, Top, Tiles | Static | Static. |
| `patioTable4` Teak patio table | 2x1 · floor | — | outdoor / seating | Frame, Top | Static | Static. |
| `patioChair` Patio chair | 1x1 · floor | sit:sit coffee:drink | outdoor / seating | Frame, Cushion | Static | Static. Pulled in to the table it faces while sat on, as far as the knees fit under it. |
| `adirondackChair` Adirondack chair | 1x1 · floor | relax:relax | outdoor / seating | Chair | Static | Static. |
| `benchGarden3` Garden bench | 3x1 · floor | sit:sit | outdoor / seating | Frame, Slats | Static | Static. |
| `outdoorBar` Outdoor bar | 2x1 · floor | — | outdoor / seating | Body, Counter, Foot rail | Static | Static. |
| `parasol` Parasol | 1x1 · floor | — | outdoor / seating | Canopy, Pole, Edge | Ambient (E2) | Open; `stir`: the canopy stirs in the wind, ambient (not on Low). (+use) it folds. |

**In play** (signed off 2026-10-17):
- Checked standing free in the garden in four facings: sat on (park bench, garden bench, garden chair, patio chair),
  relaxed in (Adirondack chair), coffee and sitting in the sun at the bistro set, a picnic and a coffee at the
  picnic table. The tables, the bar and the parasol were looked at in four facings.
- Coffee at the bistro set, picnic table and patio chair needs a working coffee machine on the same lot
  (`requires` / `provides`), as at any table.
- The patio chair is a seat at a table (`seatAt: ["table"]`; its coffee is `needsTable`), checked at the round table
  and at the teak table. Once seated, the sitter shuffles in with the chair until the top's near edge is a fixed
  reach ahead of the hips, and pushes it back while rising. How far it can come is read from the table's drawn
  triangles (`tableTuck.ts`): up to 0.45 m, the chair overlapping into the table's tile. The sim's positions are
  unchanged.
- Every colour part lands on the part its label names: the round table's Frame is its rim and pedestal, Top and
  Tiles the mosaic; the parasol's Edge is the valance.

## OU2. Loungers, swings and the hot tub (5)

- **Family:** `tools/art/blender/families/outdoor.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `sunLounger` Sun lounger | 1x2 · floor | sunbathe:relax doze:nap | outdoor / seating | Frame, Cushions, Towel | Use | `recline`: the back lowers 30° for sunbathe and doze. |
| `hammock` Hammock | 1x2 · floor | swing:relax doze:nap | outdoor / seating | Frame, Fabric, Stripes | Loop (E1) | `sway`: swings ±8° while lain in (E1). |
| `porchSwing` Porch swing | 2x1 · floor | swing:relax | outdoor / seating | Seat, Frame, Chains, Cushion | Loop (E1) | `swing`: the seat swings ±10° while sat in (E1). |
| `swingSet` Swing set | 2x1 · floor | swing:play | outdoor / play | Frame, Seats | Loop (E1) | `swing`: the seat swings while played (E1). |
| `hotTubCedar` Cedar hot tub | 2x2 · floor | soak:bath coverOn:fetch | outdoor / seating | Cedar, Cover | Use + Loop | `cover_off`: the cover slides off and folds back for soak and stays off (C1) until "Put the cover on" (`coverOn`); `bubbles`: the water bubbles while soaking (E1). |

**In play** (signed off 2026-10-17):
- Each checked in four facings: the lounger's back lowers for sunbathe and doze and comes up after; the hammock's
  `sway` and the two `swing` loops play while ridden and stop after.
- Lying, seen 1, 4 and 8 s into the use (also on Low WebGL2 at night): the resident sits on first, then lies along
  the lounger and in the hammock's sling.
- The hot tub's cover lifts and the water bubbles for a soak; after it, "Put the cover on" follows (`closeAfter`)
  and the cover shuts.
- Known: a swing or a sunbathe ends within seconds when the need it fills is full (fun 0.8 to 1 in about 8 s), and
  the next one is refused until it drops. That is the needs, not the model.

## OU3. Grills and fire (3)

- **Family:** `tools/art/blender/families/outdoor.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `grill` Charcoal grill | 1x1 · floor | cook:cook | outdoor / cooking | Body, Lid | Use | `lid_open`, the lid swings up 95° on its back hinge for the whole of cook, over red coals and food on the grate (no night glow on the coals: it showed through the shut lid). |
| `grillStation` Gas grill station | 2x1 · floor | cook:cook feast:cook | outdoor / cooking | Cart, Hood | Use | `hood_up`, the barrel hood swings up 75° for cook and feast; the counter at the content's 0.9 m. |
| `firePit` Fire pit | 1x1 · floor | relax:relax | outdoor / cooking | Bowl, Stones | Use | `flames`, tongues grow on the logs for relax, aglow at night with the embers (the light is the content's); content `seat` puts the resident on the front capstones, not in the fire. |

**In play** (signed off 2026-10-17):
- Each checked in four facings: the charcoal grill's lid is up for the whole of "Grill dinner", over the coals and
  food; the gas station's hood is up for cook and for the feast; the fire pit's flames grow for "Warm up by the
  fire", the resident on its front capstones, and the embers and flames glow at night. It is a fire (content
  `light.fire`): lit by sitting down by it in the dark or by the player, it burns (light, glow and flames) until the
  morning, and its light lies in a pool on the lawn round it (`gardenPools.ts`).
- Each shuts when the use ends. Nothing needed fixing.

## OU4. Play and yard (9)

- **Family:** `tools/art/blender/families/yard.py` (notes: `yard.md`; a family of its own rather than `outdoor.py`,
  so that OU3–OU5 were built without all editing one script and pack)

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `sandbox` Sandpit | 2x2 · floor | sandcastle:tend | outdoor / play | Frame | Use | `castle`: a sand castle (keep, four turrets, walls, a flag) rises out of the sand (2.5 s) and stands while it's built. Far version. |
| `bikeRack` Bike rack | 2x1 · floor | tuneUp:tinker | outdoor / play | Rack, Bike | Use + Loop | `wheel_up`: the bike tips up 8.5° on its front tyre, its back axle onto the stand's cradle; `wheel_turn`: the back wheel turns while tuned up (E1 loop). Far version. |
| `raisedBed` Raised vegetable bed | 2x1 · floor | tend:tend harvest:harvest | outdoor / yard | Bed | Growth + Use | `grow` (E4): lettuces, carrots and tomato bushes ripe at rest, the bed bare when picked, seedlings when planted; `harvest`: the crop is picked away (the stage then goes to picked); `rustle`: tending sways the plants every 2.4 s, on top of the stage. Far version. |
| `shedSmall` Garden shed | 2x2 · floor | potter:tinker | outdoor / yard | Wood, Roof, Door | Use | `door_open`: the ledged door swings 100° out for the whole of potter, the dark inside with a bench and tools behind. The door is a colour part of its own (one plain draw). Far version. |
| `greenhouse` Greenhouse | 2x3 · floor | tend:tend harvest:harvest | outdoor / yard | Frame, Wall | Use | `door_open`: the glass door swings 70° and the roof vent lifts 25° for tend and harvest. 4 draws (plain, glass, door, vent), as the nature lover's glasshouse. Far version. |
| `compostBin` Compost bin | 1x1 · floor | compost:tend | outdoor / yard | Bin | Use | `lid_open`: the slatted lid lifts 75° for compost; `hold` 2 s. Far version. |
| `rainBarrel` Rain barrel | 1x1 · floor | — | outdoor / yard | Barrel | Static | Static: ribbed barrel, brass tap, the downpipe into its lid. Far version. |
| `clothesline` Washing line | 3x1 · floor | washing:clean takeIn:clean | outdoor / yard | Posts, Washing | Use + Open | `washing`: the washing comes down from the line (1.5 s) and is the `open` clip (C1): it stays out until taken in. Far version (washing in). |
| `firewoodStack` Log store | 1x1 · floor | — | outdoor / yard | Shelter | Static | Static: the iron rack under a pent iron roof, logs ends out. Far version. |

**In play** (signed off 2026-10-17):
- Each checked in four facings: the sand castle rises for a sandcastle; the bike tips onto its stand and its back
  wheel turns for a tune-up; the shed's door opens for pottering, the dark inside showing; the greenhouse's door and
  roof vent open for tend and harvest; the compost lid is up for 2 s.
- The raised bed's plants rustle every 2.4 s while tended. A new bed is planted, so harvest isn't offered until it
  is ripe; a ripe bed is harvested, the crop picked away and the bed left at picked.
- The washing comes down onto the line for "Hang out the washing", stays out after, and "Take the washing in"
  (`takeIn`) takes it in.
- Colour parts land where named; the log store's Shelter is the whole iron rack and roof (one material).

## OU5. Pond, small garden things and lights (9)

- **Family:** `tools/art/blender/families/garden_things.py` (notes: `garden_things.md`; its own family so OU3 and OU4
  could be built alongside)

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `garden.pond` Garden pond | 2x2 · floor | fish:look feedFish:garden | outdoor / decor | Stones | Use + Ambient | `fish`: a koi rises 4 cm to the surface, turning, held for feedFish; `surface` (up and down, 2.4 s) every 7 s while watching (fish); `ripple` ambient (E2). Opaque gloss water, reeds still (satin): 3 draws, 4408 tris. |
| `planterBox` Planter box | 1x1 · floor | water:water | outdoor / decor | Box | Static | Static: plank trough, leaves on `foliage` (wind, tint), flowers; far version (`far_of`). 3 draws, 3752 tris. |
| `birdhouse` Birdhouse | 1x1 · floor | — | outdoor / decor | House, Roof | Static | Static. 2 draws, 1028 tris. |
| `birdFeederPole` Bird feeders | 1x1 · floor | birdwatch:look | outdoor / decor | Pole, Feeders | Use | `bird`: a blue tit appears on the second perch for birdwatch, held while watching; the robin is always there. 3 draws, 2932 tris. |
| `dogHouse` Dog house | 1x1 · floor | — | outdoor / decor | House, Roof | Static | Static: name plate REX, bowl. 2 draws, 2224 tris. |
| `sundial` Sundial | 1x1 · floor | — | outdoor / decor | Plinth, Dial | Static | Static. 1 draw, 1316 tris. |
| `steppingStones` Stepping stones | 1x3 · rug | — | outdoor / decor | Stones | Static | Static, flat: 4 cm slabs that show on the lawn. 1 draw, 320 tris. |
| `lampPost` Garden lamp post | 1x1 · floor | — | outdoor / lights | Post, Lantern | Static | Static. The panes at the light (2.3 m) glow at night (E8); far version (0.25). 1 draw, 744 tris. |
| `stringLights` String lights | 2x1 · floor | — | outdoor / lights | Posts, Bulbs | Ambient (E2) | `sway` ±4° in 3.2 s, ambient; the bulbs glow at night. 3 draws, 2524 tris. |

**In play** (signed off 2026-10-17):
- Each checked in four facings: the koi comes up for "Feed the fish" and surfaces every 7 s while watching the fish,
  with the ambient ripples; the blue tit appears for "Watch the birds"; the planter box is watered.
- The lamp post's panes and the string lights' bulbs glow at night, on WebGPU and on Low WebGL2. The birdhouse, dog
  house, sundial and stepping stones are still.
- Known, not changed: one stepping stone has a small dark nick on its edge (a dent in the boulder shape, a few pixels
  from the game camera).

## Garden decor (6)

These six have no group code of their own.

- **Family:** `tools/art/blender/families/garden_decor.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `garden.gnome` Garden gnome | 1x1 · floor | pat:tend | outdoor / decor | Hat, Tunic, Belt | Static | Static: a gnome with a red hat, white beard and a tunic bell. |
| `garden.birdbath` Bird bath | 1x1 · floor | birdwatch:look | outdoor / decor | Bowl & column, Plinth | Static | Static: plinth, tapering column and a shallow bowl of water, a small bird on the rim. |
| `garden.fountain` Stone fountain | 1x1 · floor | wish:look listen:relax | outdoor / decor | Fountain, Plinth | Static | Static: a two-tier fountain, its water opaque and still. |
| `garden.lantern` Stone lantern | 1x1 · floor | — | outdoor / lights | Lantern, Base | Static | Static: a Japanese stone lantern; its fire box is lit at night (the content's `light`). |
| `garden.wheelbarrow` Wheelbarrow planter | 1x1 · floor | water:water smell:smell | outdoor / decor | Tray, Handles, Legs & tyre | Static | Static: a green wheelbarrow full of flowers and leaves. |
| `garden.boulder` Garden boulders | 1x1 · floor | — | outdoor / decor | Large boulder, Middle boulder, Small boulder | Static | Static: three weathered stones, moss on top and tufts round the base. |

**In play** (signed off 2026-10-17):
- Checked in the north yard in four facings with their uses: pat the gnome, watch the birds, listen to the water and
  make a wish, water and smell the planter. The lantern and boulders are still; nothing floats or sinks.
- Colours checked by day in two facings with test colours far from the defaults, at night on WebGPU and on Low
  WebGL2, and in the colour studio. Every colour lands on the part its label names.
- The gnome's Tunic takes in its sleeves; its boots sit wholly inside the tunic bell, so only the belt is named.
  The fountain's Fountain is the basins, column and finial. The lantern's lit fire box and the boulders' moss stay as
  made.
