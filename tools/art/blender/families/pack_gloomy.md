# Gloomy pack — notes

`pack_gloomy.py` builds the six P7 gloomy models into `web/public/assets/models/pack_gloomy/` and their entries in
`packs/pack_gloomy/manifest.json`, an override pack listed in the base manifest. Built 2026-10-14; they sat in
Validate › Gloomy until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`),
backs on the wall's face (−0.43; the tub's ledge −0.93). Seats keep the default 0.45 (the cello's stool, the desk's
chair, the window seat's cushion top); the tub keeps `SEATS`/`BEDS` 0.25 (its inside floor is at 0.095).

| Model | Moving part and clip | Played by |
|---|---|---|
| `recordCabinet` | the lid over the turntable's well, hinged along its back: `lid_open` lifts it 80° (0.9 s) | `listen`, `newAlbum` |
| `recordCabinet` | the platter and record, origin on the spindle: `platter_spin`, one turn clockwise in 1.8 s (33 rpm), a loop | `listen`, `newAlbum` (loop) |
| `cello` | the cello, built upright on its stand, origin on the end pin's foot: `off_stand` (0.9 s) moves it across to between the player's knees and leans it back 14° | `practise`, `lament` |
| `journalDesk` | the tall candle's flame, built full size on its wick and scaled to nothing: `candle` lights it (0.6 s); its material glows at night. Its top is at 0.75 (2026-10-21: 0.78, over a desk's 0.72–0.76: M-packs-08), everything on it 3 cm lower with it | `journal`, `poetry` |
| `rainLamp` | the rain, 18 lines with drops from the canopy to the pool, squashed to nothing under the canopy: `rain` runs them down (0.6 s). Under the canopy its light (`lamp_light`), the pool and the rain (`pool`, `rain`) glow at night with the content's `light` (range 2.5 m, at 1.3 m), switched as a lamp (2026-10-21, M-packs-07) | `watchRain` |
| `rainLamp` | the ripples on the pool: `ripple` swells them 12 % and back (2 s), ambient (`idle`) | always (not on Low) |
| `windowSeat` | the seat cushion with its throw and a book, origin on the underside: `sink_in` presses them 2 cm (0.6 s) | `brood`, `doze` |
| `soakingTub` | the water up to 0.52 with a little foam, origin under the tub's floor, flat there at rest: `fill` raises it (1.5 s) | `soak` |

The music animation poses the player round the model's own cello (characters.ts, `def.includes('cello')`), so the
cello is a part of the model and `off_stand` brings it to where the pose expects it (the old part's spot, end pin
at depth 0.3). The old part's music stand is gone (it stood where the cello is played); an open book of music
lies on the floor and the bow hangs on the cello's stand.

The rain lamp is an open frame (a canopy on three posts over a pool, a little figure under an umbrella on a rock),
not a glass column: no `glass` draw. The plan's "A rain (E2)" is the ambient `ripple` on the pool; the rain itself
runs only while watched, as the table's `rain` clip says.

The window seat's pane is opaque gloss with raindrops and runs on it, not see-through glass: it stands against the
house wall, which glass would show.

**Draw calls: 3 each, the record cabinet 4.** The cabinet's walnut (`wood`) body, its lid and the platter loop (the
plan's extra "L the platter turns (E1)"): 4, `check-glb`'s one warning. Its lid pull is wood, not brass, so the
lid stays one draw. Elsewhere: the cello's body and stand `wood`, the cello itself one plain part; the desk `wood`,
the chair's upholstery plain satin; the window seat all plain but its cushion part (`fabric` throw and cushion, a
satin book); the tub's ledge `wood`, the rest plain.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.gloomy.recordCabinet` | 0.90 × 0.93 × 0.82 m | 3,922 | 4 | Body, Trim |
| `model.gloomy.cello` | 0.78 × 1.43 × 0.74 m | 6,424 | 3 | Body, Trim, Cello |
| `model.gloomy.journalDesk` | 1.50 × 1.20 × 0.83 m | 7,134 | 3 | Body, Trim |
| `model.gloomy.rainLamp` | 0.81 × 1.50 × 0.62 m | 7,384 | 3 | Body, Trim |
| `model.gloomy.windowSeat` | 1.84 × 1.97 × 0.71 m | 5,828 | 3 | Body, Cushion, Trim |
| `model.gloomy.soakingTub` | 1.86 × 0.92 × 1.85 m | 6,848 | 3 | Body, Trim |

