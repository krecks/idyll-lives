/**
 * The viewed house's interior lights on PlayCanvas: placing them, keeping them in their rooms,
 * their fades and the glows.
 *
 * - The lights: the household's lamps first, then a dim fill from the ceiling of each room without
 *   one (`placeLights`, `shared/house.ts`). Clustered lights (PlayCanvas's clustered lighting, the
 *   only kind on WebGPU): as many as the preset allows (`clusteredLamps`: every lamp, up to 32;
 *   otherwise the four the biggest rooms get first), each a point light, or a spot light for a
 *   lamp whose shade sends its light one way (`RoomLight.dir`). Always there and always enabled; by
 *   day (and in the menus' town) dark: a light of no intensity is left out of the clusters, and
 *   switching lights never compiles a shader. `?lights=forward` (WebGL2 only) draws them as forward
 *   lights instead (all of them points, so their shaders never change).
 * - Lamps light their own room (`shaders/roomLights.ts`): the rooms of the lot in view, a texel per
 *   tile on every storey shown (`markRooms`, `RoomLightState`); `?roomlights=0` turns it off to
 *   compare. With four lights (the low preset) they aren't kept in (`many`): so few light the
 *   house anyway.
 * - Light units: in the look's units a lamp of strength `I` lights a surface to `albedo / π · I /
 *   d²` (the physical falloff; a shade's cone as a spherical Gaussian), PlayCanvas to `albedo · I ·
 *   falloff`, and the game's materials take the physical falloff (`roomLights.ts`): the look's
 *   strengths / π (`LAMP_UNITS`, as `SUN_UNITS` in `lightUnits.ts`). The falloff reaches zero at the
 *   light's range there (PlayCanvas's window), while the look was tuned with lamps lit `1/d²` to
 *   well past their reach: the range is set twice as far (`REACH`), so a lamp lights its room as far
 *   as it did when the look was tuned (within a few per cent, measured on the bench's night poses)
 *   and fades out beyond. Four lights end at their range (the glTF falloff).
 * - Fades: a lamp switched off or on fades in a quarter of a second (`setLampsOff`); a room's fill
 *   goes out over a second or so as everyone in it falls asleep (or a windowless room empties), and
 *   its windows' glow with it, or with its lamps all switched off (`windowGlowOn`). In the basement
 *   and in rooms without windows the lights are on by day too, as bright as at night.
 * - The glows: the shades' and fires' (`LotContents.applyGlow`, through `glow`), the windows' by
 *   room (`HouseBuilder.setWindowGlow`) and the garden's pools (the renderer's, `hooks.pools`).
 *
 * Per frame (`frame`): the fades, no allocations; lights and glows change only when one moved.
 */

import * as pc from './engine';
import type { HouseBuilder } from './house';
import type { GlowLight, LotContents } from './lotContents';
import { MaterialLibrary, type RoomMask } from './materials';
import { benchDiag } from '../../debug/benchFlags';
import type { WorldStructure } from '../../core/protocol';
import { LOOK } from '../look';
import type { RenderQuality } from '../quality';
import type { Lighting, RGB } from '../shared/environment';
import type { RoomLight } from '../shared/house';
import { WALL_HEIGHT } from '../shared/walls';
import type { ViewRect } from '../types';

/** Interior lights without clustered lamps (the preset's `clusteredLamps` off), and with. */
const ROOM_LIGHTS = 4;
const CLUSTERED_LIGHTS = 32;
/** Brightness of a lamp at night (times its content `intensity`), in the look's units. */
const LAMP_POWER = 7;
/** A room without a lamp gets this share of the light it had before lamps gave light. */
const FILL_POWER = 0.35;
/**
 * In the look's units a lamp of strength 1 lights a white surface a metre away to `1 / π` (a
 * Lambert surface, with the physical `1/d²` falloff the game's materials take), PlayCanvas to 1:
 * the look's lamp strengths / π (`lightUnits.ts`).
 */
