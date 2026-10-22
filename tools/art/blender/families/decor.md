# Decor — notes

`decor.py` builds the 23 decor models into `web/public/assets/models/decor/` and their entries in
`packs/decor/manifest.json`: wall art and clocks (D1), ornaments (D2) and textiles (D3). Built 2026-10-14; they sat in Validate ›
Wall Decor, Ornaments and Textiles until checked in play.

Each is built where the game places it (`F`, game axes):
- **Wall things** (the pictures, the clocks, the shelf, the curtains) hang at their real height with their back on the
  wall face: `G.settle(centre=False, floor=False)`, `floor=False` and `centre=` on the `Model`. `check-glb` warns
  that they float (right on a wall).
- **The hanging plant** hangs from the ceiling (2.8 m), centred, `floor=False`.
- **The fireplace** has a raised hearth bench (0.42 m, `HEARTH`) with the firebox above it: residents warming up
  sit on the bench's front edge, the fire behind them (content `seat`: height 0.42, forward 0.36; the fire's
  `light.height` 0.7). Since 2026-10-21 (review I-decor-01) the bench runs out to `HEARTH_FRONT` 0.45, 0.46 m in
  front of the firebox's opening (a real raised hearth's 0.4–0.5 m), so the sitter's back is 0.1 m clear of the
  opening; it ran 0.14 m out and she sat in the opening, her back in the flames. The model is 0.88 m deep, `centre`
  (0, 0.01).
- **Against the wall**, the grandfather clock and the fireplace have their backs on the wall face, and so do their
  old parts (moved forward 4 and 5 cm, 2026-10-14), and the curtains' (moved back 5 cm onto it, where the
  models' rail brackets reach). The fireplace's old part is redrawn as the model: the chimney breast round a
  dark firebox over the raised hearth bench, so a resident warming up sits on it.
- **The rest** stand centred on their tile, or lie flat (the rugs).
- **Details** (2026-10-14): the ginger jar's blue sprigs are painted-on blossom sprigs (a curving stem, three leaves,
  a five-petalled flower; `_sprig`), not V-shaped pairs that read as crosses; the cat sits on its rump with a thigh
  bulging at either side and its hind feet tucked forward, not on one ball; the coat on the stand hangs by its
  collar, flaring a little to the hem, its sleeves at its sides (`_coat`), not a capsule.
- **The curtains** (2026-10-21, review M-decor-02): a window's opening is its edges less 0.12 m at each end (0.76 m over
  one edge, 1.76 m over two), so each pair's rail is the opening plus its two gathered panels: 1.22 m for `curtains`,
  2.38 m for `curtainsWide` (new: window length over two edges) and `curtainsLong`. Open, the panels stack on the wall
  beside the glass, not over it; the rails run 0.1–0.2 m past the footprint, as real rails run past the frame
  (`check-glb` warns they reach over what hangs next to them). The window-length pairs hang to 0.85 m, just under
  the sill (0.9), where they stopped 10 cm above it and left a strip of glass bare when drawn.
- **The bronze horse** (2026-10-21, review M-decor-01): `bronze` is a golden cast bronze (`#A47B45`, metal) that
  catches the light on its edges, its mane and tail a dark green patina (`bronze_dark`); it was a dark brown that
  read as wood beside its oak plinth.

## Detail pass (2026-10-22)

Rebuilt with the forms and the kit (`G.kit.moulding`, `F.sweep`, `G.loft`, `F.extrude`, `F.soft`, `G.jitter`): same keys,
sizes (within 2 %), centres, parts, clips, motions, paint and draws; every model under its size class's target.
- **Pictures**: each frame one mitred moulding (`G.kit.moulding`, new in the kit): the print's flat face with a sight
  lip, the photo's thin black section, the painting's gilt bead-cove-and-lip with carved shells at its corners and
  crest and a dark bead round the canvas, the canvas's oak L-section float frame. Mounts have bevel-cut windows. The
  art is extruded shapes: the print's curving stem and paired pointed leaves with veins; the photo's two ranges with
  snow on their ridges, haze, pines along the shore and their reflection, ripples (fewer triangles than its boxes
  had, more to see); the painting's far hills, path, fence, chimney, windows and door; the canvas wrapped round its
  stretcher (`F.soft`), a torn-edged block, a brushed line and arc, small marks.
