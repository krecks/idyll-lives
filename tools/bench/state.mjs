#!/usr/bin/env bun
/**
 * Makes a game state without a browser: the new game a world's code (or scene.mjs's recipe) starts, or a save, run
 * to an exact tick by the sim's own WebAssembly (\`web/src/core/wasm-pkg\`, \`bun run wasm\`) under Bun, as the sim
 * worker runs it, and saved paused. What \`?state=<name>\` (\`debug-states/\`) and \`?bench=<name>\`
 * (\`web/public/bench/<name>.json.gz\`) open, and what \`__benchTime.load\` swaps into an open page.
 *
 *   bun tools/bench/state.mjs --code country-lrstm-7 --home house --hour 13 --name day13
 *   bun tools/bench/state.mjs --scene --world country --options town=village --seed 7 --home town --hour 13 --name town-day --bench
 *   bun tools/bench/state.mjs --from town-day --ticks 600 --name town-day-10s
 *
 * Where the game comes from (one of):
 * - \`--code <world code>\` (default \`country-lrstm-7\`, gameplay.mjs's): the five steps of New game as the screens run
 *   them (\`ui/screens/Create*.svelte\`, \`ChooseHousehold.svelte\`): the land, the layout proposed, the town built at
 *   the build's first seed, its households from the code's people seed, then Play on a household offered on the first
 *   page whose home is \`--home house|flat|any\` (the \`--household\`-th of them, from 0) and Move in, the homes still for
 *   sale filled at \`--tier\` (full: every preset's).
 * - \`--scene\`: scene.mjs's recipe (\`--world\`, \`--options\`, \`--seed\`, \`--home first|town|flat\`): the world generated
 *   at once, its neighbours and a household of four from the seed, moving into a home for sale.
 * - \`--from <file or name>\`: a save as it is (a \`.json\`, a \`.json.gz\`, or the name of one in \`debug-states/\` or
 *   \`web/public/bench/\`).
 *
 * To when: \`--hour 13\` (5 minutes before it on day 1, as scene.mjs plays to), \`--at 2:07:30\` (day 2, 07:30),
 * \`--tick <n>\` (the sim's tick count), or \`--ticks <n>\` past where it starts. The sim's dice: the people seed's (as
 * Move in has them), or the page's \`?seed=\` as \`--sim-seed\` (\`--scene\` takes \`--seed\`, as scene.mjs's page does).
 *
 * The sim simulates in full detail only round what the camera watches (\`sim_core::lod\`): here the player's lot is the
 * view and \`--watch 320\` the square of that side round it (about what the opening camera streams), as simtick.mjs's;
 * \`--watch all\` the whole town. A page that plays the state watches what its own camera streams, so the residents out
 * of full detail move otherwise from there on.
 *
 * The state is cached in \`bench-results/.states/\` by what makes it: the town file (so the generator's and the
 * content's code are in it), the sim's wasm, the dice, the watch and the tick; \`--fresh\` makes it again. \`--name\`
 * copies it to \`debug-states/<name>.json\` (\`?state=<name>\`), with \`--bench\` also to \`web/public/bench/<name>.json.gz\`
 * (\`?bench=<name>\`). Prints one line a script reads: \`state: <path> tick <n> day <d> minute <m> made|cached <ms>\`.
 * A state is this build's: another build makes its own from the same code at the same tick (an older one can't
 * read a newer save).
 *
 * The work (`lib/stateMake.mjs`) runs under Node, bundled by Bun on each run: the generator's floating point is V8's,
 * as in Chrome, so the town is the one the page makes (under Bun's engine a world came out with other lots).
 */

import { spawnSync } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const out = ROOT + 'bench-results/.states/';
mkdirSync(out, { recursive: true });
const built = await Bun.build({ entrypoints: [ROOT + 'tools/bench/lib/stateMake.mjs'], target: 'node', outdir: out, naming: 'state-node.mjs' });
if (!built.success) {
  for (const log of built.logs) console.error(log);
  process.exit(1);
}
const run = spawnSync('node', [out + 'state-node.mjs', ...process.argv.slice(2)], { stdio: 'inherit', env: { ...process.env, LL_ROOT: ROOT } });
process.exit(run.status ?? 1);
