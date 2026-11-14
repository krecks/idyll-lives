/**
 * What every game shares and the menus need first: the content and the asset registry (loaded once a
 * page, at boot) and the simulation's compiled wasm. Apart from `session.ts`, which is a script chunk
 * of its own (`host.ts` loads it once the menu is up, or when a game starts): the menus don't wait
 * for the session's and Build mode's code.
 */
import { AssetRegistry } from '../assets/registry';
import { Content } from '../content/content';
import { perfTimeAsync } from '../debug/perf';
import { createSimPreviews } from '../render/preview';
import { createItemPreviews } from '../render/preview/items';
import { settings as liveSettings } from '../settings/settings.svelte';
import { services } from '../ui/services';
import { useTownContent } from './town';

/** The simulation's wasm, compiled once a page (from boot on) and handed to each game's worker; null if it couldn't be (the worker then fetches it). */
let simModule: Promise<WebAssembly.Module | null> | null = null;
export function compileSim(): Promise<WebAssembly.Module | null> {
  simModule ??= (async () => {
    const url = new URL('../core/wasm-pkg/sim_wasm_bg.wasm', import.meta.url);
    try {
      return await (typeof WebAssembly.compileStreaming === 'function' ? WebAssembly.compileStreaming(fetch(url)) : WebAssembly.compile(await (await fetch(url)).arrayBuffer()));
    } catch (err) {
      console.warn('[session] the simulation could not be compiled ahead; each game compiles its own', err);
      return null;
    }
  })();
  return simModule;
}

/** Shared, immutable data: loaded once per page. */
let shared: Promise<{ content: Content; assets: AssetRegistry }> | null = null;
export function loadGameData(): Promise<{ content: Content; assets: AssetRegistry }> {
  const params = new URLSearchParams(location.search);
  const base = import.meta.env.BASE_URL;
  shared ??= Promise.all([
    perfTimeAsync('data: content', () => Content.load(`${base}content/base.json`)),
    perfTimeAsync('data: assets', () => AssetRegistry.load(`${base}assets/manifest.json`, params.getAll('pack'))),
  ]).then(([content, assets]) => {
    services.content = content;
    services.assets = assets;
    useTownContent(content);
    services.previews = createSimPreviews({ assets, emotions: content.emotions.map((e) => e.id), backend: () => liveSettings.renderer });
    services.items = createItemPreviews(assets, () => liveSettings.renderer);
    // (The simulation compiles meanwhile, once: every game's worker takes it as it is.)
    void compileSim();
    return { content, assets };
  });
  return shared;
}

