# Fitness

The five F1 home gym models are built by `families/fitness.py` (notes: `fitness.md`): the treadmill's belt
(`belt_run`, `G.scroll`), the bike's pedals (`pedals_turn`) and the yoga mat (`unroll`) move. Runners jog on the
treadmill's belt (`characters.ts`, content `stand.height`), riders' feet stay on the pedals (content `seat.pedals`),
and yoga is a flow of standing and floor poses on the unrolled mat, from its front end (content `anim: exercise`,
standing; `STEP_IN` and the `yoga` clip in `characters/clips/games.ts`: mountain, salute, folds, a short downward dog,
warrior I, a child's pose and sitting on the heels; G-26). The mat is short for the dog (0.9 m: G-40). The weight bench and the dumbbell rack are for teens and up (content `stages`, G-45): a child is refused ("too young to …") and never lifts on their own.

Lifting holds the weight (content `held`): the bench's bar and one pair of the rack's dumbbells are parts of their
own, hidden while in use (T0 `squash` to 0: `bar_off`, `pair_off`), and the game draws that part in the hands, in
the object's colours, so there are never two.

## F1. Home gym (5)

- **Family:** `tools/art/blender/families/fitness.py`

| Item | Size · layer | Uses (id:anim) | Home | Colour parts | Clip | What it shows |
|---|---|---|---|---|---|---|
| `treadmill` Treadmill | 1x2 · floor | run:run | fitness / — | Frame, Pads | Loop (E1) | `belt_run`: the belt's slats run back at 0.4 m/s under the jogging runner for the whole run (E1). |
| `weightBench` Weight bench | 1x2 · floor | lift:lift | fitness / — | Frame, Pads | Use | `bar_off`: the bar on the rack shrinks away 0.8 s into the lift (`BAR_DELAY`), as the lifter lies back and the game draws it in the hands (content `held`). |
| `exerciseBike` Exercise bike | 1x1 · floor | cycle:exercise | fitness / — | Frame, Pads | Loop (E1) | `pedals_turn`: the cranks and pedals turn once every 1.2 s while cycled, the rider's feet on them (E1). |
| `yogaMat` Yoga mat | 1x1 · floor | yoga:exercise | fitness / — | Mat | Use | `unroll`: the rolled mat unrolls flat for yoga and rolls up again at the end. |
| `dumbbells` Dumbbell rack | 1x1 · floor | lift:lift | fitness / — | Rack, Weights | Use | `pair_off`: one pair leaves the rack for lift, as the weight bench; the game draws it in the hands in the rack's "Weights" colour. |

**In play** (signed off 2026-10-17):
- Each bought on free floor in all four facings and against the north wall, and used there: the belt loops under the
  runner, the rider's feet stay on the turning pedals, the mat unrolls and rolls up again, the bar and the pair leave
  the bench and the rack; by day and at night, WebGPU and Low WebGL2, every colour part and the colour studio
  ("In use").
- A held part is drawn in the object's player colours (its prop key carries them, `<model>#<part>@<colours>`): a
  coloured rack's dumbbells are its "Weights" colour in the hands; uncoloured, they are the model's dark grey.
- The bench's bar stays on its hooks while the lifter sits down, shrinks away 0.8–1.0 s into the lift and is in the
  hands from about 1.0 s on (the `sitDown` clip less its last 0.35 s), never missing.
- The bench press (G-24, 2026-10-21): the feet are down on the floor either side of the bench, 0.3 m out from its
  middle (`furniture.json` `beds.weightBench` `feet`, characters.ts `benchLegs`), the knees bent off the pad, not
  stretched along it onto the leg rollers. Teens, grown-ups and elders reach the floor from the 0.5 m pad.
  Known, not changed: the pad is 50 cm high, over a flat bench's 40–48 (M-fitness-01, left to the Blender wave).
- The treadmill's belt runs slower than a jog: faster blurs at the fastest game speed. The loop jumps home each round
  under the motor hood and rear roller cap, where it doesn't show.
