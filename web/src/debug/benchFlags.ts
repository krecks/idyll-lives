/**
 * The rendering benchmark's URL flags (`?bench=<scene>`, see `bench.ts`), read once. A leaf module:
 * the renderer and the session read them without pulling in the benchmark.
 *
 * `?diag=…` works without `?bench` too (a comma list): it leaves features out of the picture to
 * measure what each costs, or freezes time for screenshots that compare pixel for pixel.
 */

export type BenchCamera = 'orbit' | 'wide' | 'build' | 'dusk' | 'mirror' | 'mirror-still';

export const BENCH_DIAGS = ['noshadows', 'noaa', 'nonature', 'nolamps', 'nomirrors', 'noglass', 'nonight', 'freeze'] as const;
export type BenchDiag = (typeof BENCH_DIAGS)[number];

export interface BenchFlags {
  /** `public/bench/<scene>.json.gz`: the game state to play. */
  scene: string;
  /** `?quality=` (the presets in `render/quality.ts`; the renderer reads it itself). */
  quality: 'low' | 'medium' | 'high' | null;
  /** `?hour=`: the lighting held at that hour (the renderer reads it itself); null: the game's clock. */
  hour: number | null;
  cam: BenchCamera;
  /** Render scale: 0.5..1, or `auto` (the player's resolution setting). */
  res: number | 'auto';
  /** Seconds measured, after `warmup` seconds of the same camera path. */
  secs: number;
  warmup: number;
  /** Frames per second at most (`fpsCap`; 0: every frame the display shows). */
  cap: number;
  /** Game speed while measured (0 paused). */
  speed: number;
  /** The game's interface shown (`hud=0` hides it, and the summary). */
  hud: boolean;
  /** GPU time measured (`gpu=0`: not, to see what measuring costs). */
  gpu: boolean;
  /** With `freeze`: the camera path at this moment (s), held. */
  at: number;
  diag: ReadonlySet<BenchDiag>;
  /** Free text kept with the result (the runner's matrix cell). */
  label: string;
}

const params = new URLSearchParams(location.search);

/** `?diag=…`: features left out (unknown names are ignored). */
export function benchDiag(): ReadonlySet<BenchDiag> {
  const names = (params.get('diag') ?? '').split(',').map((s) => s.trim());
  return new Set(BENCH_DIAGS.filter((d) => names.includes(d)));
}

const num = (key: string, fallback: number, min: number, max: number): number => {
  const v = Number(params.get(key));
  return params.has(key) && Number.isFinite(v) ? Math.min(max, Math.max(min, v)) : fallback;
};

function parse(): BenchFlags | null {
  const scene = params.get('bench');
  if (!scene || !/^[\w-]+$/.test(scene)) return null;
  const quality = params.get('quality');
  const cam = params.get('cam');
  const diag = benchDiag();
  const res = params.get('res');
  const camera: BenchCamera = cam === 'wide' || cam === 'build' || cam === 'dusk' || cam === 'mirror' || cam === 'mirror-still' ? cam : 'orbit';
  const frozen = diag.has('freeze');
  return {
    scene,
    quality: quality === 'low' || quality === 'medium' || quality === 'high' ? quality : null,
    hour: params.has('hour') && Number.isFinite(Number(params.get('hour'))) ? Number(params.get('hour')) : null,
    cam: camera,
    res: res === 'auto' || res === null ? 'auto' : num('res', 1, 0.5, 1),
    secs: num('secs', 20, 0, 600),
    warmup: num('warmup', 5, 0, 120),
    cap: num('cap', 0, 0, 1000),
    // Build mode stands still; the dusk run hurries through the evening; a frozen picture doesn't move.
    speed: frozen || camera === 'build' ? 0 : num('speed', camera === 'dusk' ? 5 : 1, 0, 5),
    hud: params.get('hud') !== '0',
    gpu: params.get('gpu') !== '0',
    at: num('at', 0, 0, 3600),
    diag,
    label: params.get('label') ?? '',
  };
}

/** The benchmark asked for in the URL, or null (a normal game). */
export const BENCH: BenchFlags | null = parse();
