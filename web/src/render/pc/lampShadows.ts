/**
 * The lamps' shadows (the preset's `lampShadows`: how many lamps cast at once; Low has none): a lamplit room's
 * things and residents shadow its lamps' light on the floor, on the walls and on each other.
 *
 * - Which lamps: the household's lamps that can cast (`RoomLight.casts`, `shared/house.ts` `lampCasts`), lit, in
 *   a room on the storey in view, nearest the camera's target first (`choose`; one that casts keeps its place
 *   until another is nearer by a margin, `LAMP_SHADOWS.keep`). A lamp's shadows fade in and out as it takes or
 *   gives up its place (`LAMP_SHADOWS.fade` seconds, the light's `shadowIntensity`).
 * - One map a lamp: a lamp that casts is a spot light (PlayCanvas's clustered lighting gives a spot light one
 *   map in its shadow atlas, a point light six), shining along its shade's way, or straight down for a lamp
 *   without one, which lights all round as before (`lamps.ts` `shape`). The map covers a cone of
 *   `LAMP_SHADOWS.fov` degrees round that way (PlayCanvas would make it twice the light's cone: 180° for a
 *   shade's 90°), wide enough for the floor of the lamp's room from most places a lamp stands (at 156°, 4.7 m
 *   round a lamp for every metre it is above the floor); past it the lamp shines unshadowed, the shadows
 *   fading out over the cone's last `LAMP_SHADOWS.edge` degrees (`shaders/roomLights.ts` `gameLampShadow`).
 *   Seen straight down a map's texels are the same size all over the floor: 1024 of them over 9.4 m for each
 *   metre of the lamp's height (2.3 cm under a ceiling lamp), filtered 3 × 3.
 * - The atlas: 2048 texels a side in four squares (`atlasSplit`), so a lamp keeps its square whatever its size
 *   on screen (PlayCanvas sorts the squares by that otherwise, and draws a map again when its square moves).
 * - What casts: the lot's things and the residents (`casters`, the renderer's list), handed to PlayCanvas's
 *   cull in place of the layers' casters. Not the house (a lamp lights only its own room, `roomLights.ts`:
 *   its walls would only shadow themselves), and no lamp that can cast: it would shadow its own light (a
 *   lantern's panes all of it), and all copies of a model are one draw (`LotContents.lampCasters`).
 * - When a map is drawn: once when its lamp takes a place, then only when something that casts changed
 *   (`moved`: a thing placed, moved or sold, the storey; a moving part, at the preset's `lampShadowRate`) or a
 *   resident in its room moved (`residents`, at that rate too). Otherwise it is kept: no draws, only the read
 *   where the lamp lights.
 *
 * The engine's part: `ShadowRendererLocal.cull` is wrapped (as the sun's is, `sunShadows.ts`) to give a lamp's
 * map its cone and its casters. Per frame (`frame`): the choice and the fades, no allocations.
 */

import * as pc from './engine';
import { localShadows } from './internals';
import type { RoomLight } from '../shared/house';
import { pickCasters } from '../shared/lampShadows';
import { tuned } from '../tune';

export const LAMP_SHADOWS = tuned('lampShadows', {
  /** The cone a lamp's map covers (degrees across), and its last degrees (from the edge in) over which the shadows fade out. */
  fov: 156,
  edge: 5,
  /** The atlas's side (texels): four maps of half of it. */
  atlas: 2048,
  /** PlayCanvas's depth bias and the receivers' offset along their normal (m) for a lamp's map. */
  bias: 0.1,
  normalBias: 0.03,
  /** How much of a lamp's light its shadow takes (1: all of it). Under 1 some is left, as light off the walls would leave it: with all of it gone a room with one lamp has shadows the night grade turns navy. */
  strength: 0.8,
  /** Seconds a lamp's shadows take to fade in or out as it takes or gives up its place. */
  fade: 0.4,
  /** A lamp that casts keeps its place until another is nearer the camera's target than this share of its distance. */
  keep: 0.7,
});

/** The warm-up's lamp (`LampShadows.warm`): this high over the lot (m), reaching this far (its map's near plane a thousandth of it). */
const WARM_HEIGHT = 30;
const WARM_RANGE = 1000;

/** A lamp's light as the shadows need it (`lamps.ts` `RoomLamp`). */
export interface ShadowLamp {
  entity: pc.Entity;
  light: pc.LightComponent;
  use: RoomLight | null;
  on: number;
}

export interface LampShadowHooks {
  /** The storey in view, and the storey and first lot row of a lot row (`LampView`). */
  storey(): number;
  storeyOf(z: number): number;
  rowOf(k: number): number;
  /** Lamp `i` is drawn as one that casts (`on`), or as it was. */
  shape(i: number, on: boolean): void;
}

interface Cast {
  /** Its lamp's place among the lamps as last placed (`placed`). */
  key: string;
  casting: boolean;
  /** Its shadows' strength (0..1, eased). */
  level: number;
  /** A redraw held back by the rate, and when its map was last drawn. */
  pending: boolean;
  last: number;
}

