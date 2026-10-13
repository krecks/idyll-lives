/**
 * Player settings, persisted per browser in localStorage. Some apply live; the ones
 * listed in `RESTART_KEYS` take effect the next time a game is started or loaded.
 */

import { QUALITY, type QualitySettings } from '../render/quality';

export type QualityPreset = keyof typeof QUALITY;

export interface Settings {
  renderer: 'auto' | 'webgl';
  /** Fraction of native resolution, 0.5..1. */
  resolutionScale: number;
  /** Frames per second at most (saves power and heat; 60 unless chosen); 0: no cap. */
  fpsCap: number;
  /** Shadow map size (1024, 2048 or 4096). */
  shadowDetail: number;
  /** 4× multisampling; off: FXAA. */
  antiAliasing: boolean;
  /** Every lamp lights (clustered lighting) rather than the four nearest the biggest rooms. */
  allLamps: boolean;
  /** The nearest mirrors reflect the room live. */
  mirrors: boolean;
  ambientOcclusion: boolean;
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

export const RESTART_KEYS: readonly (keyof Settings)[] = ['renderer', 'shadowDetail', 'antiAliasing', 'allLamps', 'mirrors', 'ambientOcclusion'];

// Legacy key from the project's old name; kept so saves survive (and settings with them).
const STORAGE_KEY = 'open-sims-wasm.settings';

export const DEFAULT_SETTINGS: Settings = {
  renderer: 'auto',
  resolutionScale: 1,
  fpsCap: 60,
  shadowDetail: QUALITY.high.shadowMapSize,
  antiAliasing: QUALITY.high.msaaSamples > 1,
  allLamps: QUALITY.high.clusteredLamps,
  mirrors: QUALITY.high.mirrors > 0,
  ambientOcclusion: QUALITY.high.ambientOcclusion,
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

function load(): Settings {
  try {
    const stored = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}') as Partial<Settings> & { quality?: QualityPreset };
    // Settings from before the quality preset was split up: what that preset set.
    const old = stored.quality && stored.shadowDetail === undefined ? QUALITY[stored.quality] : undefined;
    const preset = old ? { shadowDetail: old.shadowMapSize, antiAliasing: old.msaaSamples > 1, allLamps: old.clusteredLamps } : {};
    const merged = { ...DEFAULT_SETTINGS, ...preset, ...stored };
    // Settings that are gone (bloom, tilt-shift, the quality preset, the visual style) aren't kept.
    for (const k of Object.keys(merged)) if (!(k in DEFAULT_SETTINGS)) delete (merged as Record<string, unknown>)[k];
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

/** Settings › Performance's quick picks: each sets the image, light and shadow settings at once (`high`: the defaults). */
export const GRAPHICS_PRESETS = {
  low: { resolutionScale: 0.75, shadowDetail: QUALITY.low.shadowMapSize, antiAliasing: QUALITY.low.msaaSamples > 1, allLamps: QUALITY.low.clusteredLamps, mirrors: false, ambientOcclusion: false },
  medium: { resolutionScale: 1, shadowDetail: QUALITY.medium.shadowMapSize, antiAliasing: QUALITY.medium.msaaSamples > 1, allLamps: QUALITY.medium.clusteredLamps, mirrors: false, ambientOcclusion: false },
  high: {
    resolutionScale: DEFAULT_SETTINGS.resolutionScale,
    shadowDetail: DEFAULT_SETTINGS.shadowDetail,
    antiAliasing: DEFAULT_SETTINGS.antiAliasing,
    allLamps: DEFAULT_SETTINGS.allLamps,
    mirrors: DEFAULT_SETTINGS.mirrors,
    ambientOcclusion: DEFAULT_SETTINGS.ambientOcclusion,
  },
} satisfies Record<string, Partial<Settings>>;
export type GraphicsPreset = keyof typeof GRAPHICS_PRESETS;

/** The preset the graphics settings match, or `custom`. */
export function graphicsPreset(s: Settings): GraphicsPreset | 'custom' {
  for (const [id, preset] of Object.entries(GRAPHICS_PRESETS)) {
    if (Object.entries(preset).every(([k, v]) => s[k as keyof Settings] === v)) return id as GraphicsPreset;
  }
  return 'custom';
}

/** Renderer quality from settings; `?quality=low|medium|high|ultra` still overrides for testing. */
export function qualityFromSettings(s: Settings, params: URLSearchParams): QualitySettings {
  const preset = QUALITY[params.get('quality') as QualityPreset] as QualitySettings | undefined;
  const chosen: QualitySettings = preset ?? {
    shadowMapSize: s.shadowDetail,
    msaaSamples: s.antiAliasing ? 4 : 1,
    ambientOcclusion: s.ambientOcclusion,
    snapshotRendering: true,
    clusteredLamps: s.allLamps,
    mirrors: s.mirrors ? QUALITY.high.mirrors : 0,
  };
  return {
    ...chosen,
    snapshotRendering: chosen.snapshotRendering && params.get('snapshot') !== '0',
  };
}
