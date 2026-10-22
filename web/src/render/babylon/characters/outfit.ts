/**
 * Turns a Sim's appearance (creator colours, hairstyle, gender) into its outfit: which garments it
 * wears (ids from the body's `garments`; the shell parts `top.*`, `bottom.*`, `shoes.*` of those
 * that have one, see crates/tools/src/characters/clothes.rs) and the per-Sim shader parameters (skin
 * colour, top, bottom and shoe colours, the cuts that paint the clothes on the body, hair colour).
 * The top follows the creator's outfit colour; shoes keep their own unless one is chosen. Garments
 * and colours chosen in the household creator (`appearance.top`, `bottom`, `shoes`, `bottomColor`,
 * `shoesColor`) are worn as chosen; whatever is missing (Sims from older saves) is picked
 * deterministically from the Sim's id, so a Sim always wears the same clothes. Pyjamas, a towel and
 * swimwear are painted the same way (`Wear`). Babies wear a romper instead of a skirt (`ROMPER`).
 */

import type { Appearance } from '../../../game/household';
import type { BodyData, GarmentCut } from './data';

const TROUSERS = ['#33415C', '#2B2D33', '#4A5568', '#B9A27E', '#6B5A48', '#3E4A3A', '#E8E2D6', '#5C3B3B'];
const SKIRTS = ['#2B2D33', '#5C3B3B', '#33415C', '#7A6A8E', '#B9A27E', '#3E4A3A'];
const SLEEP_TOP = ['#9DB4D9', '#D9A5B4', '#B9D2B0', '#E6D2A6'];

const TOPS: Record<string, readonly string[]> = {
  female: ['top.tee', 'top.tee', 'top.tank', 'top.blouse', 'top.blouse', 'top.long', 'top.shirt', 'top.jacket'],
  male: ['top.tee', 'top.tee', 'top.vneck', 'top.polo', 'top.long', 'top.long', 'top.shirt', 'top.jacket', 'top.suit'],
};
const BOTTOMS: Record<string, readonly string[]> = {
  female: ['bottom.trousers', 'bottom.trousers', 'bottom.capri', 'bottom.skirt', 'bottom.skirt', 'bottom.shorts'],
  male: ['bottom.trousers', 'bottom.trousers', 'bottom.trousers', 'bottom.shorts', 'bottom.suit'],
};
const SHOE_PARTS = ['shoes.sneakers', 'shoes.sneakers', 'shoes.trainers', 'shoes.boots', 'shoes.dress'] as const;
/**
 * What a baby wears for a skirt: shorts in the top's colour, one piece. A skirt's shell can't follow
 * a baby's legs, drawn up and fallen apart in the crib or in someone's arms.
 */
const ROMPER = 'bottom.shorts';
/** Shoes that go with formal clothes. */
const FORMAL = new Set(['top.suit', 'bottom.suit', 'bottom.skirt']);

/** Garment choices offered per gender, most common first (`pickOutfit` weights by repetition). */
export const GARMENTS = { TOPS, BOTTOMS, SHOES: SHOE_PARTS as readonly string[] };

/** A random outfit for a new Sim (the creator colour stays the top's colour). */
export function pickOutfit(gender: string, rand: () => number = Math.random): Pick<Appearance, 'top' | 'bottom' | 'shoes' | 'bottomColor'> {
  const bottom = pick(BOTTOMS[gender] ?? BOTTOMS.male, rand());
  const top = pick(TOPS[gender] ?? TOPS.male, rand());
  return {
    top,
    bottom,
    shoes: pickShoes(top, bottom, rand()),
    bottomColor: pick(bottom === 'bottom.skirt' ? SKIRTS : TROUSERS, rand()),
  };
}

function pickShoes(top: string, bottom: string, r: number): string {
  return FORMAL.has(top) || FORMAL.has(bottom) ? (r < 0.6 ? 'shoes.dress' : 'shoes.boots') : pick(SHOE_PARTS, r);
}

