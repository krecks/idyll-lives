# Bookworm pack — notes

`crates/art/packs/src/pack_bookworm` builds the six P1 bookworm models into `web/public/assets/models/pack_bookworm/` and their
entries in `packs/pack_bookworm/manifest.json`, an override pack listed in the base manifest. Built
2026-10-14; they sat in Validate › Bookworm until checked in play.

Each is built where the game places it (`F`, game axes; `settle!(centre = false)` and `.centre(..)` on its `Model`),
with one moving part. Pack families write to a pack of their own (`pack_<trait>`), keyed `model.<trait>.<id>`, so
the trait's pack keeps the old parts. The bedside stack stood 3 cm into the wall; it is centred on its `Model`
`centre` now (`settle_all()`), its back on the wall face at −0.43, as the wall-standing ones' old parts stood, so
the model and the wall agree within a centimetre.

The writer's bureau stands with its back on the wall's face and its chair in front; the writer faces the bureau
(content `"seat": {"height": 0.45, "forward": 0.12, "facing": "back"}`, as the piano). It is built as it stands
(2026-10-22; it was built as the old part first stood, the chair between the wall and the desk, and turned round),
its flap dropping forward onto the desk (+90° about X).

| Model | Moving part and clip | Played by |
|---|---|---|
| `readingNook` | the seat cushion and throw, origin on their underside: `sink_in` presses them 2 cm (0.6 s) | `curlUp` |
| `libraryWall` | the rolling ladder: `ladder` rolls it 40 cm along the brass rail, towards the middle (1.2 s) | `research`, `browse` |
| `writingBureau` | the writing flap over the pigeonholes, hinged on its foot: `fall_front` drops it 90° onto the desk (0.8 s) | `draft`, `letter` |
| `teaTrolley` | the teapot, hinged on its foot under the spout: `pour` tips it 30° (0.8 s), `hold` 1.5 s | `brew` |
| `firesideLounge` | the flames on the grate's logs, built flat: `flames` grows them (0.8 s). The hood hangs right over the grate on its flue, its rim at 0.6 (2026-10-21: at 1.0, over the air between the chairs: M-packs-10) | `readByFire`, `readAloud` |
| `bedsideStack` | the top book's cover, hinged on the spine: `book_up` opens it 170° to the right (0.8 s). The chest is nightstand-sized, 0.6 wide with four drawers (2026-10-21: a 0.8 m chest of six, M-packs-12); the floor pile in front of it | `chapter` |

**Draw calls: 3 each** (the bureau 4: its flap carries the wood panel and a brass pull, as it always did). The
still body takes its plain finishes and one textured finish, the moving part one more. So that it stays at 3:
- the library ladder is all wood, its rail hooks, forks, wheels and ties too (a metal on it costs a second draw);
- the nook's back cushions and pillows, the bureau chair's pad and the trolley's cloth are plain satin (the
  nook's seat cushion and throw, alone on their moving part, are `fabric`);
- the lounge's wood is plain satin (`walnut`), as its chairs and rug take the `fabric` draw;
- the fire is an open grate under a hood, so the flames show from the room (the old stove faced the
  chairs, its fire hidden from the camera); no glass.

Book spines are plain satin, unbevelled boxes: a wall of books costs no draw call and few triangles. Their page
edges are a second (plain) material on the same box (`pages`: the tops of standing books, three sides of lying
ones), so they show cream with no more triangles; china takes its glazed bands the same way (`band`).

## Detail pass (2026-10-22)

Rebuilt with the forms and the kit (README "The toolkit"); keys, sizes, centres, parts, clips, motions and paint as
before. Triangles in the table below (was: before the pass).
- `readingNook`: the cases' shelves with a lipped front edge and a kick; books with page edges, some leaning; a
  cornice moulding (`kit::moulding!`) round the header, an arched, beaded valance; a pendant on its cord with a
  rolled rim and bulb; frame-and-panel drawers (`kit::shaker!`) with turned knobs; stuffed, piped back cushions and
  pillows (`soft!`); the seat cushion piped, the throw a swept, wobbled cloth folded over its front.
- `libraryWall`: a crown moulding and a moulded plinth run round the case, beads on the uprights and shelf
  edges; a desk globe with land and a brass meridian; a lofted marble bust with hair; the rail on wall roses with
  turned finials; the ladder's stiles swept, wooden hooks bent over the rail, wheels in forks, two tie rods.