- **Clocks**: the wall clock's rolled bezel and sixty-tick minute track; shaped hands (`_hand_outline`: a slim taper,
  the hour hand a spade) on both clocks. The grandfather clock: moulded plinth, waist, neck and cornice
  (`kit.moulding`), a fielded base panel, quarter columns, a trunk door with an arched window and bead, turned brass
  hood columns, an arched pediment under urn finials, a brass dial plate with spandrels, a chapter ring with Roman
  hours (stroke by stroke, `_numerals`) and a minute track, winding holes, a lens-shaped bob. Still 4 draws.
- **Wall shelf**: shelves with rounded front edges; bound books (boards, pages, spine bands, the last leaning on the
  row); a lipped pot with a pothos trailing heart leaves along the shelf (the size keeps them on it); a framed picture.
- **Ornaments**: vases thrown with feet, grooves and rolled lips; twigs and stems swept, plumes and blooms jittered,
  pointed leaves; the jar's sprigs one sweep each, a foot ring and a flanged lid with a turned knob. Pedestals
  (`_pedestal`) with moulded foot and cap and a fillet framing a sunk panel on each face. The bust's head is lofted
  chin to crown (nose, lids, lips, chin, ears) under a curly cap, its chest cut at the arms with collarbones. The cat's
  tail is one sweep, legs swept, ears lofted with hollows, whisker pads, toes. The horse: a lofted barrel, swept
  neck, head, legs with knees and hocks, hooves, mane, forelock and tail.
- **Coat stand**: a turned pole with beads, legs curving to pads, brass S-hooks; the coat has a collar, lapels,
  buttons, pocket flaps, swept sleeves and cuffs; a fringed scarf over a hook; a turned hat with a band.
- **Hanging plant**: a knotted macramé hanger (strands crossing into a net under the pot, a tasselled knot), a rolled
  pot rim, jittered foliage, vines with heart leaves hanging down.
- **Fireplace**: brick lining (`brick`), fielded panels, a crown moulding, an oak shelf with a moulded nosing on scroll
  corbels, moulded jambs and lintel with corner blocks, rosettes and a keystone, three hearth slabs (top still at
  `HEARTH`), a swept grate with finials, jittered logs with pale ends (`log_end`), lofted flames; the mantel's things
  moved in front of the flue (they stood inside it, unseen).
- **Curtains**: pleats deeper at a thickened hem, easing to the heading tape; turned finials, swept brackets.
- **Rugs**: woven motifs as stepped outlines (`_diamond`): medallions, corner pieces, the round rug's leaf garland
  and star, the patterned rug's three-tier medallion and zigzag border; knotted fringes swept (the large rug's body
  is 2.75 m, its new fringe making the 2.85 m).

The pictures are flat colour shapes (matte, one draw call with the frame), not textures. Kept so (2026-10-14): a
512 px painted texture would be a draw call of its own on each picture (1 to 2) and a download, for art seen
small from the game camera, on a wall thing most houses have several of. Revisit for a close-up view.

## What moves

| Model | Clip | Kind | What |
|---|---|---|---|
| `wallClock`, `grandfatherClock` | `hour_hand`, `minute_hand` | `clock` (`at` 610) | the hands tell the game's time: each clip turns its hand once round clockwise (`G.spin`, 12 s), and the game holds it where the time has it (ten past ten at rest, the catalogue picture). No second hand: at normal speed a game minute is a real second |
| `grandfatherClock` | `pendulum` | `idle` | the pendulum swings ±4° about its top, a beat each 0.8 s (`G.rock`, 1.6 s a round), all the time near the camera (2026-10-21) |
| `hangingPlant` | `sway` | use `water`, `hold` 0.8 s | the plant, pot and hanger swing 5° out about the ceiling hook (`G.swing`, eased) and, played back after the hold, home: one sway (it was a `G.rock`, which played back swung twice) |
| `fireplace` | `flames` | use `relax` | an ember bed and tongues of flame (gloss) grow from nothing on the logs (`G.grow`) for the whole use; the light is the content's |
| `curtains`, `curtainsWide`, `curtainsLong` | `close` | night | the two pleated panels, gathered open at the sides, widen from their outer ends until they meet (`G.squash` in x, about the rail end) |

