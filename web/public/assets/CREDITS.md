# Third-party assets

All of the third-party *art* below is **CC0 1.0 (public domain)**. You don't have to credit it, but we list it so it's easy to trace and replace. The voice model, the dictionary, the language model, the typeface and the decoders further down have their own licences, and so do the libraries the game is built with (`../THIRD-PARTY-LICENSES.txt`, written by every build: each one with its licence text).
Downloaded 2026-10-07. Textures were resized and re-encoded (JPG). Fabric, wood, foliage, grass, wall, siding, roof, carpet, wallpaper and painted-wood albedo maps were converted to greyscale "detail" maps, so manifest, house and room colours set the hue.

## Textures (`textures/`)

| Folder | Source asset | Authors | URL | Licence |
|---|---|---|---|---|
| `floor_laminate/` | Laminate Floor 02 | Dario Barresi, Charlotte Baglioni | https://polyhaven.com/a/laminate_floor_02 | CC0 |
| `wall_plaster/` | Painted Plaster Wall | Amal Kumar | https://polyhaven.com/a/painted_plaster_wall | CC0 |
| `road_asphalt/` | Asphalt 02 | Rob Tuytel | https://polyhaven.com/a/asphalt_02 | CC0 |
| `path_hexpaving/` | Hexagonal Concrete Paving | Stephan Seeliger | https://polyhaven.com/a/hexagonal_concrete_paving | CC0 |
| `fabric_linen/` | Rough Linen | colormass, Rico Cilliers | https://polyhaven.com/a/rough_linen | CC0 |
| `wood_oak/` | Oak Veneer 01 | Jenelle van Heerden | https://polyhaven.com/a/oak_veneer_01 | CC0 |
| `bark/` | Bark Brown 02 | Rob Tuytel | https://polyhaven.com/a/bark_brown_02 | CC0 |
| `foliage/` | Forest Leaves 03 | Rob Tuytel, Dimitrios Savva | https://polyhaven.com/a/forest_leaves_03 | CC0 |
| `grass/` | Grass 001 | ambientCG (Lennart Demes) | https://ambientcg.com/view?id=Grass001 | CC0 |
| `siding_wood/` | Wood Siding 009 (greyscale detail, board rows staggered) | ambientCG (Lennart Demes) | https://ambientcg.com/view?id=WoodSiding009 | CC0 |
| `brick/` | Red Brick 03 | Rob Tuytel | https://polyhaven.com/a/red_brick_03 | CC0 |
| `roof_shingles/` | Grey Roof Tiles 02 (greyscale detail) | Rob Tuytel | https://polyhaven.com/a/grey_roof_tiles_02 | CC0 |
| `stone_foundation/` | Rustic Stone Wall 02 | Amal Kumar | https://polyhaven.com/a/rustic_stone_wall_02 | CC0 |
| `floor_wood/` | Wood Floor 040 | ambientCG (Lennart Demes) | https://ambientcg.com/view?id=WoodFloor040 | CC0 |
| `floor_tile/` | Tiles 141 | ambientCG (Lennart Demes) | https://ambientcg.com/view?id=Tiles141 | CC0 |
| `carpet/` | Carpet 012 (greyscale detail) | ambientCG (Lennart Demes) | https://ambientcg.com/view?id=Carpet012 | CC0 |
| `wallpaper/` | Plaster 001 (greyscale detail) | ambientCG (Lennart Demes) | https://ambientcg.com/view?id=Plaster001 | CC0 |
| `painted_wood/` | Painted Wood 008 A (greyscale detail) | ambientCG (Lennart Demes) | https://ambientcg.com/view?id=PaintedWood008A | CC0 |
| `grass_lush/` | Grass 004 (greyscale detail) | ambientCG (Lennart Demes) | https://ambientcg.com/view?id=Grass004 | CC0 |
| `door_wood/` | Wood 066 | ambientCG (Lennart Demes) | https://ambientcg.com/view?id=Wood066 | CC0 |

## Environment (`env/`)

| File | Source asset | Authors | URL | Licence |
|---|---|---|---|---|
| `belfast_open_field.env` | Belfast Open Field (1k HDRI, prefiltered to a Babylon `.env`) | Dimitrios Savva, Jarod Guest | https://polyhaven.com/a/belfast_open_field | CC0 |

## Models