/** The cone and casters of the lights that have them (by PlayCanvas's scene light). */
const maps = new WeakMap<object, () => pc.MeshInstance[]>();
const patched = new WeakSet<object>();

/** Hands `app`'s local shadow renderer the cone and the casters of lights that have their own. */
function patch(app: pc.AppBase): void {
  const sr = localShadows(app);
  if (!sr || patched.has(sr)) return;
  patched.add(sr);
  const cull = sr.cull.bind(sr);
  sr.cull = (light, comp, casters) => {
    const own = maps.get(light);
    if (!own) return cull(light, comp, casters);
    // (`cull` reads the light's cone for its map's field of view, nothing else.)
    const cone = light._outerConeAngle;
    light._outerConeAngle = LAMP_SHADOWS.fov / 2;
    try {
      cull(light, comp, own());
    } finally {
      light._outerConeAngle = cone;
    }
  };
}

/** PlayCanvas's lighting set up for lamps that cast (before the first shader is built). */
export function enableLampShadows(app: pc.AppBase): void {
  patch(app);
  const lighting = app.scene.lighting;
  lighting.shadowsEnabled = true;
  lighting.shadowType = pc.SHADOW_PCF3_32F;
  lighting.shadowAtlasResolution = LAMP_SHADOWS.atlas;
  lighting.atlasSplit = [2];
  const { fov, edge } = LAMP_SHADOWS;
  const cos = (degrees: number) => Math.cos((degrees * Math.PI) / 180);
  app.graphicsDevice.scope.resolve('game_lampShadowCone').setValue(new Float32Array([cos(fov / 2 - 1), cos(fov / 2 - 1 - edge)]));
}

/** What a light is made with to cast as a lamp does (`enableLampShadows`): nothing until it is asked to. */
export const LAMP_SHADOW_LIGHT = { shadowType: pc.SHADOW_PCF3_32F, shadowUpdateMode: pc.SHADOWUPDATE_NONE, shadowResolution: LAMP_SHADOWS.atlas / 2 } as const;

export class LampShadows {
  private readonly casts: Cast[];
  /** Per lamp, each frame: how far it is from the camera's target (`Infinity`: it can't cast now), whether it casts, whether it should. */
  private readonly distance: number[];
  private readonly casting: boolean[];
  private readonly want: boolean[];
  private readonly interval: number;
  /** Lamps casting or fading (none: `frame` has nothing to do once nothing is wanted either). */
  private active = 0;

  /**
   * `lamps`: the lights (`lamps.ts`, as many as there will ever be); `count`: how many cast at once (at most the
   * atlas's four); `rate`: redraws a second for what moves (0: every change); `casters`: what casts now.
   */
  constructor(
    private readonly lamps: readonly ShadowLamp[],
    private readonly count: number,
    rate: number,
    private readonly casters: () => pc.MeshInstance[],
    private readonly hooks: LampShadowHooks,
  ) {
    this.count = Math.min(4, count);
    this.interval = rate > 0 ? 1000 / rate : 0;
    this.casts = lamps.map(() => ({ key: '', casting: false, level: 0, pending: false, last: -Infinity }));
    this.distance = lamps.map(() => Infinity);
    this.casting = lamps.map(() => false);
    this.want = lamps.map(() => false);
    for (const lamp of lamps) {
      lamp.light.shadowBias = LAMP_SHADOWS.bias;
      lamp.light.normalOffsetBias = LAMP_SHADOWS.normalBias;
      lamp.light.shadowIntensity = 0;
    }
  }

  /**
   * The lamps were placed anew (`Lamps.assign`): a lamp that cast still does, from the light that is its now,
   * and every map is drawn again.
   */
  placed(): void {
    let kept: Map<string, number> | null = null;
    for (let i = 0; i < this.casts.length; i++) {
      const c = this.casts[i];
      const key = keyOf(this.lamps[i].use);
      if (c.casting && c.key !== key) {
        (kept ??= new Map()).set(c.key, c.level);
        this.stop(i);
      }
      c.key = key;
    }
    if (kept) {
      for (let i = 0; i < this.casts.length; i++) {
        const c = this.casts[i];
        const level = c.casting || !c.key ? undefined : kept.get(c.key);
        if (level === undefined || this.active >= this.count) continue;
        this.start(i);
        c.level = level;
        this.lamps[i].light.shadowIntensity = level * LAMP_SHADOWS.strength;
      }
    }
    this.moved(true);
  }

  /**
   * Something that casts changed: every map is drawn again, at once (`urgent`: a thing placed or sold, the
   * storey) or at the rate (a moving part).
   */
  moved(urgent: boolean): void {
    if (!this.active) return;
    for (let i = 0; i < this.casts.length; i++) if (this.casts[i].casting) this.draw(i, urgent);
  }

  /** Residents moved: the maps of the lamps in `rooms` (where residents are) are drawn again, at the rate. */
  residents(rooms: ReadonlySet<number>): void {
    if (!this.active) return;
    for (let i = 0; i < this.casts.length; i++) {
      const room = this.lamps[i].use?.room;
      if (this.casts[i].casting && room && rooms.has(room)) this.draw(i, false);
    }
  }

