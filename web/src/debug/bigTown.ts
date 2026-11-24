/**
 * The benchmark's real big world (`?bench=bigworld`, `&big=<place>`, `&options=town:city,…`): a Big
 * town in that place with those options made in the page as the Create World screen makes one (`generateWorld`, then
 * `populate`), turned into the structure the renderer draws as the menus' preview does
 * (`previewWorld`). The simulation can't hold one yet (towns over 256 m), so the game plays the
 * bench scene's village while the renderer draws this; nothing of it goes in git.
 *
 * `big=roads`: the road hierarchy's fixture set in a flat countryside instead (`roadFixture.ts`).
 *
 * The bench keeps its own focus lot in it (`benchFocus`): the lot the roaming camera settles on is
 * drawn in full straight away (the simulation, playing the village, has no view to send for it).
 */

import type { AssetRegistry } from '../assets/registry';
import type { Content } from '../content/content';
import type { WorldOptions, WorldStructure } from '../core/protocol';
import { withLand } from '../game/landFollow';
import { generateWorld, loadTemplates, populate } from '../game/town';
import { previewWorld } from '../game/townPreview';
import { BENCH } from './benchFlags';
import { perfTimeAsync } from './perf';

/**
 * The big world in place `kind` (a place, or a kind from before places: `LEGACY_KINDS`; an unknown
 * one: the countryside) with `options` (default the URL's, `options=`; the place's own where not
 * given), seed fixed: the same town every run.
 */
export async function benchBigWorld(content: Content, assets: AssetRegistry, kind: string, options: Partial<WorldOptions> = BENCH?.options ?? {}): Promise<WorldStructure> {
  // (`big=roads`: the road hierarchy's fixture in a flat countryside with no town: `roadFixture.ts`.)
  if (kind === 'roads') {
    const base = await benchBigWorld(content, assets, 'country', { town: 'hamlet', villages: 'none', hills: 'flat', woods: 'open', ...options });
    return (await import('./roadFixture')).roadFixtureWorld(base);
  }
  return perfTimeAsync('bench: big world', async () => {
    const t = await loadTemplates();
    let laid = generateWorld(content, t, kind, 7, undefined, undefined, options);
    // (`&follow`: its land graded under its town, as a new game's is: `landFollow.ts` `withLand`.)
    if (typeof location !== 'undefined' && new URLSearchParams(location.search).has('follow')) laid = withLand(laid, previewWorld(content, t, laid, null));
    const draft = populate(content, assets, t, laid);
    return previewWorld(content, t, draft);
  });
}
