/**
 * Player settings, persisted per browser in localStorage. Some apply live; the ones
 * listed in `RESTART_KEYS` take effect the next time a game is started or loaded.
 */

import { isGraphicsPreset, QUALITY, type GraphicsPreset, type RenderQuality } from '../render/quality';
import { MIN_SCALE, type Resolution } from '../render/babylon/resolution';
import type { Shading } from '../render/types';

export interface Settings {
  /** The graphics preset (`render/quality.ts`): Medium unless chosen. */
  graphics: GraphicsPreset;
  /**
   * How lit surfaces are drawn (`render/babylon/softLook.ts`): `soft`, the game's painterly
   * shading; or `classic`, the PBR shading it replaced. Applies live.
   */
  shading: Shading;
  /** `auto`: the screen's own resolution up to about 1920×1080 pixels; or a share of it along each side, 0.5..1 (`render/babylon/resolution.ts`). */
  resolution: Resolution;
  /** Frames per second at most (saves power and heat; 60 unless chosen); 0: no cap. */
  fpsCap: number;
  /** `webgl` draws with WebGL2 even where WebGPU works (Troubleshooting). */
  renderer: 'auto' | 'webgl';
  autonomy: boolean;
  /** 0 disables autosave. */
  autosaveMinutes: number;
  cameraSensitivity: number;
  uiScale: number;
  clock24h: boolean;
  showFps: boolean;
  reducedMotion: boolean;
  /** Interface sounds (buying, building, picking things up). */
  sound: boolean;
  /** Seconds without input before the camera starts watching on its own; 0: never. */
  directorDelay: number;
  /** How often the watching camera moves on: `calm` lingers, `lively` cuts sooner. */
  directorPace: 'calm' | 'lively';
  /** Fade the interface while the camera watches on its own. */
  hideHudWhileWatching: boolean;
  /** Slow down to normal speed for big moments (a first kiss, a new job, a fight at home). */
  autoSlow: boolean;
  /** Time-lapse while everyone at home sleeps or is at work. */
  skipQuietHours: boolean;
  /** The game pauses while another window has focus (and plays on when it's back). */
  pauseOnBlur: boolean;
  /**
   * Clicking residents and the floor: `inspect` looks at a resident first (commands from their
   * panel); `always` gives orders straight away, as before.
   */
  directControl: 'inspect' | 'always';
  /** Residents speak out loud (docs/design/voices.md). Downloads the voice model when first on. */
  voices: boolean;
  /** Language residents speak (English only, for now). */
  voiceLanguage: 'en';
  /**
   * Where the voice model runs: `auto` uses the graphics card and steps down to the processor
   * when it is missing or too slow; `gpu` and `cpu` keep the player's choice. Applies live.
   */
  voiceDevice: 'auto' | 'cpu' | 'gpu';
  /** 0..1. */
  voiceVolume: number;
  /**
   * Who writes residents' lines: `lines`, the hand-written ones; `llm`, a small language model
   * in the browser, from who they are and what's going on (`talk/`; downloads the model).
   */
  voiceWriter: 'lines' | 'llm';
  /** Spoken lines also show as text in the speech bubble. */
  subtitles: boolean;
  /** Birds, crickets, and things in use at home (audio/world.ts). */
  worldSound: boolean;
  /** 0..1. */
  worldVolume: number;
  /** Every sound, 0..1. */
  masterVolume: number;
  /** Silent while another window has focus (sound always stops while the tab is hidden). */
  muteInBackground: boolean;
  /** Build mode's dock: `compact` keeps one row of the catalog (more of the lot in view), `expanded` shows several. */
  catalogSize: 'compact' | 'expanded';
  /** First-use tips already dismissed (ids from `FirstTip`). */
  tipsSeen: string[];
}

export const RESTART_KEYS: readonly (keyof Settings)[] = ['renderer', 'graphics'];

// Legacy key from the project's old name; kept so saves survive (and settings with them).
const STORAGE_KEY = 'open-sims-wasm.settings';

export const DEFAULT_SETTINGS: Settings = {
  graphics: 'medium',
  shading: 'soft',
  resolution: 'auto',
  fpsCap: 60,
  renderer: 'auto',
  autonomy: true,
  autosaveMinutes: 5,
  cameraSensitivity: 1,
  uiScale: 1,
  clock24h: true,
  showFps: false,
  reducedMotion: false,
  sound: true,
  directorDelay: 20,
  directorPace: 'calm',
  hideHudWhileWatching: false,
  autoSlow: true,
  skipQuietHours: true,
  pauseOnBlur: false,
  directControl: 'inspect',
  voices: false,
  voiceLanguage: 'en',
  voiceDevice: 'auto',
  voiceVolume: 0.8,
  voiceWriter: 'lines',
  subtitles: true,
  worldSound: true,
  worldVolume: 0.6,
  masterVolume: 1,
  muteInBackground: true,
  catalogSize: 'compact',
  tipsSeen: [],
};

