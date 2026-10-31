/**
 * Model entries as the manifests write them, read the same way whichever engine draws them: the
 * fields beyond the base schema (`ModelExtras`: `fit`, `materials`, `parts`, `motions`), when a
 * model's clips play (`Motions`), the parts the player paints (`painted`), the box drawn without a
 * file (`StandIn`), a player's resizing (`Resize`), and a placed plant's natural variation.
 */

import { paintSlots } from '../../assets/paint';
import type { ModelEntry, PrimitivePart, Vec3 } from '../../assets/types';
import type { Content, ObjectDef } from '../../content/content';
import type { PartColors } from '../../core/protocol';
import { compose, quatAxisAngle, type Mat4 } from './math';

export type TintSlot = NonNullable<PrimitivePart['tint']>;
export type BoneName = NonNullable<PrimitivePart['bone']>;

/** What shapes a stand-in box: the thing's content layer and, on a top, its radius. */
export interface StandIn {
  /** `item`: food, a dish, something in a hand: a small block (`ITEM_STAND_IN`). */
  layer: ObjectDef['layer'] | 'item';
  /** Things on a top: the radius (m) of the disc they take (`Content.radius`). */
  radius: number;
}

/** The stand-in of an item on a top or in a hand: a block about 10 cm across. */
export const ITEM_STAND_IN: StandIn = { layer: 'item', radius: 0.05 };

/** The box a kind of thing is drawn as without its model's file (`BuildOptions.standIn`). */
export function standInOf(content: Content, def: ObjectDef | undefined): StandIn {
  return { layer: def?.layer, radius: content.radius(def) };
}

/** A player's resizing: its own size (metres across and deep, before turning) and how much taller. */
export interface Resize {
  size: [number, number];
  /** Centimetres taller (negative: shorter). */
  tall: number;
  /** Things hanging from the ceiling grow downwards. */
  ceiling?: boolean;
}

/** Shapes beyond the base schema. */
export interface ExtraShapes {
  /**
   * `lathe`: profile [radius, y] points in unit space (radius 0..0.5, y -0.5..0.5), revolved
   * around the vertical axis and scaled by `size`. Walk the outer surface upwards and any inner
   * surface downwards (the surface faces the right-hand side of the walk); bends sharper than
   * 40 degrees get a hard edge.
   */
  profile?: [number, number][];
}

/** Optional model entry fields beyond the base schema (read defensively; see `models.ts`). */
export interface ModelExtras {
  /** Exact size [width x, height y, depth z] in metres after `rotationY`; replaces `scale`. */
  fit?: Vec3;
  /** With `fit`: `back` puts the back (-Z) of the model on the footprint's back edge instead of centring it. */
  align?: 'centre' | 'back';
  /** Restyles materials by glTF material name; `*` applies to every material not listed. */
  materials?: Record<string, MaterialOverride>;
  /** Extra primitive parts drawn with the model (in the final, fitted space). */
  parts?: PrimitivePart[];
  /** When the file's clips play (`objectMotion.ts`). */
  motions?: MotionsEntry;
}

export interface MaterialOverride {
  /** Replace with a game finish (`material.finish.*`) coloured by `color` (default: the file's colour). */
  finish?: string;
  /** sRGB hex: the finish colour, or (without `finish`) a multiplier on the file's base colour. */
  color?: string;
  roughness?: number;
  metallic?: number;
  /** Drop the meshes that use this material. */
  hide?: boolean;
  /**
   * Mirror glass, reflecting the room live (`mirrors.ts`) from its face towards the front (+z) or
   * the back: a flat piece (the face is across its thinnest side) of the still body, square or
   * round as it is drawn.
   */
  mirror?: 'front' | 'back';
  /**
   * Glows in this colour (sRGB hex): a lamp's shade at night, as the lamps come on and out with its
   * room's lights (`TemplateGlow`); a screen while a resident uses it, with `screen`.
   */
  glow?: string;
  /** A screen: lit only while a resident uses the object (a TV, a monitor), not with the room. */
  screen?: boolean;
}