const LAMP_UNITS = 1 / Math.PI;
/**
 * Furthest reach (m) of a light, the largest bought lamp's range: room fills of big rooms reach no
 * further.
 */
const LAMP_MAX_RANGE = 7;
/** Range of a clustered light over its reach (see the file's header; forward lights end at the range). */
const REACH = 2;
/** A shade's cone (degrees from its direction): full along it, about a quarter at 45°, next to none side-on (`roomLights.ts`). */
const SHADE_CONE = 90;
/** Seconds over which a room's lights go out (or come back on) as its residents fall asleep (or wake). */
const LIGHTS_OUT_FADE = 1.2;
/** Seconds a lamp takes to go off or on at its switch. */
const LAMP_SWITCH_FADE = 0.25;
/** The lights' exposure at night (the look's brightest): a light on by day is as bright as at night. */
const NIGHT_EXPOSURE = Math.max(...LOOK.keys.map((k) => k.exposure));

/** The lot as the renderer has it now. */
export interface LampView {
  /** The storey in view (-1: the basement). */
  storey(): number;
  /** The storey a lot row is on. */
  storeyOf(z: number): number;
  /** The first lot row of storey `k`. */
  rowOf(k: number): number;
  /** The menus' town is shown (the lamps are dark behind it). */
  town(): boolean;
}

/** What the lamps light besides the lot's surfaces. */
export interface LampHooks {
  house: HouseBuilder;
  lot: LotContents;
  /** The garden's pools of lamplight again (their lamps' fades: `Lamps.lampOn`). */
  pools(): void;
}

/** One interior light: its entity, what it lights (null: unused), and how far on (1 on, 0 out). */
interface RoomLamp {
  entity: pc.Entity;
  light: pc.LightComponent;
  use: RoomLight | null;
  on: number;
}

export class Lamps {
  /** Whether the lights are clustered (else forward lights: `?lights=forward` on WebGL2). */
  readonly clustered: boolean;
  /**
   * Every lamp lights, each in its own room (the preset's `clusteredLamps`); else the four lights
   * the biggest rooms get first light past their rooms' walls and to their range (a few lamps light
   * the house anyway).
   */
  private readonly many: boolean;
  private readonly lib: MaterialLibrary;
  private readonly lamps: RoomLamp[] = [];
  private readonly components: pc.LightComponent[] = [];
  /** The lights of the viewed house as placed (before the lamps go round them). */
  private placed: RoomLight[] = [];
  /** More lights than lamps to go round: lamps switched off give theirs up (`assign`). */
  private short = false;
  /** The lamps switched off (object ids; sim-core `lights.rs`), and since when nothing fades. */
  private lampsOff: ReadonlySet<number> | null = null;
  private settled = true;
  /** Per lamp going off, off or coming on (object id): 1 on, 0 off, eased between; unlisted lamps are on. */
  private readonly lampFade = new Map<number, number>();
  /** The viewed lot's lamps (all of them, also those without a light to go round) by room. */
  private lotLamps: RoomLight[] = [];
  /** Rooms whose lamps are all switched off: their windows go dark as when everyone is asleep. */
  private readonly unlitRooms = new Set<number>();
  /** Per room whose windows are dimmed: 1 glowing, 0 out, eased between; unlisted rooms glow. */
  private readonly windowGlowOn = new Map<number, number>();
  /** Rooms without windows on the storey in view (lit by day too). */
  private dimmed: ReadonlySet<number> = new Set();
  /** Rooms where everyone is asleep and rooms with someone awake, this frame (the renderer's). */
  private dark: ReadonlySet<number> = new Set();
  private awake: ReadonlySet<number> = new Set();
  /** Night level of the lamps at the last lighting update (`Lighting.lamps`), and the exposure then. */
  private lampsLevel = 0;
  private exposure = 1;
  /** How the shades glow (`LotContents.applyGlow`): one object, its level updated. */
  private readonly glow: GlowLight;
  // Scratch for the fades (`forEach` with these, so the per-frame fades allocate nothing).
  private step = 0;
  private changed = false;
  private readonly vTmp = new pc.Vec3();
  private readonly qTmp = new pc.Quat();
  /** The lights or the glows changed (the mirrors' held reflections are drawn again: `mirrors.ts`). */
  onChanged: (() => void) | null = null;