/** Graphics settings before the three presets (0.1): the quality preset, then its parts set one by one. */
interface OldGraphics {
  quality?: string;
  resolutionScale?: number;
  shadowDetail?: number;
  antiAliasing?: boolean;
  allLamps?: boolean;
  mirrors?: boolean;
  ambientOcclusion?: boolean;
}

/**
 * The preset for graphics settings from before the presets were fixed: Low if they were Low (the
 * old quality preset, or the parts as the old Low set them), anything else Medium (High and the old
 * defaults too: Medium is everyone's default now).
 */
export function migrateGraphics(old: OldGraphics): GraphicsPreset {
  if (old.quality === 'low') return 'low';
  const low = old.resolutionScale === 0.75 && old.shadowDetail === 1024 && old.antiAliasing === false && old.allLamps === false && old.mirrors === false && !old.ambientOcclusion;
  return low ? 'low' : 'medium';
}

/** The version of the stored settings (`settingsVersion`); bump it when a migration is added. */
const SETTINGS_VERSION = 2;

/**
 * Ordered migrations: `MIGRATIONS[n]` takes settings at version `n` to `n + 1`. Settings saved
 * before there was a version are version 0. Each must be safe to run on settings that already
 * have the new shape (an old save may have skipped versions, and the stored object may be odd).
 */
const MIGRATIONS: readonly ((s: Record<string, unknown>) => void)[] = [
  // 0 → 1: the three graphics presets replaced 0.1's quality preset and its parts.
  (s) => {
    if (s.graphics === undefined) s.graphics = migrateGraphics(s as OldGraphics);
  },
  // 1 → 2: Babble's language codes (0.14–0.19) became the one language there is.
  (s) => {
    if (s.voiceLanguage !== 'en') s.voiceLanguage = DEFAULT_SETTINGS.voiceLanguage;
  },
];

const BOOLEAN_KEYS = [
  'autonomy',
  'clock24h',
  'showFps',
  'reducedMotion',
  'sound',
  'hideHudWhileWatching',
  'autoSlow',
  'skipQuietHours',
  'pauseOnBlur',
  'voices',
  'subtitles',
  'worldSound',
  'muteInBackground',
] as const satisfies readonly (keyof Settings)[];

/** A number in `lo..hi`, or `fallback` if it isn't one. */
function clampNumber(value: unknown, lo: number, hi: number, fallback: number): number {
  return typeof value === 'number' && Number.isFinite(value) ? Math.min(hi, Math.max(lo, value)) : fallback;
}

/** One of `options`, or `fallback` if it isn't one. */
function oneOf<T extends string>(value: unknown, options: readonly T[], fallback: T): T {
  return options.includes(value as T) ? (value as T) : fallback;
}

/**
 * Makes what the merge produced safe to use: numbers within the ranges the interface offers,
 * switches really on or off, and choices one of the ones there are. (A stored file may be from
 * another version, hand-edited, or truncated.)
 */
