# Resident voices

*Plan for residents speaking their thoughts and conversations out loud. Status (0.3.0):
residents speak with **Supertone's Supertonic 3** on the graphics card (WebGPU), or on the
processor (WASM) when Auto steps down: ten bundled voices, each resident with their own pitch,
depth and speed (in since 2026-10-12). 0.1.0 shipped
KittenTTS nano (see "History: KittenTTS nano and Paradee-8M"). In between, on 2026-10-12 and
never in a release, Kyutai's Pocket TTS spoke like a reference clip, so players could record
their own voice, but on the CPU a line took 1–2 s and on WebGPU it was four times slower (see
"History: Pocket TTS"). The sections "The model: Pocket TTS", "Voices" and "A resident's own
voice" describe that model and are kept as history. Phases 1–6 are in. Since 0.2.0 a voice-engine
trap restarts the voice worker (at most 3 times a session), and sound stops in a hidden tab.
Since 0.3.0 Auto runs the voices on the graphics card and steps down to the processor when it is
missing or too slow. Measurements and research behind the choices are at the end.*

## Built so far

- **Model (`web/src/voice/supertonic.ts`):** Supertonic 3 (Supertone, BigScience OpenRAIL-M),
  four ONNX graphs with their weights stored as 16-bit floats (`crates/tools/src/fp16.rs`, cast back
  to 32-bit when the model loads, so the GPU does the same arithmetic) and gzipped (`.onnxz`,
  unpacked in the worker): 185 MB instead of 398 as published. Measured against the published
  weights, on the GPU, a line differs less than it does between two random seeds (spectral
  match 0.61–0.77, against 0.55–0.59), with the same timing and the same pitch per voice. 4-bit
  doesn't fit: 91 % of the weights are convolutions, which ONNX Runtime has no 4-bit form of on
  WebGPU (and the int8 build ran its int8 operators on the CPU: 25 times slower). The graphs:
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
  Bea, Dora, 188–216 Hz) raised to 245–290 Hz and made smaller. No voice cloning: the creator's
  "Record a voice" and the expression slider (Pocket's temperature) are gone; saves keep a
  recording but it isn't used.
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
  fallback, still faster than Pocket TTS. The device test with LFM2.5: 0.42 s from
  asking for a line to hearing it (Pocket TTS on the CPU: 2.1 s); speech made in 0.10 of its
  length. The first line after loading takes about 0.2 s more (the GPU sets up its kernels). The
  same test on an M1 Pro (2026-10-18, 8 passes): 0.86 s and 0.24 of its length — the model alone
  writes 96–98 tokens a second (`tools/llm/bench.mjs`; 76–81 with the game and the voices drawing
  on the same GPU).
- **Browsers (M3 Pro, 2026-10-18, the device test):** Chrome/Brave 0.38–0.42 s a line (160–170
  tokens/s); Safari 26.5 0.71 s (114 tokens/s); Firefox 158 3.5–4.5 s (9.7 tokens/s). Firefox's
  WebGPU is much slower for both models here (speech 0.7 s against 0.18 s), so the test turns the
  language model off there and the settings say why; voices still run. Not the computer: the
  same Mac is fast in the other browsers.
- **Download:** 218.7 MB once (the model 185.2, the runtime's WebGPU build 28.7, voices and text
  files 3.2, dictionary 1.3, our WASM 0.16), kept in the browser's cache (packed).
- **Lines of 1–5 sentences (the language model's):** each line is asked for a length picked by
  its seed (1: 20 %, 2: 25 %, 3: 25 %, 4: 15 %, 5: 15 %; `lineSentences` in `talk/writer.ts`),
  and the generation keeps to it (`GenParams.sentences` in `llm/generate.ts`): until the last
  sentence is finished only plain speech can be written (no end of turn, quotes, line breaks,
  emoji or other scripts), a full stop joined to a closing quote (`."`) is written as the full
  stop alone, and the line ends with its last sentence. Asked alone, LFM2.5 350M wrote two
  sentences when asked for three to five. 20 of 20 sampled lines had the length asked for, 125–600
  ms each on the GPU.
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
- **Download on request.** The model (173 MB with the runtime and voices) is fetched when the
  player turns voices on, or when they hear a voice in the creator, and kept.
- **Consent.** A player's recording is their own voice or someone's who agreed (Kyutai's terms
  for Pocket TTS forbid cloning a voice without its owner's consent; the creator says so).

## The model: Pocket TTS

[Pocket TTS](https://github.com/kyutai-labs/pocket-tts) (Kyutai, January 2026; weights CC BY
4.0) is a 100M-parameter "continuous audio language model": a transformer writes 12.5 frames a
second of Mimi codec latents, one after another, conditioned on the text and on a reference
clip of the voice, and the Mimi decoder turns them into 24 kHz audio. Kyutai measured about 6×
real time natively on a MacBook Air M4 CPU. We use the ONNX export by KevinAHM
(`KevinAHM/pocket-tts-onnx`, revision `58a6d00c`, CC BY 4.0), its `english_2026-04` bundle, int8,
as published:

| File | Size | Job |
|---|---|---|
| `mimi_encoder_int8.onnx` | 20.8 MB | reference clip (24 kHz) → embeddings, 12.5 per second |
| `text_conditioner_int8.onnx` | 16.4 MB | SentencePiece token ids → embeddings |
| `flow_lm_main_int8.onnx` | 76.3 MB | the transformer: one frame per run; six layers of attention caches (1,000 positions) as state in and out |
| `flow_lm_flow_int8.onnx` | 10.0 MB | the flow network: conditioning, noise → the frame's latent (one Euler step, `lsd_steps` 1) |
| `mimi_decoder_int8.onnx` | 22.7 MB | latents → audio, 1,920 samples a frame, in chunks of 15 frames |
| `bundle.json`, `bos_before_voice.npy`, `tokenizer.json` | 0.3 MB | state manifests and sizes, the embeddings put before a voice's, the tokenizer (from the ungated `kyutai/pocket-tts-without-voice-cloning`) |

**One line (`pocket.ts`):** the voice's state is made once (`prepareVoice`: the clip shaped, encoded,
`bos_before_voice` and its embeddings fed to the transformer; the result kept with each
attention cache cut to the positions in use, about 6 MB, twelve voices). The text is cleaned up
as the reference does (one line, a capital first, a full stop at the end, pieces of at most 50
tokens split between sentences), embedded and fed; then frame by frame the transformer gives a
conditioning vector and an end-of-speech score, the flow network makes the frame's latent from
noise of deviation √temperature, and the line ends a few frames after the score passes −4 (3
more for lines of up to four words, as the reference). The decoder makes the audio; our WASM
trims it, shapes it and finds its mouth shapes.

**The reference clip carries the voice:** pitch, timbre, pace and recording quality. Kyutai's own
default voices include voice donations that are whispered or very creaky; ours are clear speech
(see "Voices"). The model follows changes to the clip: measured on two voices, shaping the clip's
pitch by ×0.88/×1.12 moved the output by 172/228 Hz from 197 (a woman's) and 77/88 from 81 (a
man's), and pitch 1.25 with size 1.15 took a woman's 197 Hz to 288 Hz; the length stays. Speed in
the clip moved the output's pace too (a clip read 18 % faster gave lines 14 % shorter), but speed
is given per line instead (see below).

**Expression** is the flow's noise (`temperature`): Kyutai's reference uses 0.7; ours spread 0.55–0.8
per resident, ×0.6–1.4 in the creator, and tones move it (angry and happy livelier, sad calmer).

**Dropped words:** now and then the model starts a line a word or two late ("…a lovely garden"):
it is the model's own (its raw output already lacks the word), at some seeds and voices. Over 144
test lines (24 voices, three lines, two temperatures; Whisper small) 4 began wrong with the clip
trimmed to 0.1 s after the voice, 1 with 0.6 s of silence after it, so every clip now ends with
that pause (`dsp::clip`). Lines are short and residents say many, so the rest is left as it is.

**Threads and the GPU:** one thread gives about 1.6× real time for whole lines; the worker uses
two where the page is cross-origin isolated and the computer has at least six cores (the game, the
simulation and the language model keep the others). Kyutai saw no GPU speed-up for batch size 1,
and on WebGPU it was about four times slower here (2026-10-12): it only runs on the CPU.

## Shaping a voice (`crates/voice/src/dsp.rs`)

Pocket TTS has no pitch or speed inputs, so the sound is shaped around it, all in our Rust WASM
(the same samples in every browser):

- **`clip`:** any sound at any rate to the clip the encoder takes: 24 kHz (band-limited resampling,
  `resample.rs`), the silence around it trimmed (as the output is), at most 10 s, its loudness
  brought to −22 dBFS RMS of its sounding frames (the peak under 0.95). The bundled voices go
  through it at build time, recordings in the worker.
- **TD-PSOLA (`psola`):** the pitch is found every 10 ms (normalised autocorrelation, 50–500 Hz,
  searched at half the rate and refined, the shortest strong peak taken so a multiple of the
  period isn't, median-smoothed over five frames); the sound is cut into Hann grains two periods
  long, one period apart (5 ms where unvoiced), and laid out again with the grains closer
  (higher) or further apart (lower), each taken from its place in time, so pitch and length
  change independently and the formants stay.
- **`shape(pitch, depth, speed)`:** resample by depth (every frequency, formants included, ×depth),
  then PSOLA puts the pitch at `pitch` and the length at 1/speed. About 50–250 ms for a line.
- **`reference`** is `shape` on a clip without speed: what the encoder hears for a voice.

**Two places for pitch and size (`VoiceParams`):** in the clip (`clipPitch`, `clipDepth`), and on
each line (`pitch`, `depth`, `speed`). In the clip the model speaks the line that way, naturally,
but every new value is a new voice to get ready (about 2.4 s the first time in a browser: the
encoding is kept in the Cache API after that). On the line it costs a few milliseconds but is
signal processing. So:

- a resident's own small variation (pitch ±6 %, depth ±3 %), elders, and tones go **on the line**:
  the 24 clips are shared by everyone;
- children's voices (a woman's clip raised to 265 Hz and made 1.08 smaller, one per voice) and
  the pitch and depth chosen in the creator go **into the clip**, rounded to 0.02 so similar
  choices share one;
- speed is always on the line.

## Voices

Each resident's voice comes from their **stable resident id**, gender and life stage
(`voiceFor`); a voice chosen in the creator is saved on top (see "A resident's own voice"):

```ts
interface VoiceParams { clip: string; clipPitch: number; clipDepth: number; pitch: number; depth: number; speed: number; temperature: number }
```

**The bundled voices (`PRESETS`):** twelve women (Anna, Azelma, Caro, Eponine, Eve, Fantine,
Iris, Jane, Lucy, Mary, Nora, Vera; 161–238 Hz) and twelve men (Bill, Charles, Felix, George,
Hugo, Leon, Michael, Oscar, Owen, Paul, Peter, Stuart; 79–134 Hz), each with its measured pitch
(`f0`, pYIN). Twenty are VCTK speakers (sentence 23, as enhanced by Kyutai; mostly British
accents), four are LibriVox readers via Voice-Zero. Chosen by licence (no non-commercial sets:
Expresso and EARS are left out) and for clear, natural speech: two of Kyutai's defaults from the
voice donations were left out, one whispered (no voiced frames) and one very creaky, and Alba
MacKenna's casual voice (122 Hz) for not reading clearly as a woman's.

**Who speaks with what:** men from the men's voices, women from the women's, other genders from
all 24, picked by the id. Children of any gender: a woman's voice raised to 245–290 Hz (265 Hz in
the clip, the rest on the line) and made smaller (1.05–1.12). Babies don't speak (the creator
plays a baby as the child they will be). On top, from the id: pitch 0.94–1.06 (teens 1.04–1.12),
depth 0.97–1.03 (teens 1.03–1.07), speed 0.88–1.12 (children ×1.04), expression 0.55–0.8, and the
elders' factors (women's pitch ×0.93 and depth ×0.98, men's ×1.04 and ×1.02, everyone's speed
×0.92). Measured over eleven residents (`tools/voice/samples.mjs`): men 88–141 Hz, women
202–251 Hz, the child 265 Hz.