Every model of a thing in the world (furniture, appliances, decor, the town's street furniture and
shops, trees, bushes, flowers and garden things, food and kitchen tools) is built from primitive
parts in the manifests (original, CC0), drawn by `tools/art/furniture.py`, `furniture_prims.py`,
`objects_extra.py`, `catalog.py`, `catalog2.py`, `community.py`, `items.py`, `nature.py` and
`pack_<trait>.py`. They take their surfaces from the finishes and textures above. No downloaded
models are used.

The garden decor (`models/garden/`, override pack `packs/gardenDecor/manifest.json`) is built in Blender by
`tools/art/blender/families/garden_decor.py` on the shared `tools/art/blender/gamekit/` toolkit and exported as `.glb`
(glTF, original, CC0). The other primitive-part entries stay as the fallback.

The kitchen fridge (`models/kitchen/fridge.glb`, override pack `packs/kitchen/manifest.json`) is built in
Blender by `tools/art/blender/families/kitchen.py` on `gamekit` and exported as `.glb` (glTF, original, CC0).
Its primitive-part entry stays as the fallback.

The fold-down wall bed (`models/living/wallBed.glb`, pack `packs/living/manifest.json`) is built in Blender by
`tools/art/blender/families/living.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). A new object:
its pack entry carries a simple primitive-part placeholder as the fallback.

The sofa bed (`models/living/sofaBed.glb`, same pack) is built in Blender by `tools/art/blender/families/living.py`
on `gamekit` and exported as `.glb` (glTF, original, CC0), with a `bed_out` clip (the mattress pulling out) held
while it's open. Its primitive-part entry stays as the fallback.

The bathroom sinks (`models/bathroom/sink.glb`, `sinkPedestal.glb`, `vanityDouble.glb`, `sinkVessel.glb`,
`sinkConsole.glb`, override pack `packs/bathroom/manifest.json`) are built in Blender by
`tools/art/blender/families/bathroom.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). Each has a
`water_on` clip (the tap stream) played while a resident washes. The double vanity keeps its mirror glass as a
primitive `part` (a file can't reflect yet); its cabinet and mirror frames are in the file. The primitive-part
entries stay as the fallback.

The bathroom toilets (`models/bathroom/toilet.glb`, `toiletCompact.glb`, `toiletSmart.glb`, `toiletClassic.glb`,
`toiletWallHung.glb`, override pack `packs/bathroom/manifest.json`) are built in Blender by
`tools/art/blender/families/bathroom.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). Each lifts
its lid (`lid_up`) while a resident uses it. Their primitive-part entries stay as the fallback.

The bathroom showers (`models/bathroom/shower.glb`, `showerStall.glb`, `showerWalkIn.glb`, `showerRain.glb`, override
pack `packs/bathroom/manifest.json`) are built in Blender by `tools/art/blender/families/bathroom.py` on `gamekit` and
exported as `.glb` (glTF, original, CC0). Each rains from its head (`water_on`) while a resident showers. Their
primitive-part entries stay as the fallback.

The bathroom baths (`models/bathroom/bathtub.glb`, `bathtubClawfoot.glb`, `bathtubFreestanding.glb`, override pack
`packs/bathroom/manifest.json`) are built in Blender by `tools/art/blender/families/bathroom.py` on `gamekit` and
exported as `.glb` (glTF, original, CC0). Each runs its tap and fills with water (`tap_on`, `fill`) while a resident
bathes, and drains after. Their primitive-part entries stay as the fallback.

The bookworm pack models (`models/pack_bookworm/readingNook.glb`, `libraryWall.glb`, `writingBureau.glb`,
`teaTrolley.glb`, `firesideLounge.glb`, `bedsideStack.glb`, override pack `packs/pack_bookworm/manifest.json`) are
built in Blender by `tools/art/blender/families/pack_bookworm.py` on `gamekit` and exported as `.glb` (glTF,
original, CC0). Each moves one part while it is used: the nook's cushion sinks, the ladder rolls, the bureau's flap
drops, the teapot pours, the fire grows and the top book opens. Their primitive-part entries stay as the fallback.

The hot-headed pack models (`models/pack_hotHeaded/heavyBag.glb`, `smashBin.glb`, `thunderDrums.glb`,
`coolDownCushion.glb`, `grudgeJournal.glb`, `coldPlunge.glb`, override pack `packs/pack_hotHeaded/manifest.json`) are
built in Blender by `tools/art/blender/families/pack_hotHeaded.py` on `gamekit` and exported as `.glb` (glTF,
original, CC0). Each moves while it is used: the bag swings, the crate's lid flips up, the cymbals rock, the zafu
sinks, the desk drawer slides out and the plunge cover rolls back. Their primitive-part entries stay as the fallback.

The kind pack models (`models/pack_kind/neighbourlyOven.glb`, `givingJar.glb`, `knittingRocker.glb`,
`welcomeTeaTable.glb`, `songbirdFeeder.glb`, `hearthSettee.glb`, override pack `packs/pack_kind/manifest.json`) are
built in Blender by `tools/art/blender/families/pack_kind.py` on `gamekit` and exported as `.glb` (glTF, original,
CC0). Each moves while it is used: the oven door drops, a coin drops into the jar, the rocker rocks, the teapot pours,
the feeder's roof lifts and songbirds fly out, and the settee's cushions sink. Their primitive-part entries stay as the
fallback.

The laundry models (`models/laundry/laundryWasher.glb`, `laundryDryer.glb`, `towelRail.glb`, `towelLadder.glb`,
`laundryBasket.glb`, `bathMat.glb`, override pack `packs/laundry/manifest.json`) are built in Blender by
`tools/art/blender/families/laundry.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). The washer and
the dryer open their door and turn their drum while a resident does the laundry; the rail, ladder, basket and mat
are still. Their primitive-part entries stay as the fallback.

