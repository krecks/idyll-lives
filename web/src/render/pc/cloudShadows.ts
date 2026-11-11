/**
 * Cloud shadows on PlayCanvas: slow, soft patches of shade drifting over the land by day, so the light is
 * not the same from one edge of the picture to the other and the sunlit parts read as sunlit. Only the sun
 * is dimmed under one (`shaders/clouds.ts`): the sky's light stays, so a cloud's shade is cooler than the
 * sun beside it, as a tree's is.
 *
 * The clouds are a small tiling map (`cloudCover`: fractal noise, cut off so that most of the land is in the
 * sun), made once a device and read by every lit fragment where it stands: no draw, no pass. They drift with
 * the wind (`WIND.dir`), hold still with Reduce motion and while time is frozen (`drift` is not called), and
 * fade out as the sun goes down (`light`): the moon casts none.
 */

import * as pc from './engine';
import { fbm, smoothstep } from '../../game/noise';
import { WIND } from '../shared/nature';
import { tuned } from '../tune';

/**
 * `span`: the metres the map covers before it repeats; `cells`: the noise's cells across it (a cloud's shadow
 * is a cell or two wide); `cover`: the noise's values over which a cloud thickens from none to whole (higher:
 * fewer, smaller clouds); `strength`: the share of the sun a cloud's middle takes; `speed`: the drift (m/s);
 * `at`: where the clouds are as a game opens (m, along x and z).
 */
export const CLOUDS = tuned('clouds', {
  span: 380,
  cells: 12,
  cover: [0.55, 0.68],
  strength: 0.5,
  speed: 1.3,
  at: [85, 20],
});

/** Texels a side of the map. */
const SIZE = 256;

/** How much of a cloud is over each texel of the map (a byte, tiling). */
export function cloudCover(size = SIZE): Uint8Array {
  const C = CLOUDS;
  const out = new Uint8Array(size * size);
  for (let j = 0; j < size; j++) {
    for (let i = 0; i < size; i++) {
      // (Three octaves sum to 0.875 at most: scaled to 0..1.)
      const n = fbm(((i + 0.5) / size) * C.cells, ((j + 0.5) / size) * C.cells, 733, 3, C.cells) / 0.875;
      out[j * size + i] = Math.round(255 * smoothstep(C.cover[0], C.cover[1], n));
    }
  }
  return out;
}

const blanks = new WeakMap<pc.GraphicsDevice, pc.Texture>();
const without = new WeakSet<pc.GraphicsDevice>();

/** `device`'s shaders are built without cloud shadows (a preset that has none, `QualitySettings.cloudShadows`): before its first material is made. */
export function withoutCloudShadows(device: pc.GraphicsDevice): void {
  without.add(device);
}

/** Whether `device`'s shaders read the clouds (`shaders/chunks.ts`'s patch). */
export function cloudShadowsOn(device: pc.GraphicsDevice): boolean {
  return !without.has(device);
}

/** No clouds on `device` until a renderer has some (before its first shader is built: `patchEngineChunks`). */
export function noCloudShadows(device: pc.GraphicsDevice): void {
  if (blanks.has(device)) return;
  const blank = new pc.Texture(device, { name: 'noClouds', width: 1, height: 1, format: pc.PIXELFORMAT_R8, mipmaps: false, levels: [new Uint8Array(1)] });
  blanks.set(device, blank);
  device.scope.resolve('game_cloudMap').setValue(blank);
  device.scope.resolve('game_cloud').setValue(new Float32Array(4));
}

export class CloudShadows {
  private readonly texture: pc.Texture;
  private readonly id: pc.ScopeId;
  private readonly mapId: pc.ScopeId;
  /** The map's offset (its own units, 0..1), 1 / its span and the clouds' strength now: the uniform's own array. */
  private readonly value = new Float32Array([0, 0, 1 / CLOUDS.span, 0]);
  /** How far the clouds have drifted (m). */
  private x = CLOUDS.at[0];
  private z = CLOUDS.at[1];

  constructor(app: pc.AppBase) {
    const device = app.graphicsDevice;
    noCloudShadows(device);
    this.texture = new pc.Texture(device, { name: 'cloudShadows', width: SIZE, height: SIZE, format: pc.PIXELFORMAT_R8, mipmaps: false, minFilter: pc.FILTER_LINEAR, magFilter: pc.FILTER_LINEAR, addressU: pc.ADDRESS_REPEAT, addressV: pc.ADDRESS_REPEAT, levels: [cloudCover()] });
    this.mapId = device.scope.resolve('game_cloudMap');
    this.id = device.scope.resolve('game_cloud');
    this.mapId.setValue(this.texture);
    this.place();
  }

  private place(): void {
    // (The shader reads the map at PlayCanvas's x and z, the game's z mirrored: the drift along z turned round with it.)
    this.value[0] = -this.x / CLOUDS.span;
    this.value[1] = this.z / CLOUDS.span;
    this.id.setValue(this.value);
  }

  /** The clouds drift on with the wind for `dt` ms. */
  drift(dt: number): void {
    if (dt <= 0) return;
    const d = (CLOUDS.speed * dt) / 1000;
    // (Kept within one span: the map tiles.)
    this.x = (this.x + WIND.dir[0] * d) % CLOUDS.span;
    this.z = (this.z + WIND.dir[1] * d) % CLOUDS.span;
    this.place();
  }

  /** The time of day: `day` 0 (night: none) to 1 (the sun well up). */
  light(day: number): void {
    this.value[3] = CLOUDS.strength * Math.min(1, Math.max(0, day));
    this.id.setValue(this.value);
  }

  destroy(): void {
    const device = this.texture.device;
    const blank = blanks.get(device);
    if (blank) this.mapId.setValue(blank);
    this.value[3] = 0;
    this.id.setValue(this.value);
    this.texture.destroy();
  }
}
