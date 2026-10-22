/**
 * Makes sample worlds of every kind (web/src/game/world.ts) with the shipped content, the way a
 * new game does (a world, its neighbours, a player household moving into an empty house), and
 * writes their town files for the simulation to check
 * (`cargo test -p sim-core --test generated_worlds -- --ignored`, reading WORLDS).
 *
 * Run: bun tools/worlds/sample.ts worlds.json [worlds per kind]
 */

import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import type { HouseGenConfig } from '../../web/src/game/housegen.ts';
import type { Templates } from '../../web/src/game/town.ts';

// The loaders fetch from the page's address: here, files under web/public.
const root = new URL('../../web/public/', import.meta.url);
Object.assign(globalThis, { location: { href: root.href, search: '' } });
const fetchFile = async (url: string | URL) => {
  const text = readFileSync(fileURLToPath(new URL(url, root)), 'utf8');
  return { ok: true, status: 200, text: async () => text, json: async () => JSON.parse(text) };
};
Object.assign(globalThis, { fetch: fetchFile });

const { Content } = await import('../../web/src/content/content.ts');
const { AssetRegistry } = await import('../../web/src/assets/registry.ts');
const { objectInfos } = await import('../../web/src/game/housegen.ts');
const { randomHousehold } = await import('../../web/src/game/household.ts');
const { assembleTown, generateWorld, houseTemplate, populate, vacantSlots } = await import('../../web/src/game/town.ts');
const { WORLD_KINDS } = await import('../../web/src/game/world.ts');

const content = await Content.load(new URL('content/base.json', root).href);
const assets = await AssetRegistry.load(new URL('assets/manifest.json', root).href);
const read = (name: string) => JSON.parse(readFileSync(fileURLToPath(new URL(`content/${name}`, root)), 'utf8'));
const templates: Templates = { ...(read('houses.json') as Templates), gen: read('housegen.json') as HouseGenConfig, info: objectInfos(content), lampTiles: content.lampTiles };

const out = process.argv[2] ?? 'worlds.json';
const perKind = Number(process.argv[3] ?? 3);
const worlds: unknown[] = [];
for (const kind of WORLD_KINDS) {
  for (let i = 1; i <= perKind; i++) {
    const seed = i * 104729 + kind.length;
    const town = populate(content, assets, templates, generateWorld(content, templates, kind, seed));
    const home = vacantSlots(town)[0];
    if (!home) throw new Error(`${kind} ${seed}: no empty house`);
    const player = randomHousehold(content, assets, 2);
    const file = JSON.parse(assembleTown(content, templates, town, player, home.index));
    worlds.push({ kind, seed, name: town.name, town: file });
    const farms = town.slots.filter((s) => s.lot === 'farm').length;
    const basements = town.slots.filter((s) => s.kind === 'house' && houseTemplate(templates, s.template)?.basement).length;
    console.log(
      `${kind} ${seed}: ${town.name} (${town.world?.tagline}) ${town.width}×${town.depth}, ${town.streets.length} street(s), ` +
        `${town.slots.filter((s) => s.kind === 'house').length} houses${farms ? ` (${farms} farms)` : ''}, ${basements} with a basement, ${town.households.length} households, ` +
        `${file.sims.length} residents, ${file.objects.length} things`,
    );
  }
}
writeFileSync(out, JSON.stringify(worlds));
console.log(`${worlds.length} worlds → ${out}`);
