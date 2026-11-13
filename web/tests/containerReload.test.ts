// A model file loaded again on the same app (`loadContainer` in render/pc/container.ts): PlayCanvas's resource loader
// keeps what it loaded by URL and hands a second load that, without opening the file: none of its buffer views came
// through the hook, the models waited `VIEWS_WAIT_MS` for them and built a box. The catalog's turntable showed a grey
// box for a sofa or a fridge whose file the pictures' app had let go of (past `ModelFiles`' 96 idle files) and wanted
// again. On the engine's null device, with a stand-in for PlayCanvas's glTF container that opens a file as the real
// one does: its JSON through the `global.preprocess` hook, then each view through `bufferView.postprocess`.
import { afterAll, describe, expect, test } from 'bun:test';
import * as pc from '../src/render/pc/engine';

// (The renderer's modules read the page's address as they load: one put in for them, the real one back after.)
const realLocation = globalThis.location;
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
afterAll(() => Object.assign(globalThis, { location: realLocation }));
const { loadContainer, VIEWS_WAIT_MS } = await import('../src/render/pc/container');

interface Hooks {
  global: { preprocess(gltf: object): void };
  bufferView: { postprocess(view: object, data: Uint8Array): void };
}

/** An app on the null device whose `container` files are opened by a stand-in: a glTF of `views` views, each a byte of its index. */
function appWithFiles(views: number) {
  const canvas = { width: 1, height: 1, style: {}, addEventListener() {}, removeEventListener() {}, getBoundingClientRect: () => ({ width: 1, height: 1 }) } as unknown as HTMLCanvasElement;
  const app = new pc.AppBase(canvas);
  const options = new pc.AppOptions();
  options.graphicsDevice = new pc.NullGraphicsDevice(canvas);
  options.componentSystems = [];
  options.resourceHandlers = [];
  app.init(options);
  let opened = 0;
  app.loader.addHandler('container', {
    handlerType: 'container',
    load(_url: unknown, callback: (err: unknown, data?: unknown) => void, asset: pc.Asset) {
      opened++;
      // (Each opening parses the file afresh: a new JSON, new views.)
      const gltf = { bufferViews: Array.from({ length: views }, () => ({})) };
      const hooks = asset.options as unknown as Hooks;
      hooks.global.preprocess(gltf);
      setTimeout(() => {
        gltf.bufferViews.forEach((v, i) => hooks.bufferView.postprocess(v, new Uint8Array([i])));
        callback(null, { data: { gltf, materials: [], textures: [] }, renders: [], materials: [], animations: [] });
      }, 0);
    },
    open: (_url: unknown, data: unknown) => data,
    patch() {},
  } as unknown as pc.ResourceHandler);
  return { app, opened: () => opened };
}

/** The file's views as read, or 'late' if they didn't all come well before `VIEWS_WAIT_MS`. */
async function views(app: pc.AppBase, url: string) {
  const late = new Promise<'late'>((resolve) => setTimeout(() => resolve('late'), VIEWS_WAIT_MS / 5));
  const file = await Promise.race([loadContainer(app, url), late]);
  return file === 'late' ? file : file.views.map((v) => v?.[0]);
}

describe('a model file loaded again on the same app', () => {
  test('is opened again, every view with it', async () => {
    const { app, opened } = appWithFiles(3);
    expect(await views(app, '/assets/models/living/sofa.glb')).toEqual([0, 1, 2]);
    // Let go of (`ModelFiles`' eviction) and wanted again: the turntable after the catalog's pictures.
    expect(await views(app, '/assets/models/living/sofa.glb')).toEqual([0, 1, 2]);
    expect(opened()).toBe(2);
  });

  test("leaves nothing in the loader's cache", async () => {
    const { app } = appWithFiles(2);
    await views(app, '/assets/models/kitchen/fridge.glb');
    expect(app.loader.getFromCache('/assets/models/kitchen/fridge.glb', 'container')).toBeUndefined();
  });
});
