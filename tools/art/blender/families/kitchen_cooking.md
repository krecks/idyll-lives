# Kitchen cooking — notes

`kitchen_cooking.py` builds the kitchen's cookers and small appliances into
`web/public/assets/models/kitchen_cooking/` and their entries in `packs/kitchen_cooking/manifest.json`:
the six **K2 cookers** — the electric cooker (`model.stoveElectric`), the gas range (`model.stoveGas`), the
own tower (`model.wallOven`), the range cooker (`model.rangeCooker`), the hob counter (`model.cooktopCounter`)
and the microwave counter (`model.microwave`) — and the four **K3 small appliances** — the coffee machine
(`model.coffeeMachine`), the breakfast counter (`model.breakfastCounter`) and the two dishwashers
(`model.dishwasher`, `model.dishwasherCompact`). They replace the primitive-part placeholders of
`catalog.py` / `catalog2.py`, which stay as the fallback. Built 2026-10-14.

A cooker is a box on the floor with a hob on top and an oven below; a small appliance is a counter with a
machine on it, or a dishwasher under a worktop. The parts are placed in game axes (`F`, a `GameFrame`
shifted back by the placeholder's depth centre, so the model is centred on its tile).

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
| `model.stoveElectric` | 0.600 × 0.980 × 0.659 m | 2,192 | 3 | oven door, knob | `oven_open` 1.4 s (bake); `knob_on` 0.5 s (cook) |
| `model.stoveGas` | 0.920 × 1.113 × 0.704 m | 2,904 | 4 | oven door, knob, flame | `oven_open` (bake); `burner_on` (cook, gourmet) |
| `model.wallOven` | 0.700 × 2.110 × 0.706 m | 1,668 | 3 | upper door, tray | `oven_open` 1.4 s: door drops and tray slides 10 cm (roast, bake) |
| `model.rangeCooker` | 1.580 × 1.140 × 0.754 m | 3,632 | 4 | left oven door, knob, flame | `oven_open` (bake); `burner_on` (cook, gourmet) |
| `model.cooktopCounter` | 1.000 × 0.963 × 0.790 m | 2,368 | 3 | knob, flame | `burner_on` (cook) |
| `model.microwave` | 1.000 × 1.260 × 0.670 m | 1,644 | 4 | door, turntable | `door_open` 1.0 s, `hold` 1.2 s (reheat); `turntable` loops (reheat) |

The microwave door hinges on its left edge and swings 100°; its glass turntable turns a full turn every 2 s
while it heats. The gas range and the range cooker carry a pot and a pan on the hob; the flame lights on a
free burner beside them. `stoveGas`, `rangeCooker` and `microwave` take 4 draw calls (the most allowed);
the rest take 3.

## K3 — small appliances and dishwashers

A dishwasher is a tub under a worktop with a door hinged along its bottom edge (built closed, drops 85°)
and a lower rack that slides out 30 cm as the door opens; the `dishwasher` wears the kitchen's cabinet
fronts and the `dishwasherCompact` a stainless door. The coffee machine slides its cup back under the
spout and runs a thin coffee stream (the sinks' `_water` way: built full size, origin at the spout tip,
scaled to nothing along its length, `grow` runs it out). The breakfast counter's toaster lever slides down
and the toast pops up at the end of `toast_down`; its blender lid lifts for `blend` (the jug blades need
E1).

| Model | Size (w × h × d) | Triangles | Draws | Moving parts | Clips (uses) |
|---|---|---|---|---|---|
| `model.coffeeMachine` | 1.000 × 1.410 × 0.665 m | 1,748 | 3 | cup, coffee stream | `brew` 0.6 s: cup slides under the spout, stream grows (espresso) |
| `model.breakfastCounter` | 1.000 × 1.390 × 0.665 m | 2,224 | 4 | lever, toast, lid | `toast_down` (toast); `blend` 0.4 s: lid lifts (smoothie) |
| `model.dishwasher` | 1.000 × 0.960 × 0.685 m | 1,160 | 3 | door, rack | `door_open` 0.9 s: door drops 85°, rack slides 30 cm (dishes) |
| `model.dishwasherCompact` | 1.000 × 0.950 × 0.790 m | 1,160 | 3 | door, rack | `door_open` 0.9 s (dishes) |

The coffee machine and both dishwashers take 3 draw calls; the breakfast counter takes 4 (its three
moving parts each cost one). All match their placeholder bounds (`check_glb.py`: 31 models, 0 failed;
Khronos glTF validator: 0 errors, 0 warnings each). The ten sit in Validate › Cookers and
Validate › Appliances until checked in play.

## Rebuild

```
python3 -B tools/art/blender/build.py kitchen_cooking            # the .glbs, the pack, check_glb
python3 -B tools/art/blender/build.py kitchen_cooking --preview  # previews/<id>.png, <id>@<clip>.png
```
