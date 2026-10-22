# Street furniture — notes

`street_furniture.py` builds the ten CM2 street furniture models (docs/objects/community.md) into
`web/public/assets/models/street_furniture/` and their entries in `packs/street_furniture/manifest.json`: the street
lamp (`model.streetLamp`), the long park bench (`model.parkBench`), the drinking fountain (`model.drinkingFountain`),
the litter bin (`model.litterBin`), the bus shelter (`model.busStop`), the post box (`model.postBox`), the notice board
(`model.noticeBoard`), the phone box (`model.phoneBooth`), the town fountain (`model.fountainTown`) and the founder's
statue (`model.townStatue`). The street dressing props (`model.street.*`) are not part of it. Built
2026-10-14; they sat in Validate › Street until checked in play.

They stand on the town's park lots (`web/public/content/houses.json` `parks`) and can be bought for a garden. Each is
built in the game's axes (`F`) where the game places it, at the content's heights:

- the bench's seat top at 0.45 m, centred on depth 0 (`seat`: 0.45, forward 0);
- the shelter's bench top at 0.46 m, centred at depth −0.16 (`seat`: 0.46, forward −0.16);
- the fountain's rim top at 0.5 m, its front rim's middle at depth 1.32 (`seat`: 0.5, forward 1.3);
- the lamp's lantern from 3.46 to 3.88 m round the content's `light.height` 3.5.

The bench, the litter bin and the notice board stand a few centimetres back of their tile's middle, as their
old parts do (`G.settle(centre=False)` and `centre=` on the `Model`); the rest are centred by `G.settle`.

## One palette per model

Every model's main part is `body` (the part the player colours), but its colour and finish differ: a cast-iron lamp
post (metal), a red gloss post box and phone box, a stone fountain (matte), a walnut notice board (wood), a bronze
statue (metal). A `G.Palette` has one colour and finish per name, so each build function makes its own palette
(`_palette(body=…)`, the shared colours plus its own body) and the family's palette (`PALETTES`) answers the pack's
`materials` with the model just built.

## The clips

- **Drinking fountain** (`water_on`, use `drink`): the arc of water from the bubbler's mouth into the bowl, a swept
  tube with its origin at the mouth and scaled to 0.001 on all three axes at rest; `G.grow` on X, Y and Z
  grows it out of the mouth in 0.35 s. It is `spray`, gloss, never emissive. Played back when the drink ends.
- **Post box** (`flap`, use `post`, `hold` 1 s): the slot's flap, hinged on its top edge in a housing proud of the
  pillar, swings 65° in (its bottom edge to the back) over a dark opening, in 0.5 s.
- **Phone box** (`door_open`, use `call`: `delay` 0.3 s, `hold` 2.4 s, `out` 1.6 s): the door swings 90° out, square
  to the front, in 0.8 s. The caller stands back from the tile in front while it opens, steps inside onto the plinth
  (`STEP_IN` in characters.ts) and it shuts behind them; when the call ends it opens again and they step back out.
  (It swung in until 2026-10-17, to clear a caller who stayed on the tile in front; with them inside it would sweep
  through them.) Seen in play.
- **Town fountain** (`water_run`, `idle`): the water runs all the time near the camera (not on Low). Two parts move
  in one clip of 1.2 s, each a `G.squash(..., back=True)` pulse that ends where it starts: the jet on the top knob
  rises to 1.3 times its height and falls back, and the splashes where the eight streams land in the basin bob up
  to 1.8 times theirs. The streams spilling from both bowls are still, on the body. `check-glb` gives no loop
  warning: a pulse comes back to rest exactly.
- The lamp, the bench, the bin, the shelter, the notice board and the statue are still.

## Night

- The lamp's four frosted panes are `lantern` (satin) with `glow` `#FFD596`: they glow from dusk as the lamps come up
  (E8), out of doors so no room's lights put them out. The lamp's light itself is the content's `light`, unchanged.
- The phone box's four TELEPHONE signs are `sign` (matte) with `glow` `#FFF1D2`.

## Far versions (E7)

