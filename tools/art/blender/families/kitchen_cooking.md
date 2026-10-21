# Kitchen cooking — notes

`kitchen_cooking.py` builds the kitchen's cookers and small appliances into
`web/public/assets/models/kitchen_cooking/` and their entries in `packs/kitchen_cooking/manifest.json`:
the six **K2 cookers** — the electric cooker (`model.stoveElectric`), the gas range (`model.stoveGas`), the
own tower (`model.wallOven`), the range cooker (`model.rangeCooker`), the hob counter (`model.cooktopCounter`)
and the microwave counter (`model.microwave`) — and the four **K3 small appliances** — the coffee machine
(`model.coffeeMachine`), the breakfast counter (`model.breakfastCounter`) and the two dishwashers
(`model.dishwasher`, `model.dishwasherCompact`). Built 2026-10-14.

A cooker is a box on the floor with a hob on top and an oven below; a small appliance is a counter with a
machine on it, or a dishwasher under a worktop. The parts are placed in game axes (`F`, a `GameFrame`),
and each model stands where the game places it, its back on the wall (below).

## Materials

Each model maps its materials onto the game's finishes, so it costs one draw call per finish group. The
hob and the oven windows are opaque glossy black (`hob`, a plain finish) rather than the see-through
`glass` finish, which would be a group of its own on every part; only the microwave's window is real
`glass`, and its turntable is a plain gloss plate.

| Material | Finish | Used for |
|---|---|---|
| `enamel` | gloss | a white cooker body; the player's colour (`paint`) |
| `enamel2` | gloss | the range cooker's cream body |
| `cab` | satin | cabinet fronts (the oven tower, the hob counter) |
| `worktop` | gloss | a counter worktop |
| `dark` | satin | oven fronts, plinths, control strips |
| `hob` | gloss | ceramic/induction hobs and oven windows (opaque black) |
| `glass` | glass | the microwave's see-through window |
| `plate` | gloss | the microwave's turntable plate |
| `chrome` | chrome | handles and knobs |
| `grate` | satin | burner heads, grates and hob rings |
| `flame` | gloss | a gas flame ring (gloss, not emissive) |
| `heat` | gloss | the hob counter's and the electric cooker's ring glowing red (gloss, so it reads by day) |
| `charcoal` | satin | the microwave's body |
| `display` | gloss | the microwave's display |
| `tray` | metal | the oven tower's tray; a dishwasher's rack |
| `pot`, `pan` | metal / satin | the stock pot and frying pan on the gas range |
| `steel` | metal | a stainless appliance front (a dishwasher's door) |
| `toaster` | gloss | the breakfast counter's toaster body |
| `jug` | gloss | the blender's jug (opaque, not `glass`, to save a draw call) |
| `bean` | gloss | the coffee machine's bean hopper |
| `cup` | gloss | the cup under the spout |
| `coffee` | gloss | the coffee stream |
| `fruit` | gloss | the breakfast counter's fruit and toast |

## Clips

A door is hinged along its bottom edge (built closed) and drops 85°; a knob turns 90°; a gas flame ring is
built full size but scaled to nothing and grows in every axis while the burner is on (the sinks'
`water_on` way, so it is hidden at rest). The door/tray/knob/flame are built first, the model settled,
then the clips keyed (`settle()` must come before `swing()`/`slide()`).

| Model | Size (w × h × d) | Triangles | Draws | Moving parts | Clips (uses) |
|---|---|---|---|---|---|
| `model.stoveElectric` | 0.600 × 0.980 × 0.659 m | 2,280 | 4 | oven door, knob, hot ring | `oven_open` 1.4 s (bake); `burner_on` 0.5 s: the knob turns and the front ring on its side glows red (cook) |
| `model.stoveGas` | 0.920 × 1.113 × 0.704 m | 2,904 | 4 | oven door, knob, flame | `oven_open` (bake); `burner_on` (cook, gourmet) |
| `model.wallOven` | 0.700 × 2.110 × 0.706 m | 1,668 | 3 | upper door, tray | `oven_open` 1.4 s: door drops and tray slides 10 cm (roast, bake) |
| `model.rangeCooker` | 1.580 × 1.140 × 0.754 m | 3,632 | 4 | left oven door, knob, flame | `oven_open` (bake); `burner_on` (cook, gourmet) |
| `model.cooktopCounter` | 1.000 × 0.955 × 0.660 m | 2,188 | 3 | knob, hot ring | `burner_on` (cook): the knob turns, the front-right ring glows red |
| `model.microwave` | 1.000 × 1.250 × 0.670 m | 2,076 | 4 | door, turntable | `door_open` 1.0 s, `hold` 1.2 s (reheat); `turntable` loops (reheat) |

Seen from the front, the microwave's controls are on the right and its door hinges on its left edge (game +x is
the viewer's left) and swings 100°; its glass turntable turns a full turn every 2 s
while it heats. The gas range and the range cooker carry a pot and a pan on the hob; the flame lights on a
free burner beside them. `stoveGas`, `rangeCooker`, `stoveElectric` (since its ring glows, 2026-10-21) and
`microwave` take 4 draw calls (the most allowed); the rest take 3.

**By the step's tag** (2026-10-21, review G-16): a cooker is a station of recipes started elsewhere (a meal from
the fridge), where the snapshot gives no interaction, only the step's animation tag. So the hobs play `burner_on`
for `stir` too, and the ovens `oven_open` for `bake` (the Bake steps, and a meal's Cook step in an oven or a
microwave: content `animAt`). The microwave opens its door for reheat or `bake`, shuts it (`hold` 1.2 s) and only
then turns its plate (`turntable`, `delay` 2.2 s, `every` 1.9 s, a hair under its 2 s turn: a loop that waits for the door).