export interface OutfitParams {
  /** Garments worn (ids; those with a shell are also mesh part names; empty = none). */
  parts: string[];
  /** Skin colour (linear rgb). */
  skin: [number, number, number];
  /** The body's skin reference colour (linear rgb, 1); the shader tints relative to it. */
  skinRef: [number, number, number];
  top: [number, number, number];
  bottom: [number, number, number];
  /** Shoe colour (linear rgb: the creator's, else the shoe's own). */
  shoe: [number, number, number];
  /** 1 if the top is painted over the bottom (its hem below the waistband), else 0. */
  over: number;
  /** What shows in an open jacket's V (`SHIRTS`; 0: none, the top's own colour). */
  shirt: number;
  /** Its neck line (m, the base body's height; `OutfitFit.neck`): the shirt shows below it, above it only its collar. */
  neck: number;
  /** 1 for clothes without hems, cuffs or a waistband (the towel, swimwear), else 0. */
  plain: number;
  /** Age (0 young .. 1 elder): the shader deepens the skin's cavities with it. */
  age: number;
  hair: [number, number, number];
  /** Iris colour (linear rgb). */
  eyes: [number, number, number];
  /** 1 if the Sim wears hair (its scalp takes the hair colour), else 0. */
  scalp: number;
  /** Coverage cuts: top sleeve / hem / crew / scoop, bottom waist / leg, shoe height. */
  cuts: [number, number, number, number, number, number, number];
  /** The shells not drawn with it (`SHELL_BITS`): the day clothes draw all theirs (0). */
  bare: number;
}

function hash(n: number): number {
  let h = Math.imul(n ^ 0x9e3779b9, 0x85ebca6b);
  h = Math.imul(h ^ (h >>> 13), 0xc2b2ae35);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

function pick<T>(list: readonly T[], r: number): T {
  return list[Math.floor(r * list.length) % list.length];
}

function hexToLinear(hex: string | undefined, fallback: string): [number, number, number] {
  const h = typeof hex === 'string' && /^#[0-9a-f]{6}$/i.test(hex) ? hex : fallback;
  const c = [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16) / 255);
  return c.map((v) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4)) as [number, number, number];
}

function luminance(c: readonly number[]): number {
  return c[0] * 0.2126 + c[1] * 0.7152 + c[2] * 0.0722;
}

/** Eye colours for looks without one (older saves): the creator's `palette.eyes`, mostly brown. */
export const EYE_COLOURS = ['#4A2E1B', '#6B4226', '#6B4226', '#8A6237', '#5F7A3E', '#4E7BA6', '#7C8A96'];

/** Share of each (linear) swatch channel real skin reflects (light swatches; see `skinColour`). */
const SKIN_REFLECT = [0.68, 0.6, 0.56];

/**
 * A creator skin swatch as the skin's albedo (linear). Swatches are bright UI colours; real skin
 * reflects less, and less blue and green than red (blood under the skin), so light swatches read
 * as warm skin rather than chalk. Deep swatches keep more of their own value, so their faces keep
 * the shading, lips and cavities that shape them.
 */
function skinColour(hex: string | undefined): [number, number, number] {
  const c = hexToLinear(hex, '#D9A882');
  const keep = 1 - Math.min(1, Math.max(0, (luminance(c) - 0.03) / 0.12));
  return c.map((v, k) => v * (SKIN_REFLECT[k] + (1 - SKIN_REFLECT[k]) * keep * keep)) as [number, number, number];
}

const OFF = -10;

/** What shows in an open jacket's V (`OutfitParams.shirt`; their colours are material.ts `SHIRTS`). */
const SHIRTS = { white: 1, charcoal: 2, grey: 3 } as const;

/** The shirt under `topPart` in `top` (linear): a suit's is white, a jacket's a tee that stands out from it. */
function shirtUnder(topPart: string, top: readonly number[], r: number): number {
  if (topPart === 'top.suit') return SHIRTS.white;
  if (topPart !== 'top.jacket') return 0;
  return luminance(top) > 0.2 ? SHIRTS.charcoal : r < 0.5 ? SHIRTS.white : SHIRTS.grey;
}

