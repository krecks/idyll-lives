/**
 * A glTF file read through PlayCanvas's container (`containerOptions`: meshopt, KTX2) for the
 * models (`models.ts`), which read the geometry from its buffer views themselves: the views are
 * kept as they're decoded, through the container's buffer-view hook.
 */

import * as pc from './engine';
import { containerOptions, setupDecoders } from './loaders';

/** What PlayCanvas's container gives of a glTF file: the JSON, its buffer views decoded, its materials and their textures. */
export interface Container<G extends { bufferViews?: unknown[] }> {
  gltf: G;
  views: (Uint8Array | undefined)[];
  materials: (pc.StandardMaterial | null)[];
  textures: pc.Asset[];
}

/** The longest a file waits after it loads for the last of its views to be decoded (ms). */
export const VIEWS_WAIT_MS = 5000;

/**
 * The file at `url` through PlayCanvas's glTF container, its buffer views kept as they're decoded.
 * PlayCanvas makes none of its own meshes (`skipMeshes`): the geometry is read from the views
 * (`models.ts`); its materials stay.
 *
 * Each call reads the file afresh. PlayCanvas's resource loader keeps what it loaded by URL for
 * its app and hands a second load of the URL that instead, without opening the file again: none
 * of its views would come through the hook, and the models would wait `VIEWS_WAIT_MS` for them
 * and build a box. A file let go of (`ModelFiles` keeps the last 96 nobody uses: the catalog's
 * pictures go past that in a minute) and wanted again is loaded again, so what the loader kept is
 * dropped once it's taken (and the loader keeps no file's container for the page's life either).
 */
export function loadContainer<G extends { bufferViews?: unknown[] }>(app: pc.AppBase, url: string): Promise<Container<G>> {
  setupDecoders();
  const decoded = new Map<object, Uint8Array>();
  // Every view is decoded, but PlayCanvas's `load` waits only for those it uses itself (with
  // `skipMeshes`, not the geometry read here): when it comes, the rest may still be on their way,
  // later in some browsers than others (Firefox). Read once all of them are in.
  let views = Infinity;
  let allIn: () => void = () => {};
  const decodedAll = new Promise<void>((resolve) => (allIn = resolve));
  const options = {
    ...containerOptions,
    bufferView: {
      ...containerOptions.bufferView,
      postprocess: (view: object, data: Uint8Array) => {
        decoded.set(view, data);
        if (decoded.size >= views) allIn();
      },
    },
    // (Its meshes would be uploaded only to be let go of: the geometry is read from the views here.
    // Without them a skinned node has no mesh to skin: skins aren't read here either.)
    skipMeshes: true,
    global: {
      preprocess: (gltf: { nodes?: { skin?: number }[]; bufferViews?: unknown[] }) => {
        for (const node of gltf.nodes ?? []) delete node.skin;
        views = gltf.bufferViews?.length ?? 0;
        if (decoded.size >= views) allIn();
      },
    },
  };
  return new Promise((resolve, reject) => {
    const asset = new pc.Asset(url, 'container', { url }, undefined, options as never);
    let timer: ReturnType<typeof setTimeout> | undefined;
    const viewsIn = () => Promise.race([decodedAll, new Promise((resolve) => (timer = setTimeout(resolve, VIEWS_WAIT_MS)))]).finally(() => clearTimeout(timer));
    asset.once('load', () => {
      // (The next load of the URL opens the file again, its views through the hook: above.)
      app.loader.clearCache(asset.getFileUrl() ?? url, 'container');
      void viewsIn().then(read).catch(reject);
    });
    const read = () => {
      const resource = asset.resource as pc.ContainerResource & { data: { gltf: G; materials: pc.StandardMaterial[]; textures?: pc.Asset[] } };
      const gltf = resource.data.gltf;
      const container = { gltf, views: (gltf.bufferViews ?? []).map((v) => decoded.get(v as object)), materials: resource.data.materials.map((m) => m ?? null), textures: [...(resource.data.textures ?? [])] };
      // (No meshes were made, `skipMeshes`); the materials (and their textures) stay with the file.
      for (const render of resource.renders) {
        render.unload();
        app.assets.remove(render);
      }
      for (const sub of [...resource.materials, ...resource.animations]) app.assets.remove(sub);
      app.assets.remove(asset);
      resolve(container);
    };
    asset.once('error', (err: unknown) => {
      app.assets.remove(asset);
      reject(new Error(`${url}: ${String(err)}`));
    });
    app.assets.add(asset);
    app.assets.load(asset);
  });
}
