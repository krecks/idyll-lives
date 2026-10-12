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

Unlike the art above, these are **CC BY 4.0** and **Apache-2.0** (attribution required), and CC0. They are downloaded at build time, pinned and checked by SHA-256, and are not stored in git.

| File | Source | Authors | URL | Licence |
|---|---|---|---|---|
| `pocket-en-2026-04/*.onnx`, `bundle.json`, `bos_before_voice.npy` | Pocket TTS, English bundle `english_2026-04`, ONNX export (int8: `flow_lm_main_int8`, `flow_lm_flow_int8`, `mimi_decoder_int8`, `mimi_encoder_int8`, `text_conditioner_int8`; revision `58a6d00c`), **unchanged** | Kyutai (model); KevinAHM (ONNX export) | https://huggingface.co/kyutai/pocket-tts, https://huggingface.co/KevinAHM/pocket-tts-onnx, https://github.com/kyutai-labs/pocket-tts | CC BY 4.0 |
| `pocket-en-2026-04/tokenizer.json` | Pocket TTS's English tokenizer (revision `575d1437`), unchanged | Kyutai | https://huggingface.co/kyutai/pocket-tts-without-voice-cloning | CC BY 4.0 |
| `presets-2.pcm`, `presets-2.json` | 24 voice clips from Kyutai's voice repository (revision `323332d3`), each **modified**: resampled to 24 kHz, the silence around it trimmed, cut to 10 s, its loudness evened out (`crates/voice/src/dsp.rs`, `clip`), packed as 16-bit samples | see below | https://huggingface.co/kyutai/tts-voices | CC BY 4.0, CC0 |
| `en-us.lexz` | Misaki US English dictionaries (`us_gold`, `us_silver`), merged and gzipped | hexgrad | https://github.com/hexgrad/misaki | Apache-2.0 |

**The bundled voices** (names as the game shows them; the list is `PRESETS` in `web/src/voice/voices.ts`):

- From the **VCTK corpus** (CSTR Voice Cloning Toolkit, Centre for Speech Technology Research, University of Edinburgh; https://datashare.ed.ac.uk/handle/10283/3443), CC BY 4.0, sentence 23 of each speaker as enhanced by Kyutai: Anna (p228), Azelma (p303), Eponine (p262), Eve (p361), Fantine (p244), Iris (p225), Jane (p339), Lucy (p253), Mary (p333), Nora (p236), Vera (p229), Charles (p254), Felix (p243), George (p315), Hugo (p232), Leon (p251), Michael (p360), Oscar (p270), Owen (p226), Paul (p259).
- From **Voice-Zero** (a selection of LibriVox recordings, https://github.com/OwenTyme/voice-zero), CC0: Caro (Caro Davy), Bill (Bill Boerst), Peter (Peter Yearsley), Stuart (Stuart Bell).

**Players' own voices.** A voice a player records (or loads from a sound file) in the household creator stays in their browser and in their saves; Pocket TTS speaks like it. Kyutai's terms forbid cloning a voice without its owner's explicit and lawful consent, so the creator asks for the player's own voice or that of someone who agreed.

Our own code reads the text plain for the model and phonemizes it only for the mouth shapes: `crates/voice/src/g2p.rs` ports the English rules of Misaki (Apache-2.0); no espeak-ng code is used. Pitch, size and speed are shaped around the model in `crates/voice/src/dsp.rs` (our own TD-PSOLA); the model's noise comes from our own hash in `web/src/voice/pocket.ts`.

The model runs with **ONNX Runtime Web** 1.30.0 (`onnxruntime-web`, its WebAssembly build `ort-wasm-simd-threaded.wasm`, bundled with the game), © Microsoft Corporation, MIT licence: https://github.com/microsoft/onnxruntime.

## Language models for residents' lines

Optional (Settings → Audio → What residents say → Language model), downloaded by the player's browser only when chosen, each as its 4-bit ONNX build, unchanged. The default is fetched at build time into `../llm/` (pinned and checked by SHA-256, not stored in git); the others download from Hugging Face at a pinned revision, checked by SHA-256. Models a player adds from Hugging Face keep their own licences.

| Model | Files | By | Source | Licence |
|---|---|---|---|---|
| LFM2.5-350M (default) | `lfm2.5-350m/model_q4.onnx`, `model_q4.onnx_data`, `tokenizer.json`, `tokenizer_config.json` (revision `7dd49995`) | Liquid AI (model), onnx-community (export) | https://huggingface.co/onnx-community/LFM2.5-350M-ONNX | LFM Open License v1.0 |
| SmolLM2-360M-Instruct | `onnx/model_q4.onnx`, tokenizer (revision `a10cc151`), from Hugging Face | Hugging Face | https://huggingface.co/HuggingFaceTB/SmolLM2-360M-Instruct | Apache-2.0 |
| Gemma 3 270M (instruction-tuned) | `onnx/model_q4.onnx`, `model_q4.onnx_data`, tokenizer (revision `2dbbfdb1`), from Hugging Face | Google (model), onnx-community (export) | https://huggingface.co/onnx-community/gemma-3-270m-it-ONNX | Gemma Terms of Use |

The tokenizer runs with **Tokenizers.js** (`@huggingface/tokenizers`), © Hugging Face, Apache-2.0: https://github.com/huggingface/tokenizers.js. On the GPU the model runs with ONNX Runtime Web's WebGPU build (`ort-wasm-simd-threaded.asyncify.wasm`).

## Decoders (`../decoders/`, refreshed by `tools/assets/decoders.mjs`)

Production builds ship compressed copies of the assets above (KTX2 textures, meshopt models, made by `tools/assets/optimize.mjs`; the art itself is unchanged). The game decodes them with these files, served from its own folder instead of Babylon's CDN:

| File(s) | What | Source | Authors | Licence |
|---|---|---|---|---|
| `babylon.ktx2Decoder.js`, `ktx2Transcoders/1/uastc_*.wasm` | KTX2 texture decoder and UASTC transcoders | Babylon.js 9.29.0 (cdn.babylonjs.com) | Babylon.js contributors | Apache-2.0 |
| `ktx2Transcoders/1/msc_basis_transcoder.js`, `.wasm` | Basis Universal transcoder | Basis Universal, built by Babylon.js | Binomial LLC | Apache-2.0 |
| `zstddec.wasm` | Zstandard decompression (UASTC KTX2) | zstddec, built by Babylon.js | Don McCurdy; Zstandard by Meta | MIT; BSD-3-Clause |
| `meshopt_decoder.js` | meshopt geometry decoder | meshoptimizer 1.3.0 | Arseny Kapoulkine | MIT |

The build tools that make the compressed copies are not shipped: the Basis Universal encoder (Apache-2.0, through ktx2-encoder, MIT), glTF-Transform (MIT), meshoptimizer (MIT) and sharp (Apache-2.0, with libvips, LGPL-3.0).
