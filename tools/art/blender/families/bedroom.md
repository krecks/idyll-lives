# Bedroom — notes

`bedroom.py` builds the nine beds of BR1 into `web/public/assets/models/bedroom/` and their entries in
`packs/bedroom/manifest.json`: `model.bed`, `model.bedSingle`, `model.bedBunk`, `model.bedFuton`, `model.bedKing`,
`model.bedDouble`, `model.bedQueenUpholstered`, `model.bedCanopy` and `model.bedDaybed`. Built 2026-10-14; they sit in Validate › Beds until checked in play (back to Bedroom › Beds).

## Shared

- **Mattresses centred, at the renderer's mattress heights.** The game lays sleepers centred on the footprint (head about
  0.84 m behind the middle, feet 0.86 m in front) at characters.ts `BEDS` (`BUNKS` for the bunk's two). Each
  mattress top is that height: `bed` 0.58, `bedSingle` 0.5, `bedBunk` 0.44 and 1.38, `bedFuton` 0.4, `bedKing`
  0.59. The oak double, the queen and the four-poster are 0.52 and the daybed 0.51, as the old parts were: they
  got `BEDS` entries with those tops. Headboard faces stay behind depth -0.87, so heads don't go into them.
- **The head end stands on the wall's face** (depth -0.93): headboards, posts, rails and the canopy's head end
  stop there, not the mattress, so the sleepers keep their places. Each bed's bounds therefore stand 1–2.5 cm
  forward of the middle (the Model's `centre`, printed by `_settle`); the old parts were cut back the same way
  (`to_wall`, the posted beds' head ends moved forward).
- **The duvet turns down (`turn_down`).** The game lays its own quilt over each sleeper on `BLANKET_BEDS` (now
  also `bedDouble`, `bedQueenUpholstered`, `bedCanopy`), so the bed's duvet isn't a second cover over them. It is
  two parts: the lower two thirds (`_duvet`, one bevelled box draped down the sides and the foot, still) and
  the top third (`duvet_flap`, flat on the mattress with the sheet turned down over its head edge), hinged
  across the fold on the duvet's top 3 mm up. `turn_down` turns the flap 180° about X in 1 s, laying it over the
  rest toward the foot, and flattens it as it goes (`_turn_down`). The game's quilt lies 10 cm above the mattress
  off the body times the sleeper's size (8.5 cm for the smallest), rounding down to 6 cm over its outer 8 cm. A
  flap folded at full thickness topped out at 10.1 cm (its sheet at 10.5) and showed through it, along the
  sides of the single, the bunk and the king and between two sleepers in a wide bed. Folded, it is now 40 % of
  its thickness (`FOLDED`), its sheet's top 7.5 cm above the mattress, and on the one-tile beds and the king it
  narrows to 8 cm inside the quilt's edge (`QUILT_EDGE`: 0.46 m from a one-tile bed's middle, 1.0 m on a two-tile
  bed's outer side); the king's runner is 1.93 m across, just round its duvet, for the same reason. Without a
  quilt (as the resident lies down, before it comes up) the bed still shows turned down. `motions` plays it for
  `sleep` and `nap` with no `hold`: it turns down as they get in and back up after. One sleeper in a wide bed
  turns down both halves; the quilt's inner edge meets the fold on the empty half.
- **Pillows** lie where the sleepers' heads go (about -0.68), under the duvet's head edge (-0.3 to -0.42);
  no scatter cushions where heads lie.
- **Materials** are named for what and which colour they are (`frame_oak`, `headboard_linen`, `bedding_teal`):
  one family palette, each bed its own defaults. `paint` is always Frame, Headboard, Bedding (that order);
  mattress, sheet, pillows, legs, the runner and the canopy keep their colours.
- **Draw calls: 3** (the still bed's plain/wood/fabric groups, the flap's fabric), the daybed 2.

## The beds

| Model | Size (w × h × d) | Triangles | Draws | Clip | Note |
|---|---|---|---|---|---|
| `model.bed` | 1.90 × 1.13 × 1.88 m | 2,540 | 3 | `turn_down` | upholstered linen base, four-channel headboard, teak legs, sage runner |
| `model.bedSingle` | 0.96 × 0.95 × 1.90 m | 2,776 | 3 | `turn_down` | oak posts, five-slat headboard, low footboard |
| `model.bedBunk` | 0.96 × 1.90 × 1.89 m | 4,260 | 4 | `turn_down` (lower bunk, slot 0), `turn_down_upper` (slot 1) | oak; the ladder (right, at the foot) is still; each bunk's duvet turns down for its own sleeper (`slot` in `motions.uses`), at a fourth draw |
| `model.bedFuton` | 1.96 × 0.995 × 1.91 m | 3,112 | 3 | `unfold` (held open) | ash slatted platform, low headboard; the back leans up at rest |
| `model.bedKing` | 1.98 × 1.40 × 1.90 m | 3,360 | 3 | `turn_down` | tall five-channel headboard with wings, sage runner |
| `model.bedDouble` | 1.62 × 0.95 × 1.89 m | 2,964 | 3 | `turn_down` | oak frame, eight teak slats, blue duvet |
| `model.bedQueenUpholstered` | 1.76 × 1.30 × 1.89 m | 2,876 | 3 | `turn_down` | stone upholstery, five rounded channels |
| `model.bedCanopy` | 1.73 × 2.135 × 1.90 m | 4,960 | 3 | `turn_down` | walnut posts with finials, panelled headboard, linen valance and four tied curtains (still) |
| `model.bedDaybed` | 0.96 × 0.84 × 1.91 m | 4,720 | 2 | — | white spindle back and arms, sage fitted cover, rose cushions (static) |

`check_glb.py`: 0 failed; its one warning is the bunk's 4 draw calls. The old `futon` was the futon lying flat
(0.66 m); it is the folded sofa now, as the model rests (2026-10-14). Khronos validator: 0 errors, 0 warnings
each.

The beds fill their 2-deep tiles, and their headboards reached 1–5 cm past the wall face (−0.93), as their
old parts did; moving them forward would have pushed their feet past the tiles or their headboards onto the
sleepers. Since 2026-10-15 the head ends are tucked instead (above): the padded headboards are 3.5–4 cm thick
(were 5–6), the posted ones' head ends (single, bunk, oak double, four-poster, daybed, futon frame) stand
2.5–5 cm further forward, the king's wings 3.7 cm. Mattresses, pillows and feet are where they were, but for the
daybed's mattress and the futon's unfolded head end, which stop 3–5 cm sooner, at the new head end.

## The futon

At rest a sofa: the front of the futon is the seat (depth -0.2 to 0.93) and its head end (`futon_back`, with
the pillows) leans up 66° from its hinge at the seat's back edge. It is built flat (the bed), hinged, turned up to
rest, and `unfold` swings it down flat in 1 s: one mattress at 0.4 m from -0.92 to 0.93, pillows at the head.
`motions`: `uses` sleep, nap and unfold, `open` (held while the sim's `open` is set: content `leaves: open` on
sleep, nap and Unfold, `when`/`leaves` on Fold away). Behind the back, at rest, the slats show.

## The four-poster's top is open

Its old part had a solid linen roof; the model has a valance round the top rails and curtains at the posts, so
residents asleep in it stay in view from above.

## Rebuild

```
python3 -B tools/art/blender/build.py bedroom            # every .glb, the pack, check_glb
python3 -B tools/art/blender/build.py bedroom --preview  # previews/bed.png, bed@turn_down.png, …
```