The kitchen farmhouse and double counter sinks (`models/kitchen/farmhouseSink.glb`, `counterSinkDouble.glb`, pack
`packs/kitchen/manifest.json`) are built in Blender by `tools/art/blender/families/kitchen.py` alongside the
fridge, on `gamekit`, and exported as `.glb` (glTF, original, CC0), also with a `water_on` clip. Their
primitive-part entries stay as the fallback.

The other kitchen fridges (`models/kitchen/fridgeMini.glb`, `fridgeBuiltIn.glb`, `fridgeFrenchDoor.glb`,
`fridgeRetro.glb`, `wineFridge.glb`, same pack) are built in Blender by `tools/art/blender/families/kitchen.py`
on `gamekit` and exported as `.glb` (glTF, original, CC0). Each hinges a door built closed and swings it open
while a resident takes food or a drink, held for a few seconds; the French-door freezer and the wine fridge's
top rack slide, and the built-in freezer door and French freezer drawer carry clips no use plays yet. Their
primitive-part entries stay as the fallback.

The kitchen cookers (`models/kitchen_cooking/stoveElectric.glb`, `stoveGas.glb`, `wallOven.glb`,
`rangeCooker.glb`, `cooktopCounter.glb`, `microwave.glb`, override pack `packs/kitchen_cooking/manifest.json`)
are built in Blender by `tools/art/blender/families/kitchen_cooking.py` on `gamekit` and exported as `.glb`
(glTF, original, CC0). Each drops an oven door on its bottom hinge while a resident bakes and turns a knob
while they cook, with a ring of blue flame growing on the gas ones; the oven tower's tray slides out, and the
microwave swings its door and turns its turntable. Their primitive-part entries stay as the fallback.

The kitchen small appliances and dishwashers (`models/kitchen_cooking/coffeeMachine.glb`,
`breakfastCounter.glb`, `dishwasher.glb`, `dishwasherCompact.glb`, same pack) are built in Blender by
`tools/art/blender/families/kitchen_cooking.py` on `gamekit` and exported as `.glb` (glTF, original, CC0).
The coffee machine slides its cup under the spout and runs a coffee stream; the breakfast counter's toaster
lever slides and its toast pops, and its blender lid lifts; the dishwashers drop their door and slide a rack
out. Their primitive-part entries stay as the fallback.

The kitchen base counters (`models/kitchen_counters/counter.glb`, `counterCorner.glb`, `counterEnd.glb`,
`counterCornerOuter.glb`, `counterDrawers.glb`, `counterOpenShelf.glb`, `butcherBlock.glb`,
`counterMarble.glb`, override pack `packs/kitchen_counters/manifest.json`) are built in Blender by
`tools/art/blender/families/kitchen_counters.py` on `gamekit` and exported as `.glb` (glTF, original,
CC0). The plain, corner, marble and butcher-block counters slide their top (or knife) drawer out 25 cm
while a resident makes a sandwich; the others are static base units with doors, drawers or open shelves.
Their primitive-part entries stay as the fallback.