/** A use's clip in a manifest's `motions.uses`: its name, or the name and its own `hold` (and `delay`, `every`, `slot`, `out`); or several of these. */
type UseEntry = string | { clip: string; hold?: number; delay?: number; every?: number; slot?: number; out?: number; over?: 1 | -1 };

/** A model entry's `motions` as the manifest writes it: when its clips play (see `Motions`). */
interface MotionsEntry {
  uses?: Record<string, UseEntry | UseEntry[]>;
  hold?: number;
  open?: string;
  loops?: Record<string, string>;
  idle?: string;
  night?: string;
  growth?: string;
  ride?: string;
  running?: string;
  clock?: ClockClips;
}

/** When a model's clips play: while residents use the object, and while it's open. */
export interface Motions {
  /**
   * The clips a use plays, by the interaction's id or, failing that, its animation tag (`fetch`,
   * `drink`), each with the seconds after the use starts that it plays back (a door that closes
   * again; absent: when the use ends). The manifest gives a clip's name (with the model's `hold`),
   * `{clip, hold}`, or a list of them (a shower's door shutting after 1 s while its water runs on).
   */
  uses: Record<string, readonly UseClip[]>;
  /**
   * The clip a use plays round and round while it lasts (a rocking chair, a washer's drum), by
   * interaction id or animation tag: from rest to rest, finishing its round when the use ends.
   */
  loops: Record<string, string>;
  /** The clip held open while the object is open (a wall bed folded down; the sim's `open`, set by `leaves`). */
  open?: string;
  /** The clip that goes round all the time, near the camera (a ceiling fan, a fountain's water; `ambientRange`). */
  idle?: string;
  /** The clip held open from dusk to dawn, while the lamps are lit (curtains drawn), whoever is asleep. */
  night?: string;
  /**
   * A crop's clip, held where its growth stage is (`GROWTH_TIMES`): ripe at rest, then picked,
   * growing and just planted along it, so each stage plays on to the next.
   */
  growth?: string;
  /** The clip whose (first) part carries the residents sitting or lying on the object: a rocking chair's seat, a swing. */
  ride?: string;
  /** The clip that goes round while the object runs on its own, nobody at it (a washer's drum; content `runs`, the sim's `running`). */
  running?: string;
  /** A clock's hands, set from the game's time (`ClockClips`). */
  clock?: ClockClips;
}

export interface UseClip {
  clip: string;
  /** Seconds after it starts that it plays back (absent: when the use ends). */
  hold?: number;
  /** Seconds after the use starts that it starts (a net swishing after the shot). */
  delay?: number;
  /** Plays again every so many seconds while the use lasts, from rest to rest each time (a net swishing after each shot). */
  every?: number;
  /** Only for the resident in this slot (content `slots`, from 0: the one nearest its place, `slotOf`, as sim-core's `slot_position` has them; a bunk's 1 is the upper one). */
  slot?: number;
  /** Opens again for so many seconds when the use ends (a phone box's door letting the caller out). */
  out?: number;
  /**
   * Plays along with the use instead (the snapshot's `progress`): 1 from rest to its end over the use's minutes, -1
   * from its end back to rest; the washing hung out piece by piece and taken in, a crop picked as the hands go (the
   * sim's state changes when the use ends: content `gradual`, a crop's `picks`).
   */
  over?: 1 | -1;
}

/**
 * A clock's hands (`motions.clock`): each clip turns its hand once round from rest, the `hour` one
 * in twelve hours and the `minute` one in an hour, and the game holds each where the game's time
 * has it. `at`: the time the hands show at rest, in minutes past twelve (default 0: twelve o'clock).
 */
export interface ClockClips {
  hour?: string;
  minute?: string;
  at?: number;
}

/** Of a placed object's `colors`, those on parts its model lets the player paint; undefined for none. */
export function paintColors(entry: ModelEntry | undefined, colors: PartColors | undefined): PartColors | undefined {
  if (!colors) return undefined;
  const shown = Object.fromEntries(paintSlots(entry).flatMap(({ material }) => (colors[material] ? [[material, colors[material]]] : [])));
  return Object.keys(shown).length ? shown : undefined;
}

/**
 * `entry` with each part its `paint` names in its colour from `colors` (`#rrggbb` by material, a
 * placed object's colours), each keeping its finish (mapped by name or `*`); a material without a
 * finish has its file colour multiplied by it. The entry itself when nothing is painted.
 */
