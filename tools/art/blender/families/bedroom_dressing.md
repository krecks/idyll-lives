# Bedroom dressing — notes

`bedroom_dressing.py` builds the bedroom's mirrors and dressing table (BR4) into
`web/public/assets/models/bedroom_dressing/` and their entries in `packs/bedroom_dressing/manifest.json`:
`model.mirror` (`mirror.glb`, the cheval mirror), `model.dressingTable` (`dressingTable.glb`), `model.mirrorWall`
(`mirrorWall.glb`) and `model.mirrorFloor` (`mirrorFloor.glb`). They replace the primitive-part placeholders of
`furniture.py` / `objects_extra.py` (the cheval mirror), `catalog.py` (the dressing table, the wall mirror) and
`catalog2.py` (the standing mirror), which stay as the fallback. Built 2026-10-14. They sit in Validate › Dressing
until checked in play.

## Shared

- **Mirror glass (E6).** Each mirror's glass is one flat box (or oval) of a mirror material: `mirror`
  (`'gloss', 'front'`) on the three mirrors, `mirror_back` (`'gloss', 'back'`) on the dressing table, whose
  mirror faces the seat at the back. The game lays the reflection on the glass's box, across its thinnest
  side, so **the glass stands upright**: the placeholders of the cheval and standing mirrors lean back 6° and 8°;
  these don't (a tilted pane's box would put the reflection beside it). The dressing table's glass is an oval,
  so the game draws an oval reflection; the others are rectangles.
- **Gloss, not see-through.** The mirror materials map to `gloss`, as the placeholders' mirrors: only the
  High preset reflects live, so elsewhere (the gallery, Low and Medium, a mirror beyond the nearest two) the
  pane shows as pale gloss, never what is behind it.
- **Frames** are swept loops (`_loop`): a rounded-rectangle cross-section round a rounded rectangle or an
  ellipse, so a frame is one closed mesh. The glass is set into the frame from the front: its face is behind
  the moulding's inner lip, so the reflection (2 mm in front of the face) stays under the lip.

## The models

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.mirror` | 0.80 × 1.86 × 0.56 m | 2,892 | 3 | Frame (wood) |
| `model.dressingTable` | 0.86 × 1.53 × 0.85 m | 5,848 | 4 | Table (wood), Frame (brass) |
| `model.mirrorWall` | 0.62 × 0.96 × 0.09 m | 1,136 | 2 | Frame (black metal) |
| `model.mirrorFloor` | 0.56 × 1.78 × 0.45 m | 1,048 | 2 | Frame (brass) |

- **`mirror`** — a cheval mirror: two turned posts on feet and a low stretcher, the swing frame between them on
  brass pivots at 0.95 m, a crest with a finial. Centred on its tile (its placeholder is 5 cm back).
- **`dressingTable`** — where its placeholder stands (`centre=(0, -0.006)`): the pouf at the back (depth −0.22,
  its top 0.45 m: characters.ts `SEATS` h 0.45, fwd −0.22), the table (top 0.76 m) in front of it on four
  tapered brass legs, two drawer cases either side of the kneehole, the oval mirror on a brass stand at the far
  edge facing back at the seat, with a wooden back. On the top: two perfume bottles and a jewellery box.
  - **The drawer.** The case on the seated resident's right (game +x: the game is left-handed) slides its drawer
    out 18 cm towards the seat in 0.8 s (`drawer_open`, `G.slide`), its tray hidden in the case at rest, a dark
    `inside` panel behind it. `motions` plays it for `getReady` and holds it 3 s, then shuts it.
  - **Draw calls: 4** (the most allowed; `check_glb.py` warns). The still body's wood, its plain finishes, the
    mirror glass, and the drawer (wood, with a wooden knob so it stays one). The pouf is a plain `satin`, not
    `fabric`, to stay at 4.
- **`mirrorWall`** — on the wall layer: built in place at the placeholder's height (frame 1.0–1.9 m, ledge at
  0.97 m), its back on the wall face (depth −0.43); `G.settle(centre=False, floor=False)` and
  `G.Model(..., floor=False, centre=(0, -0.385))`. A black metal frame, the glass, and a ledge with a cup and a
  toothbrush (as the placeholder: house generation hangs it over basins too).
- **`mirrorFloor`** — a full-length mirror in a slim brass frame, upright on a brass stand: two sled feet, a
  clamp up from each to the frame's bottom, a rear bar and a strut to a plate on the frame's back. Where its
  placeholder stands (`centre=(0, -0.155)`: the frame 5 cm behind the middle, the strut behind it).

`check_glb.py`: 0 failed. Warnings: the dressing table's 4 draw calls (above), and the wall mirror "floats
93.6 cm above its origin" (it is on the wall, as it should be). Khronos validator: 0 errors, 0 warnings each.

## In the game

Checked 2026-10-14 headless (WebGPU and WebGL2): the gallery next to the placeholders, the clips bench
(`drawer_open`), and in play: each bought, the wall mirror on a wall, a resident sitting at the dressing table
to get ready (the drawer opens beside them and shuts after the hold), turned to other facings. With
`&quality=high` (live mirrors are High only, the nearest two) all four reflect the room; the dressing table's oval
shows the seated resident.

## Rebuild

```
python3 -B tools/art/blender/build.py bedroom_dressing            # every .glb, the pack, check_glb
python3 -B tools/art/blender/build.py bedroom_dressing --preview  # previews/mirror.png, dressingTable@drawer_open.png, …
```