The grandfather clock takes **4 draw calls** (the most allowed): its body, all plain (1), the two hands and the
pendulum. Since 2026-10-21 (review A-decor-01) its walnut is a polished satin, not the textured wood, which gives
the draw back for the pendulum: a brass rod and bob, its origin on its top under the hood, that swings ±4° a beat
each 0.8 s, all the time near the camera (`pendulum`, `idle`, `G.rock`; at rest on Low), seen through the trunk
door's window. The trunk door stays still: the clock is wound at its dial (`wind`), and a fifth draw would be over
the budget. The curtains'
tabs are their own fabric, so the moving panels cost one draw call each.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.printFramed` | 0.50 × 0.71 × 0.03 m | 1,012 | 1 | Frame |
| `model.photoPrint` | 0.75 × 0.50 × 0.03 m | 520 | 1 | Frame |
| `model.paintingClassic` | 0.84 × 0.65 × 0.05 m | 1,772 | 1 | Frame |
| `model.canvasAbstract` | 1.50 × 0.90 × 0.05 m | 1,200 | 2 | Frame |
| `model.wallClock` | 0.36 × 0.36 × 0.07 m | 2,144 | 3 | Case, Face |
| `model.wallShelf` | 0.90 × 0.66 × 0.24 m | 3,272 | 3 | Shelves |
| `model.grandfatherClock` | 0.60 × 2.12 × 0.40 m | 6,394 | 4 | Case, Face |
| `model.vaseTall` | 0.33 × 1.27 × 0.23 m | 3,318 | 2 | Body |
| `model.urnPorcelain` | 0.36 × 0.63 × 0.36 m | 3,940 | 1 | Body |
| `model.vasePedestal` | 0.40 × 1.60 × 0.40 m | 3,292 | 2 | Vase, Pedestal |
| `model.bustMarble` | 0.44 × 1.54 × 0.44 m | 3,224 | 1 | Bust, Plinth |
| `model.statueCat` | 0.31 × 0.60 × 0.41 m | 3,636 | 1 | Body |
| `model.statueHorse` | 0.47 × 1.41 × 0.44 m | 2,238 | 2 | Body |
| `model.coatRack` | 0.44 × 1.79 × 0.36 m | 5,080 | 3 | Body |
| `model.hangingPlant` | 0.46 × 1.81 × 0.47 m | 4,966 | 3 | Pot, Cord |
| `model.fireplace` | 1.96 × 2.80 × 0.88 m | 5,492 | 3 | Surround, Mantel |
| `model.curtains` | 1.30 × 1.67 × 0.10 m | 3,540 | 3 | Fabric, Rail |
| `model.curtainsWide` | 2.46 × 1.67 × 0.10 m | 4,564 | 3 | Fabric, Rail |
| `model.curtainsLong` | 2.46 × 2.58 × 0.10 m | 4,564 | 3 | Fabric, Rail |
| `model.rugRound` | 1.90 × 0.02 × 1.90 m | 3,388 | 1 | Rug |
| `model.rugLarge` | 2.85 × 0.02 × 1.85 m | 2,396 | 1 | Rug |
| `model.rugRunner` | 0.80 × 0.02 × 2.80 m | 1,808 | 1 | Rug |
| `model.rugPattern` | 2.84 × 0.02 × 1.86 m | 3,592 | 1 | Rug |

`check-glb`: 23 models, 0 failed. Khronos validator: 0 errors, 0 warnings each.

## Rebuild

```
cargo art decor            # the 23 .glb, the pack, check-glb
cargo art decor --preview  # previews/decor/<id>.png, <id>@<clip>.png
```
