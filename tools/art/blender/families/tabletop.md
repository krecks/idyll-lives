# Tabletop — notes

`tabletop.py` builds the 39 tabletop things (docs/objects/tabletop.md, TT1–TT5) into `web/public/assets/models/tabletop/`
and their entries in `packs/tabletop/manifest.json`. Built 2026-10-14; they sat in Validate (Kitchenware, Tableware, Books And Pots, Small Ornaments, Desk And
Bath Bits) until checked in play, then went back to `tabletop` and their old group (`kitchenware`, `tableware`, `books`,
`plants`, `ornaments`, `bath`, `desk`, `toys`).

## What they keep

- **Origin at the base, centred** (the `surface` layer: the renderer lifts them to their host's top). `G.settle` puts
  them there.
- **Size** no wider than the content's `radius` (the disc a thing takes on a top; the gate warns of one reaching
  over 10 % past it). The four widest (`plateSet`, `teapotSet`, `espressoMachineSmall`, `monitor`, radius 0.23–0.3 m) don't fit
  on a counter's top with its edge margin, as before: they go on tables and desks.
- **One draw call** where it can: every finish is a plain one. Wood is `satin` brown, towels and the teddy `matte`,
  leaves `satin` green, so none needs a textured finish. Glass that is seen through is `glass` (a second draw call):
  the cookie jar, the wine glasses and the snow globe. Elsewhere glass is opaque `gloss` (the blender's jug shows the
  smoothie as its lower half).
- **Triangles** at most half the size class's target, as they're placed many times on every lot: 750 under 0.35 m,
  2,000 for the five larger (`espressoMachineSmall`, `plateSet`, `teapotSet`, `vaseFlowers`, `monitor`). The table
  below has each.
- **Player colour**: the main part's material, labelled `Body` (the cookie jar: its `Lid`); sets of like things (mugs,
  books, towels, blocks) list each one.

## The models (detail pass, 2026-10-22)

Rebuilt with the forms and the kit (README "The toolkit"): turned things are lofted rings (`turned`: a profile of
(r, z) rows, few sides, rims kept sharp and few-sided bodies shaded round), handles, spouts and stems swept
(`G.kit.fillet` round their bends), slabs extruded with their holes (`slab`), flat faces a single polygon (`disc`: a
drink's surface), the plants' leaves and flowers plants.py's. Sixteen were over half the target before (up to
1,496); every one is under it now, with more on it. Sizes, centres, parts, clips and draws as before.

