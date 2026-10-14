# Bookworm pack — notes

`pack_bookworm.py` builds the six P1 bookworm models into `web/public/assets/models/pack_bookworm/` and their
entries in `packs/pack_bookworm/manifest.json`, an override pack listed after `packs/bookworm/manifest.json`. Their
primitive-part placeholders (`tools/art/pack_bookworm.py`, in `packs/bookworm`) stay as the fallback. Built
2026-10-14; they sit in Validate › Bookworm until checked in play.

Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`),
with one moving part. Pack families write to a pack of their own (`pack_<trait>`), keyed `model.<trait>.<id>`, so
the trait's pack keeps its placeholders.

| Model | Moving part and clip | Played by |
|---|---|---|
| `readingNook` | the seat cushion and throw, origin on their underside: `sink_in` presses them 2 cm (0.6 s) | `curlUp` |
| `libraryWall` | the rolling ladder: `ladder` rolls it 40 cm along the brass rail, towards the middle (1.2 s) | `research`, `browse` |
| `writingBureau` | the writing flap over the pigeonholes, hinged on its foot: `fall_front` drops it 90° onto the desk (0.8 s) | `draft`, `letter` |
| `teaTrolley` | the teapot, hinged on its foot under the spout: `pour` tips it 30° (0.8 s), `hold` 1.5 s | `brew` |
| `firesideLounge` | the flames on the grate's logs, built flat: `flames` grows them (0.8 s) | `readByFire`, `readAloud` |
| `bedsideStack` | the top book's cover, hinged on the spine: `book_up` opens it 170° to the right (0.8 s) | `chapter` |

**Draw calls: 3 each.** The still body takes its plain finishes and one textured finish, the moving part one
more. So that it stays at 3:
- the nook's back cushions and pillows, the bureau chair's pad and the trolley's cloth are plain satin (the
  nook's seat cushion and throw, alone on their moving part, are `fabric`);
- the lounge's wood is plain satin (`walnut`), as its chairs and rug take the `fabric` draw;
- the fire is an open grate under a hood, so the flames show from the room (the placeholder's stove faced the
  chairs, its fire hidden from the camera); no glass.

Book spines are plain satin, unbevelled boxes: a wall of books costs no draw call and few triangles.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.bookworm.readingNook` | 1.88 × 1.71 × 0.65 m | 4,820 | 3 | Wood, Fabric, Cushions |
| `model.bookworm.libraryWall` | 1.88 × 2.34 × 0.82 m | 6,492 | 3 | Wood, Ladder |
| `model.bookworm.writingBureau` | 0.85 × 1.35 × 0.76 m | 5,144 | 3 | Wood, Fabric, Chair |
| `model.bookworm.teaTrolley` | 0.75 × 0.94 × 0.49 m | 3,892 | 3 | Wood, Fabric, Frame |
| `model.bookworm.firesideLounge` | 1.70 × 1.96 × 1.85 m | 5,834 | 3 | Wood, Fabric, Rug |
| `model.bookworm.bedsideStack` | 0.85 × 0.93 × 0.74 m | 3,624 | 3 | Wood, Fabric, Drawers |

The library wall has no fabric to colour; its second part is the ladder.

`check_glb.py`: 6 models, 0 failed, no warnings. Khronos validator: 0 errors, 0 warnings each. In play
(`wants-asking`, WebGPU by day, `office-night` on WebGL at night): the nook and the lounge used, the cushion
sinking and the fire growing; the bureau turned to three facings; colours on the clips bench.

## Rebuild

```
python3 -B tools/art/blender/build.py pack_bookworm            # the six .glb, the pack, check_glb
python3 -B tools/art/blender/build.py pack_bookworm --preview  # previews/<id>.png, <id>@<clip>.png
```