**Tones** (`withTone`, on the line): happy pitch ×1.06, speed ×1.06, livelier; angry ×1.04,
×1.12, livelier still; sad ×0.94, ×0.9, calmer; flirty ×0.97, ×0.94.

### A resident's own voice

The household creator's *Identity* tab has a *Voice* section:

- **A voice:** *Own* (the generated one) or one of the bundled voices that fit the gender (all 24
  for other genders; picking one plays it), or *Recording* when they have one.
- **Their own voice:** *Record a voice* (the microphone, with a level meter, 3–12 s, *Stop*) or
  *Load a sound* (any sound file the browser can read; its first 12 s are played silently into the
  recorder, so it is compressed the same way). Under it: "Only use your own voice, or the voice of
  someone who agreed to it. The recording stays on this computer and in your saves." Choosing a
  bundled voice sets a recording aside (it can be chosen again until the creator closes); a
  separate button removes it.
- **Pitch** (lower to higher), **Speed** (slower to faster), **Depth** (smaller to larger),
  **Expression** (calmer to livelier), a dice (a fitting voice or a new generated one, and every
  slider anywhere in its range; a recording stays), a button back to the default (own voice,
  factors 1; a recording stays), and **Hear** ("Hi, I'm Ada! This is how I sound." in that voice;
  a baby is heard as the child they will be).

Every control is heard for grown-ups and children alike, except the voice choice for children,
who keep a child's voice and take their chosen voice or recording from their teens; changing the
gender drops a bundled voice of the other sex (a recording stays). Hearing works with resident
voices off in Settings and doesn't turn them on; the engine is unloaded again when the creator
closes.

The choice is stored in the resident's appearance, which the simulation keeps as opaque JSON, so
the save format doesn't change:

```ts
appearance.voice = { pitch: 1.08, speed: 0.95, depth: 0.96, expressive: 1.2, seed: 1439030069, preset: 'Mary' }
appearance.voice = { pitch: 1, speed: 1, depth: 1, seed: 1439030069, recording: { audio: '<base64>', type: 'audio/webm;codecs=opus', sha: '3f…' } }
```

- **`preset`** (optional): a bundled voice by name, a teen's or grown-up's voice instead of the
  generated one.
