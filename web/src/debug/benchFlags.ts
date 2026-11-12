/**
 * The rendering benchmark's URL flags (`?bench=<scene>`, see `bench.ts`), read once. A leaf module:
 * the renderer and the session read them without pulling in the benchmark.
 *
 * `?diag=…` works without `?bench` too (a comma list): it leaves features out of the picture to
 * measure what each costs, or freezes time for screenshots that compare pixel for pixel.
 * `albedo` and `normals` draw every lit surface as its base colour or world normal instead of
 * its shading (`render/pc/shaders/diagView.ts`), and `nosims` leaves the residents out: geometry
 * and colour compared without the lighting (the reference pictures for the PlayCanvas port). `linear` draws the lit colour as it is,
 * without tone mapping, exposure or grading: the light compared without the image processing. `notown` leaves the town view's
 * drawing out (`render/pc/townView.ts`: the backdrop, the far lots, crowns, movers, the ground's map), the far LOD as before it.
 */

import { TOWN_MAX } from '../render/shared/lens';
import { isResolution, type Resolution } from '../render/shared/resolution';

export type BenchCamera = 'orbit' | 'wide' | 'build' | 'dusk' | 'mirror' | 'mirror-still' | 'roam' | 'street' | 'town';

export const BENCH_DIAGS = ['noshadows', 'noaa', 'nonature', 'nowoods', 'nofarwoods', 'noflowers', 'nograss', 'nolamps', 'nomirrors', 'noglass', 'nonight', 'noao', 'noclouds', 'freeze', 'albedo', 'normals', 'nosims', 'linear', 'shaders', 'notown'] as const;
export type BenchDiag = (typeof BENCH_DIAGS)[number];

export interface BenchFlags {
  /** `public/bench/<scene>.json.gz`: the game state to play (`bigworld`: the village's, with a real Big world drawn: `big`). */
  scene: string;
  /** The scene's town laid out `tile × tile` times for the renderer, as a big world (`debug/tileWorld.ts`); 1: as it is. */
  tile: number;
  /** A real Big world of this kind for the renderer (`debug/bigTown.ts`; `bench=bigworld`: `big=` or 'small'); null: the scene's town. */
  big: string | null;
  /** `?quality=` (the presets in `render/quality.ts`; the renderer reads it itself). */
  quality: 'low' | 'medium' | 'high' | null;
  /** `?hour=`: the lighting held at that hour (the renderer reads it itself); null: the game's clock. */
  hour: number | null;
  cam: BenchCamera;
  /** Resolution (`render/shared/resolution.ts`): 720, 1080, 1440 or 2160, or `auto` (the player's setting); `1`, the old full scale, is 2160 (every line the window has). */
  res: Resolution | 'auto';
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
  /** With `freeze`: the camera path at this moment (s), held. With `hold=1` (and no `freeze`), the camera is held there while the game and the wind go on. */
  at: number;
  hold: boolean;
  /**
   * The orbit's radius (to `TOWN_MAX`: the town view) and angle from vertical instead of its path's (`radius=`, `beta=`), and its target moved by
   * (`tx=`, `tz=`, m): a close look at the lawn. `townCentre=1`: round the drawn world's main town (its first area's middle) instead of the home lot.
   */
  orbit: { radius: number | null; beta: number | null; tx: number; tz: number; town: boolean };
  /** `walls=up|cutaway|down`: the walls held that way (`up` from the game's distance shows the house whole, roof on); null: the game's own. */
  walls: 'up' | 'cutaway' | 'down' | null;
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
  const walls = params.get('walls');
  const camera: BenchCamera = cam === 'wide' || cam === 'build' || cam === 'dusk' || cam === 'mirror' || cam === 'mirror-still' || cam === 'roam' || cam === 'street' || cam === 'town' ? cam : 'orbit';
  const frozen = diag.has('freeze');
  return {
    scene,
    tile: num('tile', 1, 1, 6),
    big: scene === 'bigworld' ? (params.get('big') ?? 'small') : params.get('big'),
    quality: quality === 'low' || quality === 'medium' || quality === 'high' ? quality : null,
    hour: params.has('hour') && Number.isFinite(Number(params.get('hour'))) ? Number(params.get('hour')) : null,
    cam: camera,
    res: res === 'auto' || res === null ? 'auto' : res === '1' ? 2160 : isResolution(Number(res)) ? (Number(res) as Resolution) : 'auto',
    secs: num('secs', 20, 0, 600),
    warmup: num('warmup', 5, 0, 120),
    cap: num('cap', 0, 0, 1000),
    // Build mode stands still; the dusk run hurries through the evening; a frozen picture doesn't move.
    speed: frozen || camera === 'build' ? 0 : num('speed', camera === 'dusk' ? 5 : 1, 0, 5),
    hud: params.get('hud') !== '0',
    gpu: params.get('gpu') !== '0',
    at: num('at', 0, 0, 3600),
    hold: params.get('hold') === '1',
    orbit: { radius: params.has('radius') ? num('radius', 24, 2, TOWN_MAX) : null, beta: params.has('beta') ? num('beta', 0.9, 0.1, 1.5) : null, tx: num('tx', 0, -200, 200), tz: num('tz', 0, -200, 200), town: params.get('townCentre') === '1' },
    walls: walls === 'up' || walls === 'cutaway' || walls === 'down' ? walls : null,
    diag,
    label: params.get('label') ?? '',
  };
}

/** The benchmark asked for in the URL, or null (a normal game). */
export const BENCH: BenchFlags | null = parse();