## Generated art (original, CC0)

| File | What | How |
|---|---|---|
| `textures/sky/clouds.png` | Cloud band of the sky | `tools/art/clouds.py` (procedural noise) |
| `icons/rotate.svg`, `wall.svg`, `door.svg`, `eraser.svg`, `eyedropper.svg`, `environment.svg`, `fence.svg`, `gate.svg`, `roof.svg`, `upgrade.svg`, `focused.svg`, `inspired.svg`, `energized.svg`, `relaxed.svg` | UI and emotion icons | Hand-written SVG in the style of the existing set |

## Characters (`characters/`)

The residents are generated in code (original, CC0): their bodies at every life stage, faces and
their expressions and visemes, eyes, hair, beards and clothes by `tools/characters/prim`
(`build.py` writes `rig.json` and the `.bin` files), and their animation clips by
`render/babylon/characters/clips` (baked when the game loads). Bed blankets are generated at
runtime in `render/babylon/characters/blanket.ts`. No downloaded models, textures or animations
are used. The facial expressions (`MOODS` in `render/babylon/characters/face.ts`) and the idle
micro-expressions (`MICRO`) are adapted from the expressions of TalkingHead by Mika Suominen (MIT,
https://github.com/met4citizen/TalkingHead).

## Street and gardens (`street/`)

The neighbourhood dressing (`render/babylon/street.ts`: sidewalks, kerbs, streetlights, mailboxes, hydrants, bins, cars, picket fences, flower beds; neighbour porches, chimneys and garages in `house.ts`) is built from primitives in code (original, CC0). The hedge and flower-clump models are primitive placeholders in `street/manifest.json`. Only the sidewalk surface is third-party:

| File | Used for | Source asset | Authors | URL | Licence |
|---|---|---|---|---|---|
| `street/sidewalk/albedo.webp`, `normal.webp`, `arm.webp` | `material.sidewalk` | Concrete Pavement (1k; albedo made a greyscale detail map, resized to 512 px, ARM to 256 px, WebP) | Charlotte Baglioni | https://polyhaven.com/a/concrete_pavement | CC0 |

## Resident voices (`../voice/`, fetched by `tools/voice/fetch.mjs`)

Unlike the art above, these come with their own licences: the model under the **BigScience OpenRAIL-M** licence (with use restrictions), the dictionary under **Apache-2.0**. They are downloaded at build time, pinned and checked by SHA-256, and are not stored in git.

| File | Source | Authors | URL | Licence |
|---|---|---|---|---|
| `supertonic-3/*.onnxz` | Supertonic 3 (revision `3cadd1ee`): duration predictor, text encoder, vector estimator and vocoder, **modified**: every float32 weight of 1,024 values or more stored as float16 and cast back to float32 when the model loads (`tools/voice/fp16.mjs`), then gzipped; nothing retrained (`supertonic-3/NOTICE`) | Supertone Inc. | https://huggingface.co/Supertone/supertonic-3, https://github.com/supertone-inc/supertonic | BigScience OpenRAIL-M (`supertonic-3/LICENSE`) |
| `supertonic-3/tts.json`, `unicode_indexer.json`, `F1.json`…`F5.json`, `M1.json`…`M5.json` | Supertonic 3 (revision `3cadd1ee`): config, text indexer and ten voice styles, **unchanged** | Supertone Inc. | https://huggingface.co/Supertone/supertonic-3 | BigScience OpenRAIL-M (`supertonic-3/LICENSE`) |
| `en-us.lexz` | Misaki US English dictionaries (`us_gold`, `us_silver`), merged and gzipped | hexgrad | https://github.com/hexgrad/misaki | Apache-2.0 (`misaki-LICENSE`) |

**Use restrictions.** Supertonic 3's licence (`supertonic-3/LICENSE`, shipped with it) forbids certain uses of the model and of what it makes. Among them: breaking the law; exploiting or harming minors; spreading verifiably false information to harm others, or personal information that can be used to harm someone; passing machine-made content off as human without saying it is machine-made; defaming, disparaging or harassing others; impersonating people without their consent; and discriminating against or harming people for their social behaviour, personal characteristics or legally protected categories. The full list is the licence's Attachment A. Anyone who gets the game with the model is bound by the same restrictions; the in-game Credits say so and link the licence. The voice names the game shows (Ada, Bea, Cleo, Dora, Elsie, Arlo, Ben, Cal, Dex, Ezra) are ours; the model calls its voices F1–F5 and M1–M5.

Our own code reads the text plain for the model and phonemizes it only for the mouth shapes: `crates/voice/src/g2p.rs` ports the English rules of Misaki (Apache-2.0); no espeak-ng code is used. Pitch, size and the 24 kHz resampling are done around the model in `crates/voice/src/dsp.rs` and `resample.rs` (our own TD-PSOLA and resampler); the model's starting noise comes from our own hash in `web/src/voice/supertonic.ts`.

The model runs with **ONNX Runtime Web** 1.30.0 (`onnxruntime-web`, its WebGPU build `ort-wasm-simd-threaded.asyncify.wasm`, bundled with the game), © Microsoft Corporation, MIT licence: https://github.com/microsoft/onnxruntime.

## The language model for residents' lines

Optional (Settings › Voices › What residents say › Lines › Language model), its 4-bit ONNX build, unchanged; the weights file is shipped gzipped and unpacked by the player's browser. Fetched at build time into `../llm/` (pinned and checked by SHA-256, not stored in git). Like the voices, it runs only on WebGPU.

| Model | Files | By | Source | Licence |
|---|---|---|---|---|
| LFM2.5-350M | `lfm2.5-350m/model_q4.onnx`, `model_q4.onnx_dataz` (`model_q4.onnx_data` gzipped, unpacked by the browser), `tokenizer.json`, `tokenizer_config.json` (revision `7dd49995`) | Liquid AI (model), onnx-community (export) | https://huggingface.co/onnx-community/LFM2.5-350M-ONNX | LFM Open License v1.0 (`lfm2.5-350m/LICENSE`, from Liquid AI's https://huggingface.co/LiquidAI/LFM2.5-350M at revision `9e6c6ccf`: the export has no copy) |

**The LFM Open License v1.0** allows reproducing, changing and distributing the model free of charge, on conditions that include giving every recipient a copy of the licence (shipped with the model as `lfm2.5-350m/LICENSE`). Commercial use by an organisation whose annual revenue is 10 million US dollars or more isn't licensed (its section 5).

The tokenizer runs with **Tokenizers.js** (`@huggingface/tokenizers`), © Hugging Face, Apache-2.0: https://github.com/huggingface/tokenizers.js. The model runs with ONNX Runtime Web's WebGPU build (`ort-wasm-simd-threaded.asyncify.wasm`).

## Typeface

**Inter** by Rasmus Andersson and the Inter Project Authors (`@fontsource-variable/inter`), SIL Open Font License 1.1: its licence ships next to the font files as `inter-LICENSE.txt`.

## Decoders (`../decoders/`, refreshed by `tools/assets/decoders.mjs`)

Production builds ship compressed copies of the assets above (KTX2 textures, meshopt models, made by `tools/assets/optimize.mjs`; the art itself is unchanged). The game decodes them with these files, served from its own folder instead of Babylon's CDN:

| File(s) | What | Source | Authors | Licence |
|---|---|---|---|---|
| `babylon.ktx2Decoder.js`, `ktx2Transcoders/1/uastc_*.wasm` | KTX2 texture decoder and UASTC transcoders | Babylon.js 9.30.0 (cdn.babylonjs.com) | Babylon.js contributors | Apache-2.0 |
| `ktx2Transcoders/1/msc_basis_transcoder.js`, `.wasm` | Basis Universal transcoder | Basis Universal, built by Babylon.js | Binomial LLC | Apache-2.0 |
| `zstddec.wasm` | Zstandard decompression (UASTC KTX2) | zstddec, built by Babylon.js | Don McCurdy; Zstandard by Meta | MIT; BSD-3-Clause |
| `meshopt_decoder.js` | meshopt geometry decoder | meshoptimizer 1.3.0 | Arseny Kapoulkine | MIT |

The build tools that make the compressed copies are not shipped: the Basis Universal encoder (Apache-2.0, through ktx2-encoder, MIT), glTF-Transform (MIT), meshoptimizer (MIT) and sharp (Apache-2.0, with libvips, LGPL-3.0).
