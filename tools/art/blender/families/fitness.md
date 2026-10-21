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

- **The slats** (`slats`): 15 strips across the belt, 10 cm apart. `belt_run` moves them one slat back (towards
  the rear) in 0.25 s at an even speed (`G.scroll`, 0.4 m/s), a loop for the whole run (`loops: {run: belt_run}`).
  That is slower than a jog, but faster would blur: at the fastest game speed (2.5×) the slats already move a third
  of their pitch a frame at 30 frames a second (it was 0.2 m/s, which read as a belt creeping along). The game
  jumps the loop home at the end of each round; the slats then stand where their neighbours were, and the first
  and last are under the motor hood and the rear roller cap, so it doesn't show. `check-glb` warns that the
  loop ends 10 cm from where it starts: right for a belt.

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

At rest the mat lies rolled up at the back of its tile, with a foam block and a bottle. `unroll` (1 s, a
use: `uses: {yoga: unroll}`, back when the yoga ends) moves two parts together:

- **The flat mat** (`mat`), its origin on its back edge, built full and squeezed to 0.001 deep: it grows to the
  front (`G.grow`, axis Y).
- **The roll** (`roll`), its origin on the floor under its axis: it rolls forward with the mat's edge (`G.slide`)
  and shrinks onto the floor (`G.squash` to 0.12), leaving a curled front edge.

The mat is 1.6 cm thick: the house's floor is drawn 1 cm up, so a thinner one disappears. Rolled at rest, it
stands off the middle of its footprint (its `Model`'s `centre`, which the gate checks it is centred on) and is
shallower than it: picking follows the rolled mat as drawn, and the shadow the tile. Yoga is done sitting
(`pose: sit`, content `seat` 0.03 m), so the resident sits cross-legged on the unrolled mat.

## The weights

Lifting holds the weight (content `held`, `characters.ts` `writeHeld`): the lifter presses the bench's bar lying
down, and curls a pair of dumbbells standing at the rack. The weight is a part of its own, hidden while it's in
the hands (squashed to 0.001 in 0.2 s, `uses: {lift: …}`), and the game draws that part in the hands, so there
is never two of it.

- **Weight bench**: 2 cm forward of where the game placed it (both now stop at the wall's face), a padded bench (top 0.5 m, `BEDS`), a rack at the head end with the bar on red J-hooks (the
  lifter's head lies under it), a leg roller, and two dumbbells on the floor by the feet. The bar with its plates
  is `bar`, its origin at its middle (`BAR`); `bar_off` hides it (`"held": {"part": "bar", "grip": "bar"}`),
  `BAR_DELAY` (0.8 s) into the lift: the game draws the bar in the hands only once the lifter has sat down and
  lies back (0.98 s in), so it leaves the hooks then rather than vanishing for the lie-down.
- **Dumbbell rack**: two tilted shelves of two hex dumbbell pairs each on A-frames (the lower shelf's heavier),
  a rubber mat in front. The upper shelf's left pair (`LIFTED`) is two parts, `dumbbell_l` and `dumbbell_r`,
  each with its origin at its middle; `pair_off` hides both (`"held": {"part": ["dumbbell_l", "dumbbell_r"],
  "grip": "pair"}`).

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.treadmill` | 0.79 × 1.44 × 1.85 m | 2,736 | 2 | Frame, Pads |
| `model.weightBench` | 0.90 × 1.20 × 1.85 m | 4,444 | 2 | Frame, Pads |
| `model.exerciseBike` | 0.53 × 1.11 × 0.90 m | 2,216 | 2 | Frame, Pads |
| `model.yogaMat` | 0.92 × 0.23 × 0.27 m (rolled) | 768 | 3 | Mat |
| `model.dumbbells` | 0.77 × 0.72 × 0.71 m | 3,996 | 3 | Rack, Weights |

`check-glb`: 5 models, 0 failed. Khronos validator: 0 errors, 0 warnings each.

## Rebuild

```
cargo art fitness            # the five .glb, the pack, check-glb
cargo art fitness --preview  # previews/fitness/<id>.png, <id>@<clip>.png
```
