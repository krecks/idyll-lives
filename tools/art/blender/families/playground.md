# Playground — notes

`playground.py` builds the eight CM1 playground models into `web/public/assets/models/playground/` and their entries
in `packs/playground/manifest.json`: the slide (`model.slide`), the climbing frame (`model.climbingFrame`), the seesaw
(`model.seesaw`), the spring rider (`model.springRider`), the merry-go-round (`model.merryGoRound`), the basketball
hoop (`model.basketballHoop`), the ping-pong table (`model.pingPongTable`) and the park chess table
(`model.parkChessTable`). They replace the primitive-part placeholders of `tools/art/community.py`, which stay as the
fallback. Built 2026-10-14; they sit in Validate › Playground until checked in play.

Each is built where its placeholder stands (`F`, game axes) and stood on the floor without moving it across
(`G.settle(centre=False)`, `centre=` on its `Model`), so the seats stay where `community.json` seats the riders. Town
things, seen from far: every model has a far version (`far=0.25`; the chess table's by hand, below).

## The moving ones

- **Seesaw** (3 × 1 tiles): a 2.5 m plank on a blue A-frame stand, rubber bumpers under its ends. The content's two
  slots fall at x = ±0.75 m (3 m wide, 2 slots) at 0.56 m: the seats are there, level at rest. The plank, its seats
  and handles are one part (`plank`) turning about the axle (0.46 m up). `seesaw` rocks it ±12° in 1.4 s (`G.rock`
  about Blender Y), so the low end just meets its bumper. `motions`: `{'loops': {'seesaw': 'seesaw'}, 'ride':
  'seesaw'}`.
- **Spring rider**: a red horse on a green coil spring (a swept helix). The saddle's top is the content's seat (0.78 m,
  4 cm back). The horse and everything above the spring (`horse`) rocks ±10° in 1 s about the spring's top (`rock`).
  `motions`: `{'loops': {'ride': 'rock'}, 'ride': 'rock'}` (the interaction id is `ride`).
- **Merry-go-round** (2 × 2): a 1.9 m red platform 0.25 m up (the content's seat 0.26 m), four bent rails and a centre
  column; the hub stays still. `spin` turns the platform (`platform`) once in 3.5 s: a whole turn, so riders carried
  round never snap back at the loop's end, and well under a turn a second. `motions`: `{'loops': {'spin': 'spin'},
  'ride': 'spin'}`. `check_glb.py` has no clip-length check; 3.5 s is longer than the usual "about 1 s", needed for a
  whole turn at a believable speed.
- **Basketball hoop**: a post on a ballast base at the back of its tile, a backboard and the rim at 2.94 m. The net
  (`net`, a diamond mesh of thin strands, `fabric`) is its own part hung from the rim: `swish` sways it 9° and
  stretches it 18 % down and back in 0.6 s, from rest to rest, after each shot: 1.4 s into `shoot`, then every
  3.2 s while it lasts. `motions`: `{'uses': {'shoot': {'clip': 'swish', 'delay': 1.4, 'every': 3.2}}}`.
- **Park chess table**: a stone table in the front half with the board inlaid (5 cm squares), and two stone stools
  behind it where the content seats its players side by side (x = ±0.25 m, 27 cm back, 0.47 m up), facing the board.
  A game is under way; one white pawn (`pawn`) slides one square on, away from the players (`move`, 0.6 s), for the
  whole game (no `hold`: it goes back when they get up). `motions`: `{'uses': {'chess': 'move'}}`.

How riding works: `objectMotion.ts` `ride(id)` gives the seat's move from rest (the first part the `ride` clip moves),
and `characters.ts` moves a sitting or lying resident by it. All three rides use `"pose": "sit"`, so their riders go
with the plank, the horse and the platform. Seen in play: the seesaw's rider goes up and down on the seat, the spring
rider's rocks on the saddle, the merry-go-round's goes round sitting on the platform's edge.

## The still ones

- **Slide** (1 × 2): a blue steel tower with a wooden deck at 1.4 m and a red pyramid roof, a ladder at the back, and
  a yellow chute swept down a smoothstep curve (flat at the top and at its run-out) to the front.
- **Climbing frame** (2 × 2): a wooden fort, deck at 1.1 m, railings on three sides, a pitched roof with a flag, a rope
  net up the front and a fireman's pole beside it (the content's description).
- **Ping-pong table** (2 × 1): a green top with white lines on two concrete legs, a steel net, two bats and a ball.
  All plain finishes: one draw call.

## Far versions

`far=0.25` decimates the model at rest. On the chess table that folded the inlaid board into spikes, so it has a far
build of its own (`parkChessTableFar`): the table, the squares and the stools at fewer sides and only the two kings
(1,216 triangles, under half the full model's). Its revolves take 12 sides so its bounds stay those of the full model.

| Model | Size (w × h × d) | Triangles (far) | Draws | Colour parts |
|---|---|---|---|---|
| `model.slide` | 0.75 × 2.36 × 1.93 m | 1,928 (478) | 2 | Frame, Slide, Roof |
| `model.climbingFrame` | 1.81 × 3.45 × 1.99 m | 1,656 (382) | 3 | Frame, Roof, Net |
| `model.seesaw` | 2.50 × 0.85 × 0.42 m | 1,808 (452) | 2 | Frame, Seats, Plank |
| `model.springRider` | 0.40 × 1.11 × 0.96 m | 3,508 (876) | 2 | Horse, Saddle, Spring |
| `model.merryGoRound` | 1.96 × 1.04 × 1.96 m | 2,924 (721) | 2 | Platform, Rails, Stripes |
| `model.basketballHoop` | 0.95 × 3.50 × 0.97 m | 2,912 (720) | 2 | Frame, Backboard, Hoop |
| `model.pingPongTable` | 1.90 × 0.95 × 0.99 m | 1,148 (276) | 1 | Table, Legs |
| `model.parkChessTable` | 0.80 × 0.85 × 0.88 m | 3,468 (1,216) | 2 | Table, Stools |

`check_glb.py`: 8 models, 0 failed, no warnings (the seesaw's placeholder, built for 2 × 1 tiles, was widened to the
content's 3 × 1 with its seats at ±0.75 m, so the fallback, picking and shadow match the model). Khronos
validator: 0 errors, 0 warnings on all 16 files.

## Rebuild

```
python3 -B tools/art/blender/build.py playground            # the 16 .glb, the pack, check_glb
python3 -B tools/art/blender/build.py playground --preview  # previews/<id>.png, <id>@<clip>.png, <id>.far.png
```