## K3 — small appliances and dishwashers

A dishwasher is a tub under a worktop with a door hinged along its bottom edge (built closed, drops 85°)
and a lower rack that slides out 30 cm as the door opens, at the left of its 1 m unit (G-32, 2026-10-21: the whole
front was the door before): the `dishwasher` is a 60 cm machine beside a 40 cm cupboard (a drawer over a door),
the `dishwasherCompact` a slimline 45 cm one beside a 55 cm stack of three drawers, both with stainless doors under
one worktop. The cupboard and drawers are in the still body. The coffee machine slides its cup back under the
spout and runs a thin coffee stream (the sinks' `_water` way: built full size, origin at the spout tip,
scaled to nothing along its length, `grow` runs it out). The breakfast counter's toast stands up in the
toaster at rest; `toast_down` presses it down with the lever, and playing back after the `hold` pops it up.
Its blender lid lifts for `blend` (the jug blades need E1).

| Model | Size (w × h × d) | Triangles | Draws | Moving parts | Clips (uses) |
|---|---|---|---|---|---|
| `model.coffeeMachine` | 1.000 × 1.400 × 0.665 m | 1,748 | 3 | cup, coffee stream | `brew` 0.6 s: cup slides under the spout, stream grows (espresso) |
| `model.breakfastCounter` | 1.000 × 1.380 × 0.665 m | 2,224 | 4 | lever, toast, lid | `toast_down` (toast); `blend` 0.4 s: lid lifts (smoothie) |
| `model.dishwasher` | 1.000 × 0.950 × 0.685 m | 1,592 | 3 | door (0.58 m), rack | `door_open` 0.9 s: door drops 85°, rack slides 30 cm, open for the whole use (dishes; clearing away at it, the `wash` tag) |
| `model.dishwasherCompact` | 1.000 × 0.950 × 0.660 m | 1,808 | 3 | door (0.43 m), rack | `door_open` 0.9 s (dishes, `wash`), as the dishwasher |

The coffee machine and both dishwashers take 3 draw calls; the breakfast counter takes 4 (its three
moving parts each cost one). `check-glb`: 31 models, 0 failed; its five warnings are those same draw counts
(the gas range, the range cooker, the electric cooker, the microwave, the breakfast counter). Khronos glTF validator: 0 errors, 0 warnings
each). The ten sat in Validate › Cookers and
Validate › Appliances until checked in play.

## Rebuild

```
cargo art kitchen_cooking            # the .glbs, the pack, check-glb
cargo art kitchen_cooking --preview  # previews/kitchen_cooking/<id>.png, <id>@<clip>.png
```

## Where it stands (review, 2026-10-14)

Each floor model is centred where the game places it (its `Model`'s `centre`, which `G.settle()` uses),
not on its tile: its back is on the wall face (-0.43), as the old part's, a run of counters and
appliances lines up, and its top is under its content `surface`. Since the
2026-10-14 fix-ups the old parts no longer reach into the wall: both moved forward onto the face.
There is one counter depth, the catalogue's 0.66 m (worktop -0.43..0.23): the catalog2 counters keep
the old 0.79-deep numbers and are drawn through `COUNTER66` (gamekit `GameFrame(depths=)`,
the old `shallow`). Islands stand free, centred.

Also from the review: the hob clip (`burner_on`) lasts the whole use, and only `oven_open`
shuts after `HOLD` (a `{clip, hold}`); each oven has a dark `oven_cavity` face just inside its closed door,
and each dishwasher's dark tub face sits just inside its door, so a dropped door shows an oven or a tub
and the tray and rack slide out of it; the microwave is hollow (floor, top, back, side and the control
column), so its turntable shows through the window and the open door.
