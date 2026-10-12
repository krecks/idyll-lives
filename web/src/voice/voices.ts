/**
 * How each resident sounds (docs/design/voices.md, "Voices"), all with **Pocket TTS**, which
 * speaks like a reference clip:
 *
 * - Teens and grown-ups: one of the bundled voices (`PRESETS`, twelve women's and twelve men's;
 *   men from the men's, women from the women's, everyone else from all), with their own pitch,
 *   size (depth), speed and expressiveness on top.
 * - Children, whatever their gender: a woman's voice made higher and smaller.
 * - Babies don't speak.
 * - A voice chosen in the household creator (`VoiceChoice`, kept in the appearance) can name a
 *   bundled voice or carry the player's own recording; either is theirs from their teens.
 *
 * Derived from the resident id, gender and life stage, so a resident always sounds the same.
 * Plain TypeScript without imports (Node runs it for the checks and the build script).
 */

/**
 * The bundled voices: short recordings Pocket TTS speaks like. Each is a clip from Kyutai's voice
 * repository (`kyutai/tts-voices`), only from sets whose licence lets us ship them: Voice-Zero
 * (CC0) and VCTK (CC BY 4.0; credited in `web/public/assets/CREDITS.md`), clear, natural
 * voices (two voice donations, one whispered and one creaky, were left out). `tools/voice/fetch.mjs` downloads them (pinned by SHA-256),
 * turns each into a clip as the voice encoder takes it (24 kHz, trimmed, at most 10 s, loudness
 * evened out) and packs them in this order. Names are what the creator shows (Kyutai's where it
 * has one); `sex` is the speaker's, which picks who a preset fits. Saves store the name.
 */
export interface Preset {
  name: string;
  sex: 'female' | 'male';
  /** The speaker's median pitch in the clip, Hz (pYIN, 2026-10-12): children's voices aim for a pitch from it. */
  f0: number;
  /** Path in `kyutai/tts-voices`. */
  file: string;
  sha256: string;
  licence: 'CC0-1.0' | 'CC-BY-4.0';
  /** Who to credit. */
  credit: string;
}

/** The revision of `kyutai/tts-voices` the clips come from. */
export const PRESET_REVISION = '323332d33f997de8394f24a193e1a76df720e01a';

const VCTK = 'VCTK corpus (CSTR, University of Edinburgh)';

