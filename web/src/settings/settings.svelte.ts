/**
 * Player settings, persisted per browser in localStorage. Some apply live; the ones
 * listed in `RESTART_KEYS` take effect the next time a game is started or loaded.
 */

import { isGraphicsPreset, QUALITY, type GraphicsPreset, type RenderQuality } from '../render/quality';

export interface Settings {
  /** The graphics preset (`render/quality.ts`): Medium unless chosen. */
  graphics: GraphicsPreset;
  /** Fraction of native resolution, 0.5..1. */
  resolutionScale: number;
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
  resolutionScale: 1,
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

function load(): Settings {
  try {
    const stored = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}') as Partial<Settings> & OldGraphics;
    const merged = { ...DEFAULT_SETTINGS, ...stored };
    if (stored.graphics === undefined) merged.graphics = migrateGraphics(stored);
    // Settings that are gone (bloom, tilt-shift, the quality preset and its parts, the visual style) aren't kept.
    for (const k of Object.keys(merged)) if (!(k in DEFAULT_SETTINGS)) delete (merged as Record<string, unknown>)[k];
    if (!isGraphicsPreset(merged.graphics)) merged.graphics = DEFAULT_SETTINGS.graphics;
    // Babble (0.14–0.19) is gone: anything but a language we have becomes the default.
    if (!['en'].includes(merged.voiceLanguage)) merged.voiceLanguage = DEFAULT_SETTINGS.voiceLanguage;
    return merged;
  } catch {
    return { ...DEFAULT_SETTINGS };
  }
}

export const settings: Settings = $state(load());

/** Settings are kept unless a benchmark changed them for its page alone (`unsavedSettings`). */
let persist = true;

/** From now on settings changes apply to this page only (the benchmark's own, `debug/bench.ts`). */
export function unsavedSettings(): void {
  persist = false;
}

$effect.root(() => {
  $effect(() => {
    const json = JSON.stringify(settings);
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