Every model has a `<id>.far.glb` for the town overview (the menus' town). The lamp, the bench, the bin, the shelter,
the post box, the notice board and the phone box use `far=0.25`: the model at rest, joined and decimated to a quarter.
The drinking fountain, the town fountain and the statue use `_far(build)`: the same, but the vertices that sit on one
another are merged first (turned parts meet on their axis in many vertices, and the fountains' pillars and copings
decimated unmerged tore into spikes), the vertices below 0.1 m (the feet) and on the model's outermost faces are left
out of the decimation. They sit on the floor at the full model's size.

## The models

Rebuilt in detail on 2026-10-22 with the forms and the kit, each at most half its size class's triangle target
(they stand by the dozen on the town's lots and the town overview draws them all).

- **Street lamp**: a square chamfered plinth, a fluted bell rising to a moulded collar, a fluted post with a collar part
  way, a neck with the lamplighter's ladder bar (ball ends), four iron scrolls (`kit.scroll`) from a turned capital out
  under the lantern; the lantern's frosted panes widening upwards in an iron frame between a moulded floor and top, a
  hipped roof curving in, a vented crown and a turned finial. One draw call.
- **Park bench**: five rounded slats and three back slats, 2.5 m long, with domed bolts, on three cast-iron supports in
  one piece each (`kit.bench_end`: back leg into the upright, seat rail, stretcher, a curl under the seat; armrests
  curling down in scrolls on the end two); its three sitters a metre apart on its three tiles.
- **Drinking fountain**: a granite slab, a turned stone pillar (moulded base, fluted shaft, moulded capital), a bowl with
  a rolled lip falling to a grated drain, a chrome bubbler bent forward in one piece on a turned foot, its mouth and an
  arched guard, and a chrome foot pedal on a bent arm with a ribbed tread. The arc of water is one swept tube, thinning.
- **Litter bin**: a green bin, pressed slats between two rolled beads, a rounded foot, a rolled iron rim round a dark
  mouth with a black bag's edge over it, a label with a bin sign; on a square iron post with a pyramid cap, two curved
  straps round its back.
- **Bus shelter**: four posts on bolted foot plates, a curved roof on beams, a fascia with a blue stripe, a gutter; glass
  at the back (two panes on a mullion) and the right-hand end in rails, white safety bands; a two-slat bench on
  cantilevered brackets with slim legs; the timetable and a route map on the back glass; the stop's blue flag on the
  roof with the bus on both faces.
- **Post box**: a red pillar box turned in one piece (foot moulding, band, shaft, stepped cornice, domed cap) on a black
  plinth, the door's outline cast round the front with a keyhole, the slot's arched hood, sill and cheeks round a dark
  opening and its flap (with its word), the collection times in a framed plate, a brass crown over the slot.
- **Notice board**: a cork board in a moulded walnut frame with a ledge, between two posts on iron feet, under a boarded
  roof (three overlapping boards a side), a ridge and shaped barge boards; six pinned notices with lines of writing,
  one a leaflet with tear-off strips.
- **Phone box**: a red kiosk on a stepped plinth, glazed on all four sides (three columns, eight rows of panes), panelled
  kick panels, a band with four TELEPHONE signs, a cornice, an arched pediment with a crown on each side and the shallow
  dome between them; the door with a bar pull (`kit.pull`). Inside, the phone (case, dial, handset on its cradle, a
  coiled cord, the coin box) on a panel, and a shelf of directories.
- **Town fountain**: a square basin on a granite slab: walls on a plinth band, three raised panels a face, a moulded
  coping with a rounded nose all round (`kit.moulding`, its top the seat), water; in the middle a moulded foot, a fluted
  column, a capital, the wide lower bowl gadrooned underneath with a rolled lip, a waisted stem, the small top bowl
  gadrooned too, a turned knob. Six streams from the top bowl and eight from the lower one, each one swept piece
  thinning as it falls, splash into crowns of spray.
- **Founder's statue**: a granite plinth (two steps, a moulded base, the die with a sunk panel on each face, a moulded
  cornice) with a bevelled brass plaque of three raised lines; a bronze woman in a long coat falling in folds, its
  lapels, collar and buttons, her right arm pointing up and away in one sleeve with a cuff, her left holding a book to
  her side.

| Model | Size (w × h × d) | Triangles (before) | Far | Draws | Colour parts |
|---|---|---|---|---|---|
| `model.streetLamp` | 0.65 × 4.24 × 0.47 m | 2,732 (780) | 678 | 1 | Lamp post |
| `model.parkBench` | 2.50 × 0.89 × 0.57 m | 3,892 (2,160) | 972 | 2 | Frame |
| `model.drinkingFountain` | 0.56 × 1.09 × 0.56 m | 2,728 (1,524) | 570 | 2 | Pillar |
| `model.litterBin` | 0.47 × 0.98 × 0.58 m | 1,600 (1,704) | 400 | 1 | Bin |
| `model.busStop` | 2.98 × 2.89 × 1.21 m | 3,980 (2,616) | 968 | 3 | Frame |
| `model.postBox` | 0.50 × 1.53 × 0.52 m | 2,810 (2,132) | 688 | 2 | Box |
| `model.noticeBoard` | 1.02 × 2.07 × 0.35 m | 1,868 (1,320) | 436 | 2 | Frame |
| `model.phoneBooth` | 0.95 × 2.70 × 0.95 m | 3,900 (3,100) | 920 | 4 | Kiosk |
| `model.fountainTown` | 2.96 × 2.47 × 2.96 m | 5,452 (5,784) | 1,232 | 3 | Stone |
| `model.townStatue` | 1.80 × 3.39 × 1.80 m | 3,410 (2,476) | 820 | 1 | Statue |

Draws are counted as `check-glb` does, each moving part apart: the bench (iron, wood), the shelter (frame, wood,
glass), the drinking fountain and the post box (body, the moving part), the notice board (plain, wood), the town
fountain (body, jet, splashes). The lamp and the phone box take one more at night for their glow.

`check-glb`: 10 models, 0 failed. One warning, expected:

- **Phone box: 4 draw calls** (the most allowed, as the washer): the kiosk, its glazing, the door and the door's
  glazing. The plan's folding door would be two moving leaves, each with its own glass: 6 draws, over the limit, so
  the door is one leaf, as on the real red kiosk. Opaque panes would save one, but then the kiosk and its phone could
  not be seen through.

Khronos validator: 0 errors, 0 warnings on all twenty files (each model and its far version).

## Rebuild

```
cargo art street_furniture            # the ten .glb and .far.glb, the pack, check-glb
cargo art street_furniture --preview  # previews/street_furniture/<id>.png, <id>@<clip>.png, <id>.far.png
```
