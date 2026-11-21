/**
 * A world's code: what makes the world as generated, and its people, written as one short line the
 * player can copy, share and type in again (step 1's code field, `?code=` in the page's URL). It
 * names the place, the land's size, how hilly and wooded it is, the land's seed, and (where the
 * player chose them) the town and the villages, the town's seed and the people's. Brush strokes
 * and town edits are not in it.
 *
 * Written as `place-flags-seed[-townSeed[-peopleSeed]]`, all lower case: the flags are a letter
 * each for the size, the hills, the woods, then the town and the villages (`_` for one not
 * chosen, left off at the end), and the seeds are in base 36. So `river-mhs-k2x9q` is a medium,
 * hilly river valley with some woods, and `island-lfwcm-k2x9q-1b-7` a large, flat, wooded island with a
 * city and many villages, its town and its people from seeds of their own.
 */

import type { LandSize, TownSize, WorldOptions } from '../core/protocol';
import { WORLD_KINDS, type WorldKind } from './world';

export interface WorldCode {
  kind: WorldKind;
  size: LandSize;
  hills: WorldOptions['hills'];
  woods: WorldOptions['woods'];
  /** The land's seed. */
  seed: number;
  town?: TownSize;
  villages?: WorldOptions['villages'];
  /** The town's own seed, if not the land's. */
  townSeed?: number;
  /** The people's own seed (`people.ts` `peopleStreams`), if not drawn from the town's. */
  peopleSeed?: number;
}

/** Each flag's letters, in the flags' order. */
const SIZES: Record<LandSize, string> = { small: 's', medium: 'm', large: 'l' };
const HILLS: Record<WorldOptions['hills'], string> = { flat: 'f', rolling: 'r', hilly: 'h', mountains: 'm' };
const WOODS: Record<WorldOptions['woods'], string> = { open: 'o', some: 's', wooded: 'w' };
const TOWNS: Record<TownSize, string> = { hamlet: 'h', village: 'v', town: 't', city: 'c' };
const VILLAGES: Record<WorldOptions['villages'], string> = { none: 'n', few: 'f', many: 'm' };
/** A flag left unchosen. */
const UNSET = '_';
/** The largest seed (seeds are unsigned 32-bit). */
const SEED_MOST = 0xffff_ffff;

/** The value a flag's letter stands for in `table`, if any. */
function valueOf<T extends string>(table: Record<T, string>, letter: string | undefined): T | undefined {
  return (Object.keys(table) as T[]).find((k) => table[k] === letter);
}

/** A seed written in base 36; undefined: not a seed. */
function seedOf(text: string): number | undefined {
  if (!/^[0-9a-z]{1,7}$/.test(text)) return undefined;
  const n = parseInt(text, 36);
  return n <= SEED_MOST ? n : undefined;
}

const written = (seed: number) => (seed >>> 0).toString(36);

/** `code` as one line (see the top of this file). */
export function writeCode(code: WorldCode): string {
  const flags = (SIZES[code.size] + HILLS[code.hills] + WOODS[code.woods] + (code.town ? TOWNS[code.town] : UNSET) + (code.villages ? VILLAGES[code.villages] : UNSET)).replace(/_+$/, '');
  const seeds = [written(code.seed), code.townSeed === undefined ? '' : written(code.townSeed), code.peopleSeed === undefined ? '' : written(code.peopleSeed)];
  while (seeds.length > 1 && seeds[seeds.length - 1] === '') seeds.pop();
  return [code.kind, flags, ...seeds].join('-');
}

/**
 * The code `text` writes (`writeCode`; any case, spaces round it ignored). A bare number (in
 * decimal, as the older seed field took it) is the land's seed alone: the rest stays as the player
 * has it. Null: not a code.
 */
export function readCode(text: string): WorldCode | { seed: number } | null {
  const line = text.trim().toLowerCase();
  if (/^\d+$/.test(line)) {
    const seed = Number(line);
    return seed <= SEED_MOST ? { seed } : null;
  }
  const parts = line.split('-');
  if (parts.length < 3 || parts.length > 5) return null;
  const [kind, flags, ...seedTexts] = parts;
  if (!(WORLD_KINDS as readonly string[]).includes(kind) || flags.length < 3 || flags.length > 5) return null;
  const size = valueOf(SIZES, flags[0]);
  const hills = valueOf(HILLS, flags[1]);
  const woods = valueOf(WOODS, flags[2]);
  if (!size || !hills || !woods) return null;
  const town = flags[3] === undefined || flags[3] === UNSET ? undefined : valueOf(TOWNS, flags[3]);
  const villages = flags[4] === undefined || flags[4] === UNSET ? undefined : valueOf(VILLAGES, flags[4]);
  if ((flags[3] !== undefined && flags[3] !== UNSET && !town) || (flags[4] !== undefined && flags[4] !== UNSET && !villages)) return null;
  // (A town's seed left out between the land's and the people's is an empty part; the last never is.)
  const seed = seedOf(seedTexts[0]);
  if (seed === undefined || seedTexts.at(-1) === '') return null;
  const townSeed = seedTexts[1] ? seedOf(seedTexts[1]) : undefined;
  const peopleSeed = seedTexts[2] ? seedOf(seedTexts[2]) : undefined;
  if ((seedTexts[1] && townSeed === undefined) || (seedTexts[2] && peopleSeed === undefined)) return null;
  return {
    kind: kind as WorldKind,
    size,
    hills,
    woods,
    seed,
    ...(town ? { town } : {}),
    ...(villages ? { villages } : {}),
    ...(townSeed === undefined ? {} : { townSeed }),
    ...(peopleSeed === undefined ? {} : { peopleSeed }),
  };
}
