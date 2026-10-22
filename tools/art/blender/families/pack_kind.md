# Kind pack — notes

`pack_kind.py` builds the six P9 kind models into `web/public/assets/models/pack_kind/` and their entries in
`packs/pack_kind/manifest.json`, an override pack listed in the base manifest. Built 2026-10-14; they sat in
Validate › Kind until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`).
Seats stay where the renderer puts residents: the rocker's cushion and the poufs at 0.45–0.49 (default seat), the
settee's seat cushions at 0.45 at x = ±0.5, its sleepers on the default 0.5 bed height. The tea table's poufs
(40 cm across) stand at x = ±0.75 either side of its low table (top 0.36, 0.4 × 0.56 m), and its two face each
other straight across it (content `places` `[±0.75, 0]` facing ±90; G-39, 2026-10-21: the poufs were at ±0.5
beside a 0.42 m table, the sitters turned 45°): the knees stop short of the table's ends, the feet before them.
The ferns stand at the back between the poufs and the table.

| Model | Moving part and clip | Played by |
|---|---|---|
| `neighbourlyOven` | the range's oven door, hinged on its foot: `oven_open` drops it 85° forward (0.8 s), shut again after 3 s (`hold`; 2026-10-21). A beadboard back rises from the worktop's back to carry the jar shelf, standing free or on the wall (2026-10-21, M-packs-01; Q-06). | the `bake` step of `familyPie` and `treatsToShare` (by its tag, so the mixing at its worktop leaves the oven shut) |
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
| `model.kind.neighbourlyOven` | 1.83 × 1.72 × 0.63 m | 7,914 | 3 | Body (the range's enamel), Fabric (the towel), Cupboards |
| `model.kind.givingJar` | 0.64 × 1.32 × 0.66 m | 4,912 | 3 | Body (the pedestal), Fabric (the ribbon) |
| `model.kind.knittingRocker` | 0.74 × 1.05 × 0.86 m | 4,820 | 3 | Body (the chair), Fabric (the cushion) |
| `model.kind.welcomeTeaTable` | 1.94 × 0.50 × 0.87 m | 7,380 | 3 | Body (the table), Fabric, Second pouf, Rug |
| `model.kind.songbirdFeeder` | 0.90 × 1.70 × 0.88 m | 6,724 | 3 | Body (the feeder house), Fabric (the seed sack), Roof |
| `model.kind.hearthSettee` | 1.90 × 0.95 × 0.82 m | 7,360 | 3 | Body (the velvet), Fabric (the quilt) |

The range sits on the wall's face (−0.43), 2.5 cm forward of the old part's; the jar is 3.5 cm taller with the
coin standing in its slot; the rocker's yarn basket stands 2 cm further out.

`check-glb`: 6 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each. Clips bench: every clip
at rest and open on WebGPU and WebGL (the rocker from the side, the coin close up). In play (`wants-asking`, WebGPU):
each object used by a resident: the oven door open while the pie bakes, the coin, the knitter rocking, the tea
poured, the feeder's roof up for the refill and the songbirds out while watched, the settee's cushions; all six
turned in the yard (the settee to all four facings, the range to 1, 2 and 3); colours on the clips bench; at night
(`office-night`, WebGL) in the house.

## Detail pass (2026-10-22)

Rebuilt with the forms and the kit, keeping every key, size, centre, part, hinge, clip, `motions` and `paint`
(triangles before → after: counter 5,592 → 7,914, jar 3,068 → 4,912, rocker 2,840 → 4,820, tea table 4,616 → 7,380,
feeder 5,004 → 6,724, settee 3,188 → 7,360; draws unchanged).

- `neighbourlyOven`: raised shaker doors (`kit.shaker`) with turned brass knobs, a butcher block with strips; a
  lattice pie of woven strips, a wire cooling rack, turned biscuits, a turned rolling pin in a patch of flour, a cake
  with icing drips and a cherry on a turned stand; a capped beadboard back, shaped shelf brackets, lofted jars with
  cork lids and labels; the range's backguard clock, burners under cast-iron grates (`kit.fillet` tubes), turned
  knobs, a rail on returns with a gingham towel draped over it (swept, jittered, checked), a stock pot with handles.
  The oven door: a moulded window frame and a bar pull.
- `givingJar`: a richer turned pedestal with a bead; more coins and curled notes; a turned cork, a ribbon tied in a
  bow, a heart tag (an outline) on its string; a framed chalk sign with a chalked heart; a flowering shrub of jittered
  clumps in a rimmed pot; the jar with a thick rim and lip. The coin has a face.
- `knittingRocker`: runners swept on the rockers' circle (now touching the floor: the old arc of boxes stood 2.5 cm
  up), swept legs, stiles and spindles, turned finials, arms rolling into hand-rests, a bowed crest, a shaped seat, a
  piped cushion with ties, a throw folded over the crest; a woven basket (fluted loft) with a rim and handle, wrapped
  yarn balls, a strand to the floor, knitting needles.
- `welcomeTeaTable`: a braided oval rug; cable-knit poufs (fluted lofts) with piped, buttoned tops; swept legs, books
  with pages, a tea light in a cup; cups with handles on saucers, a milk jug; ferns of arching fronds. The teapot:
  lofted, with a band, lid, knob, a curving spout and a looped handle.
- `songbirdFeeder`: a bracketed tray with a swept lip, a house with corner trims, a hopper of seed, a ringed door with
  its perch; the kit's songbirds (`kit.songbird`: a robin at the door, a tit on the tray, a finch and a sparrow in the
  bath); a bath with a rolled rim; jittered flowering shrubs; a soft seed sack with its top rolled, a tin of seed and
  a scoop. The roof: shingle rows and a ridge.
- `hearthSettee`: a piped soft base and back, puffed back cushions, rolled arms with welts (living.py's), turned
  legs, buttoned pillows, a lantern with bands, a cap and a ring. The seats: piped cushions with their tops at 0.45, a
  strip quilt of five fabrics draped over the right cushion and down its front, stitched across.

`check-glb`: 6 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each.

## Rebuild

```
cargo art pack_kind            # the six .glb, the pack, check-glb
cargo art pack_kind --preview  # previews/pack_kind/<id>.png, <id>@<clip>.png
```