| Model | Size (w × h × d) | Triangles (before) | Draws | What it has |
|---|---|---|---|---|
| `kettle` | 0.23 × 0.24 × 0.17 m | 724 (696) | 2 | jug body swelling to the shoulder on a power base, beak spout, D handle in one piece, red switch, water window; the lid and knob turned, the lid flipping up (`lid_open`) |
| `toaster` | 0.28 × 0.23 × 0.15 m | 710 (584) | 1 | rounded body on a plinth, two slots with toast (crust round the crumb), lever in its guide, browning dial and pointer, cancel button, chrome trim |
| `blender` | 0.18 × 0.36 × 0.15 m | 736 (648) | 1 | tapering base with a dial and two buttons, collar, soft-square jug (smoothie its lower half), handle in one piece, lid and filler cap |
| `standMixer` | 0.30 × 0.36 × 0.17 m | 708 (1,180) | 1 | rounded base, column, lofted tilt head swelling to its nose with a chrome band and hub, speed lever, steel bowl with batter, beater (a cut-out paddle) |
| `espressoMachineSmall` | 0.23 × 0.31 × 0.39 m | 1,874 (844) | 1 | body rounded over the top, head with buttons and light, group, portafilter with two spouts, slotted grate over the drip tray, gauge (bezel, face, needle), steam knob and bent wand, water tank, cup-warmer rail on posts, a cup with crema and handle, a cup upturned on top |
| `knifeBlock` | 0.12 × 0.30 × 0.23 m | 584 (352) | 1 | leaning block with eased edges on its foot, six knives (a steel bolster under each riveted handle, sized to its blade) |
| `utensilCrock` | 0.13 × 0.32 × 0.12 m | 708 (668) | 1 | glazed crock with a rolled lip and a blue band; wooden spoon (one swept piece), slotted spatula, balloon whisk (two wire loops), ladle with its bowl |
| `cuttingBoard` | 0.34 × 0.10 × 0.23 m | 724 (792) | 1 | board with a hand hole, a lofted bloomer with three scores, two tomatoes with calyxes, two slices with their cut faces, a cook's knife (blade, bolster, handle) |
| `breadBin` | 0.34 × 0.21 × 0.20 m | 644 (380) | 2 | body between D-shaped wooden ends (eased), a toe rail, a loaf inside; the roll-top's wooden bar pull (`kit.pull`), rolling back (`roll_open`) |
| `spiceJars` | 0.30 × 0.11 × 0.12 m | 700 (792) | 1 | rack with shaped ends, back board and round front rail; five shouldered jars with paper labels and screw lids |
| `cookieJar` | 0.16 × 0.25 × 0.16 m | 604 (696) | 2 | glass jar with shoulder and neck, four cookies, glazed lid with a turned knob |
| `fruitBowl` | 0.28 × 0.19 × 0.28 m | 726 (1,088) | 1 | bowl on a foot ring; red and green apples (shaped, stalks), two oranges with navels, two bananas (five-sided, dark tips) |
| `plateSet` | 0.38 × 0.10 × 0.32 m | 990 (988) | 1 | dinner and side plates (foot ring, well, rim) with blue bands, napkin, a four-tined fork, a knife with its blade shaped, tumbler |
| `cakeStand` | 0.28 × 0.26 × 0.28 m | 724 (1,352) | 1 | footed stand with a knop, layer cake with filling, icing running down in drips, six piped rosettes, cherry and stalk |
| `teapotSet` | 0.40 × 0.17 × 0.26 m | 1,856 (1,496) | 1 | tray with a gallery and hand holes in its ends, round teapot with a band, curving spout, loop handle, lid and knob; two cups of tea on saucers |
| `mugs` | 0.22 × 0.09 × 0.21 m | 746 (1,092) | 1 | three mugs with thick rims and coffee, one-piece loop handles, a tea bag's string and tag |
| `wineGlasses` | 0.21 × 0.30 × 0.19 m | 690 (960) | 2 | bottle with shoulders, label and foil capsule; two open glasses (foot, stem, bowl) with wine |
| `candlesticks` | 0.17 × 0.33 × 0.07 m | 608 (840) | 2 | turned brass sticks (stepped foot, knop, drip pan), candles with rounded tops, curled wicks, a run of wax; teardrop flames |
| `bookStack` | 0.25 × 0.13 × 0.20 m | 372 (344) | 1 | four hardbacks, boards proud of the page block, rounded spines with gilt bands, a ribbon marker |
| `booksUpright` | 0.32 × 0.21 × 0.13 m | 692 (624) | 1 | arched brass bookends with a round cut-out on their feet; eight books, boards proud of the pages, rounded spines, paper bands |
| `vaseFlowers` | 0.18 × 0.41 × 0.17 m | 1,444 (1,428) | 1 | vase with belly and flared lip; daisies (ray petals, raised eyes), small daisies, three tulips, a cosmos, on curving stems, leaves on stems and at the neck |
| `vaseSmall` | 0.08 × 0.28 × 0.08 m | 532 (492) | 1 | bud vase with slim neck and lip; one rose (two cupped rings round a furled bud), sepals, a leaf on the stem |
| `potSucculent` | 0.16 × 0.14 × 0.16 m | 732 (524) | 1 | pot with a rim on grit; an echeveria: three tiers of fat upturned leaves round a heart |
| `potCactus` | 0.18 × 0.32 × 0.13 m | 686 (896) | 1 | terracotta pot with a rolled rim on sand; a ribbed column with two ribbed arms bent up (fillet), a pink flower with its eye |
| `potSmall` | 0.16 × 0.26 × 0.16 m | 728 (1,008) | 1 | glossy pot with a rim, soil; a peace lily: arching pointed leaves on stalks, two white spathes with spadices on tall stems |
| `photoFrame` | 0.18 × 0.23 × 0.11 m | 288 (260) | 1 | moulded frame (`kit.moulding`, mitred), back board, white mount, a photo of a hillside with sun and a figure, strut |
| `tableClock` | 0.11 × 0.16 × 0.05 m | 722 (976) | 3 | drum case rolled at the back with a raised bezel, face with flat hour marks, domed bells, hammer, carrying loop, splayed legs on bun feet |
| `globe` | 0.21 × 0.31 × 0.24 m | 728 (940) | 1 | the continents as the sphere's own faces in land colour, tilted in a brass meridian ring with pole pins, turned column and foot |
| `snowGlobe` | 0.13 × 0.18 × 0.13 m | 800 (804) | 2 | turned base with a brass band and a collar round the dome's foot; inside a snowy mound, a three-tier fir, a cabin with a snowy roof, a snowman, flakes |
| `trophy` | 0.15 × 0.23 × 0.13 m | 694 (780) | 1 | two-step plinth with plaque, turned stem with a knop, flaring cup with a rolled rim, scrolled handles |
| `sculptureSmall` | 0.15 × 0.22 × 0.12 m | 400 (652) | 1 | a Möbius band on edge (a flat band turning half over), brass ball, plinth with a name plate |
| `soapSet` | 0.26 × 0.17 × 0.12 m | 742 (668) | 1 | stone tray, pump dispenser (collar, pump, spout in one piece), tumbler with two toothbrushes (bristles), soap on drainer ridges |
| `towelStack` | 0.27 × 0.15 × 0.20 m | 684 (456) | 1 | three folded towels, each two soft layers, a woven band across the front |
| `perfumeTray` | 0.28 × 0.14 × 0.18 m | 684 (688) | 1 | mirrored tray in a mitred brass gallery; faceted bottle with a brass stopper, a round flacon on its edge with an atomizer tube and bulb, a tall bottle with a cap |
| `deskOrganiser` | 0.27 × 0.16 × 0.20 m | 524 (344) | 1 | two-tier letter tray (scooped sides, back, posts), paper and a loose sheet in each; pen pot with two capped, clipped pens and a sharpened pencil |
| `laptopClosed` | 0.32 × 0.02 × 0.22 m | 372 (324) | 2 | base and lid with rounded corners and eased edges, hinge barrel, thumb notch, ports, logo; a keyboard, touchpad and screen under the lid, which opens (`lid_open`) |
| `monitor` | 0.54 × 0.46 × 0.34 m | 1,486 (404) | 1 | rounded foot, curved neck, puffed back housing, slim bezel round the screen, logo and power light; keyboard of 57 separate keys; lofted mouse with a wheel |
| `stuffedToy` | 0.19 × 0.28 × 0.19 m | 722 (1,300) | 1 | pear-shaped body and round head (lofted), tummy and muzzle, nose, eyes, cupped ears with pale insides, arms and legs swept, paws, foot pads, a bow and knot |
| `toyBlocks` | 0.22 × 0.20 × 0.17 m | 692 (226) | 1 | five blocks with rounded edges, a raised letter (A, L, T, H, O: outlines with holes) on two faces each, a prism roof |

