/**
 * The world a bench tool makes: `--world <place>` (`web/src/game/world.ts` `WORLD_KINDS`, or a kind
 * from before places, `LEGACY_KINDS`: `small`, `medium` and `dense` are the countryside as a village,
 * a town and a city) and `--options town=city,villages=many` (`web/src/game/worldOptions.ts`; the
 * place's own where not given). A place or an option the game doesn't know stops the tool, rather
 * than making the countryside instead.
 *
 * `flag(name, fallback)`: the tool's own reader of its command line.
 */

import { LEGACY_KINDS, WORLD_KINDS } from '../../web/src/game/world.ts';
import { OPTION_FIELDS, parseOptions } from '../../web/src/game/worldOptions.ts';

/** Stops the tool with `message`. */
function stop(message) {
  console.error(message);
  process.exit(1);
}

export function worldFlags(flag, fallback = 'country') {
  const world = flag('world', fallback);
  if (world !== null && !WORLD_KINDS.includes(world) && !Object.hasOwn(LEGACY_KINDS, world)) stop(`--world ${world}: no such place (${[...WORLD_KINDS, ...Object.keys(LEGACY_KINDS)].join(', ')})`);
  const options = parseOptions(flag('options', ''), (entry) => stop(`--options: ${entry} is no option (${OPTION_FIELDS.map((f) => `${f.key}=${f.choices.map((c) => c.value).join('|')}`).join(', ')})`));
  /** The same as the page's URL flags take them (`options=town:city,…`). */
  const query = Object.entries(options).map(([k, v]) => `${k}:${v}`).join(',');
  return { world, options, query };
}
