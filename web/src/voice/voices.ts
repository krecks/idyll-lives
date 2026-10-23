/**
 * How each resident sounds (docs/design/voices.md, "Voices"), all with **Supertonic 3**, which
 * speaks in ten fixed voices:
 *
 * - Teens and grown-ups: one of the bundled voices (`PRESETS`, five women's and five men's; men
 *   from the men's, women from the women's, everyone else from all), with their own pitch, size
 *   (depth) and speed on top.
 * - Children, whatever their gender: one of the higher women's voices made higher and smaller.
 * - Babies don't speak.
 * - A voice chosen in the household creator (`VoiceChoice`, kept in the appearance) can name a
 *   bundled voice; it's theirs from their teens.
 *
 * Derived from the resident id, gender and life stage, so a resident always sounds the same.
 * Plain TypeScript without imports (Node runs it for the checks and the build script).
 */

/**
 * The bundled voices: Supertonic 3's voice styles (`F1`…`M5`, fetched by `cargo fetch-voice`).
 * Names are what the creator shows (ours; the model only numbers them); `sex` picks who a voice
 * fits. Saves store the name.
 */
export interface Preset {
  name: string;
  sex: 'female' | 'male';
  /** The model's voice style. */
  style: string;
  /** The voice's median pitch, Hz (measured on the check's lines, 2026-10-12): children's voices aim for a pitch from it. */
  f0: number;
}

export const PRESETS: readonly Preset[] = [
  { name: 'Ada', sex: 'female', style: 'F1', f0: 188 },
  { name: 'Bea', sex: 'female', style: 'F2', f0: 216 },
  { name: 'Cleo', sex: 'female', style: 'F3', f0: 164 },
  { name: 'Dora', sex: 'female', style: 'F4', f0: 193 },
  { name: 'Elsie', sex: 'female', style: 'F5', f0: 158 },
  { name: 'Arlo', sex: 'male', style: 'M1', f0: 117 },
  { name: 'Ben', sex: 'male', style: 'M2', f0: 87 },
  { name: 'Cal', sex: 'male', style: 'M3', f0: 97 },
  { name: 'Dex', sex: 'male', style: 'M4', f0: 112 },
  { name: 'Ezra', sex: 'male', style: 'M5', f0: 91 },
];

/** The preset with this name, or undefined. */
export function presetByName(name: unknown): Preset | undefined {
  return typeof name === 'string' ? PRESETS.find((p) => p.name === name) : undefined;
}

/** A voice: the model's voice style and speed, and the pitch and size put onto each finished line. */
export interface VoiceParams {
  /** Supertonic's voice style (`Preset.style`). */
  style: string;
  /** Pitch, size (above 1 smaller) and speaking rate (above 1 faster): factors. */
  pitch: number;
  depth: number;
  speed: number;
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
  /** More or less expressive: Pocket TTS's temperature, no longer used (Supertonic has none). */
  expressive?: number;
  /**
   * Stands in for the resident id in the generated voice: someone made in the creator has no id
   * until they move in, and the voice heard there must be the one they keep.
   */
  seed?: number;
  /** A bundled voice (`PRESETS` name) as a teen's or grown-up's voice; missing: the generated one. */
  preset?: string;
  /** A player's recording for Pocket TTS to speak like: no longer used (Supertonic doesn't clone voices); such residents get the generated voice. */
  recording?: unknown;
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

/** How far a chosen voice may stray from the generated one: about two semitones, a little faster or slower, larger or smaller. */
export const VOICE_RANGE = { pitch: [0.88, 1.12], speed: [0.88, 1.12], depth: [0.92, 1.08] } as const;
/** And where it stops, so a deep elder made lower or a child made higher still sounds pleasant. */
const PITCH_LIMITS = [0.5, 1.5] as const;
const SPEED_LIMITS = [0.8, 1.2] as const;
const DEPTH_LIMITS = [0.78, 1.16] as const;

type Range = readonly [number, number];
/** A generated voice per group: pitch and depth ranges (factors on the bundled voice). */
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
 * Children of any gender: a woman's voice raised to 245–290 Hz (`Preset.f0`) and made smaller;
 * only the higher women's voices, so none is raised by more than about half.
 */
const CHILD_F0: Range = [245, 290];
const CHILD_MIN_F0 = 180;
const CHILD_DEPTH: Range = [1.05, 1.12];

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

/** The bundled voices that fit a gender (all for others), by name. */
export function presetsFor(gender: string | undefined): string[] {
  const sex = sexOf(gender);
  return PRESETS.filter((p) => sex === 'other' || p.sex === sex).map((p) => p.name);
}

/** The bundled voice a resident speaks with when nothing is chosen (children: a woman's). */
function generatedPreset(key: number, sex: Sex, child: boolean): Preset {
  const pool = PRESETS.filter((p) => (child ? p.sex === 'female' && p.f0 >= CHILD_MIN_F0 : sex === 'other' || p.sex === sex));
  return pool[Math.floor(unit(key, 4) * pool.length)];
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
  const preset = (child ? undefined : presetByName(chosenPreset(choice))) ?? generatedPreset(key, sex, child);
  let pitch: number;
  let depth: number;
  if (child) {
    pitch = lerp(CHILD_F0, p) / preset.f0;
    depth = lerp(CHILD_DEPTH, d);
  } else {
    const group = stage === 'teen' ? TEEN : ADULT;
    pitch = lerp(group.pitch, p);
    depth = lerp(group.depth, d);
  }
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
    pitch = clamp(pitch * factor(choice.pitch, VOICE_RANGE.pitch), PITCH_LIMITS);
    depth = clamp(depth * factor(choice.depth, VOICE_RANGE.depth), DEPTH_LIMITS);
    speed = clamp(speed * factor(choice.speed, VOICE_RANGE.speed), SPEED_LIMITS);
  }
  return { style: preset.style, pitch, depth, speed };
}

/** Line tone on top of the resident's voice, on the line itself (the size stays): pitch and speed. */
export function withTone(v: VoiceParams, tone: string | undefined): VoiceParams {
  const t = (pitch: number, speed: number): VoiceParams => ({ ...v, pitch: v.pitch * pitch, speed: v.speed * speed });
  switch (tone) {
    case 'happy':
      return t(1.06, 1.06);
    case 'angry':
      return t(1.04, 1.12);
    case 'sad':
      return t(0.94, 0.9);
    case 'flirty':
      return t(0.97, 0.94);
    default:
      return v;
  }
}

export { unit as hashUnit };