## The lids and the roll-top (2026-10-26)

Things on tops have uses since G-63 (review I-tabletop-01); three of them open for theirs (review §9.2, F2), each a
part of its own (two draws):

- `kettle`: the `lid` is hinged on the handle's side (it was at its back edge), as a jug kettle's is, the knob
  moved to its free side; `lid_open` flips it up 80° in 0.4 s while the kettle is filled for *Make a cup of tea*
  (`tea`, held 1.6 s, then shut).
- `breadBin`: the `roll_top`, a half-tube shell round the bin's axis, rolls back 100° in 0.6 s (`roll_open`, about
  Blender X) for *Make a sandwich* (`sandwich`, held 2.4 s): over the top and down behind, inside the base, showing
  a loaf on the bin's floor (a soft box, 108 triangles, hidden at rest).
- `laptopClosed`: the `lid`, hinged at its back edge, opens 105° in 0.8 s (`lid_open`) and stays open while the
  laptop is used (by its uses' animation, `type`: browse, code, write, homework). Under it, hidden at rest, the
  screen (`laptop_screen`, dark glass that glows a cool white at night with the room's lights, `#A9C8E8`) and on the
  base a keyboard and a touchpad.

In play (`check.mjs play --state review-stage --objects kettle,breadBin,laptopClosed --use all`, headless WebGPU) each
clip played with its use at the counter. The play runner sets the laptop down at rot 0 whatever way the counter
faces, so its screen faced along the counter there; set down facing the one using it (as the house generator does,
about the way its host faces) the screen faces them.

## The alarm clock

`tableClock` tells the game's time: its hour and minute hands are parts of their own, each turning about the face's
middle once round, `G.spin(hand, '<name>_hand', 1, seconds=12, axis='Y')`, and the game holds each where the time has
it (`motions.clock`, `at` 610: ten past ten at rest, the catalogue picture), on every preset. A positive turn about
Blender +Y is clockwise seen from the front. The red second hand is gone (a game minute is a real second at normal
speed); its red hub stays on the face. Three draws: the body and the two hands.

## The candles (2026-10-21)

`candlesticks` was drawn with its flames lit all day and dark at night (review M-tabletop-01). Its two flames are now
one part of their own (`flames`, one more draw: two), built lit and sunk 3.4 cm into the candles' tops at rest, hidden
in the wax (1.3 cm across to their 2.2). `lit` slides them up over the wicks (`G.slide`, 0.6 s) and is the model's
`night` clip, held open dusk to dawn: by day the wicks are dark, at night the flames stand on them. The flame's
material glows (`#FFC46A`, as the lantern's): bright while the room's lights are on, out with them (a dark room's
candles show their flames unlit). Its height is the candles' at rest: 0.33 m.

## The alarm clock's size (2026-10-21)

`tableClock` was 0.15 × 0.23 × 0.06 m, half again a real twin-bell clock's height (review M-tabletop-02). It is built
at 0.72 (`S`): the case 10.7 cm across, 0.11 × 0.16 × 0.05 m in all, its ticks and hands as thick as
before.