The desk's apron runs only along its far edge and its ends (the old part's deep apron took the writer's hands).

`check-glb`: 6 models, 0 failed; the record cabinet's 4-draw warning above. Khronos validator: 0 errors, 0
warnings each. Gallery next to the old parts; clips bench at rest, open and mid-loop on WebGPU and WebGL,
coloured (cabinet, window seat). In play (`wants-asking`, WebGPU): the cello played (in the player's arms), the
record played (lid up, record turning), brooding on the window seat, writing at the desk by the lit candle, watching
the rain, soaking in the tub. At night (`office-night`, WebGL): the window seat in four facings, the tub, the
cabinet, the desk and the lamp. Not seen: the cello in the night yard (no free spot for its facing), `doze` on the
window seat.

## Detail pass (2026-10-22)

Rebuilt with the forms and the kit, keeping every key, size, centre, part, hinge, clip, `motions` and `paint`
(triangles before → after: cabinet 2,552 → 3,922, cello 3,924 → 6,424, desk 2,768 → 7,134, lamp 5,334 → 7,384,
window seat 3,668 → 5,828, tub 4,120 → 6,848; draws unchanged).

- `recordCabinet`: splayed tapered legs with brass ferrules, an arched apron, a slatted door in a walnut frame (an
  outline with a hole) with a brass bar pull (`kit.pull`), a record pulled half out of the LPs, the turntable's
  pivot, S-curved tonearm (`kit.tube`), counterweight, headshell and speed buttons; a turned platter and record; the
  lid a frame round a sunk panel; a candle with a melted top and drips on a dish; a lofted vase of branching twigs
  with honesty pods. Trimmed to the 0.35–1 m target (slats unbevelled, 7-sided pods).
- `cello`: the body lofted from a figure-eight outline, its plates arched over the ribs (a little shallow, to keep
  the old 0.74 m depth), f-holes swept in S-curves, a tailpiece with fine tuners, a carved bridge (feet, kidneys, a
  heart: an outline with a hole), a D-section neck, a tapering fingerboard, nut, pegbox, scroll (`kit.scroll`), pegs
  with heads, four strings over the bridge, the end pin in its collar. The stool is turned with a padded, piped,
  buttoned top on three splayed legs and a foot ring; the stand's feet, cup, post, cradle and a yoke round the neck
  are swept; the bow is cambered with its frog, tip and hair; the music lies open with curved pages and staves.
- `journalDesk`: cabriole legs (`kit.cabriole`), scalloped aprons, a gallery pierced by nine pointed arches, a
  moulded shelf on scroll brackets; the journal open with ruled curved pages, a square inkwell with a brass collar
  and a quill with its vane, books with boards and page blocks, a wilted rose and fallen petals, three dripping
  candles on a brass dish. The chair: a buttoned plum back, sabre back legs, finials, a pointed crest, turned front
  legs, an H stretcher.
- `rainLamp`: a moulded base, tapered posts with brass collars, a pagoda canopy with a scalloped eave (lofted rings),
  a turned finial; a boulder rock with moss; the figure in a lofted coat and hat under a scalloped umbrella with a J
  handle; boulder pebbles.
- `windowSeat`: the bench on a recessed plinth with two shaker drawers (`kit.shaker`) and brass pulls, a rounded
  nosing; a sash of four lights (an outline with holes), a moulded casing (`kit.moulding`), a nosed sill and apron;
  runs on the glass swept; candles on dishes, a mug with a handle; soft pillows. The seat part: a piped soft cushion,
  a throw draped over its front edge (swept, jittered), a book.
- `soakingTub`: a smoother tub profile with a rolled rim and a foot, two moon bands; the ledge's ends in raised
  shaker panels, a bullnosed stone top (swept); dripping candles, soft folded towels with bands, a vase of arching
  fern fronds, a soft mat with a fringe; foam as boulders. (Moon-phase discs on the tub were tried and dropped: their
  vertices darkened the tub's long faces in the baked shading.)

`check-glb`: 6 models, 0 failed; the record cabinet's 4-draw warning (as before). Khronos validator: 0 errors,
0 warnings each.

## Rebuild

```
cargo art pack_gloomy            # the six .glb, the pack, check-glb
cargo art pack_gloomy --preview  # previews/pack_gloomy/<id>.png, <id>@<clip>.png
```