export function painted(entry: ModelEntry | undefined, colors: PartColors | undefined): ModelEntry | undefined {
  const shown = paintColors(entry, colors);
  if (!entry || !shown) return entry;
  const materials = { ...((entry as ModelEntry & ModelExtras).materials ?? {}) };
  for (const [name, color] of Object.entries(shown)) materials[name] = { ...(materials[name] ?? namedFinish(name) ?? materials['*']), color };
  return { ...entry, materials } as ModelEntry;
}

/** A model entry's `motions`, if it has any. */
export function motionsOf(entry: ModelEntry | undefined): Motions | undefined {
  const m = (entry as (ModelEntry & ModelExtras) | undefined)?.motions;
  if (!m) return undefined;
  const uses = Object.fromEntries(
    Object.entries(m.uses ?? {}).map(([use, value]) => [use, (Array.isArray(value) ? value : [value]).map((u) => (typeof u === 'string' ? { clip: u, hold: m.hold } : { ...u, hold: u.hold ?? m.hold }))]),
  );
  const loops = m.loops ?? {};
  const clock = m.clock?.hour || m.clock?.minute ? m.clock : undefined;
  return Object.keys(uses).length || Object.keys(loops).length || m.open || m.idle || m.night || m.growth || m.running || clock
    ? { uses, loops, open: m.open, idle: m.idle, night: m.night, growth: m.growth, ride: m.ride, running: m.running, clock }
    : undefined;
}

/**
 * A material named for its finish, `stone@matte` (`tunic@fabric#body`): drawn with
 * `material.finish.matte` (unless the entry maps it), if `lib` has it (without one: whatever it names).
 */
export function namedFinish(name: string, lib?: { hasFinish(key: string): boolean }): MaterialOverride | undefined {
  const named = /.@([a-z]+)$/.exec(name.replace(TINT_IN_NAME, ''));
  const finish = named && `material.finish.${named[1]}`;
  return finish && (!lib || lib.hasFinish(finish)) ? { finish } : undefined;
}

/** A tint slot in a material's name: `tunic#body` is tinted as a primitive part with `tint: 'body'` is. */
const TINT_IN_NAME = /#(body|skin|hair)(?=@|\||$)/;

/** The tint slot a file's material carries in its name (`TINT_IN_NAME`), or null. */
export function tintInName(name: string | null | undefined): TintSlot | null {
  return (name ? (TINT_IN_NAME.exec(name)?.[1] as TintSlot | undefined) : undefined) ?? null;
}

/**
 * A model entry's own placement of its file's model space: its `scale`, `rotationY` and `offset`;
 * with `fit`, only its turn (the fit is found from the geometry built, `fitInto`). The matrix
 * layout of `shared/math.ts` (`compose`).
 */
export function entrySpace(entry: ModelEntry, out: Mat4 = new Float32Array(16)): Mat4 {
  const fit = (entry as ModelEntry & ModelExtras).fit;
  const s = fit ? 1 : (entry.scale ?? 1);
  const [ox, oy, oz] = entry.offset ?? [0, 0, 0];
  const turn = quatAxisAngle(0, 1, 0, ((entry.rotationY ?? 0) * Math.PI) / 180);
  return fit ? compose(s, s, s, turn, 0, 0, 0, out) : compose(s, s, s, turn, ox, oy, oz, out);
}

/**
 * The natural variation of a placed plant (manifest `vary`): its own turn (radians) and size,
 * fixed by the tile it stands on so it looks the same after a reload. None when `vary` is 0.
 */
export function placementVariation(vary: number, x: number, z: number): { turn: number; size: number } {
  if (!vary) return { turn: 0, size: 1 };
  return { turn: hash3(x, z, 101) * Math.PI * 2, size: 1 + (hash3(x, z, 102) * 2 - 1) * vary };
}

/** A hash of three integers in [0, 1). */
export function hash3(x: number, y: number, z: number): number {
  let h = (Math.imul(x, 374761393) + Math.imul(y, 668265263) + Math.imul(z, 1440662683)) | 0;
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}
