# Fitness — notes

`fitness.py` builds the five F1 home gym models into `web/public/assets/models/fitness/` and their entries in
`packs/fitness/manifest.json`: the treadmill (`model.treadmill`), the weight bench (`model.weightBench`), the
exercise bike (`model.exerciseBike`), the yoga mat (`model.yogaMat`) and the dumbbell rack (`model.dumbbells`).
Built 2026-10-14; they sat in Validate › Fitness until checked in play.

Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`,
except the bike, which is centred).

## The treadmill

The belt runs along the depth, the console at the back, tilted 35° to face the runner. The runner jogs on the
belt facing the console: `characters.ts` moves them forward from the tile in front onto its middle, at the belt's
top (content `stand.height`, 0.156 m), as it does for the trampoline.

- **The slats** (`slats`): 6 strips across the belt, 30 cm apart (`SLAT`). `belt_run` moves them one slat back
  (towards the rear) in 0.13 s at an even speed (`G.scroll`, 2.25 m/s, a run; a clip of 4 frames: 3 failed the
  gate's even-speed test), a loop for the whole run
  (`loops: {run: belt_run}`). It was 0.4 m/s with 10 cm slats, a slow walk under a running stride (review
  A-fitness-02, 2026-10-21): the long pitch is what lets the belt run that fast and still be seen going back, as
  each frame moves the slats under half a pitch (4 cm a frame at 60 frames a second, 9 cm at the fastest game
  speed, 2.5×). The game jumps the loop home at the end of each round; the slats then stand where their neighbours
  were, and the first and last are under the motor hood and the rear roller cap (the last placed a slat short of
  the cap's far edge), so it doesn't show. `check-glb` warns that the loop ends 30 cm from where it starts: right
  for a belt.

## The exercise bike

(The bike and the yoga mat stand with their backs on the wall face, -0.43, on their `Model` `centre`; they and
their old parts reached 2 cm into the wall.)

An upright bike: stabiliser feet, a flywheel with covers at the front, the saddle 0.76 m up (content `seat`), the
handlebars and a console. **The cranks and pedals** (`pedals`) are joined round the axle; `pedals_turn` turns them
once in 1.2 s (`G.spin`), a loop while cycled (`loops: {cycle: pedals_turn}`). The pedals are round, so they look
the same at any angle. The rider pedals: content `"seat": {"height": 0.76, "forward": -0.12, "pedals": {"height":
0.3, "forward": 0.04, "crank": 0.16}}` puts each foot on its pedal, in time with `pedals_turn` (the right one at
the bottom at rest; `characters.ts` `pedal`).

## The yoga mat

A full 1.8 × 0.62 m mat on a 1 × 2 footprint (2026-10-21, review M-fitness-02: it unrolled to 0.9 m on one tile). At
rest it lies rolled up at the back, on the wall's face, with a foam block and a bottle. `unroll` (1 s, a
use: `uses: {yoga: unroll}`, back when the yoga ends) moves two parts together:

- **The flat mat** (`mat`), its origin under the roll's axis, built full and squeezed to 0.001 deep: it grows to the
  front (`G.grow`, axis Y). Flattened there, it is mostly inside the roll (`check-glb` still warns of 42 % showing:
  a 1.6 cm strip under the roll's curve, at the back on the floor).
- **The roll** (`roll`), its origin on the floor under its axis: it rolls forward with the mat's edge (`G.slide`)
  and shrinks onto the floor (`G.squash` to 0.12), leaving a curled front edge.

The mat is 1.6 cm thick: the house's floor is drawn 1 cm up, so a thinner one disappears. Rolled at rest, it
stands off the middle of its footprint (its `Model`'s `centre`, which the gate checks it is centred on) and is
shallower than it: picking follows the rolled mat as drawn, and the shadow the tiles. Yoga is a standing flow
(`characters/clips/games.ts` `yoga`), the yogi stepped onto the mat 0.4 m in front of its middle (`YOGA_AT`), facing
along it, so the dog's hands and the squat's heels stay on it.

## The weights

Lifting holds the weight (content `held`, `characters.ts` `writeHeld`): the lifter presses the bench's bar lying
down, and curls a pair of dumbbells standing at the rack. The weight is a part of its own, hidden while it's in
the hands (squashed to 0.001 in 0.2 s, `uses: {lift: …}`), and the game draws that part in the hands, so there
is never two of it.

- **Weight bench**: 2 cm forward of where the game placed it (both now stop at the wall's face), a flat padded bench
  (top 0.44 m, `PAD_TOP`, `beds.weightBench`: a bench-press bench is 0.43–0.45, so the feet reach the floor either
  side; 2026-10-21, review M-fitness-01: 0.50 m, with a leg roller at the foot where the shins would come down), a
  rack at the head end with the bar on red J-hooks (the lifter's head lies under it), and two dumbbells on the floor
  by the feet. The bar with its plates
  is `bar`, its origin at its middle (`BAR`); `bar_off` hides it (`"held": {"part": "bar", "grip": "bar"}`),
  `BAR_DELAY` (0.8 s) into the lift: the game draws the bar in the hands only once the lifter has sat down and
  lies back (0.98 s in), so it leaves the hooks then rather than vanishing for the lie-down.
- **Dumbbell rack**: two tilted shelves of two hex dumbbell pairs each on A-frames (the lower shelf's heavier),
  back on the wall's face (`RACK_SHIFT`), and a 0.7 × 0.48 m rubber mat filling the front of the tile, where the
  lifter steps in to curl (`characters.ts` `STAND_BACK` -0.6; 2026-10-21, review I-fitness-02: they stood on the tile
  in front, the mat empty between them and the rack). The upper shelf's left pair (`LIFTED`) is two parts, `dumbbell_l` and `dumbbell_r`,
  each with its origin at its middle; `pair_off` hides both (`"held": {"part": ["dumbbell_l", "dumbbell_r"],
  "grip": "pair"}`).

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.treadmill` | 0.79 × 1.44 × 1.85 m | 4,348 | 2 | Frame, Pads |
| `model.weightBench` | 0.90 × 1.20 × 1.85 m | 5,048 | 2 | Frame, Pads |
| `model.exerciseBike` | 0.53 × 1.11 × 0.90 m | 3,152 | 2 | Frame, Pads |
| `model.yogaMat` | 0.92 × 0.23 × 0.27 m (rolled) | 1,952 | 3 | Mat |
| `model.dumbbells` | 0.77 × 0.72 × 0.93 m | 3,240 | 3 | Rack, Weights |