- **`recording`** (optional, wins over `preset`): the sound (base64, as the browser compressed
  it), its MIME type and the first 16 hex digits of its SHA-256, which names it (`rec:<sha>`). It
  starts as recorded (pitch and depth 1; the sliders move it from there, into the clip). The main
  thread decodes it (to 24 kHz) and hands it to the worker; a recording this browser can't decode
  (made in another browser, say) is left out and the resident has their generated voice.
- **`pitch`, `depth`** go into the clip; **`speed`** and **`expressive`** (a factor on the
  temperature) are per line. Each is a factor on the generated voice (1 or missing: as generated);
  the life stage still applies underneath. Kept within pitch 0.5–1.5, speed 0.8–1.2, depth
  0.78–1.16, expression 0.2–1.
- **`seed`** replaces the resident id in the generated voice: someone made in the creator has no
  id until they move in, and the voice heard there must be the one they keep.
- **`base`** (0.20 to 0.1.0, a KittenTTS voice by index) is ignored: those residents have their
  generated voice.
- Without `voice` (older saves, neighbours, newcomers, babies, random residents from the
  creator's dice) a resident has the generated voice. Every resident sounds different from 0.1.0
  (a new model).
- Main-thread rewrites of the appearance keep `voice`: life-stage looks and grey hair copy it
  along; the look expanded from a newcomer's or baby's seed keeps a voice stored next to it.
  Portraits ignore it (their cache key leaves it out).

There is no place to change a resident's look in a game yet; the Voice controls would go there too.

## Phonemizer: our own, in Rust, without espeak-ng

*Since Pocket TTS (which reads plain text) the phonemizer only gives the mouth shapes their
phonemes; a pronunciation mark in a line (`[read](/ɹˈɛd/)`) now shapes the mouth, and the model
reads the word. It was written for Paradee and KittenTTS:*

Paradee, like Kokoro, needs **Misaki** phonemes. `kokoro-js` gets phonemes from **espeak-ng
compiled to WASM, which is GPL-3**, and needs a conversion step (`web/misaki.js` in the Paradee
repo; without it Whisper mishears about 31 % of words). The game was MIT then (GPL-3.0-or-later since
0.1.0). We don't use espeak-ng.

Existing Rust options (checked 2026-10-09):

- **`misaki-rs`** (MIT, latest 0.6.0, September 2026): a port of Misaki with the dictionaries and
  a part-of-speech tagger built in. espeak is an optional feature that is on by default; without
  it, unknown words are spelled out letter by letter. Unclear whether it builds for `wasm32`
  (docs.rs has failed to build it since 0.3.0); about 40 MB of source; small project (10 stars).
- **Kokoros** (Apache-2.0), a Rust Kokoro runtime, builds espeak-ng in (`espeak-rs-sys`): same
  GPL problem.
- Not Rust: `@piper-plus/g2p` (MIT, rule-based, runs in the browser) outputs Piper's phonemes.

**Plan: `g2p` inside `crates/voice`** (MIT):

1. **Dictionary:** Misaki's English dictionaries (`us_gold`, `us_silver`; Apache-2.0, credited in
   `CREDITS.md`). They are already in the alphabet Paradee takes, so no mapping is needed. Ship
   them compressed; trim rare words if size matters.
