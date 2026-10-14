# Fitness — notes

`fitness.py` builds the five F1 home gym models into `web/public/assets/models/fitness/` and their entries in
`packs/fitness/manifest.json`: the treadmill (`model.treadmill`), the weight bench (`model.weightBench`), the
exercise bike (`model.exerciseBike`), the yoga mat (`model.yogaMat`) and the dumbbell rack (`model.dumbbells`).
They replace the primitive-part placeholders of `objects_extra.py` / `furniture.py` (the base manifest) and
`catalog.py`, which stay as the fallback. Built 2026-10-14; they sit in Validate › Fitness until checked in play.

Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its `Model`,
except the bike, which is centred).

## The treadmill

The belt runs along the depth, the console at the back, tilted 35° to face the runner. The runner jogs on the
belt facing the console: `characters.ts` moves them forward from the tile in front onto its middle, at the belt's
top (`TREADMILL_BELT`, 0.156 m), as it does for the trampoline.

- **The slats** (`slats`): 15 strips across the belt, 10 cm apart. `belt_run` moves them one slat back (towards
  the rear) in 0.5 s at an even speed (`G.scroll`), a loop for the whole run (`loops: {run: belt_run}`). The game
  jumps the loop home at the end of each round; the slats then stand where their neighbours were, and the first
  and last are under the motor hood and the rear roller cap, so it doesn't show. `check_glb.py` warns that the
  loop ends 10 cm from where it starts: right for a belt.

## The exercise bike

An upright bike: stabiliser feet, a flywheel with covers at the front, the saddle 0.76 m up (`SEATS`), the
handlebars and a console. **The cranks and pedals** (`pedals`) are joined round the axle; `pedals_turn` turns them
once in 1.2 s (`G.spin`), a loop while cycled (`loops: {cycle: pedals_turn}`). The pedals are round, so they look
the same at any angle. The rider's legs don't pedal (the `exercise` sitting pose).

## The yoga mat

At rest the mat lies rolled up at the back of its tile, with a foam block and a bottle. `unroll` (1 s, a
use: `uses: {yoga: unroll}`, back when the yoga ends) moves two parts together:

- **The flat mat** (`mat`), its origin on its back edge, built full and squeezed to 0.001 deep: it grows to the
  front (`G.grow`, axis Y).
- **The roll** (`roll`), its origin on the floor under its axis: it rolls forward with the mat's edge (`G.slide`)
  and shrinks onto the floor (`G.squash` to 0.12), leaving a curled front edge.

The mat is 1.6 cm thick: the house's floor is drawn 1 cm up, so a thinner one disappears. `check_glb.py` warns that
it is off centre and shallower than its placeholder (the flat mat with a roll): picking and the shadow follow the
placeholder, which is the unrolled mat's tile. Yoga is done sitting (`pose: sit`, content `seat` 0.03 m), so the resident sits
cross-legged on the unrolled mat.

## The still ones

The lift animation draws no weight in the hands (`lift`: `pickUp` standing, a bench press lying), so nothing
leaves the bench's rack or the dumbbell rack: both are joined, still.

- **Weight bench**: a padded bench (top 0.5 m, `BEDS`), a rack at the head end with the bar on red J-hooks (the
  lifter's head lies under it), a leg roller, and two dumbbells on the floor by the feet.
- **Dumbbell rack**: two tilted shelves of four hex dumbbells each on A-frames, a rubber mat in front.

| Model | Size (w × h × d) | Triangles | Draws | Colour parts |
|---|---|---|---|---|
| `model.treadmill` | 0.79 × 1.44 × 1.85 m | 2,736 | 2 | Frame, Pads |
| `model.weightBench` | 0.90 × 1.20 × 1.85 m | 4,444 | 1 | Frame, Pads |
| `model.exerciseBike` | 0.53 × 1.11 × 0.90 m | 2,216 | 2 | Frame, Pads |
| `model.yogaMat` | 0.92 × 0.23 × 0.27 m (rolled) | 768 | 3 | Mat |
| `model.dumbbells` | 0.77 × 0.72 × 0.71 m | 3,996 | 1 | Rack, Weights |

`check_glb.py`: 5 models, 0 failed. Khronos validator: 0 errors, 0 warnings each.

## Rebuild

```
python3 -B tools/art/blender/build.py fitness            # the five .glb, the pack, check_glb
python3 -B tools/art/blender/build.py fitness --preview  # previews/<id>.png, <id>@<clip>.png
```
