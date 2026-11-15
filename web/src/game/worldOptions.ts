/**
 * The options a new world is made with besides its place (`WorldOptions`: how big its main town
 * is, how many villages round it, how hilly and wooded its land), as step 1 offers them
 * (`CreateWorld.svelte`) and the tools name them (`?options=town:city,villages:many` in the page's
 * URL, `--options town=city,villages=many` on a bench tool's command line). Kept apart from
 * `world.ts` so the benchmark's flags can read them without the generator.
 */

import type { TownSize, WorldOptions } from '../core/protocol';

/** One option: what step 1 calls it, and its choices in order, each with its label. */
export interface OptionField<K extends keyof WorldOptions = keyof WorldOptions> {
  key: K;
  label: string;
  choices: readonly { value: WorldOptions[K]; label: string }[];
}

/** The options, in the order step 1 shows them. */
export const OPTION_FIELDS: readonly OptionField[] = [
  {
    key: 'town',
    label: 'Town',
    choices: [
      { value: 'hamlet', label: 'Hamlet' },
      { value: 'village', label: 'Village' },
      { value: 'town', label: 'Town' },
      { value: 'city', label: 'City' },
    ],
  },
  {
    key: 'villages',
    label: 'Villages',
    choices: [
      { value: 'none', label: 'None' },
      { value: 'few', label: 'Few' },
      { value: 'many', label: 'Many' },
    ],
  },
  {
    key: 'hills',
    label: 'Hills',
    choices: [
      { value: 'flat', label: 'Flat' },
      { value: 'rolling', label: 'Rolling' },
      { value: 'hilly', label: 'Hilly' },
      { value: 'mountains', label: 'Mountains' },
    ],
  },
  {
    key: 'woods',
    label: 'Woods',
    choices: [
      { value: 'open', label: 'Open' },
      { value: 'some', label: 'Some' },
      { value: 'wooded', label: 'Wooded' },
    ],
  },
];

/**
 * Options from a list such as `town:city,villages:many` (or `town=city`, as the bench tools take
 * them). An entry that names no option or no choice of it is left out, and told to `bad` if given.
 */
export function parseOptions(text: string | null | undefined, bad?: (entry: string) => void): Partial<WorldOptions> {
  const out: Record<string, string> = {};
  for (const entry of (text ?? '').split(',').map((s) => s.trim()).filter(Boolean)) {
    const [key, value] = entry.split(/[:=]/).map((s) => s.trim());
    const field = OPTION_FIELDS.find((f) => f.key === key);
    if (field && field.choices.some((c) => c.value === value)) out[key] = value;
    else bad?.(entry);
  }
  return out as Partial<WorldOptions>;
}

/** The same options (missing ones only equal missing ones). */
export function sameOptions(a: Partial<WorldOptions> | undefined, b: Partial<WorldOptions> | undefined): boolean {
  return OPTION_FIELDS.every((f) => a?.[f.key] === b?.[f.key]);
}

const TOWN_NAMES: Record<TownSize, string> = { hamlet: 'a hamlet', village: 'a village', town: 'a town', city: 'a city' };

/**
 * A world in a line (step 1's hint): its homes, what its main town is and how many villages lie
 * round it, how big its land is, and the neighbours it can have at most (`cap`), said only when
 * its homes, all but the player's, could hold that many.
 */
export function worldLine(w: { homes: number; villages: number; width: number; depth: number; town?: TownSize; cap: number }): string {
  const homes = `${w.homes} ${w.homes === 1 ? 'home' : 'homes'} in ${TOWN_NAMES[w.town ?? 'town']}`;
  const villages = w.villages ? ` and ${w.villages} ${w.villages === 1 ? 'village' : 'villages'}` : '';
  const room = w.homes - 1 >= w.cap ? ` · up to ${w.cap}\u00a0neighbours` : '';
  // (The land's size kept on one line.)
  return `${homes}${villages} · ${w.width}\u00a0×\u00a0${w.depth}\u00a0m${room}`;
}
