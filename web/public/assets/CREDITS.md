# Third-party assets

All of the third-party art below is **CC0 1.0 (public domain)**. You don't have to credit it, but we list it so it's easy to trace and replace.
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

## Generated art (original, CC0)

| File | What | How |
|---|---|---|
| `textures/sky/clouds.png` | Cloud band of the sky | `tools/art/clouds.py` (procedural noise) |
| `icons/rotate.svg`, `wall.svg`, `door.svg`, `eraser.svg`, `eyedropper.svg`, `environment.svg`, `fence.svg`, `gate.svg`, `roof.svg`, `upgrade.svg`, `focused.svg`, `inspired.svg`, `energized.svg`, `relaxed.svg` | UI and emotion icons | Hand-written SVG in the style of the existing set |

## Characters (`characters/`)

Sims' bodies and faces are made with **MakeHuman** (MPFB 2.0.17 for Blender 5.2.2 LTS) from the CC0 MakeHuman assets, generated 2026-10-10 with `tools/characters/mpfb/make_bodies.py` (real human proportions from MakeHuman's anthropometric averages, with no stylisation; women and men as the gender macro shapes them, plus small facial cues; every life stage from baby to elder from MakeHuman's age macro; MakeHuman's `game_engine` rig; eyes, eyebrows, eyelashes, teeth, tongue, eight hairstyles and MakeHuman's clothes, all fitted by MPFB to every life stage; MakeHuman's young, middle-aged and old skins; the ARKit face units and Meta visemes as face morphs) and converted with `tools/characters/build_mpfb.py` and `clothes.py` (meshes mirrored into Babylon's space and re-packed; the rig mapped onto the clip skeleton; eye bones for the gaze; the face morphs packed sparsely; the suits split into tops and bottoms, with the jeans also cut off as men's shorts; the clothes' textures packed into one atlas as greyscale detail tinted per resident, MakeHuman logo prints left plain; the painted stubble cleaned off the skins' scalps; the middle-aged and old skins' wrinkles laid over the young skin's colour). Hairstyles and animations come from the free **Standard** editions of three Quaternius packs (all **CC0 1.0**, licence files included in the downloads), downloaded 2026-10-08 and first converted with `tools/characters/build.py` (hairstyles with greyscale strand textures re-baked from the source hair textures so the creator's hair colour tints them, then refitted to the MakeHuman heads; animations kept as rotations and retargeted at runtime). The generated parts (eye bones, the atlases and cleaned skins derived from the CC0 sources, strand textures) are original, CC0. The facial expressions (`MOODS` in `render/babylon/characters/face.ts`) and the idle micro-expressions are adapted from the expressions of TalkingHead by Mika Suominen (MIT, https://github.com/met4citizen/TalkingHead). Bed blankets are generated at runtime in `render/babylon/characters/blanket.ts` (original, CC0).

| File(s) | Used for | Source asset | Author | URL | Licence |
|---|---|---|---|---|---|
| `male.bin`, `female.bin`, `*.baby/child/teen/adult/elder.bin` (bodies at each life stage, clothes, face morphs), `male_albedo.jpg`, `female_albedo.jpg` (young light-skinned male / female skins), `*.adult_albedo.jpg`, `*.elder_albedo.jpg` (with the wrinkles of the middle-aged and old light-skinned skins), `male_brows.png`, `female_brows.png` (eyebrow012, eyebrow003), `eye_albedo.jpg` (brownlight eye, tinted per resident), `lashes.png` (eyelashes01), `teeth.jpg`, `tongue.jpg`, `hair_*.png`, the low-detail bodies in `male.bin` / `female.bin` (proxies female1605, male1591), the hairstyles in `*.hair.*.bin` (afro01, bob02, braid01, ponytail01, short01–04, as greyscale strands) | `model.sim` bodies, faces, eyes, brows, lashes, teeth, tongue; hairstyles Afro, Bob, Braid, Ponytail, Crop, Tousled, Side-swept, Layered | MakeHuman base mesh, `game_engine` rig and system assets (`makehuman_system_assets_cc0`) | MakeHuman Community (Data Collection AB, Joel Palmius, Jonas Hauquier) | https://static.makehumancommunity.org/assets/assetpacks.html | CC0 |
| clothes in `male.bin`, `female.bin` and the stage files, `cloth_albedo.jpg`, `cloth_normal.jpg` (female_casualsuit01/02, female_sportsuit01, female_elegantsuit01, male_casualsuit01–06, male_elegantsuit01, shoes01/03/05/06) | Tops, bottoms and shoes (`top.*`, `bottom.*`, `shoes.*`) | MakeHuman system assets (`makehuman_system_assets_cc0`): clothes | MakeHuman Community (Data Collection AB, Joel Palmius, Jonas Hauquier) | https://static.makehumancommunity.org/assets/assetpacks.html | CC0 |
| face morphs in `male.bin`, `female.bin` | Expressions, blinks, gaze-following lids | Faceunits 01 (ARKit face units) | Mika Suominen | https://static.makehumancommunity.org/assets/assetpacks/faceunits01.html | CC0 |
| face morphs in `male.bin`, `female.bin` | Lip sync | Visemes 02 (Meta visemes) | Mika Suominen | https://static.makehumancommunity.org/assets/assetpacks/visemes02.html | CC0 |
| hair parts in `male.bin`, `female.bin`, `hair*_*.jpg` | `model.hair.*` (Simple Parted, Long, Buns, Buzzed, beard) | Universal Base Characters [Standard] | Quaternius | https://quaternius.itch.io/universal-base-characters | CC0 |
| `anims.bin` (idle, talk, walk, jog, sit, sit-talk, sit down / stand up, interact, pick up, dance, punches, hit, kneel, crouch, fall, push, simple-spell idle as "hands busy") | Character animation | Universal Animation Library [Standard] | Quaternius | https://quaternius.itch.io/universal-animation-library | CC0 |
| `anims.bin` (eat/drink, watering, harvest, plant seed, folded arms, no, yes, lie-to-idle, phone, lantern idle as "holding a book", chest open, rail lean, tree chopping) | Character animation | Universal Animation Library 2 [Standard] | Quaternius | https://quaternius.itch.io/universal-animation-library-2 | CC0 |

## Street and gardens (`street/`)

The neighbourhood dressing (`render/babylon/street.ts`: sidewalks, kerbs, streetlights, mailboxes, hydrants, bins, cars, picket fences, flower beds; neighbour porches, chimneys and garages in `house.ts`) is built from primitives in code (original, CC0). The hedge and flower-clump models are primitive placeholders in `street/manifest.json`. Only the sidewalk surface is third-party:

| File | Used for | Source asset | Authors | URL | Licence |
|---|---|---|---|---|---|
| `street/sidewalk/albedo.webp`, `normal.webp`, `arm.webp` | `material.sidewalk` | Concrete Pavement (1k; albedo made a greyscale detail map, resized to 512 px, ARM to 256 px, WebP) | Charlotte Baglioni | https://polyhaven.com/a/concrete_pavement | CC0 |

## Resident voices (`../voice/`, fetched by `tools/voice/fetch.mjs`)

Unlike the art above, these are **Apache-2.0** (attribution required). They are downloaded at build time, pinned and checked by SHA-256, and are not stored in git.

| File | Source | Authors | URL | Licence |
|---|---|---|---|---|
| `kitten-nano-0.8-edit2.onnx`, `kitten-nano-0.8-voices.f32` | KittenTTS nano 0.8 (fp32 ONNX `kitten_tts_nano_v0_8.onnx` and `voices.npz`, revision `7a1db645`); **modified** (see below) | KittenML (Stellon Labs) | https://huggingface.co/KittenML/kitten-tts-nano-0.8-fp32, https://github.com/KittenML/KittenTTS | Apache-2.0 |
| `en-us.lexz` | Misaki US English dictionaries (`us_gold`, `us_silver`), merged and gzipped | hexgrad | https://github.com/hexgrad/misaki | Apache-2.0 |

**Changes to KittenTTS nano 0.8:** `kitten-nano-0.8-edit2.onnx` is the upstream `kitten_tts_nano_v0_8.onnx` changed by `tools/voice/model.mjs`: the large float weights of its convolutions, matrix products and LSTMs are stored as int8 with a float scale per output channel (`DequantizeLinear` turns them back into floats, so the arithmetic stays float but the values are rounded); an added `pitch` input multiplies the predicted pitch curve; and its two random-number nodes (`RandomNormalLike`, `RandomUniformLike`) are replaced by a deterministic hash of each value's position. `kitten-nano-0.8-voices.f32` holds the first 128 style rows of its eight voices from `voices.npz`, as raw floats, unchanged. KittenTTS's input (espeak-style phonemes and its symbol table) is rebuilt in `crates/voice/src/kitten.rs` from our phonemizer; its own code and espeak-ng are not used.

The phonemizer in `crates/voice/src/g2p.rs` ports the English rules of Misaki (Apache-2.0); no espeak-ng code is used.

The model runs with **ONNX Runtime Web** 1.30.0 (`onnxruntime-web`, its WebAssembly build `ort-wasm-simd-threaded.wasm`, bundled with the game), © Microsoft Corporation, MIT licence: https://github.com/microsoft/onnxruntime.

## Decoders (`../decoders/`, refreshed by `tools/assets/decoders.mjs`)

Production builds ship compressed copies of the assets above (KTX2 textures, meshopt models, made by `tools/assets/optimize.mjs`; the art itself is unchanged). The game decodes them with these files, served from its own folder instead of Babylon's CDN:

| File(s) | What | Source | Authors | Licence |
|---|---|---|---|---|
| `babylon.ktx2Decoder.js`, `ktx2Transcoders/1/uastc_*.wasm` | KTX2 texture decoder and UASTC transcoders | Babylon.js 9.29.0 (cdn.babylonjs.com) | Babylon.js contributors | Apache-2.0 |
| `ktx2Transcoders/1/msc_basis_transcoder.js`, `.wasm` | Basis Universal transcoder | Basis Universal, built by Babylon.js | Binomial LLC | Apache-2.0 |
| `zstddec.wasm` | Zstandard decompression (UASTC KTX2) | zstddec, built by Babylon.js | Don McCurdy; Zstandard by Meta | MIT; BSD-3-Clause |
| `meshopt_decoder.js` | meshopt geometry decoder | meshoptimizer 1.3.0 | Arseny Kapoulkine | MIT |

The build tools that make the compressed copies are not shipped: the Basis Universal encoder (Apache-2.0, through ktx2-encoder, MIT), glTF-Transform (MIT), meshoptimizer (MIT) and sharp (Apache-2.0, with libvips, LGPL-3.0).
