/**
 * The benchmark's real big world (`?bench=bigworld`, `&big=<kind>`): a Big town of that world kind
 * made in the page as the Create World screen makes one (`generateWorld`, then
 * `populate`), turned into the structure the renderer draws as the menus' preview does
 * (`previewWorld`). The simulation can't hold one yet (towns over 256 m), so the game plays the
 * bench scene's village while the renderer draws this; nothing of it goes in git.
 *
 * The bench keeps its own focus lot in it (`benchFocus`): the lot the roaming camera settles on is
 * drawn in full straight away (the simulation, playing the village, has no view to send for it).
 */

import type { AssetRegistry } from '../assets/registry';
import type { Content } from '../content/content';
import type { WorldStructure } from '../core/protocol';
import { generateWorld, loadTemplates, populate } from '../game/town';
import { previewWorld } from '../game/townPreview';
import { WORLD_KINDS } from '../game/world';
import { perfTimeAsync } from './perf';

/** The big world of kind `kind` (an unknown one: 'small'), seed fixed: the same town every run. */
export async function benchBigWorld(content: Content, assets: AssetRegistry, kind: string): Promise<WorldStructure> {
  return perfTimeAsync('bench: big world', async () => {
    const t = await loadTemplates();
    const k = WORLD_KINDS.find((w) => w === kind) ?? 'small';
    const draft = populate(content, assets, t, generateWorld(content, t, k, 7));
    return previewWorld(content, t, draft);
  });
}