  /** Each frame: which lamps cast (those nearest the camera's target, game space), their fades, redraws held back. */
  frame(dt: number, targetX: number, targetZ: number): void {
    const wanted = this.choose(targetX, targetZ);
    if (!wanted && !this.active) return;
    // (With time held still, the benchmark's pictures, the shadows are as they settle.)
    const step = LAMP_SHADOWS.fade > 0 && dt > 0 ? Math.min(1, dt / LAMP_SHADOWS.fade) : 1;
    const now = performance.now();
    // Those giving up their place first: a place is free once its lamp's shadows are out.
    for (let i = 0; i < this.casts.length; i++) {
      const c = this.casts[i];
      if (!c.casting || this.want[i]) continue;
      c.level = Math.max(0, c.level - step);
      if (c.level > 0) this.lamps[i].light.shadowIntensity = c.level * LAMP_SHADOWS.strength;
      else this.stop(i);
    }
    for (let i = 0; i < this.casts.length; i++) {
      const c = this.casts[i];
      if (!this.want[i]) continue;
      if (!c.casting) {
        if (this.active >= this.count) continue;
        this.start(i);
      }
      if (c.level < 1) {
        c.level = Math.min(1, c.level + step);
        this.lamps[i].light.shadowIntensity = c.level * LAMP_SHADOWS.strength;
      }
      if (c.pending && now - c.last >= this.interval) this.draw(i, true);
    }
  }

  /**
   * One frame of the warm-up (the renderer's `warmFrame`): a lamp's map drawn from high over (`x`, `z`)
   * (PlayCanvas space), looking down on the whole lot, so the shadow pass's shaders of everything that casts in
   * a lamp's light are made now and not as the first lamp comes on at dusk. Returns what ends it; the lamp's
   * place, way and reach are then the caller's to set again (`Lamps.warm`).
   */
  warm(x: number, z: number): () => void {
    const i = this.active < 4 ? this.casts.findIndex((c) => !c.casting) : -1;
    if (i < 0) return () => {};
    const { entity, light } = this.lamps[i];
    this.start(i);
    entity.setPosition(x, WARM_HEIGHT, z);
    entity.setRotation(pc.Quat.IDENTITY);
    light.range = WARM_RANGE;
    return () => this.stop(i);
  }

  /** Marks the lamps that should cast (`want`): the `count` nearest the target that can (`pickCasters`). Returns whether any. */
  private choose(tx: number, tz: number): boolean {
    const storey = this.hooks.storey();
    for (let i = 0; i < this.casts.length; i++) {
      const lamp = this.lamps[i];
      const use = lamp.use;
      this.casting[i] = this.casts[i].casting;
      this.distance[i] = Infinity;
      if (!use || !use.casts || !use.room || lamp.on <= 0 || lamp.light.intensity <= 0) continue;
      const k = this.hooks.storeyOf(use.z);
      if (k === storey) this.distance[i] = Math.hypot(use.x - tx, use.z - this.hooks.rowOf(k) - tz);
    }
    return pickCasters(this.distance, this.casting, this.count, LAMP_SHADOWS.keep, this.want);
  }

  private start(i: number): void {
    const c = this.casts[i];
    const light = this.lamps[i].light;
    c.casting = true;
    c.level = 0;
    c.pending = false;
    c.last = performance.now();
    this.active++;
    this.hooks.shape(i, true);
    maps.set(light.light, this.casters);
    light.shadowIntensity = 0;
    light.castShadows = true;
    // (Drawn in this frame: PlayCanvas gives it its square of the atlas now.)
    light.shadowUpdateMode = pc.SHADOWUPDATE_THISFRAME;
  }

  private stop(i: number): void {
    const c = this.casts[i];
    if (!c.casting) return;
    const light = this.lamps[i].light;
    c.casting = false;
    c.level = 0;
    c.pending = false;
    this.active--;
    light.castShadows = false;
    light.shadowIntensity = 0;
    light.shadowUpdateMode = pc.SHADOWUPDATE_NONE;
    maps.delete(light.light);
    this.hooks.shape(i, false);
  }

  /** Lamp `i`'s map drawn again in the next frame (`urgent`), or once the rate allows. */
  private draw(i: number, urgent: boolean): void {
    const c = this.casts[i];
    const now = performance.now();
    if (urgent || this.interval <= 0 || now - c.last >= this.interval) {
      c.pending = false;
      c.last = now;
      this.lamps[i].light.shadowUpdateMode = pc.SHADOWUPDATE_THISFRAME;
    } else {
      c.pending = true;
    }
  }
}

/** A lamp's place: the same lamp is the same key from one placing to the next (empty: no lamp). */
function keyOf(use: RoomLight | null): string {
  return use && use.kind === 'lamp' ? `${use.lamp ?? ''}@${use.x},${use.y},${use.z}` : '';
}
