# Engine and tools first

What the clips in the category files lean on. Before this work, an object clip played only while a resident used the
object, or while the sim held it `open` (`web/src/render/babylon/objectMotion.ts`, `models.ts` `motionsOf` / `clipPose`).
A clip played forward, held, then played back. One `hold` per model. Nothing looped or played on its own.

**Status (2026-10-14): all done**, on `v0.3/object-motion`. How each is used is in `tools/art/blender/README.md` and
`docs/design/objects.md` ("Moving parts", "Level of detail", "Materials"). In short:

| Task | Use it as |
|---|---|
| T0 | `/clips.html?keys=model.<id>` (`&clip=`, `&t=`, `&renderer=webgl`); `G.spin`, `G.rock`, `G.squash` (the boulder shape is now `G.boulder`) |
| E1 | `motions.loops: {use: clip}`; `check_glb` fails a loop that doesn't end at rest |
| E2 | `motions.idle: clip`; quality knob `ambientRange` (Low 0, Medium 40 m, High 60 m) |
| E3 | `motions.night: clip`, held from dusk to dawn; curtains stay drawn over sleepers |
| E4 | content `grows: {days, fallow}`, harvest `"when": "ripe"`, `"leaves": "picked"`; `motions.growth: clip`, made with `G.stages` (ripe at rest, then picked, growing, planted) |
| E5 | `motions.uses: {use: [{clip, hold}, clip]}`: several clips, each with its own `hold` |
| E6 | a material's `"mirror": "front"` (gamekit: `glass=('#hex', 'glass', 'front')`) |
| E7 | an entry's `far` file, `<id>.far.glb`; `G.Model(far=0.25)` writes it |
| E8 | materials named `stone@matte`; packs keep the placeholder below; a material's `"glow": "#hex"` (gamekit: `{'glow': '#hex'}`) |
| C1 | the futon, the sofa bed, the washing line and the hot tub stay open (`tests/open_objects.rs`) |

## T0. Clips bench and gamekit helpers (before any category)

- **Clips bench:** turn `/water.html` (`web/src/debug/water.ts`) into a bench for any model's clips.
  - `?keys=` takes any models.
  - Every clip of each plays in a loop: rest, play, hold, play back.
  - Buttons for each clip by name, and `?t=` to hold a time for screenshots.
  - Used by every task's step 7.
- **gamekit `motion.py`:**
  - `spin(obj, clip, turns, seconds, axis)`: whole turns that end at rest (wheels, platters, fans, the train);
  - `rock(obj, clip, degrees, seconds, axis)`: out, back past rest and home (rocking chairs, swings, the seesaw);
  - `squash(obj, clip, scale, seconds)`: scales to any size and back (cushions sinking, bags flattening, a
    held instrument or weight hidden while in the hands).
  - `grow` (in the code, missing from the README's toolkit table) goes into that table.
- **Paint:** families already write labelled `paint` (`[('body', 'Cabinet'), …]`); nothing to do.

## E1. Loops while in use

- **Manifest:** `motions.loops: {useOrTag: clip}`. The clip repeats from rest to rest for as long as the use
  lasts, then finishes its round and stops at rest. It never plays backwards.
- **Renderer:** `objectMotion.ts` keeps a loop's phase per object. Loops run on game time, so they stop when the
  game is paused.
- **Check:** `check_glb.py` requires a loop clip to end where it starts.
- **For:**
  - rocking chair, rocking horse, hammocks and swings, seesaw, spring rider, merry-go-round;
  - treadmill belt, bike pedals, foosball rods, joysticks, cymbals;
  - microwave turntable, washer and dryer drums;
  - record platters, the pottery wheel, the train table, hot tub bubbles, the trampoline, the dance floor.

## E2. Ambient clips

- **Manifest:** `motions.idle: clip`, looping always.
- **Cost:**
  - drawn only within a distance of the camera;
  - off on the Low preset, which must still run on Intel HD 620 / 530;
  - shadows aren't redrawn for them.
- **For:** the ceiling fan, clock hands and pendulum, the fountains' water, pond ripples, the parasol, string
  lights, the scarecrow, the paper lantern, the lava lamp and the rain lamp.

## E3. Night state

- **Manifest:** `motions.night: clip`, held while the lighting key's `lampsLevel` is up (`house.ts`,
  `BabylonRenderer.ts` lamp level).
- **Lights out:** respect lights-out in rooms where everyone sleeps.
- **For:** curtains drawn. Shade glow is E8.

## E4. Growth state

- **Sim:** a per-object growth stage the sim advances (planted → growing → ripe → picked).
- **Renderer:** shows the stage as a held clip position.
- **For:** the edibles (`garden.vegPatch`, tomatoes, strawberries, pumpkins), the raised bed and the nature
  lover's veggie bed.
- **Until then:** a harvest clip (the crop shrinks away during the use) is the stand-in.

## E5. Per-use hold

- **Manifest:** a `uses` value may be `{clip, hold}`, not only a clip name, so one model can have a door that shuts
  after 1 s and water that runs the whole use.
- **For:** showers (door opens, water runs, door shuts), the phone booth, the sauna and the washer.

## E6. Mirrors from files

- **What it is:** a material mapped to `mirror` (front or back) gives `mirrors`, as placeholder parts do
  (objects.md engine item 5).
- **Until then:** mirror glass is a primitive `part` drawn with the file (the double vanity's old way).
- **For:** the mirror cabinet, round, wall and standing mirrors, the dressing table and the romantic vanity.

## E7. Level of detail

- **What it is:** `<id>.far.glb` (or a second mesh) for distance, as nature's `*.far` keys (objects.md engine
  item 4).
- **Needed before:**
  - trees and hedges (G5, G3);
  - street lamps and benches placed by the hundred (CM2);
  - the farm's big buildings (CM5).

## E8. Finishes by name, packs that extend, night glow

objects.md engine items 6–8:
- `stone@matte` material names map themselves;
- a pack entry without `placeholder` keeps the one it overrides;
- lamp shades and screens glow, following the lamp level, without emissive at noon.

## C1. Content: more things that stay open

Interactions with `when` / `leaves` (`docs/content-packs.md`), as the wall bed's:
- **the futon and the sofa bed:** sleep and nap leave them `open`; fold uses close them;
- **the clothesline:** the washing stays out;
- **the hot tub:** the cover stays off.

Each needs a sim test, as `wallBed`'s.
