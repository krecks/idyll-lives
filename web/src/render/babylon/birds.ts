/**
 * Garden birds on Babylon: the engine-neutral birds (`shared/birds.ts`: who lives where, how they
 * perch, fly, forage, sing and sleep, and every matrix drawn) as thin instances, a buffer per
 * species and part (body, head, tail, the two spread wings), only the parts written in a frame going
 * up.
 */

import { Vector3, VertexBuffer, type Mesh, type Scene } from './core';
import type { AssetRegistry } from '../../assets/registry';
import { buildModel, type ModelTemplate } from './models';
import { windSway } from './nature';
import { TemplateCache } from './templateCache';
import { BIRD_PARTS, birdLook, Birds as SharedBirds, noBirdLook, roofRidge as ridgeOf, type BirdEntry, type BirdLook, type BirdView } from '../shared/birds';

export type { BirdFrame, BirdHabitat, BirdQuality, BirdSpecies } from '../shared/birds';

/** A species' look on Babylon: all its meshes (for `TemplateCache`), and by part. */
interface Look extends BirdLook {
  meshes: Mesh[];
  parts: Mesh[][];
}

export class Birds extends SharedBirds<Look> {
  constructor(scene: Scene, assets: AssetRegistry, debugging = false) {
    super(birdView(scene, assets), assets, debugging);
  }
}

/** The birds' engine side on a Babylon scene. */
function birdView(scene: Scene, assets: AssetRegistry): BirdView<Look> {
  const looks = new TemplateCache<Look>(scene);
  return {
    look: (id, key) =>
      looks.get(key, async () => {
        const entry = assets.get(key, 'model') as BirdEntry | undefined;
        const template = await buildModel(scene, `bird:${id}`, entry, [1, 1], { parts: BIRD_PARTS });
        if (entry && template.meshes.length && template.motion) return lookFrom(entry, template);
        for (const mesh of template.meshes) mesh.dispose(false, false);
        return { ...noBirdLook(template.failed), meshes: [], parts: [] };
      }),
    invalidate: (url) => looks.invalidate(url).length > 0,
    disposeStale: () => looks.disposeStale(),
    show: (visible) =>
      looks.forEachBuilt((look) => {
        for (const mesh of look.meshes) mesh.setEnabled(visible && look.count > 0);
      }),
    setBuffers: (look, visible) =>
      look.parts.forEach((list, k) => {
        for (const mesh of list) {
          mesh.thinInstanceSetBuffer('matrix', look.buffers[k], 16, false);
          mesh.thinInstanceCount = look.count;
          mesh.setEnabled(visible && look.count > 0);
        }
      }),
    updated: (look, part) => {
      for (const mesh of look.parts[part]) mesh.thinInstanceBufferUpdated('matrix');
    },
    sway: (x, z) => windSway(scene, x, z),
    dispose: () => {
      looks.forEachBuilt((look) => {
        for (const mesh of look.meshes) mesh.dispose(false, false);
      });
      looks.clear();
      looks.disposeStale();
    },
  };
}

/** A species' look from its model, its parts hidden until it has birds. */
function lookFrom(entry: BirdEntry, template: ModelTemplate): Look {
  const parts: Mesh[][] = BIRD_PARTS.map(() => []);
  template.meshes.forEach((mesh, k) => parts[Math.max(0, template.motion!.parts[k])].push(mesh));
  for (const mesh of template.meshes) {
    mesh.isPickable = false;
    mesh.receiveShadows = true;
    mesh.alwaysSelectAsActiveMesh = true;
    mesh.doNotSyncBoundingInfo = true;
    mesh.setEnabled(false);
  }
  return {
    ...birdLook(
      entry,
      BIRD_PARTS.map((_, k) => template.motion!.rests?.[k]?.m),
    ),
    meshes: template.meshes,
    parts,
  };
}

/**
 * The viewed house's roof ridge as the birds' perches (`shared/birds.ts` `roofRidge`), from the
 * roofs' vertices in the world; and the roof's top (m), null without a roof.
 */
export function roofRidge(roofs: readonly Mesh[]): { ridge: number[]; top: number | null } {
  const p = new Vector3();
  return ridgeOf((fn) => {
    for (const roof of roofs) {
      const pos = roof.getVerticesData(VertexBuffer.PositionKind);
      if (!pos) continue;
      const m = roof.computeWorldMatrix(true);
      for (let i = 0; i < pos.length; i += 3) {
        Vector3.TransformCoordinatesFromFloatsToRef(pos[i], pos[i + 1], pos[i + 2], m, p);
        fn(p.x, p.y, p.z);
      }
    }
  });
}
