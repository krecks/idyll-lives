# Bedroom — notes

`bedroom.py` builds the nine beds of BR1 into `web/public/assets/models/bedroom/` and their entries in
`packs/bedroom/manifest.json`: `model.bed`, `model.bedSingle`, `model.bedBunk`, `model.bedFuton`, `model.bedKing`,
`model.bedDouble`, `model.bedQueenUpholstered`, `model.bedCanopy` and `model.bedDaybed`. Built 2026-10-14; they sat in Validate › Beds until checked in play (back to Bedroom › Beds).

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
  two parts: the lower two thirds (`_duvet`, one soft, puffed box draped down the sides and the foot, still) and
  the top third (`duvet_flap`, flat on the mattress with the sheet turned down over its head edge), hinged
  across the fold on the duvet's top 3 mm up. `turn_down` turns the flap 180° about X in 1 s, laying it over the
  rest toward the foot, and flattens it as it goes (`_turn_down`); half way over it is gathered to a fifth of its
  length (`ROLL`, a scale along the bed there and back, 2026-10-21), so it rolls over its fold as cloth, a low
  bulge, not a 0.45 m board standing up across the bed's head. The game's quilt lies 10 cm above the mattress
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
| `model.bed` | 1.90 × 1.13 × 1.88 m | 5,928 | 3 | `turn_down` | upholstered linen base piped round top and bottom, piped headboard with four lofted padded channels, teak legs, a sage runner swept over the foot |
| `model.bedSingle` | 0.96 × 0.95 × 1.91 m | 6,200 | 3 | `turn_down` | oak posts with collars and ball finials, an arched top rail over five rounded slats, a framed and panelled footboard |
| `model.bedBunk` | 0.97 × 1.90 × 1.90 m | 7,244 | 4 | `turn_down` (lower bunk, slot 0), `turn_down_upper` (slot 1, 5 s in) | oak; posts with capped tops, arched end boards (the upper bunk's with a hand hold), the ladder hooked over the upper rail; the ladder (right, at the foot) is still; each bunk's duvet turns down for its own sleeper (`slot` in `motions.uses`), at a fourth draw; the upper one once its sleeper has climbed the ladder and crawled in (`delay` 5 s, `BUNK_CLIMB_IN`: the climb is 7 s), not at the ladder's foot (A-bedroom-06) |
| `model.bedFuton` | 1.96 × 0.99 × 1.91 m | 7,280 | 3 | `unfold` (held open) | ash slatted platform with rounded rails and tenons showing, a low headboard with four slots under a rounded cap; tufted seat and back, a folded quilt |
| `model.bedKing` | 1.98 × 1.40 × 1.90 m | 7,396 | 3 | `turn_down` | piped base and headboard, five lofted padded channels, piped wings, tapered square legs, sage runner |
| `model.bedDouble` | 1.62 × 0.95 × 1.90 m | 6,100 | 3 | `turn_down` | oak frame with a bead, posts with pyramid caps, an arched top rail over eight rounded teak slats, tapered square legs, blue duvet |
| `model.bedQueenUpholstered` | 1.76 × 1.30 × 1.89 m | 6,064 | 3 | `turn_down` | stone upholstery; a headboard scalloped over five domed padded channels; piped base |
| `model.bedCanopy` | 1.73 × 2.135 × 1.90 m | 7,900 | 3 | `turn_down` | turned walnut posts (vase, rings, finial) on foot blocks, a headboard of three raised panels under an arched crest, scalloped linen valance, four pleated curtains drawn in by tie-backs (still) |
| `model.bedDaybed` | 0.96 × 0.84 × 1.92 m | 6,676 | 2 | — | white posts with ball finials, turned spindles, rounded rails; piped sage fitted cover, piped rose cushions (static) |

### Detail (2026-10-22)

Rebuilt with the forms (gamekit/forms.py) and the kit, same sizes, centres, parts, clips, draws and mattress tops:
- **Bedding.** `_mattress`: a soft box with a piped seam round its top and bottom, its middle exactly at the mattress
  top (`BEDS`), puffed a few mm. `_pillow`: plump in the middle, thin at the seams, a little crumpled (`G.jitter`).
  The duvet (`_duvet`) and its flap (`_flap`) are soft boxes puffed and rumpled, their tops still at `DUVET`
  (the flap's sheet too), so the quilts over sleepers still hide the turned-down flap. The runners are one cloth
  swept over the foot (`_runner`, its path's corners rounded by hand: Catmull-Rom overshot 9 cm off the long
  top span).
- **Headboards.** `_channel` lofts a padded column (domed for the queen), `_board` an arched board with slots,
  `_post` a post with a collar and a ball, pyramid or flat cap. The canopy's posts are turned (`G.revolve`),
  its curtains lofted through pleated sections, its valances extruded with scalloped edges.
- **Baked shading.** A panel set into a frame stops a hair inside the opening: a vertex buried in another part
  bakes black (`shade.py`), and a face with only corner vertices goes dark all over (the single bed's
  footboard and the canopy's panels did).

`check-glb`: 0 failed; its one warning is the bunk's 4 draw calls. The old `futon` was the futon lying flat
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
cargo art bedroom            # every .glb, the pack, check-glb
cargo art bedroom --preview  # previews/bedroom/bed.png, bed@turn_down.png, …
```