  constructor(
    app: pc.AppBase,
    quality: RenderQuality,
    private readonly view: LampView,
    private readonly hooks: LampHooks,
  ) {
    this.lib = MaterialLibrary.for(app);
    const params = new URLSearchParams(location.search);
    const wanted = params.get('lights');
    const scene = app.scene;
    // (Before anything is drawn: switching recompiles every shader. WebGPU has only clustered lights.)
    if (wanted === 'forward' && !app.graphicsDevice.isWebGPU) scene.clusteredLightingEnabled = false;
    this.clustered = scene.clusteredLightingEnabled;
    this.many = this.clustered && (wanted === 'clustered' || (wanted !== 'forward' && quality.clusteredLamps));
    const count = this.many ? CLUSTERED_LIGHTS : ROOM_LIGHTS;
    // Lamps cast no shadows and have no cookies or shapes: the clusters' shaders without them.
    const lighting = scene.lighting;
    lighting.shadowsEnabled = false;
    lighting.cookiesEnabled = false;
    lighting.areaLightsEnabled = false;
    lighting.maxLightsPerCell = count;
    // The look's lamp colour (sRGB, as PlayCanvas takes a light's colour).
    const c = linearHex(LOOK.lampColor);
    const color = new pc.Color(gamma(c.r), gamma(c.g), gamma(c.b));
    const off = benchDiag().has('nolamps');
    for (let i = 0; i < count; i++) {
      const entity = new pc.Entity(`roomLight${i}`);
      entity.addComponent('light', {
        type: 'omni',
        color,
        intensity: 0,
        range: 1,
        falloffMode: pc.LIGHTFALLOFF_INVERSESQUARED,
        // (No inner cone: the physical cone, `roomLights.ts`; with four lights the glTF one.)
        innerConeAngle: this.many ? SHADE_CONE : 0,
        outerConeAngle: SHADE_CONE,
        castShadows: false,
        // Only meshes that keep the dynamic bit (`MaterialLibrary.excludeFromLights`).
        mask: pc.MASK_AFFECT_DYNAMIC,
      });
      entity.enabled = !off;
      app.root.addChild(entity);
      this.lamps.push({ entity, light: entity.light!, use: null, on: 1 });
      this.components.push(entity.light!);
    }
    // Each clustered lamp lights only its own room (`?roomlights=0` turns it off to compare).
    this.lib.rooms.enabled = this.many && params.get('roomlights') !== '0';
    this.glow = {
      level: 0,
      lampOn: (id) => this.lampOn(id),
      roomOn: (room) => this.windowGlowOn.get(room) ?? 1,
      dimmed: (room) => this.dimmed.has(room),
    };
  }

  /** How far lamp `id` is on (1 on, 0 off, between while it fades; a light without a switch: 1). */
  lampOn(id: number | undefined): number {
    return id === undefined ? 1 : (this.lampFade.get(id) ?? (this.lampsOff?.has(id) ? 0 : 1));
  }

  /** Leaves `meshes` out of the lamps (the landscape, as it changes: `LotContents.landscapeDraws`). */
  exclude(meshes: Iterable<pc.MeshInstance>): void {
    this.lib.excludeFromLights(this.components, meshes);
  }

  /** The rooms without windows on the storey in view (`darkTiles`): their lights are on by day too. */
  setDimmed(rooms: ReadonlySet<number>): void {
    this.dimmed = rooms;
  }

  /**
   * The viewed house's lights on the storeys shown (`placeLights`, lifted to their storeys here):
   * the storey in view gets them first, then the nearest below it, lamps switched off last of all.
   */
  place(lights: RoomLight[]): void {
    this.placed = lights;
    this.lotLamps = lights.filter((l) => l.kind === 'lamp');
    this.findUnlitRooms();
    this.assign();
  }

