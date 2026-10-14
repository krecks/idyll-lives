# Fitness

The five F1 models are done in `families/fitness.py` (notes: `fitness.md`): the treadmill's belt (`belt_run`, the
new `G.scroll`), the bike's pedals (`pedals_turn`) and the yoga mat (`unroll`). The lift animation draws no weight
in the hands, so the bench and the rack are still. Runners now jog on the treadmill's belt (`characters.ts`), and yoga
is done sitting cross-legged on the mat (content `pose: sit`, `seat`). What's left to consider: `future.md`.

Where a resident holds the weight in the hand, the object's own weight hides while it is used (T0 `squash` to 0),
so there are never two. Check each `lift` and `run` animation against its object.

## F1. Home gym (5) — **Done** (in Validate › Fitness)

- **Family:** `tools/art/blender/families/fitness.py`
- **Validate group:** `fitness` (Validate › Fitness)
- **Placeholders to match:** `tools/art/catalog.py`, base manifest (`tools/art/furniture.py`, `nature.py`)

| Item | Size · layer | Uses (id:anim) | Back to | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `treadmill` Treadmill | 1x2 · floor | run:run | fitness / — | Frame, Pads | Loop (E1) | `belt`: the belt slats run while used (E1). Until then U: the console tilts up 15°. |
| `weightBench` Weight bench | 1x2 · floor | lift:lift | fitness / — | Frame, Pads | Use | `bar_off`: the bar on the rack shrinks away for lift if the lift animation draws one in the hands; otherwise static. |
| `exerciseBike` Exercise bike | 1x1 · floor | cycle:exercise | fitness / — | Frame, Pads | Loop (E1) | `pedals`: the pedals turn while cycled (E1). Until then static. |
| `yogaMat` Yoga mat | 1x1 · floor | yoga:meditate | fitness / — | Mat | Use | `unroll`: the rolled mat unrolls flat for yoga and rolls up again at the end. |
| `dumbbells` Dumbbell rack | 1x1 · floor | lift:lift | fitness / — | Rack, Weights | Use | `pair_off`: one pair leaves the rack for lift, as the weight bench. |
