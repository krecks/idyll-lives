# How to build a new object

*Status: the standard proposed for 0.3.0, written from the review of the first Blender models
(the six garden decor models, branch `v0.3/garden-decor`, 2026-10-13). From 0.3.0 every object's
model is built in Blender and exported as glTF: all 541 were built and signed off in play by 2026-10-17
([docs/objects/](../objects/README.md)). Files now resize, are picked by what's drawn, load
once per scene and can have [moving parts](#moving-parts) (branch `v0.3/object-motion`); the rest of
[Before Blender is the standard](#before-blender-is-the-standard) is still to do. The toolkit and
the step-by-step are in [tools/art/blender/](../../tools/art/blender/README.md).*

An object is three things, joined by keys:

| Part | Where | Owned by |
|---|---|---|
| The definition: footprint, layer, top, seat, slots, light, interactions, price | `web/public/content/*.json` (`objects`, see [content-packs.md](../content-packs.md#objects)) | The simulation. It never sees the model. |
| The model entry: file, finishes, fallback | An asset manifest (`model.<family>.<id>`) | The renderer (`render/babylon/models.ts`) |
| The model: one `.glb` | `web/public/assets/models/<family>/<id>.glb`, exported by a family script in `tools/art/blender/families/` | The art |

The content must agree with the model (a seat at the seat's height, a top where the top is), and
the model must fit the footprint the content gives it. Nothing checks that for you yet beyond the
gate below, so the review does.

## The short version

1. Write or extend a family script in `tools/art/blender/families/` on `gamekit` (copy
   `_template.py`; the steps are [tools/art/blender/README.md](../../tools/art/blender/README.md)), in
   metres, front −Y, origin at the footprint centre on the floor. Build to the real size; don't stretch to fit.
2. Name materials for what they are (`stone`, `water`, `tunic`), each with one of the game's finishes,
   in the family's `G.Palette`; the pack entry's `materials` is written from it.
3. Stay in budget: 1–3 draw calls (finish groups), triangles for the size class below.
4. Build with `python3 -B tools/art/blender/build.py <family>` (Blender 5.2 LTS): it exports, and
5. registers the model in the family's pack with `url` and `materials`, then
6. runs `check_glb.py` (no errors). Run the Khronos glTF validator too (no errors or warnings).
7. Look at it in the gallery, on a lot by day and at night, on WebGPU and WebGL2, and in the catalog. Credit it. Commit only the files you changed.

## Tools and folders

- **Blender 5.2 LTS**, run from scripts by `build.py` (`Blender --background --factory-startup --python
  families/<family>.py`). `gamekit` warns on another version. The committed files were exported with 5.2.2 (glTF I/O
  5.2.40). A new LTS is adopted by re-exporting every family and checking the shots, in one commit.
- **Scripts are the source.** A model is built by Python on `gamekit` (primitives, `revolve`,
  `boulder`, bevels, `join_all`), so its source is reviewable text and re-exporting is one command.
  A shape that can't reasonably be scripted may come from a `.blend` file next to its script,
  saved compressed and under 2 MB; the export still runs from the script, so the settings stay fixed.
- **Folders and names:**

| What | Where | Name |
|---|---|---|
| Shared toolkit | `tools/art/blender/gamekit/` | `import gamekit as G` |
| The runner | `tools/art/blender/build.py` | `build.py <family> [--only <id>] [--preview]` |
| A family's script | `tools/art/blender/families/<family>.py` | `garden_decor.py` builds `model.garden.*`; `_template.py` to copy |
| The models | `web/public/assets/models/<family>/<id>.glb` | the key's last part: `model.garden.gnome` → `garden/gnome.glb` |
| Their manifest entries | `web/public/assets/packs/<family>/manifest.json`, listed in the base manifest's `packs` | |
| Previews (`build.py <family> --preview`, `check_glb.py -- --preview`) | `tools/art/blender/previews/` (ignored by git) | `<id>.png`, `<id>@<clip>.png`, `rt_<id>.png` |

  Inside the file: one mesh object per model, named as the id (one per moving part, below); one
  material per colour, named for what it is. Material names are the keys the manifest maps, so renaming one is a manifest change.
- **Exports aren't byte for byte repeatable.** Blender 5.2's exporter writes the same triangles
  in a different order on every run. Re-export, then commit only the models you meant to change
  (`git checkout` the others).

## Scale, axes, origin and footprint

- **1 unit = 1 metre.** Blender is Z up; the exporter's +Y up makes Blender (x, y, z) the game's
  (−x, z, −y). The front faces −Y in Blender (Blender's front view, numpad 1, is the player's view
  of the front) and +Z in the game. So x runs the other way in the game's numbers (the content's,
  `G.GameFrame`'s): a part on the right of a front view is at +x in Blender and at −x in the game
  (the fridge's handle: `F` x −0.3, Blender x +0.3).
- **Origin** at the footprint's centre, on the floor: the lowest point at y = 0 for things on the
  floor. Things on a top stand on y = 0 at their middle (the game lifts them to the top); things on
  a wall have their back on the wall face, z = −0.43 in a 1-deep footprint, at their real height;
  ceiling things hang from 2.8 m ([content-packs.md](../content-packs.md#objects), `layer`).
- **Footprint:** the content's `footprint` is in whole 1 m tiles (`[1, 1]` by default). The model
  stays inside it with a few centimetres to spare, unless it is meant to meet its neighbour (a
  counter run). The contact shadow (1.15 × the footprint) and buy mode's checks follow the
  footprint; picking follows the drawn outline.
- **Real size, uniform scale.** Build to the object's real size; `G.settle` stands it on the floor,
  centred, by moving it, and the family's `G.Model(size=…, tolerance=…)` asserts its size (2 % by
  default). `gamekit.fit` scales each axis on its own to hit an exact box: it stretched the garden
  lantern's hexagon 15 % wider than deep and squashed the boulders to 75 % of their depth and 120 % of
  their height, until the garden decor was rebuilt at real size (2026-10-14). It, and the manifest's
  `fit`, are for third-party files only.
- **Heights the content names** come from the model: `seat.height`, `surface.height` and its
  rectangle (`size`, `at`), `light.height`, and `slots` spread across the width. Mark them in
  Blender with empties named `top`, `seat`, `seat.1` …, `light` (not exported), and have the
  family script assert them against the content JSON (`families/lighting.py` reads each lamp's
  `light.height` from the content and `_near_light` checks its bulb against it). The content stays the
  truth for the simulation; the script keeps the two in line.

## Budgets

Measured on 813 primitive models as the game's loader builds them (LOD 0), and set so that a
Blender model costs what a primitive one does today. Every object kind is drawn with thin
instances, so these costs are per kind of object in view, plus the triangles of each copy.

| Largest side | Placeholders today: triangles (median / 90th pct.) | Target | Most (`check_glb.py` fails above) |
|---|---|---|---|
| under 0.35 m | 1,326 / 3,048 | 1,500 | 3,000 |
| 0.35–1 m | 3,688 / 9,116 | 4,000 | 8,000 |
| 1–2.5 m | 7,440 / 20,042 | 8,000 | 16,000 |
| over 2.5 m | 4,390 / 17,780 | 12,000 | 24,000 |

- **Things placed many times** (garden plants, park and street props, anything the town has dozens
  of) aim for half the target: the town overview draws every lot's outdoor objects at full detail.
- **Draw calls:** one per finish group (below). Aim for 1–3, at most 4. Each also draws once more
  in the sun's shadow map, which in live play is redrawn every frame while residents are in view
  (graphics.md). Map every material to a finish: six garden models took 30 draws unmapped, 9 mapped.
- **Textures:** none by default; the finishes bring their own tiling textures. A texture of its
  own (a painting, a label) is one per object, at most 512 px a side (1024 for something over
  2.5 m), PNG or JPEG in the file; the build turns it into KTX2.
- **Download:** follows from the triangles: about 12 bytes a triangle after `optimize.mjs`
  (the six garden models: 445 KB as exported, 280 KB in the build). Untextured models export
  without UVs.
- **The presets** draw the same models. Low trims shadows, the lawn and lamps, not objects, and on
  the M3 Low is limited by geometry (graphics.md), so these budgets are set for Low; Medium and High
  have room to spare. There is no level of detail for objects
  yet (below).

## Materials, finishes, tints, styles and quality

The game's surfaces are a dozen finishes, the `material.finish.*` entries of the base manifest:
`matte` (roughness 0.75), `satin` (0.42), `gloss` (0.12), `metal`, `chrome`, `skin`, `hair`
(plain), `fabric`, `wood`, `bark`, `foliage` (textured, tinted by the part's colour) and `glass`
(see-through). A model's parts are coloured per part; all the plain finishes of a model merge into
one mesh, so a model costs one draw call for its plain parts plus one per textured or see-through
finish it uses. The plain finishes carry their roughness and metalness in the vertex colour's alpha
(`encodeFinish`, read by `PlainFinishPlugin` in `render/babylon/materials.ts`; until 2026-10-15 the
shader never saw it and drew every plain finish as rough metal). Metals mirror the sky just above the
horizon where they would reflect the ground: the game looks down on everything, so anything upright
and shiny (a handle, a steel fridge) would otherwise mirror the dark field of the sky map, which no
room has.

- **Hardware is metal.** Handles, knobs, pulls, latches and taps take a `metal` or `chrome` palette
  colour (chrome or brushed steel, brass, black iron as the piece's style wants), on base models and
  packs alike. Colouring such a part keeps its finish, so a tinted handle is tinted metal (anodised),
  not paint; "As made" is the plain metal. On a still part that has no plain material yet, metal
  hardware costs a draw call (a wood-only dresser body): the dresser, the low dresser, the dressing
  table and the writing bureau took their fourth for it; the record cabinet's lid would take a fifth,
  so its pull stays wood. Left as they are: what isn't furniture hardware (mugs, pans, tools, grips,
  toys' handles, plastic bins), and wood where heat wants wood (the sauna's and the pizza oven's doors).

- **Map every material to a finish** in the entry's `materials`, by material name, with `*` for
  the rest:
  ```json
  "materials": { "*": { "finish": "material.finish.matte" }, "water": { "finish": "material.finish.gloss" }, "leaf": { "finish": "material.finish.foliage" } }
  ```
  The file's base colour becomes the part's colour; its roughness and metalness are ignored (the
  finish decides), so choose the finish, not a number. An unmapped material keeps the file's own
  PBR material: a draw call of its own, a look that doesn't match the furniture, and (today) a
  material left behind each time the model is rebuilt. `check_glb.py` warns of each one.
- **Finishes by name.** A material named `<name>@<finish>` (`stone@matte`, `leaf@foliage`) is drawn
  with that finish without a `materials` entry (one in the entry still wins). Handy for files not made
  with gamekit, whose palettes write the map themselves.
- **Colours** are sRGB hex in the script, as in the manifests; `gamekit.mat` converts them to the
  linear values Blender and glTF use. (The first export wrote sRGB values as linear: every colour
  came out pale, and white-blue at night.)
- **One-sided.** Meshes are closed and materials one-sided (`gamekit.mat` sets it).
- **Water** is `gloss`, opaque. Real glass is `glass`: its own draw call,
  drawn after everything else.
- **No emissive materials.** A lamp's light is the content's `light`, which also lights the lamp's
  own shade at night; an emissive surface glows at noon as well.
- **Night glow.** A material with `"glow": "#FFD9A0"` in its `materials` entry (a lamp's shade, a
  screen) glows in that colour as the windows do: from dusk as the lamps come up, out with its
  room's lights (a lamp's with the lamp itself, switched off), nothing by day. It is a copy of those faces drawn added on
  top (one more draw call at night per part it's on, none by day); on a moving part (a laptop's lid)
  it moves with it. In gamekit:
  `shade=('#F2E6CC', 'fabric', {'glow': '#FFD9A0'})`.
- **Tints** (`body`, `skin`, `hair`) are for character parts; no object model uses one. A file's
  material carries one in its name, `tunic#body` (with a finish: `tunic@fabric#body`): a tint slot,
  as the template's `tints` say. Nothing draws by them today (residents
  have their own rig and outfits).
- **No styles.** An object has one model, `model.<id>` (the modern, cozy and minimal variants were
  removed on 2026-10-14). Another look of the same thing is another object with its own model (the
  mini fridge, the built-in fridge).
- **Colours the player picks.** A model whose entry has `paint` lists the parts the player may colour:
  `{material, label}` (the double vanity's `body` "Cabinet", `stone` "Worktop", `chrome` "Taps &
  handles"…), the main part first. Each part can take one of the content's `objectColors` (the shared
  palette) or a colour of the player's own, per placed object (sim-core `ObjectInstance::colors`,
  `#rrggbb` by material, saved as `objects[].colors`; a part left out is as made). The item panel's
  quick row colours the main part; its "Every part…" button opens the colour studio
  (`ui/buy/CustomiseColours.svelte`): the item live on a stage of its own (`render/preview/itemStage.ts`)
  beside its parts, applied on Done as one edit (one undo). The stage plays the model's clips as the
  game does (`poseParts`): the ambient one always, and once the player picks a part, every clip that
  opens (all but loops, `every`, night, growth and clock clips) opens and stays open, so what a door,
  drawer or cover hides shows; "At rest" / "In use" switches it either way. The renderer draws each part in its colour
  with its finish (`painted` in models.ts); parts not listed keep theirs. Name the main part `body`,
  give every other part its own material named for what it is, and list in `paint` only what's worth
  colouring (not drains, water or food). Each combination of colours in use is a template of its own:
  a few more draw calls per combination on a lot, the file still loaded once. Colours never change
  price or quality. Primitive models take no colour.
- **Resizing** is switched off in Buy mode for now (`RESIZING` in `web/src/game/buildmode.ts`): a model's
  moving parts don't follow a resize well (a fridge stretched wide, a wall bed). It works for files and
  will come back for the objects it suits, decided per object.
- **Quality (★) and wear** change what an object gives, not how it is drawn: nothing to model.
- **Mirrors:** a material whose `materials` entry has `"mirror": "front"` (or `"back"`) is mirror
  glass: its face reflects the room, as `mirror` parts did (the quality's
  `mirrors`: the nearest on screen, one on Low and Medium, two on High). Make the glass a flat piece, on the still body or on one moving part
  (a mirror cabinet's door: the reflection swings with it); the face is across its thinnest side,
  square or round as drawn. In gamekit, the palette takes the side: `glass=('#DCE6EA', 'gloss',
  'front')`. Map it to `gloss`, as mirrors always were: the mirrors past the nearest show their
  glass, and see-through `glass` would show what is behind it. `check_glb.py` fails glass split over
  two parts and warns of glass that isn't flat or is see-through.

## Level of detail

- **Objects** are drawn at full detail wherever play shows them: the game draws only the lots
  around the one in view. Hence the budgets above.
- **Far versions:** a model's entry may name a second file, `"far": "../../models/<family>/<id>.far.glb"`,
  at about a quarter of the triangles and standing still (no moving parts). It is drawn where the
  model is only seen from afar: the town overview behind the menus (its gardens and street,
  `BuildOptions.far`) and the far woods (`lod` 1). A model without one is drawn whole there. In gamekit,
  `G.Model(..., far=0.25)` writes it from the model at rest (joined, decimated to that share, without
  what only shows while it moves), or `far=build_function` builds one of its own; `check_glb.py`
  checks it (still, the same size, at most half the triangles). Give one to what is placed by the
  hundred or seen far off: trees and hedges, street lamps and benches, the farm's buildings.
- **Plants and trees** (`render/babylon/nature.ts`) use a reduced build at a distance, for the far
  woods too: a file's `far` (`families/plants.py`). Its parts on `foliage` and `bark` take the nature
  shader's wind and tint.

## Moving parts

A door, a lid or a drawer that moves while residents use the object: one node per moving part in
the file, moved by a clip of its own. The fridge's doors (`families/kitchen.py`), the wall bed that folds
down for a whole sleep (`families/living.py`) and the template's lid and drawer (`families/_template.py`)
are the examples; `G.hinged`, `G.turn`, `G.swing` and `G.slide` do the work.

- **One node per part, its origin on the hinge** (or where a drawer slides from), named for what it
  is (`door_fridge`). What moves with it is joined into it or parented to it (the handle). The rest
  of the model is one node (`cabinet`). A part inside another (`G.attach`: the retro fridge's lever on
  its door, the jukebox's label on its record) moves with that one's clips and plays its own on top.
- **Several clips on one part** play together: their poses multiply, the clips of the part itself
  first, then those of the parts it is inside. So a record swings up and spins (a `uses` clip on the
  record, a loop on its label inside it). Two clips on the same node also multiply, but in no set
  order (except a crop's growth stage, which comes first: **Growth** below): for a swing and a
  spin, put the spinning part inside the swinging one.
- **One clip per part**, named `<part>_open` (an NLA track of that name, made by `G.swing` or
  `G.slide`; the build exports the tracks when a model has any). Its first key is the rest pose, where
  the file has the part and where the game draws it closed: frame 1 when exporting. Add the clips after
  `G.settle`. Rotation (and translation, scale) of the
  node only. Two keys are enough; the game blends between samples of the clip, 30 a second.
- **When it plays** is the pack entry's `motions` (`objectMotion.ts`; the family's `G.Model(motions=…)`): the clip a use plays, by the
  interaction's id or its animation tag, and `hold`, the seconds after a use starts that the clip
  plays back (absent: when the use ends). Time is the residents' animation time: still while paused,
  faster at higher speeds.
  ```json
  "motions": { "uses": { "fetch": "fridge_open", "drink": "fridge_open" }, "hold": 3.5 }
  ```
- **Several clips, each its own hold:** a use's value may be `{"clip", "hold"}` (its own `hold`, over
  the model's) or a list of clips, so one use can shut a door after a second while the water runs on.
  A clip closed after its hold stays closed until the use ends.
  ```json
  "motions": { "uses": { "shower": [{ "clip": "door_open", "hold": 1 }, "water_on"] } }
  ```
- **Later in the use:** `delay` starts a clip that many seconds after the use starts (its `hold`
  counts from then): the retro fridge's lever turns first and the door swings 0.3 s later.
  ```json
  "motions": { "uses": { "fetch": [{ "clip": "lever", "hold": 0.4 }, { "clip": "fridge_open", "delay": 0.3 }] }, "hold": 3.5 }
  ```
- **Again and again:** `every` plays a clip from rest to rest once every that many seconds while the use
  lasts, the first `delay` seconds in (a net swishing after each shot). It must end where it starts, as
  a loop does (`G.rock`, `G.spin`, `G.squash(back=True)`); it never plays backwards, and when the use ends
  it finishes the round it is in.
  ```json
  "motions": { "uses": { "shoot": { "clip": "swish", "delay": 1.4, "every": 3.2 } } }
  ```
- **Per slot:** `slot` plays a clip only for the resident in that slot (content `slots`, from 0 as
  sim-core's `slot_position` spreads them; a bunk's upper bunk is 1), so each bunk's duvet turns down for
  its own sleeper. The renderer tells the slot by where the resident is. (The sofas still sink every seat
  cushion: their cushions are one part.)
  ```json
  "motions": { "uses": { "sleep": [{ "clip": "turn_down", "slot": 0 }, { "clip": "turn_down_upper", "slot": 1 }] } }
  ```
- **Loops:** `loops` names the clip a use plays round and round while it lasts (a rocking chair, a
  washer's drum), by interaction id or animation tag. It goes from rest to rest (`G.rock`, `G.spin`:
  `check_glb.py` fails a loop that ends away from where it starts), never plays backwards, and when the
  use ends it finishes its round and stops at rest. A use can play `uses` clips and a loop together.
  ```json
  "motions": { "loops": { "relax": "rocking", "read": "rocking" }, "ride": "rocking" }
  ```
- **Riders:** `ride` names the clip whose part carries the residents sitting or lying on the object
  (a rocking chair's seat, a swing, a hammock): while it plays they move and tilt with that part (the
  first one the clip moves), as if sitting on it. Build the seat in its rest pose where the game seats
  them (`SEATS`, `BEDS` heights), with its origin on the pivot.
- **Ambient:** `idle` names a clip that goes round all the time, used or not (a ceiling fan, a
  pendulum, a fountain's water), from rest to rest like a loop. It plays only within the quality's
  `ambientRange` of the camera (40 m on Medium, 60 on High) and not at all on Low, where it stays at
  rest; the sun's shadows aren't redrawn for it. Make it something that looks fine at rest. When the
  game is paused it holds where it is.
  ```json
  "motions": { "idle": "blades_turn" }
  ```
- **Clocks:** `clock` names a clock's hands: `hour` a clip turning its hand once round in 12 hours,
  `minute` one turning its hand once round in an hour (each a whole turn from rest, any length:
  `G.spin`), and `at` the time the hands show at rest, in minutes past twelve (the catalogue picture's
  ten past ten is 610). The game holds each where the game's time has it, as state rather than motion:
  on every preset, set each game minute (each five on Low) for the clocks within the ambient range
  (30 m at least, and on Low), and not at all for the others until they come near. No second hand: at
  normal speed a game minute is a real second.
  ```json
  "motions": { "clock": { "hour": "hour_hand", "minute": "minute_hand", "at": 610 } }
  ```
- **Night:** `night` names a clip held open from dusk to dawn, while the lamps are lit (curtains
  drawn): it closes as the lamps come up past half way and opens again as they go down. A room going
  dark because everyone in it is asleep doesn't open it (the curtains stay drawn over the sleepers);
  what follows a room's own lights is the shade glow (engine item 8). A game loaded at night shows it
  closed without playing it.
  ```json
  "motions": { "night": "close" }
  ```
- **Growth:** a crop (content `grows`: planted, growing, ripe, picked) holds its `growth` clip where
  its stage is: ripe at rest (the catalog picture, the fallback's look), picked a third of the way
  along (the crop gone), growing two thirds (smaller) and just planted at the end (seedlings). From
  stage to stage it plays on at the clip's pace: a harvest shrinks the crop away, replanting the
  plants, and growing goes back through picked (no crop yet) to ripe. `G.stages` makes it, a scale
  per stage for each part. A game loaded shows each crop at its stage without playing. A use's clip on a
  growing part plays on top of its stage: the stage's pose first, then the use clip's about the part's rest
  origin (`partPose`), so tending a just-planted veggie bed rustles its seedlings by their own small height
  rather than popping them to full size; a scale (the lettuce pressed into the soil) multiplies with the stage's.
  ```json
  "motions": { "growth": "grow", "uses": { "tendVeggies": { "clip": "rustle", "delay": 0.6, "every": 2.4 } } }
  ```
- **Open until closed:** an object can be open (the sim's `open`, saved), set by interactions with
  `leaves` and offered by `when` ([content-packs.md](../content-packs.md)). `motions.open` names the clip
  held open while it is: the wall bed stays down after *Fold down* or a night's sleep until *Fold up*
  (or the last one up in the morning) folds it. Put the uses that open it in `uses` too, so the clip
  starts with the use rather than when the house is redrawn. An open object is clicked by its whole
  tiles (its drawn outline is the closed one).
  ```json
  "motions": { "uses": { "sleep": "bed_down", "nap": "bed_down", "foldDown": "bed_down" }, "open": "bed_down" }
  ```
- **Cost:** each part is drawn on its own (one draw per finish group within it, and again in the sun's
  shadow map): the fridge takes 3 draws, 1 for the cabinet and 1 per door. Count parts in the 1–3.
  While a part moves, its kind's buffer is rewritten each frame and the shadows are redrawn (not for
  ambient clips or clock hands); nothing is done while all are still.
- **Resizing** moves each part's hinge with its edge, so a wider fridge's door still turns on its side.
- **Not yet:** the freezer door, which no use
  opens (no interaction takes frozen food), and tiles that free up while an object is closed (a folded
  wall bed still takes all its tiles).

## Export settings

`gamekit.export_glb` holds them (`gamekit/export.py`); don't export by hand.

| Setting | Value | Why |
|---|---|---|
| Format | GLB (one binary file) | One request; `optimize.mjs` reads GLB |
| Up axis | +Y (`export_yup`) | Blender −Y becomes the game's front, +Z |
| Modifiers | applied | Bevels and the like are baked in |
| Selection | the model's mesh objects only | No cameras, lights or helpers in the file |
| UVs | off unless textured | Nothing reads them; they split vertices and cost bytes |
| Normals | on | Smooth and hard edges as modelled |
| Materials | exported, one-sided | Their names and colours drive the finishes |
| Compression | none (no Draco) | The build adds meshopt; the game ships no Draco decoder |
| Clips | moving parts only (`clips=True`): node rotation, translation or scale, first key at rest | Thin instances move whole nodes, not bones (below) |
| Skins, morphs | none | Placed things are thin instances without bones |

## Validation

- **`python3 -I -B tools/art/blender/check_glb.py`** reads every `.glb` the manifests use, with
  its entry and its content object, and exits 1 on an error. Errors: a model the content draws (an
  object's `model` or `model.<id>`, an item's `model` or `item.<id>`) that no manifest has, or whose
  entry has no `url` once the packs are stacked (its pack missing from `packs`); an empty `model`; a
  key on the command line that no manifest has; a `url` that isn't a `.glb`; more than 4 draw calls
  after the `materials` mapping (each moving part's counted apart), triangles over the most for the
  model's size, a model below the floor, extensions the game can't read, skins, morph targets, a clip
  that animates anything but a node's transform or doesn't start at rest, a `motions` clip the file
  doesn't have, textures over 1024 px, a mapping to a finish that doesn't exist. Warnings: a clip no
  use plays, over 3 draws or the triangle target, unmapped, double-sided, see-through or emissive
  materials, unused UVs, a floating origin, a Blender-built model off its `Model`'s `centre` (the
  middle, or the `centre` it gives; `tools/art/blender/centres.json`, which the build writes for the gate
  only) by over 2 cm at rest (not its far version), one that reaches over 1 cm into the wall, one
  indoors (on the floor, a wall or the ceiling, without `vary`) that reaches over 2 cm past its
  footprint at rest, or whose part that goes round (a loop or ambient spin: a fan's blades) sweeps
  over 1 cm past it, a thing on a top that reaches more than 10 % past its content `radius` from its
  origin (its farthest vertex, as it shows), and a floor model or moving part lower than the floor
  drawn over it; each against every content object that draws the model. No content fact says a thing
  stands against a wall, so one standing off it isn't caught. Plain Python, so CI can run it; inside
  Blender, `-- --preview` also renders each file re-imported. A content file or manifest it can't read
  is a FAIL naming the file.
- **`cargo test -p sim-core --test shipped_content`** (in CI) fails when an object or item has no
  model file, or no `far` file where its entry names one, once the manifests are stacked as the game
  stacks them, and on an empty `model`.
- **The build** (`Family.run`, before it builds anything; `write_pack` again as a backstop) refuses a
  `key=` that no content object or item names and no manifest has, unless its `Model` has
  `code=True` (a model the game draws by code: a held `tool.*`, a street prop).
- **The Khronos glTF validator**: no errors and no warnings. It is on npm as `gltf-validator`
  (Apache-2.0); install it outside the repo and run `validateBytes` over the files, or drop them on
  the validator's web page.
- **Looking at it** (headless Chrome for Testing on the real GPU, as graphics.md's runner; not
  `chrome-headless-shell`, which draws in software):
  - `/gallery.html?keys=model.<family>.<id>` (WebGPU), and the same with `&renderer=webgl`.
  - A lot with the object on it (`?state=<name>` from a test dump, or a bench state with objects
    added), `&hour=13` and `&hour=22`, WebGPU and `&renderer=webgl`.
  - The catalog picture (`render/preview/items.ts`, WebGL2).
  - Its clips: `/clips.html?keys=model.<id>` plays every clip of the model in turn, the way the game
    plays it (a use's opens, holds and closes; a loop goes round); a button per clip (`&clip=`),
    `&t=` to hold a time, `&colors=body:%23hex`, `&renderer=webgl`.
  - Placed turned (all four `rot`), and resized when the engine supports it.

## Optimisation and registration

- **Optimisation is automatic.** `vite build` runs `tools/assets/optimize.mjs`, which writes a
  meshopt copy of every `.glb` (and KTX2 for embedded textures) and an index the game reads;
  the dev server serves the source files. Both load on WebGPU and WebGL2 (checked with the garden
  models). Quantising the geometry as well (`KHR_mesh_quantization`) would take the six garden
  models from 280 to 178 KB; it is an `optimize.mjs` change for files from Blender only (the foliage
  cards keep float data).
- **Registration** is an entry in the family's pack, listed in the base manifest's `packs`:
  ```json
  "model.garden.gnome": { "type": "model", "url": "../../models/garden/gnome.glb", "materials": { "*": { "finish": "material.finish.satin" } } }
  ```
  A pack's entry replaces the base entry, keeping its `vary` unless it gives its own (registry.ts
  `extended`); the base entry's other fields are copied in. The family script writes its pack from the
  manifests at export time (`G.write_pack`, called by the build), never by hand. `scale`, `rotationY`,
  `offset` and `fit` are for
  third-party files; a model built to this standard needs none of them. `parts` adds primitive
  parts drawn with the file.
- **A new object** also needs its content definition (footprint, interactions, price, category).

## Catalog pictures

Nothing to do: the catalog draws every model with the game's own loader in a small WebGL2 engine
(`render/preview/items.ts`) and caches the pictures per app version and manifest entry, so a new
file or mapping draws a new picture. Check it, as above. An `icon` key is optional.

## Credits and licences

All art is original or CC0 (look.md). A model built from a script in this repo is original:
add it to `web/public/assets/CREDITS.md` (what, which script, "original, CC0"). The scripts are
code, under the game's GPL. Downloaded models, if ever used, must be CC0, with source, author and
licence in CREDITS.md, and go through the same gate. Blender's licence doesn't extend to its output.

## Review checklist

- [ ] Built by a family script in `tools/art/blender/families/` on `gamekit`, Blender 5.2 LTS; re-exports with `build.py <family>`.
- [ ] Real size, no stretching; origin at the footprint centre on the floor (or the layer's rule); front −Y in Blender.
- [ ] Inside its footprint; heights and tops agree with the content (`seat`, `surface`, `light`, `slots`).
- [ ] Every material mapped to a finish; 1–3 draw calls; triangles within the target for its size (half for things placed many times).
- [ ] No emissive, no see-through except `glass`, no textures of its own unless needed (then ≤ 512 px).
- [ ] `check_glb.py` passes; the Khronos validator reports nothing.
- [ ] Gallery; a lot by day and at night; WebGPU and WebGL2; the catalog picture; turned four ways.
- [ ] Parts the player may colour named in `paint` (`body`), the rest in materials of their own.
- [ ] Pack entry with `url` and `materials` (written by the build); base manifest lists the pack.
- [ ] CREDITS.md updated; only the changed `.glb` files committed; previews not committed.

## Every model is a Blender file

**Every object is migrated** (2026-10-17): each has a Blender file, signed off in play, and so do the
things residents carry, the town's street props and the ground's tufts, rocks and marker. The
primitive-part entries they replaced are gone from the manifests (2026-10-18), so Blender is the only
source; the models still stand where those parts stood, so each kept its key and saves and content
didn't change. A new object gets a Blender file from the start.

## Before Blender is the standard

What the engine (`render/babylon/models.ts` and around) must do first, most important first.
Placement, turning, height, shadows, contact shadows, footprints, the fallback when a file fails (a
plain box, until a background try brings the file),
WebGPU and WebGL2, the town overview, catalog pictures, meshopt copies and player colours (`paint`) already
work for files.

1. ~~**Resizing (blocker).**~~ Done: a file's vertices take the same edge bands and rise line as
   `resizeParts` (`resizeMaps`, `resizeData`), and its moving parts' hinges move with them.
2. ~~**Picking outline.**~~ Done: `loadGltf` returns the baked geometry's bounds.
3. ~~**Materials left behind.**~~ Done: each file is imported once per scene (`loadFile`); builds
   copy its geometry and share its materials.
4. ~~**Level of detail** for files.~~ Done: an entry's `far` file, `<id>.far.glb`, is what LOD 1 and
   the town overview draw (`G.Model(far=…)` writes it).
5. ~~**Mirrors** from files.~~ Done: a material mapped with `mirror` (front or back) gives `mirrors`
   as `mirror` parts did (`fileMirror`).
6. ~~**Finishes by name.**~~ Done: a material named `stone@matte` maps itself (`namedFinish`), and
   `tunic#body` carries a tint slot (`namedTint`).
7. ~~**Packs that extend.**~~ Done: a pack entry without `vary` keeps the one of the entry it
   overrides (registry.ts `extended`); gamekit's packs no longer copy it.
8. ~~**Night glow**~~ Done: a material's `glow` (lamp shades, screens), following the lamp level and
   its room's lights.
9. **Quantised geometry** in `optimize.mjs` for files from Blender.
10. **CI** runs `check_glb.py` (it needs no Blender) with the other checks.