  /** The lights of `placed` on the interior lights (see `place`), at their brightness now. */
  private assign(): void {
    const storey = this.view.storey();
    const away = (z: number) => Math.abs(this.view.storeyOf(z) - storey);
    const off = (l: RoomLight) => (l.lamp !== undefined && this.lampsOff?.has(l.lamp) ? 1 : 0);
    // (Stable: within a storey each room's first light still comes before any second.)
    const lights = [...this.placed].sort((a, b) => off(a) - off(b) || away(a.z) - away(b.z));
    this.short = lights.length > this.lamps.length;
    for (let i = 0; i < this.lamps.length; i++) {
      const lamp = this.lamps[i];
      const l = lights[i] ?? null;
      lamp.use = l;
      lamp.on = l?.lamp !== undefined ? this.lampOn(l.lamp) : l && this.fillOut(l.room) ? 0 : 1;
      if (!l) continue;
      const k = this.view.storeyOf(l.z);
      // Game space to PlayCanvas's (the world is mirrored in z: `space.ts`).
      lamp.entity.setPosition(l.x, l.y + k * WALL_HEIGHT, -(l.z - this.view.rowOf(k)));
      // A shade's side (`dir`, clustered lights only): a spot light shining along its entity's -y.
      const dir = this.clustered ? l.dir : undefined;
      lamp.light.type = dir ? 'spot' : 'omni';
      if (dir) lamp.entity.setRotation(downTo(dir[0], dir[1], -dir[2], this.vTmp, this.qTmp));
    }
    this.applyRoomLights();
  }

  /**
   * The rooms of the lot in view on every storey shown, for keeping each lamp's light in its room
   * (`RoomLightState`): 1 outdoors, 2 and up a room, 0 a tile split by a diagonal wall (lit by all).
   */
  markRooms(world: WorldStructure, view: ViewRect | null): void {
    const rooms = this.lib.rooms;
    if (!rooms.enabled || !view) {
      rooms.show(null);
      return;
    }
    const W = world.width;
    const storey = this.view.storey();
    const [k0, n] = storey < 0 ? [storey, 1] : [0, storey + 1];
    const tiles = new Uint16Array(view.w * view.d * n);
    const ids = new Map<number, number>();
    const diagonals = new Set((world.diagonals ?? []).map((d) => d.z * W + d.x));
    for (let k = 0; k < n; k++) {
      const row = this.view.rowOf(k0 + k);
      for (let z = 0; z < view.d; z++) {
        for (let x = 0; x < view.w; x++) {
          const i = (z + view.z + row) * W + x + view.x;
          if (z + view.z + row >= world.depth || diagonals.has(i)) continue;
          const r = world.rooms[i] ?? 0;
          let id = ids.get(r);
          if (id === undefined) ids.set(r, (id = r === 0 ? 1 : ids.size + 2));
          tiles[(k * view.d + z) * view.w + x] = id;
        }
      }
    }
    const mask: RoomMask = { x0: view.x, z0: view.z, w: view.w, d: view.d, storey: k0, storeys: n, tiles };
    rooms.show(mask);
  }

  /** Lamps switched off (`Renderer.setLampsOff`): those switched since the last fade out or in. */
  setLampsOff(off: ReadonlySet<number>): void {
    // The first time (a game loading) they're as they are; afterwards a lamp switched off fades out.
    const first = !this.lampsOff;
    for (const id of off) if (!this.lampFade.has(id)) this.lampFade.set(id, first ? 0 : 1);
    this.lampsOff = off;
    this.settled = false;
    this.findUnlitRooms();
    if (this.short) this.assign();
  }

  /** The time of day's lamp level (`Lighting.lamps`) and exposure: the lights and the glows. */
  applyLighting(l: Lighting): void {
    this.lampsLevel = l.lamps;
    this.exposure = Math.max(l.exposure, 1e-3);
    this.applyRoomLights();
    this.applyGlow();
  }

