# Street furniture — notes

`street_furniture.py` builds the ten CM2 street furniture models (docs/objects/community.md) into
`web/public/assets/models/street_furniture/` and their entries in `packs/street_furniture/manifest.json`: the street
lamp (`model.streetLamp`), the long park bench (`model.parkBench`), the drinking fountain (`model.drinkingFountain`),
the litter bin (`model.litterBin`), the bus shelter (`model.busStop`), the post box (`model.postBox`), the notice board
(`model.noticeBoard`), the phone box (`model.phoneBooth`), the town fountain (`model.fountainTown`) and the founder's
statue (`model.townStatue`). The street dressing props (`model.street.*`) are not part of it. Built
2026-10-14; they sit in Validate › Street until checked in play.

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

- **Drinking fountain** (`water_on`, use `drink`): the arc of water from the bubbler's mouth into the bowl, a bent
  tube joined with its origin at the mouth and scaled to 0.001 on all three axes at rest; `G.grow` on X, Y and Z
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
  to 1.8 times theirs. The streams spilling from both bowls are still, on the body. `check_glb.py` gives no loop
  warning: a pulse comes back to rest exactly.
- The lamp, the bench, the bin, the shelter, the notice board and the statue are still.

## Night

- The lamp's four frosted panes are `lantern` (satin) with `glow` `#FFD596`: they glow from dusk as the lamps come up
  (E8), out of doors so no room's lights put them out. The lamp's light itself is the content's `light`, unchanged.
- The phone box's four TELEPHONE signs are `sign` (matte) with `glow` `#FFF1D2`.

## Far versions (E7)

Every model has a `<id>.far.glb` for the town overview (the menus' town): `far=0.25`, the model at rest, joined
and decimated to a quarter. For the drinking fountain and the statue, plain decimation lifted the bottom off the
floor (the fountain's slab by 3.9 cm, and pushed it 7 cm wider a side; the statue's step by 0.7 cm), so those two
use `_far(build)`: the same, but the vertices below 0.1 m are left out of the decimation (a vertex group on the
Decimate modifier). Their far versions sit on the floor at the full model's size.

## The models

- **Street lamp**: a fluted cast-iron base, a pole with collars, a lamplighter's ladder bar with ball ends, and a
  square lantern (frosted panes widening upwards in an iron frame) under a pyramid roof with a vent and finial. One
  draw call.
- **Park bench**: four wooden seat slats and three back slats on three cast-iron supports (front and back legs, the
  back upright, a seat rail, feet), with armrests and scrolls on the end supports.
- **Drinking fountain**: a stone slab, a turned stone pillar and bowl with a drain, a chrome bubbler at the back of
  the bowl turned to the front, and a chrome foot pedal.
- **Litter bin**: a green ribbed bin on an iron post with two straps, an iron rim round a dark opening, a label.
- **Bus shelter**: four posts, a roof with a fascia, glass at the back and the right-hand end in rails, a two-slat
  bench on brackets, the timetable on the back glass and the stop's blue flag on the roof.
- **Post box**: a red pillar box on a black plinth, a band, a cornice and a dome, the slot's hooded housing with its
  flap, the collection times and a crest.
- **Notice board**: a cork board between two walnut posts, framed, under a little gabled roof; six pinned notices.
- **Phone box**: a red kiosk, hollow and glazed on all four sides (three columns, eight rows of panes in glazing
  bars), kick panels, a band with four TELEPHONE signs, a stepped cornice and a shallow dome. Inside, the phone on a
  panel at the back and a shelf.
- **Town fountain**: a square stone basin on a granite slab, straight rims to sit on, water; in the middle a foot,
  a column, a wide lower bowl, a stem, a small top bowl and a knob. Water streams curve out over both bowls' lips
  (six from the top bowl, eight from the lower) and splash into the water below; a jet rises from the knob.
- **Founder's statue**: a granite plinth (step, die, cornice) with a brass plaque, and a bronze woman in a long coat
  on a little base, pointing up and away with her right arm and holding a book in her left.

| Model | Size (w × h × d) | Triangles | Far | Draws | Colour parts |
|---|---|---|---|---|---|
| `model.streetLamp` | 0.66 × 4.24 × 0.47 m | 780 | 186 | 1 | Lamp post |
| `model.parkBench` | 2.86 × 0.91 × 0.57 m | 2,052 | 512 | 2 | Frame |
| `model.drinkingFountain` | 0.56 × 1.09 × 0.56 m | 1,524 | 380 | 2 | Pillar |
| `model.litterBin` | 0.47 × 0.97 × 0.58 m | 1,704 | 412 | 1 | Bin |
| `model.busStop` | 2.98 × 2.89 × 0.98 m | 2,616 | 642 | 3 | Frame |
| `model.postBox` | 0.50 × 1.53 × 0.52 m | 2,132 | 528 | 2 | Box |
| `model.noticeBoard` | 1.02 × 2.07 × 0.35 m | 1,320 | 324 | 2 | Frame |
| `model.phoneBooth` | 0.95 × 2.70 × 0.95 m | 3,100 | 714 | 4 | Kiosk |
| `model.fountainTown` | 2.96 × 2.47 × 2.94 m | 5,784 | 1,430 | 3 | Stone |
| `model.townStatue` | 1.80 × 3.39 × 1.80 m | 2,476 | 614 | 1 | Statue |

Draws are counted as `check_glb.py` does, each moving part apart: the bench (iron, wood), the shelter (frame, wood,
glass), the drinking fountain and the post box (body, the moving part), the notice board (plain, wood), the town
fountain (body, jet, splashes). The lamp and the phone box take one more at night for their glow.

`check_glb.py`: 10 models, 0 failed. Two warnings, both expected:

- **Phone box: 4 draw calls** (the most allowed, as the washer): the kiosk, its glazing, the door and the door's
  glazing. The plan's folding door would be two moving leaves, each with its own glass: 6 draws, over the limit, so
  the door is one leaf, as on the real red kiosk. Opaque panes would save one, but then the kiosk and its phone could
  not be seen through.
- **Street lamp: depth 0.47 m against the old part's 0.65 m**: the old part's pyramid cap is measured across
  its corners; the real lantern is 0.47 m a side, and the ladder bar makes it 0.66 m wide. Picking and the contact
  shadow follow the old part.

Khronos validator: 0 errors, 0 warnings on all twenty files (each model and its far version).

## Rebuild

```
python3 -B tools/art/blender/build.py street_furniture            # the ten .glb and .far.glb, the pack, check_glb
python3 -B tools/art/blender/build.py street_furniture --preview  # previews/<id>.png, <id>@<clip>.png, <id>.far.png
```
