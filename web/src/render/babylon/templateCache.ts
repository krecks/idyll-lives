/**
 * The Babylon renderer's template cache (`shared/templateCache.ts`): a template set aside is hidden
 * and disposed by its meshes (and its glows'), and a scene's listeners are told of a file again
 * through `retellModelFileArrived`.
 */

import type { Mesh, Scene } from './core';
import { retellModelFileArrived } from './models';
import { TemplateCache as SharedTemplateCache } from '../shared/templateCache';

/** What a cache keeps: a template's meshes (and its glows'), and the files it was built without. */
export interface CachedTemplate {
  meshes: Mesh[];
  glows?: { mesh: Mesh }[];
  failed?: string[];
}

const meshesOf = (t: CachedTemplate) => [...t.meshes, ...(t.glows ?? []).map((g) => g.mesh)];

export class TemplateCache<T extends CachedTemplate | null> extends SharedTemplateCache<T> {
  constructor(scene: Scene) {
    super({
      retell: (url) => retellModelFileArrived(scene, url),
      gone: () => scene.isDisposed,
      hide: (t) => {
        for (const mesh of meshesOf(t)) mesh.setEnabled(false);
      },
      dispose: (t) => {
        for (const mesh of meshesOf(t)) mesh.dispose(false, false);
      },
    });
  }
}