  /**
   * The fades, each frame (`dt` in seconds; `dark` and `awake`: the rooms where everyone is asleep,
   * and those with someone awake, the renderer's `findDarkRooms`).
   */
  frame(dt: number, dark: ReadonlySet<number>, awake: ReadonlySet<number>): void {
    this.dark = dark;
    this.awake = awake;
    const switched = this.fadeLamps(dt);
    if (this.fadeRoomLights(dt) && (this.lampsLevel > 0 || this.view.storey() < 0 || this.dimmed.size > 0)) this.applyRoomLights();
    if ((this.fadeWindowGlow(dt) || switched) && (this.lampsLevel > 0 || this.dimmed.size > 0)) {
      this.hooks.house.setWindowGlow(this.windowGlowOn);
      this.applyGlow();
    }
  }

  /** The world is gone (`Renderer.clear`): no lights, no rooms to keep them in. */
  clear(): void {
    this.placed = [];
    this.lotLamps = [];
    this.unlitRooms.clear();
    this.windowGlowOn.clear();
    this.dimmed = new Set();
    this.lib.rooms.show(null);
    this.assign();
  }

  /** How far the shades' glow and the garden's pools are up (0..1): from dusk as the lamps come up, none in town. */
  private get glowLevel(): number {
    return this.view.town() ? 0 : Math.min(1, Math.max(0, (this.lampsLevel - 0.35) / 0.5));
  }

  /** The shades' glow (`LotContents.applyGlow`) and the garden's pools. */
  private applyGlow(): void {
    this.glow.level = this.glowLevel;
    this.hooks.lot.applyGlow(this.glow);
    this.hooks.pools();
    this.onChanged?.();
  }

  /** The interior lights at the night level of the last lighting update, out in rooms where everyone is asleep. */
  private applyRoomLights(): void {
    const town = this.view.town();
    const basement = this.view.storey() < 0;
    for (const lamp of this.lamps) {
      // The overview has no interiors: the game house's lamps stay off behind it.
      const use = town ? null : lamp.use;
      let intensity = 0;
      let range = 1;
      if (use) {
        // Down in the basement and in rooms without windows they're on by day too, as bright as at
        // night (by day the exposure is lower).
        const dark = basement || this.dimmed.has(use.room);
        const lit = (dark ? NIGHT_EXPOSURE / this.exposure : this.lampsLevel) * lamp.on;
        if (use.kind === 'lamp') {
          // A lamp the household bought: a warm pool around it.
          intensity = lit * LAMP_POWER * (use.power ?? 1);
          range = use.range ?? 4.5;
        } else {
          // A room without a lamp: only a dim glow from the ceiling.
          intensity = lit * FILL_POWER * (4 + Math.min(use.area, 30) * 0.35);
          range = 4 + Math.sqrt(use.area) * 1.4;
        }
      }
      // (A dark light, all of them by day, is left out of the clusters.)
      setIntensity(lamp.light, intensity * LAMP_UNITS);
      const reach = this.many ? Math.min(range, LAMP_MAX_RANGE) * REACH : range;
      if (lamp.light.range !== reach) lamp.light.range = reach;
    }
    this.onChanged?.();
  }

  /** A room's fill is out: everyone there is asleep, or nobody awake is in a room without windows. */
  private fillOut(room: number): boolean {
    return this.dark.has(room) || (this.dimmed.has(room) && !this.awake.has(room));
  }

  /** The viewed lot's rooms with lamps, all of them switched off. */
  private findUnlitRooms(): void {
    const lit = new Set<number>();
    this.unlitRooms.clear();
    for (const { room, lamp } of this.lotLamps) {
      if (lamp === undefined || !this.lampsOff?.has(lamp)) lit.add(room);
      else if (room) this.unlitRooms.add(room);
    }
    for (const room of lit) this.unlitRooms.delete(room);
  }

  /** Eases each lamp towards off or on (`lampsOff`); true if any changed. */
  private fadeLamps(dt: number): boolean {
    if (this.settled) return false;
    this.changed = false;
    this.step = Math.min(1, dt / LAMP_SWITCH_FADE);
    this.lampFade.forEach(this.fadeLamp);
    this.settled = !this.changed;
    return this.changed;
  }

