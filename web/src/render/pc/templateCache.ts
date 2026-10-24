/**
 * The PlayCanvas renderer's template cache (`shared/templateCache.ts`): a template set aside is hidden
 * by turning off its meshes' instanced draws (`instancing.ts`) and disposed by letting go of its
 * meshes, and an app's listeners are told of a file again through `retellModelFileArrived`.
 */

import type * as pc from './engine';
import { instancesOf } from './instancing';
import { disposeTemplate, retellModelFileArrived, type ModelMesh } from './models';
import { TemplateCache as SharedTemplateCache } from '../shared/templateCache';

/** What a cache keeps: a template's meshes (and its glows'), and the files it was built without. */
export interface CachedTemplate {
  meshes: ModelMesh[];
  glows?: { mesh: ModelMesh }[];
  failed?: string[];
}

export class TemplateCache<T extends CachedTemplate | null> extends SharedTemplateCache<T> {
  constructor(app: pc.AppBase) {
    let gone = false;
    app.once('destroy', () => (gone = true));
    super({
      retell: (url) => retellModelFileArrived(app, url),
      gone: () => gone,
      hide: (t) => {
        for (const m of [...t.meshes, ...(t.glows ?? []).map((g) => g.mesh)]) for (const draw of instancesOf(m.mesh)) draw.enabled = false;
      },
      dispose: (t) => disposeTemplate(t),
    });
  }
}