`check-glb`: 5 models, 0 failed; warnings: the treadmill's belt loop (expected) and the yoga mat's flat strip (29 %,
was 42 %). Khronos validator: 0 errors, 0 warnings each.

## Detail pass (2026-10-22)

Rebuilt with the forms and the kit, the same keys, sizes, centres, parts, clips (the treadmill's `belt_run` scroll and
its hood and cap covers as they were), motions, paint and draws; the held bar and dumbbells keep their origins.

- **Treadmill**: a rounded deck and roller cap with the roller's ends, ribbed foot treads, a lofted motor hood with a
  band and vents, transport wheels, oval uprights, handlebars bent in one piece with pulse plates and capped grips, a
  rounded console with a bezel, speakers and a safety key on its lanyard, a turned bottle in a holder.
- **Weight bench**: a piped pad, rounded beam, posts and feet with soft caps; uprights with peg holes and caps,
  J-hooks on pinned sleeves; the bar with knurling, sleeves, shoulders, turned plates with a red band, spring collars.
- **Exercise bike**: rounded stabilisers and beam, a profiled seat tube with a clamp and knob, a curved handlebar post,
  dished flywheel covers with hubs, a lofted saddle on its rail, handlebars bent in one piece, a resistance cable, a
  console with buttons; flat crank arms, pedal ends, a chainring.
- **Yoga mat**: the roll's ends show its spiral, two straps round it; the flat mat has rounded corners and alignment
  lines (`mat_edge`, a new matte colour); a soft foam block, a turned bottle with a label and a loop.
- **Dumbbells** (and the bench's two): hex heads chamfered, flats down, knurled handles, end caps; the rack's A-frames
  bent in one piece, tray shelves with lips and rubber liners, end caps, a brace; the floor mat in four tiles.

## Rebuild

```
cargo art fitness            # the five .glb, the pack, check-glb
cargo art fitness --preview  # previews/fitness/<id>.png, <id>@<clip>.png
```