  private readonly fadeLamp = (on: number, id: number): void => {
    const target = this.lampsOff?.has(id) ? 0 : 1;
    if (on === target && target === 0) return;
    const next = target > on ? Math.min(target, on + this.step) : Math.max(target, on - this.step);
    if (next === 1) this.lampFade.delete(id);
    else this.lampFade.set(id, next);
    this.changed = true;
  };

  /** Eases each interior light towards out (its room dark) or on, a lamp as its switch; true if any changed. */
  private fadeRoomLights(dt: number): boolean {
    let changed = false;
    const step = Math.min(1, dt / LIGHTS_OUT_FADE);
    for (let i = 0; i < this.lamps.length; i++) {
      const lamp = this.lamps[i];
      const use = lamp.use;
      if (use?.lamp !== undefined) {
        const now = this.lampOn(use.lamp);
        if (now !== lamp.on) changed = true;
        lamp.on = now;
        continue;
      }
      const target = use && this.fillOut(use.room) ? 0 : 1;
      if (lamp.on === target) continue;
      lamp.on = target > lamp.on ? Math.min(target, lamp.on + step) : Math.max(target, lamp.on - step);
      changed = true;
    }
    return changed;
  }

  /** Eases each room's window glow towards out (the room dark, or its lamps all off) or on; true if any changed. */
  private fadeWindowGlow(dt: number): boolean {
    this.changed = false;
    this.step = Math.min(1, dt / LIGHTS_OUT_FADE);
    this.dark.forEach(this.glowRoom);
    this.unlitRooms.forEach(this.glowRoom);
    this.windowGlowOn.forEach(this.fadeGlow);
    return this.changed;
  }

  /** A room that may go dark gets a glow of its own (from full). */
  private readonly glowRoom = (room: number): void => {
    if (!this.windowGlowOn.has(room)) this.windowGlowOn.set(room, 1);
  };

  private readonly fadeGlow = (on: number, room: number): void => {
    const target = this.dark.has(room) || this.unlitRooms.has(room) ? 0 : 1;
    if (on === target && target === 0) return;
    const next = target > on ? Math.min(target, on + this.step) : Math.max(target, on - this.step);
    if (next === 1) this.windowGlowOn.delete(room);
    else this.windowGlowOn.set(room, next);
    this.changed = true;
  };
}

/** sRGB of a linear value (PlayCanvas takes light colours in sRGB). */
function gamma(v: number): number {
  return Math.pow(Math.max(0, v), 1 / 2.2);
}

/** A hex colour (sRGB) in linear light, `pow(c, 2.2)` as the materials decode colours. */
function linearHex(hex: string): RGB {
  const v = parseInt(hex.slice(1), 16);
  const lin = (c: number) => Math.pow(c / 255, 2.2);
  return { r: lin((v >> 16) & 255), g: lin((v >> 8) & 255), b: lin(v & 255) };
}

/**
 * A light's intensity, linear in `value`: PlayCanvas linearises its sRGB colour times an intensity
 * below 1 (`Light._updateLinearColor`), so an intensity there is given as its sRGB.
 */
function setIntensity(light: pc.LightComponent, value: number): void {
  const v = value >= 1 ? value : gamma(value);
  if (light.intensity !== v) light.intensity = v;
}

/** The rotation turning -y (a spot light's way) to (`x`, `y`, `z`), a unit vector in PlayCanvas space. */
function downTo(x: number, y: number, z: number, axis: pc.Vec3, out: pc.Quat): pc.Quat {
  // -y to d: about (-y) × d by the angle between them (half a turn about x when d is +y).
  axis.set(-z, 0, x);
  const len = axis.length();
  if (len < 1e-6) return y < 0 ? out.set(0, 0, 0, 1) : out.setFromAxisAngle(pc.Vec3.RIGHT, 180);
  return out.setFromAxisAngle(axis.mulScalar(1 / len), (Math.acos(Math.max(-1, Math.min(1, -y))) * 180) / Math.PI);
}
