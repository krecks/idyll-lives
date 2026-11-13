# Resident voices

*Plan for residents speaking their thoughts and conversations out loud. Status: residents speak with
**Supertone's Supertonic 3** on the graphics card (WebGPU), or on the processor (WASM) when Auto
steps down: ten bundled voices, each resident with their own pitch, depth and speed (in since
2026-10-12). A voice-engine trap restarts the voice worker (at most 3 times a session), and sound
stops in a hidden tab.*

## Built so far

- **Model (`web/src/voice/supertonic.ts`):** Supertonic 3 (Supertone, BigScience OpenRAIL-M),
  four ONNX graphs with their big weights stored smaller and turned back into 32-bit floats when
  the model loads (ONNX Runtime folds it once), so the GPU does float32 arithmetic as before:
  since 0.7.0 the text encoder, vector estimator and vocoder as 8-bit integers with a scale per
  channel (`crates/tools/src/int8.rs`), the duration predictor as 16-bit floats
  (`crates/tools/src/fp16.rs`; in int8 some lines came out a frame longer or shorter); gzipped
  (`.onnxz`, unpacked in the worker): 87 MB instead of 398 as published (185 MB with 16-bit
  weights everywhere, 0.2.0 to 0.6.x). The 16-bit weights, against the published ones on the GPU:
  a line differs less than it does between two random seeds (spectral match 0.61–0.77, against
  0.55–0.59), with the same timing and the same pitch per voice. The 8-bit ones against the
  16-bit (2026-11-05, on the CPU, 30 lines by 10 residents; log-spectrogram correlation): 0.87–0.96,
  against 0.39–0.79 between two noise seeds of the 16-bit model; pitch shaping alike (×0.85 gives
  a median ×0.853 against ×0.852 over 40 lines and voices); the same speed on the GPU (the device
  test's speech 203–210 ms against 206) and the CPU; and to the owner's ear the same. 4-bit
  doesn't fit: 91 % of the weights are convolutions, which ONNX Runtime has no 4-bit form of on
  WebGPU; and int8 *operators* (the upstream int8 build) run on the CPU, 25 times slower: the
  8-bit weights here never reach an operator. The graphs:
  duration predictor (how long a line lasts), text encoder, vector estimator (flow matching: 8
  passes turn noise into the line's latents) and vocoder (44.1 kHz). A whole line in a few large passes, which suits the GPU. The loop is
  ported from Supertone's reference code (`web/helper.js`). It runs in the voice worker on
  **ONNX Runtime Web** 1.31.1, on WebGPU (the graphics card) or WASM (the processor, Auto's
  fallback). Lines are made one at a time; the Node checks run the same `supertonic.ts` (on the
  CPU).
- **Deterministic:** the starting noise is ours, a hash of the voice, speed and text, so the
  same line in the same voice gives the same samples (bit-identical in the Node check, against
  golden hashes), which the clip cache relies on.
- **Voices (`PRESETS` in `web/src/voice/voices.ts`):** Supertonic's ten voice styles, five
  women's (Ada, Bea, Cleo, Dora, Elsie) and five men's (Arlo, Ben, Cal, Dex, Ezra; the names are
  ours). Speed goes to the model; each resident's own pitch and size go onto every finished line
  (our TD-PSOLA, `crates/voice/src/dsp.rs`). Children: one of the higher women's voices (Ada,
  Bea, Dora, 188–216 Hz) raised to 245–290 Hz and made smaller.
- **Text:** the model reads plain text (`plain_text`, then the reference's clean-up); our Rust
  phonemizer (`crates/voice`, Misaki's rules) only gives the mouth shapes their phonemes. The
  model's 44.1 kHz output is resampled to 24 kHz (`LineInputs::finish`) before trimming and
  shaping, so the mouth shapes and playback are as before.
- **Tests:** `cargo test -p voice --release`; `bun run check:voice` (ONNX Runtime Web in Bun with
  the worker's `supertonic.ts`: bit-identical across runs and sessions, golden hashes, every voice
  speaks and its median pitch, pitch, depth, speed and trimming checked).
- **Measured (M3 Pro, Chrome, 2026-10-18):** lines of 1–5 sentences, spoken a piece at a time:
  the first sound 0.18–0.21 s after asking (6 passes, within a hair of 8; 4 passes: 0.12–0.14 s),
  a whole five-sentence line (10.5 s of audio) in 0.31 s, about 34× faster than it plays; frames
  at 16.7 ms. On the CPU (two threads) a line takes 0.7–1.0 s and runs 2.5× real time: Auto's
  fallback. The device test with LFM2.5: 0.42 s from
  asking for a line to hearing it; speech made in 0.10 of its
  length. The first line after loading takes about 0.2 s more (the GPU sets up its kernels). The
  same test on an M1 Pro (2026-10-18, 8 passes): 0.86 s and 0.24 of its length — the model alone
  writes 96–98 tokens a second (`tools/llm/bench.mjs`; 76–81 with the game and the voices drawing
  on the same GPU).
- **Browsers (M3 Pro, 2026-10-18, the device test):** Chrome/Brave 0.38–0.42 s a line (160–170
  tokens/s); Safari 26.5 0.71 s (114 tokens/s); Firefox 158 3.5–4.5 s (9.7 tokens/s). Firefox's
  WebGPU is much slower for both models here (speech 0.7 s against 0.18 s), so the test turns the
  language model off there and the settings say why; voices still run. Not the computer: the
  same Mac is fast in the other browsers.
- **Download:** 120.6 MB once (the model 87.1, the runtime's WebGPU build 28.7, voices and text
  files 3.2, dictionary 1.3, our WASM 0.16), kept in the browser's cache (packed). The runtime is
  cached once for the voices and the language model (`openRuntimeCache`, `ml/assets.ts`).
- **Lines of 1–5 sentences (the language model's):** each line is asked for a length picked by
  its seed (1: 20 %, 2: 25 %, 3: 25 %, 4: 15 %, 5: 15 %; `lineSentences` in `talk/writer.ts`),
  and the generation keeps to it (`GenParams.sentences` in `llm/generate.ts`): until the last
  sentence is finished only plain speech can be written (no end of turn, quotes, line breaks,
  emoji or other scripts), a full stop joined to a closing quote (`."`) is written as the full
  stop alone, and the line ends with its last sentence. Asked alone, LFM2.5 350M wrote two
  sentences when asked for three to five. 20 of 20 sampled lines had the length asked for, 125–600
  ms each on the GPU.
- **The language model's files (`cargo fetch-llm`, `crates/tools/src/llmslim.rs`):** the
  onnx-community 4-bit export of LFM2.5 350M, repacked without changing what it computes (since
  0.7.0): its tied 4-bit embedding was stored twice, for reading the tokens and for the output
  layer (33.6 MB the same, which gzip can't fold away: its window is 32 KB), and its rotary tables
  held 128,000 positions (32.8 MB) where a prompt and its reply take about 470 + 160. Now the two
  point at the same bytes and the tables keep their first 4,096 rows, which `generate.ts` never
  goes past (`ModelSpec.positions`: a prompt that doesn't fit is refused). Weights 293.6 → 228.4
  MB, the download 254.9 → 195.5 MB. Checked (M3 Pro, 2026-11-05): `check:llm`'s golden text as
  before, the benchmark's five lines word for word, and the same speed (three runs of each in
  turn, medians about 165–175 tokens a second for both).
- **Made ahead (`voice/index.ts`, `setOut`):** the snapshot shows what a resident is walking to
  use (`objectDef`, `interaction`) before the animation starts, so when they set out the dice
  decide whether they'll speak and, if so, the line is written and its speech made on the way;
  arriving plays it from the clip cache at once. A plan given up on the way is forgotten (and a
  line made ahead expires after a minute). The animation isn't known while walking, so an object
  with only `action:` lines is still picked on arrival.
- **Loaded at launch:** the voice model loads when the game starts (while voices are on;
  `voice/service.svelte.ts`), the language model too while residents' lines are its own
  (`talk/talk.svelte.ts`), and both stay loaded; switching them off unloads them.
- **Auto mode (`voice/service.svelte.ts`, `voice/index.ts`):** `voiceDevice` is `auto`, `gpu` or
  `cpu`. Auto starts on the graphics card when WebGPU has an adapter, otherwise on the processor;
  if the card cannot start the voices it tries the processor. When three lines in a row come
  later than 4 s, or making speech slows the frames, Auto unloads the card's engine, starts the
  processor one and says so in a toast (non-blocking); the frame counters get a clean window to
  judge the processor. A manual `gpu` or `cpu` choice is never overridden. The choice is a
  setting, so it persists.
- **Guards (`voice/index.ts`):** nothing new is said while the engine loads; after Auto's step
  down, voices pause for the session (with a toast) if speech still comes too late or slows the
  frames.
- **Not yet:** blending voice styles for more than ten voices.

Residents speak with a text-to-speech model that runs **entirely in the browser**: no server, no
API key, nothing leaves the player's computer.

## Goals and rules

- **Performance first.** Speech must never cost frames. It runs in its own worker on the GPU
  (a line in a few large passes, well under a frame's worth of GPU time per 16 ms), and a
  benchmark in Settings and the device test check that this computer keeps up.
- **Quiet by default.** Residents only speak when:
  1. **a resident is selected** (their Inspect panel is open, `game.inspected`): they speak their
     thoughts, and they and whoever they are talking to speak their conversation; or
  2. **the player started a conversation** (from the social menu): both residents speak that
     conversation until it ends, even if neither is selected.

  Nobody else speaks, including while the camera is watching on its own.
- **Presentation only.** Voices never feed back into the simulation. The line a resident says is
  picked with a hash of the resident id and event, not the sim's random number generator, so
  saves and replays are unaffected and `sim-core` needs no changes.
- **Every resident sounds different**, and always the same, across sessions and saves.
- **Download on request.** The model (121 MB with the runtime and voices) is fetched when the
  player turns voices on, or when they hear a voice in the creator, and kept.

## Shaping a voice (`crates/voice/src/dsp.rs`)

The sound is shaped around the model, all in our Rust WASM (the same samples in every browser):

- **TD-PSOLA (`psola`):** the pitch is found every 10 ms (normalised autocorrelation, 50–500 Hz,
  searched at half the rate and refined, the shortest strong peak taken so a multiple of the
  period isn't, median-smoothed over five frames); the sound is cut into Hann grains two periods
  long, one period apart (5 ms where unvoiced), and laid out again with the grains closer
  (higher) or further apart (lower), each taken from its place in time, so pitch and length
  change independently and the formants stay.
- **`shape(pitch, depth, speed)`:** resample by depth (every frequency, formants included, ×depth),
  then PSOLA puts the pitch at `pitch` and the length at 1/speed. About 50–250 ms for a line.

## Voices

Each resident's voice comes from their **stable resident id**, gender and life stage
(`voiceFor`); a voice chosen in the creator is saved on top (see "A resident's own voice").

**Who speaks with what:** men from the men's voices, women from the women's, other genders from
all, picked by the id. Children of any gender: a woman's voice raised to 245–290 Hz and made smaller (1.05–1.12). Babies don't speak (the creator
plays a baby as the child they will be). On top, from the id: pitch 0.94–1.06 (teens 1.04–1.12),
depth 0.97–1.03 (teens 1.03–1.07), speed 0.88–1.12 (children ×1.04), and the
elders' factors (women's pitch ×0.93 and depth ×0.98, men's ×1.04 and ×1.02, everyone's speed
×0.92).

**Tones** (`withTone`, on the line): happy pitch ×1.06, speed ×1.06; angry ×1.04,
×1.12; sad ×0.94, ×0.9; flirty ×0.97, ×0.94.

### A resident's own voice

The household creator's *Identity* tab has a *Voice* section:

- **A voice:** *Own* (the generated one) or one of the bundled voices that fit the gender (all
  for other genders; picking one plays it).
- **Pitch** (lower to higher), **Speed** (slower to faster), **Depth** (smaller to larger),
  a dice (a fitting voice or a new generated one, and every
  slider anywhere in its range), a button back to the default (own voice,
  factors 1), and **Hear** ("Hi, I'm Ada! This is how I sound." in that voice;
  a baby is heard as the child they will be).

Every control is heard for grown-ups and children alike, except the voice choice for children,
who keep a child's voice and take their chosen voice from their teens; changing the
gender drops a bundled voice of the other sex. Hearing works with resident
voices off in Settings and doesn't turn them on; the engine is unloaded again when the creator
closes.

The choice is stored in the resident's appearance, which the simulation keeps as opaque JSON, so
the save format doesn't change:

```ts
appearance.voice = { pitch: 1.08, speed: 0.95, depth: 0.96, seed: 1439030069, preset: 'Ada' }
```

- **`preset`** (optional): a bundled voice by name, a teen's or grown-up's voice instead of the
  generated one.
- **`pitch`, `depth`, `speed`:** each is a factor on the generated voice (1 or missing: as generated);
  the life stage still applies underneath. Kept within pitch 0.5–1.5, speed 0.8–1.2, depth
  0.78–1.16.
- **`seed`** replaces the resident id in the generated voice: someone made in the creator has no
  id until they move in, and the voice heard there must be the one they keep.
- **`base`** (0.20 to 0.1.0, a KittenTTS voice by index) is ignored: those residents have their
  generated voice.
- Without `voice` (older saves, neighbours, newcomers, babies, random residents from the
  creator's dice) a resident has the generated voice.
- Main-thread rewrites of the appearance keep `voice`: life-stage looks and grey hair copy it
  along; the look expanded from a newcomer's or baby's seed keeps a voice stored next to it.
  Portraits ignore it (their cache key leaves it out).

There is no place to change a resident's look in a game yet; the Voice controls would go there too.

## Phonemizer: our own, in Rust, without espeak-ng

*The phonemizer only gives the mouth shapes their phonemes; a pronunciation mark in a line
(`[read](/ɹˈɛd/)`) shapes the mouth, and the model reads the word.*

**`g2p` inside `crates/voice`:**

1. **Dictionary:** Misaki's English dictionaries (`us_gold`, `us_silver`; Apache-2.0, credited in
   `CREDITS.md`). Ship
   them compressed; trim rare words if size matters.
2. **Text clean-up:** punctuation, contractions, possessive and plural "s", "-ed" and "-ing"
   endings (Misaki's suffix rules), numbers to words.
3. **Unknown words** (mostly residents' names): public-domain letter-to-sound rules (the 1976
   NRL rules), written in Misaki's alphabet. A tiny trained model can replace them later.
4. **Words with two pronunciations** ("read", "live"): content lines can mark the pronunciation
   (Misaki's `[word](/phonemes/)` markup); dynamic text accepts the occasional slip.
5. **Build-time check:** every line in the content is phonemized in a test, and a word that falls
   back to the rules fails it, so hand-written lines always sound right.

## Settings

A **Voices** tab in Settings (`ui/screens/SettingsPanel.svelte`). Everything applies live, so
nothing is added to `RESTART_KEYS`.

| Row | Control | Notes |
|---|---|---|
| Resident voices | Toggle | Default off. |
| Language | (hidden while English is the only one) | Comes back as a row when a second language ships. |
| Runs on | Auto / Graphics card / Processor | Auto uses the graphics card and steps down to the processor when it is missing or too slow. The choice applies live. |
| Voice model | Status | Supertonic 3, its state, where it runs, and the download (about 121 MB). |
| Try it | **Hear a sample** and **Speed test** | The speed test measures the model with the scene drawing: "On the graphics card: 0.4 s per line, 7.6× real time, smooth" (M3 Pro, Chrome; "smooth" needs 1.5 s or less). A result over 3 s per line says voices may pause. |
| Voice volume | Slider | Separate from sound effects. |

New keys in `settings.svelte.ts`:

```ts
voices: boolean;                          // false
voiceLanguage: 'en';                      // 'en'
voiceDevice: 'auto' | 'cpu' | 'gpu';      // 'auto'
voiceVolume: number;                      // 0.8
```

The benchmark result is stored under its own localStorage key (`littlelives.voiceBench`), not in
`Settings`, so "Reset to defaults" does not throw away a measurement.

Turning voices on the first time: download the model (progress in the row), then speak. Until
the model is ready, residents stay quiet. The speed test is optional; Auto does not wait for it,
and the live watchdog steps down if the card turns out too slow while playing.

## Performance

### Where the work runs

| Work | Thread | Cost |
|---|---|---|
| Deciding who speaks, picking lines, playback | Main thread | Microseconds, event-driven (no per-frame work) |
| Supertonic 3 inference, shaping, mouth shapes | **Own module worker** (`voice/tts.worker.ts`) running `voice-wasm`, separate from the sim worker, lines one at a time | About 0.18–0.31 s per line on an M3 Pro's GPU, 0.7–1.0 s on two processor threads; never blocks a frame |

- **Graphics card (WebGPU):** a line in a few large passes, about 34× faster than it plays. It
  shares the GPU with rendering; Auto steps down to the processor if that costs frames.
- **Processor (WASM):** one thread, or two where the page is cross-origin isolated, leaving cores
  for the main thread, the sim worker and the browser. About 2.5× faster than it plays.
- **Auto:** starts on the card when WebGPU has an adapter and on the processor otherwise; a card
  that cannot start the voices or turns out too slow hands over to the processor. Manual `gpu` and
  `cpu` choices stay put.
- Model files are self-hosted (bundled by Vite), following the "self-host all assets" rule. The
  browser cache keeps them.

### Benchmark ("Test this computer")

Runs from Settings, in the main menu (the live 3D town draws behind it, so contention is
realistic) or in a game (speech pauses while it runs). It takes a few seconds.

1. Load the engine on its current device, if needed (progress shown).
2. Say one warm-up line, then three fixed test lines (3, 8 and 12 words).
3. Record per line: time to the finished clip, and **real-time factor** (RTF = generation time ÷
   audio length). Meanwhile measure frame times with `requestAnimationFrame` and keep the worst.
4. Show the median line time, its multiple of real time, and a verdict: **smooth** (≤ 1.5 s a
   line and no frame over 50 ms), **lines may start a moment late** (≤ 3 s), or **too slow here:
   voices may pause**.
5. Store `{ rtf, lineMs, worstFrameMs, cores, model, device, date }` in
   `littlelives.voiceBench`, outside `Settings`. Results from another model or core count are
   dropped.

The test does not choose the device by itself: Auto's live watchdog does, while the game plays.
A player can force **Graphics card** or **Processor** and the result says which one was measured.

### Guards while playing

- At most **two voices at once**, and at most one line per resident every ~8 s.
- A line that is not ready within **2 s** is dropped (the moment has passed).
- No generation while in Build or Creation, while the tab is hidden, or at speed 3× and above
  (except the conversation the player started).
- **Live watchdog:** the scheduler tracks each line's RTF and the renderer's frame times
  (`game.stats`). If three lines in a row are late, or frame time rises past the budget while
  inference runs, Auto steps the engine down from the graphics card to the processor with a
  toast. If the processor still cannot keep up, voices pause for the session with a toast
  (`voice/index.ts`: three lines in a row later than 4 s, or slowed frames). A manual device
  choice goes straight to the pause.
- **Memory:** the worker is unloaded when voices are turned
  off. Finished clips are kept in an in-memory LRU cache
  (24 MB), keyed by model, device, voice and text. Because the engine is deterministic, a cached
  clip is exactly what would be generated again.
- The performance overlay (F3) shows a voice line: model, mode, last RTF, queue length, cache
  hits.

## What residents say

Speech needs text: **hand-written lines as
content data** (`content/voice/<lang>.json`), so content packs can add and translate them. The
English file has about 1,000 lines, and the trait packs about 500 more for their own items.

- **Conversations** (`social`, per social interaction id): `start` for the one who starts it,
  `good` and `bad` for the answer once the outcome shows (halfway through; at once for a hug or a
  kiss, which the body answers). Read from the snapshot fields `social`,
  `role`, `outcome` and `partner` that the bubbles already use.
- **Using something:** a line when the selected resident starts using an object. The snapshot
  says which (`objectDef` and `interaction`, content indices, next to the animation tag in
  `action`), so lines can be per item. The most specific lines win:
  1. `object.<object id>.interactions.<interaction id>` (the gourmet meal at the fridge);
  2. `object.<object id>.lines` (anything at the treadmill);
  3. `interaction.<interaction id>` on any object (`water`, `smell`, `homework`);
  4. `action.<animation tag>` (`cook`, `run`), which also covers repairs, accidents, knocking
     and tidying up, where no object interaction is in use.
- **Thoughts of the selected resident:** when their emotion changes (`emotion`, per emotion id)
  and when the planner shows a thought (`thought`, per thought kind: `skipped`, `noPlace`,
  `kept`, `goal`, `roomLoved`, `roomDisliked`, `broken`, `accident`, `crying`).
- **Later sources plug into the same pipeline:** the optional in-browser language model for
  thoughts (see `docs/design/smarter-thoughts.md`; measured 2026-11-01: describing the utility
  AI's choice needs no shortlist, so it can ride on this pipeline as it is, while the re-rank the
  note asks about is a separate, more expensive question).

A group of lines is a list, or `{"lines": [...], "tone": ...}`; an object's `tone` applies to its
`lines`. Tones (`happy`, `sad`, `angry`, `flirty`, `question`) adjust speed and pitch. `{name}` is
the other resident's first name, `{me}` the speaker's; `[word](/phonemes/)` fixes a
pronunciation (`[read](/ɹˈɛd/)` in the past tense).

**Variation:** the line is picked with a hash of the resident and a counter, never the sim's
random numbers. A resident doesn't say any of their last three lines for the same key again
while there are others (`voice/lines.ts`), so 4–6 lines per key go a long way.

**Writing lines:** short (most under 40 characters; speech is made per line), and right for
children, teens and grown-ups alike (babies don't speak). `cargo test -p voice --release --test
lines` phonemizes every English line, in the language file and in the packs, and fails on a word
missing from the dictionary (fix it with a pronunciation or another word), on symbols the model
doesn't know, on unknown placeholders or tones, and on keys that name no social, object,
interaction, animation tag, emotion or thought kind. Words with accents (`café`, `Zoë`) are read
as without them.

**Packs** carry lines under a top-level `"voice": {"en": {...}}`, laid out like the language
file; the web side merges them over it key by key (sim-core ignores `voice`). See
`docs/content-packs.md`.

## Playback

- One shared `AudioContext` with sound effects, with
  separate gain nodes for effects and voices.
- Every line is made at one loudness: its last step (`crates/voice/src/dsp.rs` `level`, after the
  shaping and the mouth shapes) reads how loud the clip is to the ear (LUFS, ITU-R BS.1770 at
  24 kHz, gated so its pauses don't count) and turns the whole clip up or down to -25 LUFS, by one
  gain (12 dB at most, and up only as far as its peak stays under 0.95), so its length, its pauses
  and its mouth shapes are untouched. As the model makes them, 110 lines (its ten voices, ten
  residents' shaped voices) were between -28.8 and -21.1 LUFS, "Hi!" and "Ouch!" the loudest.
- Each clip plays through a `StereoPannerNode` (`voice/player.ts`, `audio/space.ts` `Placed`): left
  to right by where the resident's head is on screen (`renderer.simHead` projected, 0.7 of the way
  to either side at the screen's edges), updated about 10 times a second, so a voice comes from
  where the resident stands in the picture. There is no 3D panner and no distance: a voice is as
  loud from far as from near. It takes the reverb of the room the resident stands in (a send into
  the room's convolver; dry outdoors) and, with the camera closed in on another room, the wall's
  muffling, as the world's sounds do (`docs/design/sound.md`).
- The speech bubble appears straight away; the voice follows when its clip is ready.
- **Lip sync.** Every clip comes with a viseme track (`crates/voice/src/visemes.rs`), made in Rust
  together with the audio. The model gives no durations, so the phonemes are mapped to the 15
  Meta visemes and weighted by kind (vowels longer than plosives, diphthongs split in two). They
  are spread over the clip's voiced span, and pauses are snapped to the gaps in its loudness.
  Loudness, at 100 values a second, scales how far the mouth opens. The director passes the track
  to the renderer when the clip starts (`setSpeech`). The face mixes the visemes into the
  resident's face morphs a little ahead of the sound, blending each shape in over about 70 ms.
  Talk without a voice line uses made-up syllables from the same mouth shapes.

## Modules

Rust: `crates/voice` (engine, phonemizer, DSP) and `crates/voice-wasm` (the `wasm-bindgen`
wrapper), built by a `bun run voice` script next to the existing `wasm` script.

Web code lives in `web/src/voice/`, behind one small interface used by the game session:

| File | Job |
|---|---|
| `voice/index.ts` | `VoiceDirector`: watches the frame state and UI state, applies the "who may speak" rules and guards, queues lines. The only thing `game/session.ts` talks to. |
| `voice/lines.ts` | Picks a line for an event from content (hash-based). |
| `voice/voices.ts` | The bundled voices (`PRESETS`), and each resident's pitch, depth and speed from id, gender, life stage and the voice chosen in the creator. |
| `voice/service.svelte.ts` + `voice/tts.worker.ts` | Worker client and worker: load the model, get a voice ready, `speak(text, voice) → Float32Array` (transferred, 24 kHz), clip cache, speed test, unload. |
| `voice/player.ts` | Plays a clip on the voice bus at its place (panner, room, wall). |

## Research: other models (2026-11-05)

Looked at as replacements; none taken. **Language model: MNN-Pocket-0.3B** (Alibaba's MNN team,
Apache-2.0, Qwen3 layout with 28 layers, English and Chinese; its own card puts it at IFEval 73
against LFM2.5's 72). No browser build is published: its 4-bit build is in MNN's own format, and
the community ONNX export (`liodon-ai/MNN-Pocket-0.3B-ONNX`) leaves every operator separate, so
on WebGPU (`tools/llm/bench.mjs`, M3 Pro) its fp16 build (769 MB) wrote 14 tokens a second and
its dynamic int8 build (364 MB, int8 operators on the CPU) 6.3, against LFM2.5's 187. Built as
4-bit for WebGPU (ONNX Runtime GenAI's builder) it was 176 MB and wrote 158 tokens a second, but
asked for five sentences it ran into `!!!!.....` to the token limit, in fp16 and 4-bit alike.
**Voices:** OmniVoice (k2-fsa) has non-commercial weights (CC-BY-NC) and is 3.3 GB;
Audio8-TTS-Realistic-Air (Apache-2.0) is 2.6 GB, autoregressive and only for PyTorch on CUDA;
PiVo-1.4M (Apache-2.0, 15 MB of ONNX) has two voices at 16 kHz, a vocoder run once per 10 ms and,
by its author, a metallic sound. Supertonic 3 stays.