2. **Text clean-up:** punctuation, contractions, possessive and plural "s", "-ed" and "-ing"
   endings (Misaki's suffix rules), numbers to words.
3. **Unknown words** (mostly residents' names): public-domain letter-to-sound rules (the 1976
   NRL rules), written in Misaki's alphabet. A tiny trained model can replace them later.
4. **Words with two pronunciations** ("read", "live"): content lines can mark the pronunciation
   (Misaki's `[word](/phonemes/)` markup); dynamic text accepts the occasional slip.
5. **Build-time check:** every line in the content is phonemized in a test, and a word that falls
   back to the rules fails it, so hand-written lines always sound right.

Phase 1 also tries `misaki-rs` with `default-features = false`. If it builds for `wasm32` at an
acceptable size, we use it (or a fork) and add only steps 3 and 5.

## Settings

A **Voices** tab in Settings (`ui/screens/SettingsPanel.svelte`). Everything applies live, so
nothing is added to `RESTART_KEYS`.

| Row | Control | Notes |
|---|---|---|
| Resident voices | Toggle | Default off. |
| Language | (hidden while English is the only one) | Comes back as a row when a second language ships. |
| Runs on | Auto / Graphics card / Processor | Auto uses the graphics card and steps down to the processor when it is missing or too slow. The choice applies live. |
| Voice model | Status | Supertonic 3, its state, where it runs, and the download (about 217 MB). |
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
   `littlelives.voiceBench`, outside `Settings`. Older results without a device count as the
   graphics card (the only device before 0.3.0); results from another model or core count are
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
- **Memory:** the worker is unloaded after 5 minutes without speech, and when voices are turned
  off. Its sessions take about 300 MB (the transformer's state is 49 MB in and out per frame);
  twelve prepared voices about 75 MB more. Finished clips are kept in an in-memory LRU cache
  (24 MB), keyed by model, device, voice and text. Because the engine is deterministic, a cached
  clip is exactly what would be generated again. Encoded voices are kept in the Cache API (512 KB
  each).
- The performance overlay (F3) shows a voice line: model, mode, last RTF, queue length, cache
  hits.

## What residents say

Today's bubbles are icons only (`game/bubbles.ts`), so speech needs text: **hand-written lines as
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
  thoughts (see `docs/design/smarter-thoughts.md`).

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

- One shared `AudioContext` with sound effects (`ui/sfx.ts` is refactored to expose it), with
  separate gain nodes for effects and voices.
- Each speaking resident gets a `PannerNode` at their head position (`renderer.simHead`),
  updated about 10 times a second, so voices come from where the resident stands.
- The speech bubble appears straight away; the voice follows when its clip is ready.
- **Lip sync.** Every clip comes with a viseme track (`crates/voice/src/visemes.rs`), made in Rust
  together with the audio. Pocket TTS gives no durations, so the phonemes are mapped to the 15
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
| `voice/voices.ts` | The bundled voices (`PRESETS`), and each resident's clip, pitch, depth, speed and expression from id, gender, life stage and the voice chosen in the creator. |
| `voice/service.svelte.ts` + `voice/tts.worker.ts` | Worker client and worker: load the model, get a voice ready, recordings handed over, `speak(text, voice) → Float32Array` (transferred, 24 kHz), clip cache, speed test, unload. |
| `voice/pocket.ts` | One line with Pocket TTS (the worker and the Node checks share it). |
| `voice/record.ts` | A player's own voice from the microphone or a sound file. |
| `voice/benchmark.ts` | The benchmark and the device choice. |
| `voice/player.ts` | Voice bus, panners, clip cache. |

## Phases

| Phase | Scope | Done when |
|---|---|---|
| 1. Baseline (2–3 days) | `crates/voice` with `tract` running Paradee plus the `pitch` graph edit, in a worker next to the running game; try `misaki-rs` without espeak on `wasm32`. (Since 0.20 the baseline is ONNX Runtime Web.) | Paradee speaks hand-written phonemes in the browser; RTF and frame impact measured in Chrome, Firefox and Safari; `misaki-rs` or our own `g2p` chosen. |
| 2. Phonemizer (2–4 days) | `g2p` (or `misaki-rs` plus our fallback). | Every content line phonemizes from the dictionary; names get a plausible reading; no GPL code shipped. |
| 3. Rules | Settings tab, `VoiceDirector` with the "who may speak" rules and guards, lines content, voice bus with panning. (Babble, a made-up language, was part of this phase from 0.14 to 0.19; removed in 0.20.) | Selecting a resident or starting a conversation makes them speak; nobody else does; no frame cost. |
| 4. English | Download with progress, `VoiceSpec` (pitch, speed), clip cache, unloading, quiet while loading. | Selected residents speak English; residents sound different from each other. |
| 5. Own engine | Planned for KittenTTS; with Pocket TTS (a transformer run frame by frame, 146 MB) ONNX Runtime Web stays. Streaming playback (a line starting after its first frames) is the next speed-up if slower computers need it. | |
| 6. Benchmark | "Test this computer", Auto mode (graphics card, processor fallback), live watchdog, F3 line. In 0.3.0. | Auto picks the right mode without frame drops; a slow machine steps down with a clear message. |
| Later | Thoughts from the Planner (0.8) and from an in-browser language model; lip sync; persistent clip cache (IndexedDB, Opus via WebCodecs); a distilled male voice; more languages. | |

Every phase keeps `bun run check` clean and the frame-time budget checked, with voices on and off.

## Research: a voice the player can shape (2026-10-11/12)

The owner asked for a voice a player can configure "a little bit", in the browser (WASM or
WebGPU). Options looked at (from their pages, not all measured):

| Option | Size | Browser | What a player can change | Verdict |
|---|---|---|---|---|
| KittenTTS mini / micro 0.8 | 78 / 41 MB (dynamic int8 only) | WASM | Mix of 8 voices, speed (+ our pitch, depth) | Same 8 voices as nano; dynamic int8 ran slower than float in ONNX Runtime Web for nano |
| Kokoro-82M (q8) | 92 MB | WASM, WebGPU | Blend of 20 American voices, speed (+ ours) | Slower than real time on one thread (2026-10-10) |
| Supertonic 2 | | WASM, WebGPU | 10 voices | Archived 2026-07-23, weight licence unclear |
| Piper (VITS) | 20–60 MB a voice | WASM | Speaker id, speed, noise | Research-only data for the multi-speaker English voice |
| **Pocket TTS** | 146 MB int8 | WASM (community ports) | **Any reference clip**: presets, recordings; shaped clips | **Chosen** |
| Chatterbox Turbo | 350M params | WebGPU (Transformers.js) | Cloning, exaggeration | Too heavy next to the game |
| MOSS-TTS-Nano | 100M params | ONNX, browser extension only | Cloning | Unproven in a page |
| Qwen3-TTS VoiceDesign, NeuTTS | 0.6–1.7B, 229M–0.75B | no browser build | Voice from a description / cloning | Too big |

**Measured before building it (Phase 0, M3 Pro, Node, ONNX Runtime Web 1.30 WASM, the int8
bundle):** the speed test's three lines at 1.7–2.0× real time on one thread, 2.8× on two, 3.7× on
four (the transformer about 23 ms a frame, the decoder 17, the flow 2 on one thread); a 10 s clip
encoded in 2.1 s (int8 and fp32 alike), fed in 0.4–0.5 s; bit-identical with our noise; Whisper
(small) understood every line; and the output followed a clip read faster or slower (pitch and
pace), and our PSOLA-shaped clips (pitch alone, size alone). The attention cache's length (1,000)
is fixed in the export.

**Trailing silence on the clip and dropped first words:** 144 lines (24 voices × 3 lines × 2
temperatures) with the clip as trimmed (0.1 s after the voice): 4 began wrong; with 0.6 s of
silence after it: 1. Adopted (`dsp::clip` adds 0.5 s).

## Research: different voices (2026-10-10)

The owner found men and women sounding alike. Measured (M3 Pro, ONNX Runtime Web WASM on one
thread in Node and in Chrome for Testing; WebGPU in Chrome; UTMOS and Whisper for quality,
WeSpeaker for how different voices are, Praat for pitch and formants; three game lines):

| Option | Per line | Download | Voices | Verdict |
|---|---|---|---|---|
| Paradee with other style vectors | 0.2–0.5 s | 9 MB | 1 | Its voice is two learned constants (1×32, 1×16), not Kokoro's 256-number style, so Kokoro's voices can't be fed; changing them mostly breaks the voice. |
| **KittenTTS nano 0.8**, weight-only int8 | 0.9–1.6 s | 15.5 + 1 MB | 8 + mixes | **Chosen** (above). |
| Kokoro-82M q8, WASM | 2.8–4.5 s | 86–92 MB | 28 + blends | Slower than real time on one thread (1.0× with four threads and cross-origin isolation). |
| Kokoro-82M, WebGPU | 1.2–1.75 s, first line up to 7 s | 155–326 MB | 28 + blends | Not the 10× reported elsewhere under our constraints; fp32 halved the frame rate on a 120 Hz screen. |
| Kokoro-7M-Distill | 0.16–0.28 s | 31 MB | 1 | Takes Kokoro's style input but every voice comes out as the same woman. |
| Piper LibriTTS-R medium, int8 | 0.14–0.27 s | 22 MB | 904 | Fast, real men's voices, good scores, but fine-tuned from the lessac voice, whose data licence is research-only. The from-scratch LibriTTS high voice is CC BY 4.0 but slower than real time. |
| A distilled male Paradee | Paradee's | +9 MB per voice | 1 per model | Kept as the fallback plan (see "Voices"). |
| Pocket TTS, Supertonic, KittenTTS 2 and others | | 100 MB–1.7 GB | | Too big, or licence or archive problems. |

## Research summary (2026-10-09)

| Option | Verdict |
|---|---|
| **Paradee-8M** | Chosen. Apache-2.0, 9 MB int8, 12–14× real time on one CPU thread (our native test), quality close to Kokoro. One voice; Misaki phonemes. |
| Kokoro-82M | Optional larger model. Apache-2.0; q8 92 MB, fp32 326 MB; about real time on CPU with 4 threads, about 10× on WebGPU (M1 Max). 28 English voices that can be blended. |
| Kokoro-7M-Distill | 30 MB, faster (35×) but lower quality (UTMOS 4.18) than Paradee (Paradee paper). |
| Piper / VITS | The multi-speaker English voice is trained on a dataset that forbids commercial use; lower quality. |
| Kitten TTS | 25–78 MB, only 8 voices; the maker calls the browser builds legacy. (Chosen on 2026-10-10 for teens and grown-ups after measuring: 15.5 MB as weight-only int8, see above.) |
| Supertonic | Archived in September 2026; restrictive licence. |
| MMS-TTS | Non-commercial licence. |
| Chatterbox, OuteTTS, CSM, Dia, Orpheus | Too big or too slow next to a 3D game. |
| Browser `speechSynthesis` | Free, but voices differ per player, it can't be positioned in 3D and only one voice plays at a time. |

Sources: [Paradee-8M](https://huggingface.co/sahilmahendrakar/Paradee-8M-v1.0),
[Paradee code](https://github.com/sahilmahendrakar/paradee),
[Kokoro-82M ONNX](https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX),
[Misaki](https://github.com/hexgrad/misaki),
[misaki-rs](https://github.com/MicheleYin/misaki-rs),
[tract](https://github.com/sonos/tract),
[ONNX Runtime Web](https://onnxruntime.ai/docs/tutorials/web/) (MIT, [npm](https://www.npmjs.com/package/onnxruntime-web)),
[browser TTS benchmarks](https://briantung.me/blog/tts-engines-in-the-browser/),
[lessac dataset licence](https://www.cstr.ed.ac.uk/projects/blizzard/2013/lessac_blizzard2013/license.html).
## History: Pocket TTS (2026-10-12, after 0.1.0; never released)

Kyutai's Pocket TTS spoke like a short reference clip, so a player could record their own voice.
It made speech one 80 ms frame at a time: 1.6–2× real time on the CPU, and on WebGPU about four
times slower (many small passes). Replaced by Supertonic 3 on the GPU. What was built:

- **Model (`web/src/voice/pocket.ts`):** Pocket TTS, Kyutai's 100M-parameter English model
  (CC BY 4.0), as the published int8 ONNX export of its `english_2026-04` bundle (KevinAHM),
  unchanged, five graphs: Mimi encoder (a reference clip to embeddings), text conditioner,
  the transformer (`flow_lm_main`, one 80 ms frame per run, with its attention caches as state),
  the flow network (one Euler step from noise to the frame's latent) and the Mimi decoder (latents
  to 24 kHz). The loop around them is ported from the export's reference code. It runs in the
  voice worker on **ONNX Runtime Web** 1.31.1 (WASM with SIMD; two threads when the page is
  cross-origin isolated and the computer has six cores or more). Lines are made one at a time;
  the Node checks run the same `pocket.ts`.
- **Deterministic:** the flow's noise is ours, a hash of the voice, the text and the frame
  (Box–Muller on two counter-hashed streams), so the same line in the same voice gives
  **bit-identical samples** (checked across runs and sessions, against golden hashes), which the
  clip cache relies on. No graph edits are needed.
- **Voices (`PRESETS` in `web/src/voice/voices.ts`):** 24 bundled clips, twelve women's and
  twelve men's, from Kyutai's voice repository (VCTK, CC BY 4.0; Voice-Zero, CC0), each turned
  by `cargo fetch-voice` into a clip as the encoder takes it (24 kHz, trimmed, at most 10 s,
  loudness evened out) and packed into one 16-bit file (11.3 MB). See "Voices".
- **Shaping (`crates/voice/src/dsp.rs`, in the voice WASM):** our own TD-PSOLA moves pitch, size
  (formants, by resampling first) and speed independently. A voice's pitch and size can go
  **into the clip** the model hears (the model then speaks the whole line that way) or **onto each
  finished line** (cheap). See "Shaping a voice".
- **Players' own voices (`web/src/voice/record.ts`):** the creator records from the microphone
  or reads a sound file, compressed by the browser (Opus, 32 kbit/s, at most 12 s, about 50 KB)
  and kept in the resident's appearance, so it travels with the save. See "A resident's own
  voice".
- **Text:** the model reads plain text (`plain_text`: pronunciation marks dropped); our Rust
  phonemizer (`crates/voice`, Misaki's rules) now only gives the mouth shapes their phonemes.
- **Tests:** `cargo test -p voice --release` (phonemizer against Misaki, the shaping, mouth shapes,
  and every game line reads as plain text); `bun run check:voice` (also in CI: ONNX Runtime Web in
  Bun with the worker's `pocket.ts`, bit-identical across runs and sessions, golden hashes;
  pitch, depth and speed on the line, pitch in the clip, expression and trimming checked).
- **Browser (`web/src/voice/`):** the worker with a Cache API download (`tts.worker.ts`; encoded
  voices are kept there too), the app-wide service and speed test (`service.svelte.ts`), the
  director with the "who may speak" rules (`index.ts`; it gets a resident's voice ready when they
  start to be heard), lines per language (`content/voice/en.json`, packs add theirs), voices per
  resident (`voices.ts`), stereo playback (`player.ts`), the clip cache. Settings → Voices.
- **Measured (M3 Pro, 2026-10-12):** in Node (ONNX Runtime Web WASM) 1.6–1.8× real time on one
  thread for whole lines (the 51-character test line: 3.3 s of audio in 2.0 s), about 2.8× on two
  threads and 3.7× on four (the reference loop alone; the shaping adds 50–250 ms a line). In
  Chrome for Testing (cross-origin isolated, two threads), the Settings speed test: 1.6 s per
  line, 2.1× real time, worst frame 16.7 ms with the menu's town drawing; in a game, lines took
  0.6–2.2 s (1.5× real time), none dropped. The device test (LFM2.5 on the GPU) uses setup A:
  speech 1.2 s a line on the CPU, against 3.6 s on WebGPU (setup B, too slow). Lines of 1–5
  sentences (the model's, since 2026-10-12) are spoken a piece at a time (`speechPieces`): on the
  CPU the first sound comes after 1.1–1.7 s whatever the length and speech is made in 0.6–0.9 of
  its length, so the next piece is always ready; on WebGPU 5–7 s and 3–3.7, with gaps of up to
  15 s in a five-sentence line. So Pocket TTS only runs on the CPU (setup B is gone). Getting a
  new voice ready: about 2.4 s (encoding a 10 s clip 2.1–2.3 s, feeding it to the model 0.4 s);
  0.4 s once its encoding is kept. Whisper understood every test line in every voice; about one line in 140
  starts a word late (the model's own; see "Pocket TTS").
- **Download:** 173.5 MB once (runtime 14.2, model 146.5, voices 11.3, dictionary 1.3, our WASM
  0.16), kept in the browser's cache.
- **Guards (`voice/index.ts`):** as before: nothing new is said while the engine loads; it
  unloads after 5 minutes without a line; voices pause for the session (with a toast) after three
  lines in a row later than 4 s, or when making speech slows the frames.
- **Not yet:** Auto mode; streaming within a piece (lines are played a sentence or so at a time;
  if computers slower than the M3 Pro need it, starting a piece after its first frames is next).


## History: KittenTTS nano and Paradee-8M (0.20 to 0.1.0)

*Residents spoke with these before Pocket TTS. Kept for the measurements and the reasoning; the
files, the graph edits (`tools/voice/model.mjs`) and the KittenTTS tokenizer are gone.*

### Built for KittenTTS (0.1.0)

- **Phonemizer (`crates/voice`, `crates/voice-wasm`):** Misaki's English rules ported to Rust
  (`g2p.rs`; matches Python Misaki 0.9.4 on all 25 test sentences in `tests/data`), the merged
  Misaki dictionary, letter-to-sound guesses for unknown names (`rules.rs`), and Misaki to
  espeak-style IPA and KittenTTS's tokenizer (`kitten.rs`, see "KittenTTS nano"). The browser
  WASM only turns text into the model's inputs and trims and resizes its output (140 KB; it was
  180 KB with Paradee's tokenizer).
- **Model (0.20):** KittenTTS nano runs in the voice worker with **ONNX Runtime Web** 1.30.0
  (`onnxruntime-web/wasm`: its WebAssembly build with SIMD, one thread, no proxy worker; the
  14.2 MB runtime is bundled with the game and loaded through the same cached download), loaded
  the first time a voice is needed. It works with and without cross-origin isolation. Lines are
  made one at a time, by `voice/synth.ts` (the Node checks use the same code). Files the current
  version no longer uses (an older build's WASM, Paradee's model) are dropped from the cache.
- **Model edits (`tools/voice/model.mjs`):** `bun run voice` (`cargo fetch-voice`) fetches the
  pinned upstream model and edits it in Node (a minimal protobuf reader/writer, so
  CI needs no Python): a `pitch` input multiplies the pitch curve, and the two random nodes are
  replaced by a counter-based hash of each value's position (h = (i·48271 + salt) mod 2³¹−1, then
  twice h = (h² + c) mod 2³¹−1; Box–Muller for the normal noise). ONNX Runtime's random kernels
  keep their state between runs even with a seed, so this is what makes the same line and voice
  give **bit-identical samples**, which the clip cache relies on. The noise is as white as a
  library generator's (autocorrelation under 0.004, spectral flatness 0.998), the output is as
  close to the original model as two runs of the original are to each other, and it costs no
  time. The result is pinned by its own SHA-256 and named `kitten-nano-0.8-edit2.onnx` (a new
  edit gets a new name, so browsers never mix versions); its weights are also stored as int8
  (see "KittenTTS nano").
- **Tests:** `bun run test:voice` (Rust: phonemizer against Misaki, ids, trimming, and every game
  line tokenises for KittenTTS); `bun run check:voice` (also in CI: ONNX Runtime Web in Bun with
  the browser's WASM and the worker's `synth.ts`, bit-identical across runs and sessions, against
  golden hashes, which hold on arm64 and x86-64; pitch, speed, depth, mixes and trimming checked).
- **Browser (`web/src/voice/`):** voice worker with a Cache API download (`tts.worker.ts`),
  app-wide service and speed test (`service.svelte.ts`), the director with the "who may speak"
  rules (`index.ts`), lines per language (`content/voice/en.json`, keyed by content ids, per item and interaction;
  packs add theirs), per-resident
  pitch and speed (`voices.ts`), stereo playback (`player.ts`), the clip cache. Settings → Audio.
- **Measured in Chrome 155 (M-series Mac, production build, the speed test's three lines, nine
  runs each):**

  | | `tract` (0.19) | ONNX Runtime Web (0.20) |
  |---|---|---|
  | "Hello there!" (1.3 s of audio) | 0.39–0.52 s | 0.18–0.20 s |
  | 51-character line (3.7 s) | 0.93–1.17 s | 0.51–0.63 s |
  | 66-character line (4.4 s) | 1.11–1.50 s | 0.62–0.72 s |
  | Real-time factor | 3.0–3.9× | 5.8–7.1× |
  | Worst frame while speaking | 9–17 ms | 9–17 ms (once 368 ms, in the first test after start) |
  | Engine ready (files cached) | 0.4 s | 1.2–1.3 s (creating the session) |
  | Download (raw / gzipped) | 28.7 / 12.6 MB | 24.8 / 12.5 MB |

  ONNX Runtime Web's download: runtime 14.2 MB (3.7 MB gzipped), phonemizer 0.18 MB, worker
  0.08 MB, model 9.0 MB (7.4 MB), dictionary 1.3 MB; cached after the first time.

  With KittenTTS (0.20, production build in Chrome for Testing on an M3 Pro, served without
  cross-origin isolation like GitHub Pages): the speed test's median line 1.2 s for KittenTTS
  (2.2× real time) and 0.57 s for Paradee (6.4×), worst frame 16.8 ms for both while the menu's
  town draws; in a game, KittenTTS lines took 0.70–0.95 s and no frame drawn while they were made
  was over 16.8 ms. KittenTTS adds 15.5 MB (13.5 MB gzipped) and its voices 1.0 MB: 41.3 MB for
  everything (26.9 MB gzipped), 32.2 MB for KittenTTS alone, 24.8 MB for Paradee alone.
- **Voices by gender and life stage (`voices.ts`):** everyone speaks with KittenTTS, each with
  their own mix of two of its voices; children with a mix of the women's voices, higher and
  smaller; pitch, speed and **depth** (the size of the voice) per resident, from their id; elders
  slower. Babies don't speak. See "Voices".
- **Depth by resampling (0.20):** the voice worker runs the model at pitch P/α and speed s/α
  and resamples its output by α (`crates/voice/src/resample.rs`, in the voice WASM), so the
  formants scale by α while pitch and length stay. A few milliseconds per line (2–6 ms in Chrome
  for 1.5–2 s lines, about 5 ms for a 3.7 s line in Node).
- **Voices chosen in Creation (the household creator):** the resident's own mix or one of KittenTTS's
  voices, and pitch, speed and depth, kept in the save (see "A resident's own voice").
- **Guards (`voice/index.ts`):** nothing is said while the engine loads; it unloads after 5
  minutes without a line and reloads on the next (quiet meanwhile); a watchdog pauses voices for
  the session (with a toast) after three lines in a row later than 4 s, or when over 10 % of the
  frames drawn while speech is made exceed 50 ms and that is at least twice the share without.
  The F3 voice section shows whether voices are on, off or paused. (In 0.20 KittenTTS stepped
  down to Paradee first; with Paradee gone, there is no smaller model to step down to.)
- **Not yet:** Auto mode, our own engine. Kokoro-82M was measured again on 2026-10-10 and is not
  planned for now (see "Research").

### The model: Paradee-8M (removed for 0.1.0)

*Children spoke with Paradee-8M in 0.20, and it stood in when KittenTTS was too slow; it was
removed for 0.1.0 so only KittenTTS is used. Kept here for the measurements and the
reasoning.*

[Paradee-8M v1.0](https://huggingface.co/sahilmahendrakar/Paradee-8M-v1.0) is Kokoro-82M shrunk
to a tenth: the same architecture (Kokoro's own code) at smaller widths, 8.07M parameters,
distilled from Kokoro's `af_heart` voice.

| | |
|---|---|
| Licence | Apache-2.0 (weights and code), like Kokoro |
| Files | `paradee_int8.onnx` 9.0 MB (recommended), `paradee.onnx` 37 MB, PyTorch `text_side.pt` + `decoder.pt` |
| Input | Phoneme ids in **Misaki's** alphabet (178 symbols, max 512) and `speed` |
| Output | 24 kHz mono audio |
| Quality (paper) | UTMOS 4.41 vs Kokoro 4.52; word error rate 5.7 %, same as Kokoro |
| Speed (paper) | about 18× faster than real time, int8 ONNX, one CPU thread |
| Limits | American English only, **one voice** (female), a faint buzz on some voiced sounds |

It needs no GPU, so the GPU stays with the renderer.

What we measured ourselves (M-series Mac, one thread, "Hello! I'm hungry, I should cook
something.", 2.95 s of audio):

| Runtime | Time | Real-time factor |
|---|---|---|
| ONNX Runtime (Python, native) | 0.21–0.25 s | 12–14× faster than real time |
| `tract` 0.23 (pure Rust, native, unoptimised plan) | 0.75 s | 4× |
| `tract` 0.23 compiled to **WebAssembly** with SIMD, in Node | 1.5–1.6 s | 1.9× |

Later, on the speed test's 51-character line (3.65 s of audio, one thread, 2026-10-10): ONNX
Runtime native 0.26 s (14×); **ONNX Runtime Web 1.30** (WASM) in Node 0.53 s (6.9×) and in Chrome
0.51–0.63 s; `tract` WASM in Node 1.09 s (3.3×) and in Chrome 0.93–1.17 s. `tract` runs its
unoptimised plan (its optimiser rejects the data-dependent `Range`; splitting the graph at the
alignment reaches the optimised plan, as fast as ONNX Runtime natively, but only for concrete
lengths at about 0.5 s of re-optimisation per line), and dequantises the weights on every call.

- `tract` output matches ONNX Runtime: same length and loudness, spectra correlate 0.988 (the
  model adds random noise, so the two are never bit-identical).
- `tract`'s optimiser rejects the graph (a `Range` op with a symbolic length); only the slower
  unoptimised plan runs. The test `.wasm` was 20 MB before stripping.
- **Pitch control works:** the graph has Kokoro's explicit pitch path (`/F0_proj/Conv` predicts
  the pitch curve, the decoder synthesises from it). Multiplying that curve by an extra `pitch`
  input gave a median pitch of 157 Hz at ×0.7, 209 Hz at ×1.0 and 261 Hz at ×1.3, with the same
  length and intact audio.

### Our engine (`crates/voice`)

**Runtime today (0.20): ONNX Runtime Web.** It is a JavaScript dependency with its own 14 MB
WASM runtime (larger than the model) and treats the model as a black box, but it is about twice
as fast as `tract` was in the browser, at about the same download (12.5 MB gzipped against
12.6 MB), so it replaced `tract`, which is now only a test dependency. A small engine for exactly
this architecture can still replace it (research, 2026-10-10: about 9–10× real time estimated
with plain WASM SIMD, 13–14× with relaxed SIMD, WASM about 0.3–0.5 MB); the plan below stays for
that.

| Runtime (one thread, browser) | 51-character line | Download (gzipped) | Notes |
|---|---|---|---|
| `tract` 0.23 (0.14–0.19) | 0.93–1.17 s | 3.9 MB WASM | unoptimised plan; pure Rust |
| **ONNX Runtime Web 1.30 (0.20)** | 0.51–0.63 s | 3.7 MB runtime + 0.06 MB phonemizer | MIT; edits needed for pitch and determinism |
| Own engine (planned) | about 0.3–0.4 s (estimate) | about 0.2 MB | about 2.5 weeks |

ONNX Runtime Web would run Paradee, but it is a JavaScript dependency with its own multi-megabyte
WASM runtime (larger than the model), and it treats the model as a black box. `tract` ran it in
pure Rust, but slowly. So the plan is a small engine for exactly this architecture.

**Crate layout** (no dependency on `sim-core`; its own `voice-wasm` build, loaded lazily in the
voice worker so the sim's WASM stays small):

| Part | Job |
|---|---|
| `g2p` | Text to Misaki phonemes (see "Phonemizer") |
| `model` | Weights and the network |
| `dsp` | Pitch and noise controls; inverse STFT (the voice's size stays a resample of the output) |
| `lib.rs` | `speak(text, voice) -> Vec<f32>` (24 kHz), exposed with `wasm-bindgen` |

**Two stages:**

1. **Baseline with `tract`** (days). Runs the unmodified `paradee_int8.onnx` plus build-time
   graph edits (a `pitch` input, as tested). Already measured at 1.9× real time in WASM on one
   thread: a 3-second line in about 1.5 s. Good enough to build everything around it. Replaced
   in 0.20 by ONNX Runtime Web on the edited model (about twice as fast), which is now the
   baseline and the reference the hand-written engine is tested against.
2. **Hand-written inference** (about 1–2 weeks). A build script (`tools/voice/export.py`) reads
   the ONNX initialisers into one flat file (int8 weights with per-channel scales, float norms;
   about 9 MB). Rust implements the layer types the graph uses: embedding, the ALBERT-style text
   encoder (attention, layer norm, GELU), 1-D convolutions and transposed convolutions,
   bidirectional LSTMs (6), instance norm with style (AdaIN), leaky ReLU, upsampling, the
   harmonic sine source driven by the pitch curve, and the inverse STFT (magnitude and phase
   to audio). Shapes are known from the token count, so all buffers are allocated once and
   reused; matrix kernels use WASM SIMD128.

**Why our own engine pays off:**

- **Speed:** native ONNX Runtime already reaches 12–14×; a fixed-architecture engine with SIMD
  should reach **8× or more in the browser on one thread** (an estimate to confirm), i.e. a
  3-second line in under 0.4 s.
- **Size:** the WASM should be well under 1.5 MB (estimate), against about 20 MB for the `tract`
  test build.
- **Voice controls built in:** pitch (scale the pitch curve), speed (scale durations) and depth
  (shipped in 0.20 as a resample of the output, see "Voices from one model"). The model's own
  inverse STFT has only 11 bins, 1.2 kHz apart (n_fft 20), too coarse to move formants, so the
  earlier plan to warp the spectrum before it was dropped.
- **Deterministic audio:** the noise comes from the same position hash as in the edited model
  (done in 0.20 for ONNX Runtime Web), so the same line and voice always give the same clip.
  That makes clips cacheable and lets tests compare against golden output.
- **Same architecture as Kokoro-82M,** so the engine can later run Kokoro's CPU path too, with a
  style vector input instead of a baked-in voice.

**Tests:** golden tests run the same phoneme ids through the engine and through ONNX Runtime
(or `tract`) on the edited model, whose noise is now a deterministic hash an engine can reproduce
exactly: durations and pitch curves must match exactly, audio within a set signal-to-noise
ratio. A benchmark test fails if real-time factor regresses.

If the hand-written engine stalls, ONNX Runtime Web stays.

### KittenTTS nano (teens and grown-ups, 0.20)

[KittenTTS](https://github.com/KittenML/KittenTTS) nano 0.8 (Apache-2.0, KittenML) is a
StyleTTS2-style model like Kokoro's, about 15M parameters, with **eight voices**: Bella, Luna,
Rosie and Kiki (women), Jasper, Bruno, Hugo and Leo (men). A voice is a table of 256-number style
vectors, one per text length, fed as the `style` input, so two voices can be **mixed** linearly.
Its makers call the 0.8 ONNX models "legacy" (their KittenTTS 2 is 1.7B parameters); they are
frozen but fine for us.

- **Download and edits (`cargo fetch-voice`, `model.mjs`, `npz.mjs`):** the fp32 model
  (`kitten-tts-nano-0.8-fp32`, pinned revision and SHA-256) has its large float weights stored as
  int8 with one float scale per output channel and `DequantizeLinear` (`quantizeWeights`), so it
  is 15.5 MB but computes in float. (The upstream int8 build uses dynamic quantisation,
  `ConvInteger`/`MatMulInteger`, which ONNX Runtime Web's WASM runs *slower* than float: 2.2–2.6×
  real time against 3.0–3.7×, and it scored lower.) Then deterministic noise and a `pitch`
  input. KittenTTS's decoder reads the pitch curve inside an `If` subgraph, so the pitch is
  spliced by renaming the curve's producer and giving the scaled curve the old name: ×0.85 and ×1.15 move the pitch by the same factors and keep the timing.
  `voices.npz` becomes `kitten-nano-0.8-voices.f32`, the first 128 rows of the eight voices as raw
  floats (1 MB; a line longer than 127 characters uses the last row). All pinned by SHA-256, Node
  only.
- **Input (`crates/voice/src/kitten.rs`):** KittenTTS was trained on espeak-ng's IPA (GPL). Misaki
  was made from espeak's output by a fixed substitution table, so we apply it backwards
  (`misaki_to_espeak`: `A`→`eɪ`, `I`→`aɪ`, `O`→`oʊ`, `ʤ`→`dʒ`, `T`→`ɾ`, `ɜɹ`→`ɜː`, `əɹ`→`ɚ`, …) and put
  back the vowel length marks Misaki drops (`iː`, `uː`, `ɑː`, `ɔː`; final unstressed `i` short).
  Over all 1,026 game lines this differs from espeak-ng's own output in 2.6 % of the sounds, the
  worst being espeak's mistakes ("Mmm" read as "M-M-M", "Brr" as "B-R-R"), and the voices score
  the same with either. Then KittenTTS's own tokenisation: words and punctuation split and joined
  with spaces, its 178-symbol table, `0 … ids … 10 0`. `lines.rs` fails on any game line with a
  symbol KittenTTS doesn't know.
- **Output:** the last 5,000 samples are dropped (as KittenTTS's code does) and the silence
  around the speech trimmed (its clips start with up to 0.8 s of silence and a soft breath:
  10 ms frames, threshold 5 % of the loudest frame's RMS, 50 ms kept before and 100 ms after),
  then resized by depth. In Rust (`kitten::trim`), so every browser gets the same
  samples.
- **Speed:** about 2–3× real time on one thread, 0.5–1.3 s for a typical line in Node and Chrome
  on an M3 Pro: about half Paradee's speed (KittenTTS is twice the size). See "Guards while
  playing" for slower computers.

Measured over three lines per voice (UTMOS, a predictor of listeners' scores, 1–5; Whisper
understood every line of every voice): Paradee (0.20) scored 3.95–4.06 for women and 2.5–3.2 for
its pitched-down men (2.9–3.5 with depth); KittenTTS's men 4.1–4.2 and women 3.4 (Kiki) to 4.25.
A speaker-recognition model (WeSpeaker ResNet34) heard all of Paradee's residents as nearly the
same person (similarity 0.92 within a sex, 0.81 across; one real person scores about 0.6–0.8);
KittenTTS's mixes 0.56 within a sex and 0.16 across.

### Voices (KittenTTS)

Each resident gets a voice derived from their **stable resident id** (release 0.7), gender and
life stage, computed on the main thread (`voiceFor`); a voice chosen in the creator is saved on
top (see "A resident's own voice"):

```ts
interface VoiceParams { mix: [a, b, w]; pitch: number; speed: number; depth: number }
```

**Who speaks with what:** everyone with KittenTTS. Children have a mix of the women's voices
(KittenTTS has no child's voice), raised and made smaller; in 0.20 they spoke with Paradee
instead, because a woman's KittenTTS voice scaled up scored worse (UTMOS 2.4), and that trade was
made for 0.1.0 to keep a single model. Babies don't speak (the creator plays a baby as the
child they will be).

**A resident's mix:** a main voice and a second one, `w` of the main (0.5–1, from the
id): men from the men's voices, women from the women's, other genders from all eight, children
from the women's (a voice chosen in the creator is theirs from their teens). Kiki, very
high (about 290 Hz) and the least natural, is never the main voice, only a second one (or chosen
in the creator). Each voice is evened out:

| Voice | Sex | Measured rate (syllables/s at speed 1) | `pace` (to 4.5/s) | `pitch` |
|---|---|---|---|---|
| Bella | f | 2.95 | 1.53 | 1 |
| Luna | f | 4.04 | 1.11 | 1 |
| Rosie | f | 4.08 | 1.10 | 1 |
| Kiki | f | 4.82 | 0.93 | 1 |
| Jasper | m | 4.45 | 1.01 | 0.90 |
| Bruno | m | 4.46 | 1.01 | 1 |
| Hugo | m | 4.22 | 1.07 | 0.90 |
| Leo | m | 3.43 | 1.31 | 0.96 |

Jasper and Hugo are light voices (about 170 Hz) and sound more like men a little lower; Bruno
(108 Hz) and Leo lose naturalness when lowered (UTMOS −0.3 at ×0.9), so they stay. On top: pitch
0.94–1.06 (teens 1.04–1.12, children 1.12–1.30), depth 0.97–1.03 (teens 1.03–1.07, children
1.05–1.12), speed 0.88–1.12 (children ×1.04), and the elders' factors (women's pitch ×0.93 and
depth ×0.98, men's ×1.04 and ×1.02, everyone's speed ×0.92). Measured on six men and six women:
median pitch 113–167 Hz for men, 192–219 Hz for women; UTMOS 3.8–4.2. Children's voices (about
250–300 Hz) are not measured yet.

**Depth (the voice's size).** Pitching a woman's voice down leaves her vocal tract the same size,
so men used to sound like a pitched-down woman. Depth α scales the formants: the model runs at
pitch P/α and speed s/α, and its output is read at a step of α (band-limited, a Kaiser-windowed
sinc with 16 zero crossings; when α > 1 the cut-off moves below the new Nyquist frequency, so
nothing folds back). Every frequency scales by α, so the pitch lands back on P and the formants
move by α; every duration by 1/α, so the length lands back on the original (within a few per cent:
the model rounds durations to 25 ms frames). Done in the voice worker by the Rust WASM, so the
samples are the same in every browser and the clip cache keeps working; α < 1 makes the model's
part shorter, so men cost a little less. Range 0.75–1.3.

**Paradee's generated voices (0.20, removed after)** (factors on the model's voice, about
210 Hz; each resident's place in a range comes from hashes of their id, size following pitch a
little):

| Group | Pitch | Depth | Notes |
|---|---|---|---|
| Men | 0.52–0.76 (about 140–160 Hz: Paradee's pitch follows less than asked below ×0.8, and voicing gets rough) | 0.82–0.88 | |
| Women | 0.84–1.22 | 0.95–1.06 | |
| Others | 0.66–1.08 | 0.87–1.00 | |
| Teen boys | 0.68–0.90 | 0.88–0.95 | voices dropping |
| Teen girls / others | 0.94–1.22 / 0.80–1.12 | 1.00–1.06 / 0.94–1.03 | |
| Children (any gender) | 1.20–1.45 | 1.06–1.13 | speed ×1.04; not smaller, or they sound sped up |
| Elders | women ×0.93, men ×1.04 | women ×0.98, men ×1.02 | speed ×0.92 |

Speed 0.88–1.12 for everyone, and per line by tone (`withTone`: happy and angry faster and
higher, sad slower and lower; the size stays).

Measured on Paradee's samples (`tools/voice/samples.mjs`, three men and three women spanning
their ranges, two lines each): the spectral centroid of voiced frames below 5 kHz, which follows the
formants, is 16 % lower for men than for women (829 against 986 Hz; it was 3 % with pitch alone),
and over the whole band men went from brighter than women (2164 against 2052 Hz) to clearly darker
(1617 against 1922 Hz). Median pitch: men 133–151 Hz, women 185–246 Hz, the child 273 Hz.