/**
 * What a Sim wears for what it does (characters.ts `wearFor`): its `day` clothes; pyjamas for a
 * night's sleep in bed (`night`: a long top and trousers in a sleep colour); its day clothes with the
 * shoes off for a nap on a bed (`rest`); a towel in the shower, the bath and the sauna (`towel`: wrapped
 * under the arms, or round the hips for a man); swimwear in a hot tub (`swim`: a swimsuit, or trunks).
 * All are painted on the body; the shells the day clothes have (a jacket, a skirt, the soles) are drawn
 * only with the day clothes (`bare`), so a change of clothes adds nothing to draw.
 */
export type Wear = 'day' | 'night' | 'rest' | 'towel' | 'swim';
export const WEARS: readonly Wear[] = ['day', 'night', 'rest', 'towel', 'swim'];

/** Shell categories (`OutfitParams.bare`, as the shells' fields: 0 top, 1 bottom, 2 shoes) as bits. */
export const SHELL_BITS = { top: 1, bottom: 2, shoes: 4 } as const;
const ALL_SHELLS = SHELL_BITS.top | SHELL_BITS.bottom | SHELL_BITS.shoes;

/**
 * A base body's heights at rest (m, its fields' heights) the bath clothes are cut by: under the arms, the hips' and
 * the knees' joints; and an open jacket's neck line, 1 cm under `neck_01` (as clothes.rs cuts the shell).
 */
export interface OutfitFit {
  chest: number;
  hip: number;
  knee: number;
  neck: number;
}

const TOWELS = ['#F2EFE8', '#CFE0E6', '#E9D8C4', '#D8E3CF'];
const SWIMWEAR = ['#1F3A5F', '#B23A48', '#2E6E65', '#D98E04', '#3B3B58', '#C45A7A'];

/** Tops' colours for the outfits a Sim changes into (`spareLook`): the creator's `palette.outfit`. */
const SPARE_TOPS = ['#E98A6B', '#5F86C9', '#8FA89A', '#E2B65B', '#9C7BD1', '#4FBF85', '#34405A', '#F4F1EC', '#D45D79'];

/**
 * The look of a Sim's `outfit`-th outfit (sim-core `Sim::outfit`, changed into at a wardrobe): 0 the
 * creator's, as `appearance` has it; the others garments and colours picked from the id, each with a
 * top or a bottom other than the creator's, its top in another colour and the bottom standing out from it.
 * Everything else (skin, hair) stays.
 */
function spareLook(id: number, gender: string, appearance: Appearance | null | undefined, body: BodyData, outfit: number): Appearance | null | undefined {
  if (outfit <= 0 || !appearance) return appearance;
  const own = dayOutfit(id, gender, appearance, body, false).parts;
  let look = appearance;
  for (let tries = 0; tries < 6; tries++) {
    let k = 0;
    const rand = () => hash(id * 31 + 100 + outfit * 17 + tries * 5 + k++);
    const top = SPARE_TOPS.filter((c) => c.toLowerCase() !== appearance.body?.toLowerCase());
    look = { ...appearance, ...pickOutfit(gender, rand), body: pick(top, rand()), shoesColor: undefined };
    const parts = dayOutfit(id, gender, look, body, false).parts;
    // (Not all one colour: a bottom that stands out from the top, as `dayOutfit`'s own.)
    const contrast = Math.abs(luminance(hexToLinear(look.body, '#8FA89A')) - luminance(hexToLinear(look.bottomColor, '#33415C')));
    if ((parts[0] !== own[0] || parts[1] !== own[1]) && contrast >= 0.2) break;
  }
  return look;
}