function sanitize(s: Settings): Settings {
  for (const k of BOOLEAN_KEYS) {
    if (typeof s[k] !== 'boolean') (s as unknown as Record<string, unknown>)[k] = DEFAULT_SETTINGS[k];
  }
  // 0 is "no cap", so it isn't below the smallest cap the interface offers.
  s.fpsCap = typeof s.fpsCap === 'number' && Number.isFinite(s.fpsCap) && s.fpsCap !== 0
    ? Math.round(Math.min(500, Math.max(10, s.fpsCap)))
    : s.fpsCap === 0 ? 0 : DEFAULT_SETTINGS.fpsCap;
  s.autosaveMinutes = clampNumber(s.autosaveMinutes, 0, 60, DEFAULT_SETTINGS.autosaveMinutes);
  s.cameraSensitivity = clampNumber(s.cameraSensitivity, 0.5, 2, DEFAULT_SETTINGS.cameraSensitivity);
  s.uiScale = clampNumber(s.uiScale, 0.5, 2, DEFAULT_SETTINGS.uiScale);
  s.directorDelay = clampNumber(s.directorDelay, 0, 600, DEFAULT_SETTINGS.directorDelay);
  s.voiceVolume = clampNumber(s.voiceVolume, 0, 1, DEFAULT_SETTINGS.voiceVolume);
  s.worldVolume = clampNumber(s.worldVolume, 0, 1, DEFAULT_SETTINGS.worldVolume);
  s.masterVolume = clampNumber(s.masterVolume, 0, 1, DEFAULT_SETTINGS.masterVolume);
  s.graphics = isGraphicsPreset(s.graphics) ? s.graphics : DEFAULT_SETTINGS.graphics;
  s.shading = oneOf(s.shading, ['soft', 'classic'] as const, DEFAULT_SETTINGS.shading);
  if (s.resolution !== 'auto' && (typeof s.resolution !== 'number' || !(s.resolution >= MIN_SCALE && s.resolution <= 1))) s.resolution = DEFAULT_SETTINGS.resolution;
  s.renderer = oneOf(s.renderer, ['auto', 'webgl'] as const, DEFAULT_SETTINGS.renderer);
  s.directorPace = oneOf(s.directorPace, ['calm', 'lively'] as const, DEFAULT_SETTINGS.directorPace);
  s.directControl = oneOf(s.directControl, ['inspect', 'always'] as const, DEFAULT_SETTINGS.directControl);
  s.voiceLanguage = oneOf(s.voiceLanguage, ['en'] as const, DEFAULT_SETTINGS.voiceLanguage);
  s.voiceDevice = oneOf(s.voiceDevice, ['auto', 'cpu', 'gpu'] as const, DEFAULT_SETTINGS.voiceDevice);
  s.voiceWriter = oneOf(s.voiceWriter, ['lines', 'llm'] as const, DEFAULT_SETTINGS.voiceWriter);
  s.catalogSize = oneOf(s.catalogSize, ['compact', 'expanded'] as const, DEFAULT_SETTINGS.catalogSize);
  if (!Array.isArray(s.tipsSeen) || s.tipsSeen.some((id) => typeof id !== 'string')) s.tipsSeen = [];
  return s;
}

function load(): Settings {
  try {
    const stored = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}') as Record<string, unknown>;
    const saved = typeof stored.settingsVersion === 'number' && Number.isInteger(stored.settingsVersion) ? stored.settingsVersion : 0;
    // A file from a newer version keeps the settings this one knows; there's no way back to
    // the ones it doesn't.
    for (let from = Math.max(0, Math.min(saved, SETTINGS_VERSION)); from < SETTINGS_VERSION; from++) {
      MIGRATIONS[from]?.(stored);
    }
    delete stored.settingsVersion;
    const merged = { ...DEFAULT_SETTINGS, ...stored } as Settings;
    // Settings that are gone (bloom, tilt-shift, the quality preset and its parts, the visual style) aren't kept.
    for (const k of Object.keys(merged)) if (!(k in DEFAULT_SETTINGS)) delete (merged as unknown as Record<string, unknown>)[k];
    return sanitize(merged);
  } catch {
    return { ...DEFAULT_SETTINGS };
  }
}

export const settings: Settings = $state(load());

/**
 * The system's reduce motion, followed as it changes: the renderer takes it with the setting
 * (`reducedMotion || …`), so what applies the settings to it reads this too, and runs again.
 */
export const systemMotion = $state({ reduced: false });
if (typeof matchMedia === 'function') {
  const query = matchMedia('(prefers-reduced-motion: reduce)');
  systemMotion.reduced = query.matches;
  query.addEventListener('change', () => (systemMotion.reduced = query.matches));
}

/** Settings are kept unless a benchmark changed them for its page alone (`unsavedSettings`). */
let persist = true;

/** From now on settings changes apply to this page only (the benchmark's own, `debug/bench.ts`). */
export function unsavedSettings(): void {
  persist = false;
}

$effect.root(() => {
  $effect(() => {
    const json = JSON.stringify({ ...settings, settingsVersion: SETTINGS_VERSION });
    if (!persist) return;
    try {
      localStorage.setItem(STORAGE_KEY, json);
    } catch {
      // Storage unavailable (private mode); settings still work for this session.
    }
  });
});

/** Back to the defaults: every setting, or just `keys` (one page of Settings). */
export function resetSettings(keys?: readonly (keyof Settings)[]): void {
  if (!keys) return void Object.assign(settings, DEFAULT_SETTINGS, { tipsSeen: [] });
  for (const k of keys) (settings as unknown as Record<string, unknown>)[k] = structuredClone(DEFAULT_SETTINGS[k]);
}

/** Renderer quality: the graphics preset's (`render/quality.ts`); `?quality=low|medium|high` overrides it for testing, `?snapshot=0` turns snapshot rendering off. */
export function qualityFromSettings(s: Settings, params: URLSearchParams): RenderQuality {
  const asked = params.get('quality');
  const preset = isGraphicsPreset(asked) ? asked : s.graphics;
  const q = QUALITY[preset];
  return { ...q, preset, snapshotRendering: q.snapshotRendering && params.get('snapshot') !== '0' };
}