- `writingBureau`: now built facing the front as it stands (no `turn_round`; the flap's clip unchanged); shaker
  drawers with knobs, a rounded desk top and hutch top, an arched crest; letters with envelopes in the
  pigeonholes; an inkwell and quill; the chair with raked swept back legs, turned front legs, stretchers, a bowed
  crest rail, a vase splat and a piped pad; the flap a raised panel with its lining, a bar pull and a key.
- `teaTrolley`: each end of the frame one bent tube (the right end the push bar with a grip), galleries on
  spindles, rounded shelves, castors in forks; a lofted teapot with a glazed band, swept spout and handle; cups
  with tea and ear handles, a jug and sugar bowl; the cake with a slice cut out; the cloth hanging over its edge.
- `firesideLounge`: wing chairs with swept rolled arms, extruded shaped wings, a puffed back and piped seat,
  short cabriole legs; a fire basket of bowed bars with ring and legs, crooked logs with pale ends, a lofted hood
  with a rim, flue bands; a tripod table; a bookcase with a moulded top, divider and four rows of books; a woven
  log basket; the flames lofted, leaning tongues.
- `bedsideStack`: a rounded top, plinth and turned legs; shaker drawers with brass cup pulls and label frames;
  the open book's rounded spine and ribbon; folded reading glasses with a bridge and arms; the lamp a drum shade
  with brass rims and finial.

| Model | Size (w × h × d) | Triangles (2026-10-22) | Draws | Colour parts |
|---|---|---|---|---|
| `model.bookworm.readingNook` | 1.88 × 1.71 × 0.65 m | 6,868 (was 4,820) | 7,644 | Wood, Fabric, Cushions |
| `model.bookworm.libraryWall` | 1.88 × 2.34 × 0.82 m | 6,256 (was 6,492) | 6,256 | Wood, Ladder |
| `model.bookworm.writingBureau` | 0.85 × 1.35 × 0.76 m | 7,352 (was 5,144) | 7,368 | Wood, Fabric, Chair |
| `model.bookworm.teaTrolley` | 0.75 × 0.94 × 0.49 m | 3,768 (was 3,892) | 3,768 | Wood, Fabric, Frame |
| `model.bookworm.firesideLounge` | 1.70 × 2.80 × 1.85 m | 11,520 (was 6,070) | 12,416 | Wood, Fabric, Rug |
| `model.bookworm.bedsideStack` | 0.63 × 0.93 × 0.73 m | 3,948 (was 2,976) | 3,948 | Wood, Fabric, Drawers |

The library wall has no fabric to colour; its second part is the ladder.

`check-glb`: 6 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each. In play
(`wants-asking`, WebGPU by day, `office-night` on WebGL at night): the nook and the lounge used, the cushion
sinking and the fire growing; the bureau turned to three facings; colours on the clips bench.

## Rebuild

```
cargo art pack_bookworm            # the six .glb, the pack, check-glb
cargo art pack_bookworm --preview  # previews/pack_bookworm/<id>.png, <id>@<clip>.png
```

## The soft forms pass (2026-11-13)

The reading nook's two throw pillows are sewn flat and stuffed: their sides drawn in 1.2 cm between corners that stand out (`waist`), `resting`. Besides, its soft boxes take the shared form's new shape (the shared `soft_box`: rounded edges turning in equal parts, so the same triangles stand closer to the curve; puffed boxes' corners pinched in; cords following the seams). Same sizes (to 1.2 mm), parts, clips, seat and mattress heights (`cargo review sizes` and `uses` unchanged). 3 models changed shape: `firesideLounge`, `readingNook`, `writingBureau`; 26,604 to 27,428 triangles.

## Gate to zero (2026-11-25)

Every hard edge the gate's `edges` found is eased (55 → 0): the nook's and the library's cornices chamfer their top
arrises 4 mm, the library's plinth moulding its corners; the pendant's crown, the fire basket's rim, the hood's top
(stepping in under the flue), the embers, the flue's bands, the cups' and saucers' rims, the tin's lid, the cake's cut
and the wing chairs' arm ends (`bevel_edges`) and scrolls are rounded or bevelled; the chairs' cabriole legs reach 3 cm
up into their base, so their tops don't show at its rounded edge; the bureau chair's crest rail is eased at its ends.

The books cost what shows of them (`shell_book`): a book standing in a row its spine, top and sides with the edges
between chamfered 4 mm (20 triangles; its back on the case and its bottom on the shelf left out), one lying in a pile
the same with its back (30), where a whole chamfered box took 44. With that, and the nook's back and seat cushions
`resting` (the back ones two flat steps), the globe 16 × 10, the pendant 14 sides, the wing chairs' cushions resting
(the seat one flat step, one rounding step), their legs 8 sides, the tables' tops one bevel step, the tori's sections 3
and the scrolls 12, the trolley's frame 6 sides, castors 10, tin 16 and pot lid 12, and the bedside chest's top one
bevel step, its pulls a flat strap and its glasses' rims 3 sides, every model is under its triangle aim: the nook
7,858 (was 9,980), the library wall 7,876 (12,304), the lounge 11,922 (14,976), the trolley 3,992 (3,832), the bedside
stack 3,918 (4,396), the bureau 7,840 (8,168).

The bedside stack's lamp and the pile on the floor in front of it are `aside` (drawn, not the chest's own size; no
draw call more): it measures 0.60 × 0.66 × 0.45 m, a nightstand. The bureau keeps 4 draw calls (the body plain and
wood, the flap wood and plain: its brass pull and key and leather lining move with it).