/** The outfit for `wear` (`fit`: the base body's, for the towel and swimwear) in the Sim's `outfit`-th clothes (`spareLook`). */
export function outfitFor(
  id: number,
  gender: string,
  appearance: Appearance | null | undefined,
  body: BodyData,
  wear: Wear = 'day',
  fit?: OutfitFit,
  outfit = 0,
): OutfitParams {
  const day = dayOutfit(id, gender, spareLook(id, gender, appearance, body, outfit), body, wear === 'night');
  if (fit) day.neck = fit.neck;
  if (wear === 'day') return day;
  const c = [...day.cuts] as OutfitParams['cuts'];
  // Barefoot.
  c[6] = OFF;
  if (wear === 'rest') return { ...day, cuts: c, bare: SHELL_BITS.shoes };
  const cut = (g: string) => body.garments[g] ?? {};
  if (wear === 'night') {
    // A long top and trousers.
    const t = cut('top.long');
    const b = cut('bottom.trousers');
    return { ...day, cuts: [t.sleeve ?? c[0], t.hem ?? c[1], t.crew ?? OFF, t.scoop ?? OFF, b.waist ?? c[4], b.leg ?? c[5], OFF], over: 0, shirt: 0, bare: ALL_SHELLS };
  }
  const waist = cut('bottom.trousers').waist ?? 1;
  const f = fit ?? { chest: waist + 0.25, hip: waist - 0.15, knee: waist - 0.5, neck: waist + 0.4 };
  const thigh = f.hip - f.knee;
  const r = (k: number) => hash(id * 31 + k);
  const male = gender === 'male';
  if (wear === 'towel') {
    const towel = hexToLinear(pick(TOWELS, r(10)), TOWELS[0]);
    // The bottom's cut alone: from under the arms (round the hips) to mid-thigh (above the knees).
    const [top, hem] = male ? [waist + 0.01, f.knee + 0.15 * thigh] : [f.chest, f.knee + 0.6 * thigh];
    return { ...day, bottom: towel, cuts: [OFF, 10, OFF, OFF, top, hem, OFF], over: 0, shirt: 0, plain: 1, bare: ALL_SHELLS };
  }
  const suit = hexToLinear(pick(SWIMWEAR, r(11)), SWIMWEAR[0]);
  // Trunks; or a swimsuit, the tank top's cut down to the top of the legs.
  if (male) return { ...day, bottom: suit, cuts: [OFF, 10, OFF, OFF, waist - 0.03, f.knee + 0.65 * thigh, OFF], over: 0, shirt: 0, plain: 1, bare: ALL_SHELLS };
  const t = cut('top.tank');
  return { ...day, top: suit, cuts: [t.sleeve ?? -0.62, f.hip - 0.035, t.crew ?? OFF, t.scoop ?? 0, OFF, 10, OFF], over: 0, shirt: 0, plain: 1, bare: ALL_SHELLS };
}

