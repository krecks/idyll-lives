# Playground — notes

`playground.py` builds the eight CM1 playground models into `web/public/assets/models/playground/` and their entries
in `packs/playground/manifest.json`: the slide (`model.slide`), the climbing frame (`model.climbingFrame`), the seesaw
(`model.seesaw`), the spring rider (`model.springRider`), the merry-go-round (`model.merryGoRound`), the basketball
hoop (`model.basketballHoop`), the ping-pong table (`model.pingPongTable`) and the park chess table
(`model.parkChessTable`). Built 2026-10-14; they sat in Validate › Playground until checked in play.

Each is built where the game places it (`F`, game axes) and stood on the floor without moving it across
(`G.settle(centre=False)`, `centre=` on its `Model`), so the seats stay where `community.json` seats the riders. Town
things, seen from far: every model has a far version (`far=0.25`; the chess table's by hand, below).

## The moving ones

- **Seesaw** (3 × 1 tiles): a 2.5 m plank on a blue A-frame stand, rubber bumpers under its ends. The content's two
  slots fall at x = ±0.75 m (3 m wide, 2 slots) at 0.56 m: the seats are there, level at rest. The plank, its seats
  and handles are one part (`plank`) turning about the axle (0.46 m up). `seesaw` rocks it ±12° in 1.4 s (`G.rock`
  about Blender Y), so the low end just meets its bumper. `motions`: `{'loops': {'seesaw': 'seesaw'}, 'ride':
  'seesaw'}`.
- **Spring rider**: a red horse on a green coil spring (a swept helix). The saddle's top is the content's seat (0.62 m,
  4 cm back: a rider's seat for a toddler to a six-year-old; it was 0.78, the horse 16 cm higher on a longer coil). The horse and everything above the spring (`horse`) rocks ±10° in 1 s about the spring's top (`rock`).
  `motions`: `{'loops': {'ride': 'rock'}, 'ride': 'rock'}` (the interaction id is `ride`).
- **Merry-go-round** (2 × 2): a 1.9 m red platform, its top 0.26 m up (the content's seat; it was 0.24), four bent rails and a centre
  column; the hub stays still. `spin` turns the platform (`platform`) once in 3.5 s: a whole turn, so riders carried
  round never snap back at the loop's end, and well under a turn a second. `motions`: `{'loops': {'spin': 'spin'},
  'ride': 'spin'}`. `check-glb` has no clip-length check; 3.5 s is longer than the usual "about 1 s", needed for a
  whole turn at a believable speed.
- **Basketball hoop**: a post on a ballast base at the back of its tile, a 1.2 × 0.8 m backboard with the shooter's
  square, and the rim at 3.05 m (regulation, as park hoops are; it was 2.94 m on a 0.95 × 0.62 board). The net
  (`net`, a diamond mesh of thin strands, `fabric`) is its own part hung from the rim: `swish` sways it 9° and
  stretches it 18 % down and back in 0.6 s, from rest to rest, after each shot: 1.4 s into `shoot`, then every
  3.2 s while it lasts. `motions`: `{'uses': {'shoot': {'clip': 'swish', 'delay': 1.4, 'every': 3.2}}}`.
- **Park chess table** (2 × 1): a stone table (0.7 m square) with the board inlaid (5 cm squares), and a stone
  stool either side, left and right, where the content's `places` seat the two players facing each other across the
  board (x = ±0.6 m, 0.47 m up, their knees clear of each other under the top; until 2026-10-21 both stools stood
  behind it, the players side by side, then ±0.42 m on one tile, the knees meeting). White plays
  from the left. A game is under way; one white pawn (`pawn`) slides one square on, towards black (`move`, 0.6 s), for
  the whole game (no `hold`: it goes back when they get up). `motions`: `{'uses': {'chess': 'move'}}`.

How riding works: `objectMotion.ts` `ride(id)` gives the seat's move from rest (the first part the `ride` clip moves),
and `characters.ts` moves a sitting or lying resident by it. All three rides use `"pose": "sit"`, so their riders go
with the plank, the horse and the platform. Seen in play: the seesaw's rider goes up and down on the seat, the spring
rider's rocks on the saddle, the merry-go-round's goes round sitting on the platform's edge.

## The still ones

- **Slide** (1 × 2): a blue steel tower with a wooden deck at 1.4 m and a red pyramid roof, its eaves 2.46 m up (1 m
  over the deck: a child stands under them), a ladder at the back whose rails go on up as handrails either side of
  an open way in, an arch over the chute's mouth rising 0.55 m (a child sits up under it), and a yellow chute swept
  down a smoothstep curve (flat at the top and at its run-out) to the front. Until 2026-10-21 the eaves were 1.93 m,
  a bar crossed the deck's back at 1.86 m and a hoop rose 0.27 m over the chute (M-community-07); the slide's round
  (`clips/rides.ts` `SLIDE_HOOP`) holds the arch.
- **Climbing frame** (2 × 2): a wooden fort, deck at 1.1 m, railings on three sides, a pitched roof with a flag, a rope
  net up the front and a fireman's pole beside it (the content's description).
- **Ping-pong table** (3 × 2): a green top at the ITTF's 2.74 × 1.525 m, 0.76 m up, with white lines, on two concrete
  legs, a steel net 15.25 cm high reaching 15.25 cm past each side, two bats and a ball; the players stand at its
  ends (`places` ±1.8 m; it was 1.9 × 0.94 m on 2 × 1, its top 0.78).
  All plain finishes: one draw call.

## Far versions

`far=0.25` decimates the model at rest. On the chess table that folded the inlaid board into spikes, so it has a far
build of its own (`parkChessTableFar`): the table, the squares and the stools at fewer sides and only the two kings
(1,216 triangles, under half the full model's). Its revolves take 12 sides so its bounds stay those of the full model.

| Model | Size (w × h × d) | Triangles (far) | Draws | Colour parts |
|---|---|---|---|---|
| `model.slide` | 0.75 × 2.81 × 1.93 m | 2,020 (498) | 2 | Frame, Slide, Roof |
| `model.climbingFrame` | 1.81 × 3.45 × 1.99 m | 1,656 (382) | 3 | Frame, Roof, Net |
| `model.seesaw` | 2.50 × 0.85 × 0.42 m | 1,808 (452) | 2 | Frame, Seats, Plank |
| `model.springRider` | 0.40 × 0.95 × 0.96 m | 3,172 (792) | 2 | Horse, Saddle, Spring |
| `model.merryGoRound` | 1.96 × 1.04 × 1.96 m | 2,924 (721) | 2 | Platform, Rails, Stripes |
| `model.basketballHoop` | 1.20 × 3.71 × 0.97 m | 2,912 (714) | 2 | Frame, Backboard, Hoop |
| `model.pingPongTable` | 2.74 × 0.92 × 1.86 m | 1,148 (276) | 1 | Table, Legs |
| `model.parkChessTable` | 1.44 × 0.85 × 0.70 m | 3,468 (1,216) | 2 | Table, Stools |

`check-glb`: 8 models, 0 failed, no warnings (the seesaw, built for 2 × 1 tiles, was widened to the content's
3 × 1 with its seats at ±0.75 m, so picking and the shadow match the model). Khronos
validator: 0 errors, 0 warnings on all 16 files.

## Rebuild

```
cargo art playground            # the 16 .glb, the pack, check-glb
cargo art playground --preview  # previews/playground/<id>.png, <id>@<clip>.png, <id>.far.png
```