export const PRESETS: readonly Preset[] = [
  { name: 'Anna', sex: 'female', f0: 198, file: 'vctk/p228_023_enhanced.wav', sha256: '0a6de25cf12bf1540beb85979f306a92be81fecc051c547c5395e7e5237a3856', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p228` },
  { name: 'Azelma', sex: 'female', f0: 212, file: 'vctk/p303_023_enhanced.wav', sha256: '60e3d26cdf2efdec5df712152c839928f4d5522821e6554ae11fd96c57ab1026', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p303` },
  { name: 'Caro', sex: 'female', f0: 161, file: 'voice-zero/caro_davy.wav', sha256: '40c692c005a0268a7a5b6ebae348077d3dca6a86eb6b12bd36e343bbcd71b5f6', licence: 'CC0-1.0', credit: 'Caro Davy (LibriVox, via Voice-Zero)' },
  { name: 'Eponine', sex: 'female', f0: 161, file: 'vctk/p262_023_enhanced.wav', sha256: 'a13c27fb47627b05223691a0ef2974358a18c886e6c2f9d2762ff1d02c20926b', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p262` },
  { name: 'Eve', sex: 'female', f0: 190, file: 'vctk/p361_023_enhanced.wav', sha256: '396e7cbd066b0f3fb6d67fa26e7904076958239d736d4390f15b5fe88feb14cd', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p361` },
  { name: 'Fantine', sex: 'female', f0: 203, file: 'vctk/p244_023_enhanced.wav', sha256: '5f07d4e2a3f20a15572aae885156b43ef3fc12ef3812996fd135680d9956448b', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p244` },
  { name: 'Iris', sex: 'female', f0: 181, file: 'vctk/p225_023_enhanced.wav', sha256: '10d947a5f12cc0f3b009a91d8b22c2f9e4ecd3c42d9f63ecfbe5b8facb74004c', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p225` },
  { name: 'Jane', sex: 'female', f0: 205, file: 'vctk/p339_023_enhanced.wav', sha256: '2f12e7f155eb3118f55425394f1b049e5b1b67bdc9b3932c8ba4521420aeb84a', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p339` },
  { name: 'Lucy', sex: 'female', f0: 224, file: 'vctk/p253_023_enhanced.wav', sha256: 'a2d3f18a7b520eb1c1cd02ae66cb7e3fdc577720a42828bc09b1b2ba89b689ee', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p253` },
  { name: 'Mary', sex: 'female', f0: 183, file: 'vctk/p333_023_enhanced.wav', sha256: 'a35b0468382218e9f37a9a7494d1e4b74deaf18d7ced22265b4e325bb55c183f', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p333` },
  { name: 'Nora', sex: 'female', f0: 238, file: 'vctk/p236_023_enhanced.wav', sha256: 'dc369354c36fac04cdab3cc52399602c8611f3002d4d0c7cb4ab0f1d26929211', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p236` },
  { name: 'Vera', sex: 'female', f0: 178, file: 'vctk/p229_023_enhanced.wav', sha256: '309cf91a895830f15842b398f69a4962cb1f7e0bfab10e25dd27838e826c204b', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p229` },
  { name: 'Bill', sex: 'male', f0: 128, file: 'voice-zero/bill_boerst.wav', sha256: 'be4815e4fb760ba1b78117545a260cce4a4c124c7657bc5c6127a0fef8ba661f', licence: 'CC0-1.0', credit: 'Bill Boerst (LibriVox, via Voice-Zero)' },
  { name: 'Charles', sex: 'male', f0: 79, file: 'vctk/p254_023_enhanced.wav', sha256: '6b681a429198f16e378d53bccb08d06939da7b00144a7696111d4f8f76be7756', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p254` },
  { name: 'Felix', sex: 'male', f0: 116, file: 'vctk/p243_023_enhanced.wav', sha256: 'c5b5068c991015339bddff1d6de5d68270f7ff96efd71489c1853eaeadddfef1', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p243` },
  { name: 'George', sex: 'male', f0: 118, file: 'vctk/p315_023_enhanced.wav', sha256: '29a41f93bf5236e5b21501091d7774c255d5f3d4e62fa4f9fdf0a92a793c84ae', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p315` },
  { name: 'Hugo', sex: 'male', f0: 109, file: 'vctk/p232_023_enhanced.wav', sha256: '02c719693c27f21a996e2ead6232f1385686ae301267227efc96fe309ee91031', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p232` },
  { name: 'Leon', sex: 'male', f0: 117, file: 'vctk/p251_023_enhanced.wav', sha256: '5666c909e99d9c5bd204763d9e8418fa6ff5f50c50bc1c62a3b449d500726c0b', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p251` },
  { name: 'Michael', sex: 'male', f0: 95, file: 'vctk/p360_023_enhanced.wav', sha256: 'b6743e9195e5e3fd34fe9d1633ae93f7ffab787b249e45f6467d7d6f7a6ee6ad', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p360` },
  { name: 'Oscar', sex: 'male', f0: 105, file: 'vctk/p270_023_enhanced.wav', sha256: '1ff9bcfc72038adff8f8855e2d8c92cfe8ddd61a8c9073c487444b047412044f', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p270` },
  { name: 'Owen', sex: 'male', f0: 112, file: 'vctk/p226_023_enhanced.wav', sha256: '24d12e5ffae86407fd8190448bb630c2ce60dc955ad6254dd7262d4aaf1b1f72', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p226` },
  { name: 'Paul', sex: 'male', f0: 113, file: 'vctk/p259_023_enhanced.wav', sha256: '7aba504fe0b3b16478b69eb27ce6007e3cb42b0c1915b5f1c6a6024ae37d679b', licence: 'CC-BY-4.0', credit: `${VCTK}, speaker p259` },
  { name: 'Peter', sex: 'male', f0: 87, file: 'voice-zero/peter_yearsley.wav', sha256: 'fbb3920fda7ae26a5a8b317ffcae1d55c0bd5d89d075205f5a52b1e924b83f51', licence: 'CC0-1.0', credit: 'Peter Yearsley (LibriVox, via Voice-Zero)' },
  { name: 'Stuart', sex: 'male', f0: 134, file: 'voice-zero/stuart_bell.wav', sha256: '00c7baeb2fb7a8c1c6198e045b5e853a7ccc04002a51a09b4be3dd7c96994f73', licence: 'CC0-1.0', credit: 'Stuart Bell (LibriVox, via Voice-Zero)' },
];

/** The preset with this name, or undefined. */
export function presetByName(name: unknown): Preset | undefined {
  return typeof name === 'string' ? PRESETS.find((p) => p.name === name) : undefined;
}

/**
 * A voice. Pitch and size are given in two places: put into the clip the model speaks like
 * (`clipPitch`, `clipDepth`: the model then speaks the whole line that way, but every new value
 * costs about two seconds to prepare, so only for children's voices and voices chosen in the
 * creator), and onto each finished line (`pitch`, `depth`: a resident's own small variation,
 * about a millisecond per line, so everyone sounds a little different on a few prepared clips).
 */
export interface VoiceParams {
  /** The clip the model speaks like: `preset:<name>` or `rec:<sha>` (a recording). */
  clip: string;
  /** Pitch and size (above 1 smaller) put into the clip: factors on it. */
  clipPitch: number;
  clipDepth: number;
  /** Pitch, size and speaking rate (above 1 faster) given to each line. */
  pitch: number;
  depth: number;
  speed: number;
  /** Expressiveness: the model's sampling temperature (0 flat, about 0.7 natural, 1 lively). */
  temperature: number;
}

/** A player's recording (or a sound file they chose), kept in the appearance so it travels with the save. */
export interface Recording {
  /** The sound, base64 (compressed by the browser: Opus where it can). */
  audio: string;
  /** Its MIME type. */
  type: string;
  /** SHA-256 of `audio` (hex, the first 16 digits): names the clip. */
  sha: string;
}

/**
 * A voice chosen in the household creator (`appearance.voice`): factors on the generated voice,
 * 1 (or missing) being as generated, so it still changes as the resident grows up. Residents
 * without one (all older saves, neighbours, newcomers, babies) have the generated voice. Fields
 * are only ever added, so older choices load as they were.
 */
export interface VoiceChoice {
  /** Higher or lower than the generated voice (`VOICE_RANGE.pitch`). */
  pitch?: number;
  /** Faster or slower (`VOICE_RANGE.speed`). */
  speed?: number;
  /** Larger (below 1) or smaller (above 1) than the generated voice (`VOICE_RANGE.depth`). */
  depth?: number;
  /** More or less expressive (`VOICE_RANGE.expressive`, a factor on the temperature). */
  expressive?: number;
  /**
   * Stands in for the resident id in the generated voice: someone made in the creator has no id
   * until they move in, and the voice heard there must be the one they keep.
   */
  seed?: number;
  /** A bundled voice (`PRESETS` name) as a teen's or grown-up's voice; missing: the generated one. */
  preset?: string;
  /** The player's own recording as a teen's or grown-up's voice (wins over `preset`). */
  recording?: Recording;
  /** A KittenTTS voice (0.20 to 0.1.0, by index): no longer used; such residents get the generated voice. */
  base?: number;
}

/** 0..1 from an integer and a salt (a small integer hash, stable across sessions). */
function unit(id: number, salt: number): number {
  let h = (id * 0x9e3779b1) ^ (salt * 0x85ebca6b);
  h = Math.imul(h ^ (h >>> 16), 0x7feb352d);
  h = Math.imul(h ^ (h >>> 15), 0x846ca68b);
  h ^= h >>> 16;
  return (h >>> 0) / 4294967296;
}

/** How far a chosen voice may stray from the generated one: about two semitones, a little faster or slower, larger or smaller, flatter or livelier. */
export const VOICE_RANGE = { pitch: [0.88, 1.12], speed: [0.88, 1.12], depth: [0.92, 1.08], expressive: [0.6, 1.4] } as const;
/** And where it stops, so a deep elder made lower or a child made higher still sounds pleasant. */
const PITCH_LIMITS = [0.5, 1.5] as const;
const SPEED_LIMITS = [0.8, 1.2] as const;
const DEPTH_LIMITS = [0.78, 1.16] as const;
const TEMPERATURE_LIMITS = [0.2, 1] as const;

type Range = readonly [number, number];
/** A generated voice per group: pitch and depth ranges (factors on the clip's own voice). */
interface Group {
  pitch: Range;
  depth: Range;
}
type Sex = 'male' | 'female' | 'other';

/** The bundled voices are real men's and women's: small spreads around them. */
const ADULT: Group = { pitch: [0.94, 1.06], depth: [0.97, 1.03] };
/** Teens: the same voices a little higher and smaller. */
const TEEN: Group = { pitch: [1.04, 1.12], depth: [1.03, 1.07] };
/**
 * Children of any gender: a woman's voice raised to 245–290 Hz, whichever she is (`Preset.f0`),
 * and made smaller. Most of it goes into the clip (to 265 Hz and size 1.08, one clip per voice:
 * the model follows it, a woman's clip at pitch 1.25 and size 1.15 came out at 288 Hz from 197,
 * measured 2026-10-12), the rest onto each line.
 */
const CHILD_F0: Range = [245, 290];
const CHILD_CLIP = { f0: 265, depth: 1.08 } as const;
const CHILD_DEPTH: Range = [1.05, 1.12];
/** Expressiveness (temperature) of generated voices; the model's reference uses 0.7. */
const TEMPERATURE: Range = [0.55, 0.8];

const clamp = (v: number, [lo, hi]: Range) => Math.min(hi, Math.max(lo, v));
const lerp = ([lo, hi]: Range, t: number) => lo + (hi - lo) * t;
/** A stored factor, or 1 for a missing or broken one (the appearance is free-form JSON). */
const factor = (v: unknown, range: Range) => (typeof v === 'number' && Number.isFinite(v) ? clamp(v, range) : 1);
const sexOf = (gender: string | undefined): Sex => (gender === 'male' || gender === 'female' ? gender : 'other');

/** Whether a life stage has a child's voice (babies don't speak; they're voiced as the child they'll be). */
export function childVoice(stage: string | undefined): boolean {
  return stage === 'child' || stage === 'baby';
}

/** The bundled voice a stored choice names, if it's one we have. */
export function chosenPreset(choice: VoiceChoice | null | undefined): string | null {
  return presetByName(choice?.preset)?.name ?? null;
}

/** A stored recording, if it's whole. */
export function chosenRecording(choice: VoiceChoice | null | undefined): Recording | null {
  const r = choice?.recording;
  return r && typeof r.audio === 'string' && r.audio && typeof r.type === 'string' && typeof r.sha === 'string' && r.sha ? r : null;
}

/** The bundled voices that fit a gender (all for others), by name. */
export function presetsFor(gender: string | undefined): string[] {
  const sex = sexOf(gender);
  return PRESETS.filter((p) => sex === 'other' || p.sex === sex).map((p) => p.name);
}

/** The bundled voice a resident speaks with when nothing is chosen (children: a woman's). */
function generatedPreset(key: number, sex: Sex, child: boolean): string {
  const pool = PRESETS.filter((p) => (child ? p.sex === 'female' : sex === 'other' || p.sex === sex));
  return pool[Math.floor(unit(key, 4) * pool.length)].name;
}

/** The resident's voice at this life stage. */
export function voiceFor(id: number, gender: string | undefined, stage?: string, choice?: VoiceChoice | null): VoiceParams {
  const seed = choice?.seed;
  const key = typeof seed === 'number' && Number.isInteger(seed) ? seed : id;
  const p = unit(key, 1);
  // Size follows pitch a little (lower voices tend to be larger), with its own spread.
  const d = 0.65 * unit(key, 3) + 0.35 * p;
  let speed = 0.88 + unit(key, 2) * 0.24;
  const sex = sexOf(gender);
  // Babies don't speak (they're heard as the child they'll be); children sound alike whatever their gender.
  const child = childVoice(stage);
  const recording = child ? null : chosenRecording(choice);
  const preset = child ? null : chosenPreset(choice);
  const name = preset ?? generatedPreset(key, sex, child);
  const clip = recording ? `rec:${recording.sha}` : `preset:${name}`;
  let clipPitch = 1;
  let clipDepth = 1;
  let pitch: number;
  let depth: number;
  if (recording) {
    // Someone's own voice: it starts as it is, not moved within a group.
    pitch = depth = 1;
  } else if (child) {
    clipPitch = clamp(CHILD_CLIP.f0 / (presetByName(name)?.f0 ?? 200), [1.05, 1.5]);
    clipDepth = CHILD_CLIP.depth;
    pitch = lerp(CHILD_F0, p) / CHILD_CLIP.f0;
    depth = lerp(CHILD_DEPTH, d) / clipDepth;
  } else {
    const group = stage === 'teen' ? TEEN : ADULT;
    pitch = lerp(group.pitch, p);
    depth = lerp(group.depth, d);
  }
  let temperature = lerp(TEMPERATURE, unit(key, 5));
  switch (stage) {
    case 'child':
    case 'baby':
      speed *= 1.04;
      break;
    case 'elder':
      // Older women's voices drop; older men's rise a little and thin out; everyone slower.
      pitch *= sex === 'male' ? 1.04 : 0.93;
      depth *= sex === 'male' ? 1.02 : 0.98;
      speed *= 0.92;
      break;
  }
  if (choice) {
    // A chosen pitch and size go into the clip, so the model speaks that way.
    clipPitch = clamp(clipPitch * factor(choice.pitch, VOICE_RANGE.pitch), PITCH_LIMITS);
    clipDepth = clamp(clipDepth * factor(choice.depth, VOICE_RANGE.depth), DEPTH_LIMITS);
    speed = clamp(speed * factor(choice.speed, VOICE_RANGE.speed), SPEED_LIMITS);
    temperature = clamp(temperature * factor(choice.expressive, VOICE_RANGE.expressive), TEMPERATURE_LIMITS);
  }
  return { clip, clipPitch, clipDepth, pitch, depth, speed, temperature };
}

/** Line tone on top of the resident's voice, on the line itself (the size stays): pitch, speed and expressiveness. */
export function withTone(v: VoiceParams, tone: string | undefined): VoiceParams {
  const t = (pitch: number, speed: number, temperature: number): VoiceParams => ({
    ...v,
    pitch: v.pitch * pitch,
    speed: v.speed * speed,
    temperature: clamp(v.temperature * temperature, TEMPERATURE_LIMITS),
  });
  switch (tone) {
    case 'happy':
      return t(1.06, 1.06, 1.15);
    case 'angry':
      return t(1.04, 1.12, 1.25);
    case 'sad':
      return t(0.94, 0.9, 0.8);
    case 'flirty':
      return t(0.97, 0.94, 1.1);
    default:
      return v;
  }
}

export { unit as hashUnit };