/** Day outfit (or its colours as pyjamas, `sleep`). */
function dayOutfit(id: number, gender: string, appearance: Appearance | null | undefined, body: BodyData, sleep: boolean): OutfitParams {
  const r = (k: number) => hash(id * 31 + k);
  const has = (p: string) => p in body.garments;
  const skin = skinColour(appearance?.skin);
  const skinRef = [body.skinRef[0], body.skinRef[1], body.skinRef[2]] as [number, number, number];
  // Fabric: the creator colour, a little less saturated-bright than the UI swatch.
  let top = hexToLinear(appearance?.body, '#8FA89A').map((v) => v * 0.9) as [number, number, number];
  const hair = hexToLinear(appearance?.hair, '#5A3B2A');

  // Creator choices (when the body has them), else picks from the id.
  const chosen = (part: string | undefined, prefix: string) => (part && part.startsWith(prefix) && has(part) ? part : null);
  const isHex = (c: string | undefined) => typeof c === 'string' && /^#[0-9a-f]{6}$/i.test(c);
  const topPart = chosen(appearance?.top, 'top.') ?? pick(TOPS[gender] ?? TOPS.male, r(7));
  const picked = chosen(appearance?.bottom, 'bottom.') ?? pick(BOTTOMS[gender] ?? BOTTOMS.male, r(6));
  const skirt = picked === 'bottom.skirt';
  const romper = skirt && (body.stage === 'baby' || appearance?.stage === 'baby') && has(ROMPER);
  const bottomPart = romper ? ROMPER : picked;
  let bottom: [number, number, number];
  if (romper) {
    bottom = top;
  } else if (isHex(appearance?.bottomColor)) {
    bottom = hexToLinear(appearance!.bottomColor, '#33415C');
  } else {
    bottom = hexToLinear(pick(skirt ? SKIRTS : TROUSERS, r(2)), '#33415C');
    // Avoid a bottom that matches the top too closely.
    if (Math.abs(luminance(bottom) - luminance(top)) < 0.03) bottom = hexToLinear(pick(TROUSERS, r(3)), '#2B2D33');
  }
  const shoePart = chosen(appearance?.shoes, 'shoes.') ?? pickShoes(topPart, picked, r(8));
  if (sleep) {
    top = hexToLinear(pick(SLEEP_TOP, r(5)), '#9DB4D9');
    bottom = top.map((v) => v * 0.8) as [number, number, number];
  }

  const parts = [topPart, bottomPart, shoePart].filter(has);
  const t: GarmentCut = body.garments[topPart] ?? {};
  const b: GarmentCut = body.garments[bottomPart] ?? {};
  const s: GarmentCut = body.garments[shoePart] ?? {};
  const cuts: OutfitParams['cuts'] = [
    has(topPart) ? (t.sleeve ?? OFF) : OFF,
    t.hem ?? 10,
    t.crew ?? OFF,
    t.scoop ?? OFF,
    has(bottomPart) ? (b.waist ?? OFF) : OFF,
    b.leg ?? 10,
    has(shoePart) ? (s.shoe ?? OFF) : OFF,
  ];
  return {
    parts,
    skin,
    skinRef,
    top,
    bottom,
    shoe: hexToLinear(isHex(appearance?.shoesColor) ? appearance!.shoesColor : s.colour, '#3A3330'),
    over: t.over ? 1 : 0,
    shirt: shirtUnder(topPart, top, r(12)),
    // (Without the body's joints, from the waistband: `outfitFor` puts in the neck line when it has them.)
    neck: (body.garments['bottom.trousers']?.waist ?? 1) + 0.4,
    plain: 0,
    age: appearance?.stage === 'elder' ? 1 : appearance?.stage === 'adult' ? 0.4 : 0,
    hair,
    eyes: hexToLinear(appearance?.eyes ?? EYE_COLOURS[Math.floor(r(9) * EYE_COLOURS.length)], '#6B4226'),
    scalp: appearance?.hairStyle === 'none' ? 0 : 1,
    cuts,
    bare: 0,
  };
}

/** Writes the appearance texels (see material.ts) into `row` at `offset`. */
export function writeOutfit(o: OutfitParams, row: Float32Array, offset: number): void {
  const c = o.cuts;
  row.set([o.skin[0], o.skin[1], o.skin[2], c[0]], offset);
  row.set([o.top[0], o.top[1], o.top[2], c[1]], offset + 4);
  row.set([o.bottom[0], o.bottom[1], o.bottom[2], c[2]], offset + 8);
  row.set([o.hair[0], o.hair[1], o.hair[2], c[3]], offset + 12);
  // (r: over the bottom 1, or 1 + the shirt + its neck line / 4; else 0, or -1 if plain.)
  row.set([o.over ? 1 + (o.shirt ? o.shirt + o.neck / 4 : 0) : -o.plain, o.age, 0, c[4]], offset + 16);
  row.set([o.skinRef[0], o.skinRef[1], o.skinRef[2], c[5]], offset + 20);
  row.set([o.shoe[0], o.shoe[1], o.shoe[2], c[6]], offset + 24);
  row.set([o.eyes[0], o.eyes[1], o.eyes[2], o.scalp], offset + 28);
}
