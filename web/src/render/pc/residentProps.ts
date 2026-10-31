/**
 * What residents hold, on PlayCanvas (`characters.ts`): the things they carry (content `items`, by
 * model), the tools on the station they cook at (`tool.*`), and the part of an object they use that
 * they hold (content `held`, `<model key>#<part>`: a guitar, a mop, a dumbbell in each hand). One
 * template per key, built the first time it's asked for; each of its meshes drawn instanced, one copy
 * per resident row, placed by a matrix of its own (game space, 16 floats a row of `matrices`; a row
 * of zeros draws nothing). A prop nobody holds isn't drawn at all.
 *
 * Built without its file, a prop is a small block (`ITEM_STAND_IN`), built again when the file comes
 * (the residents' code asks the cache to, `shared/characters/residents.ts`), the block taken down once
 * the new one is there.
 */

import * as pc from './engine';
import { Instances } from './instancing';
import { buildModel, retellModelFileArrived, uploadData, type ModelMesh, type ModelTemplate } from './models';
import type { AssetRegistry } from '../../assets/registry';
import type { PartColors } from '../../core/protocol';
import { partColors } from '../shared/characters/parts';
import { invert } from '../shared/math';
import { ITEM_STAND_IN, painted } from '../shared/modelEntry';
import { bakeData, type VData } from '../shared/modelParts';
import { TemplateCache } from '../shared/templateCache';

export interface Prop {
  meshes: ModelMesh[];
  /** A draw per mesh (all off while nobody holds it). */
  draws: Instances[];
  /** Per resident row, where it's drawn (game space; zeros: not in that row's hands). */
  matrices: Float32Array;
  /** Its highest point (a mop's handle; [0, 0, 0] for a carried thing). */
  top: readonly [number, number, number];
  /** The files it was built without (it's a small block then). */
  failed?: string[];
}

export class ResidentProps {
  /** The props by key (`ResidentView.props`). */
  readonly cache: TemplateCache<Prop | null>;

  constructor(
    private readonly app: pc.AppBase,
    private readonly parent: pc.Entity,
    private readonly assets: AssetRegistry,
    /** Resident rows (a prop's matrices hold one each). */
    private readonly rows: number,
  ) {
    let gone = false;
    app.once('destroy', () => (gone = true));
    this.cache = new TemplateCache<Prop | null>({
      retell: (url) => retellModelFileArrived(app, url),
      gone: () => gone,
      hide: (p) => {
        for (const d of p.draws) d.enabled = false;
      },
      dispose: (p) => destroyProp(p),
    });
  }

  /** Per frame: draws prop `p` for the first `count` rows (its matrices as written; 0: not at all), its copies uploaded (a few kB at most). */
  show(p: Prop, count: number): void {
    const used = count > 0;
    for (const d of p.draws) {
      if (d.enabled !== used) d.enabled = used;
      if (!used) continue;
      d.setCount(count);
      for (let i = 0; i < count; i++) d.setMatrix(i, p.matrices, i * 16);
      d.update(false);
    }
  }

  destroy(): void {
    this.cache.disposeStale();
    for (const [, t] of this.cache) void t.then((p) => p && destroyProp(p));
    this.cache.clear();
  }

  /**
   * Builds prop `key` (`<model>`, or `<model>#<part>[@<colorsKey>]`: `ResidentView.buildProp`); without its
   * file, a small block (built again when the file comes).
   */
  async build(key: string): Promise<Prop | null> {
    const at = key.indexOf('#');
    const p = at < 0 ? await this.whole(key) : await this.part(key.slice(0, at), ...partColors(key.slice(at + 1)));
    if (!p) return null;
    const draws = p.meshes.map((m) => new Instances(this.app.graphicsDevice, this.parent, m.mesh, m.material, { name: `prop:${key}`, castShadows: false, receiveShadows: true, capacity: this.rows, cull: false }));
    for (const d of draws) d.enabled = false;
    return { meshes: p.meshes, draws, matrices: new Float32Array(this.rows * 16), top: p.top, failed: p.failed };
  }

  private async whole(key: string): Promise<{ meshes: ModelMesh[]; top: readonly [number, number, number]; failed?: string[] }> {
    const t = await buildModel(this.app, `prop:${key}`, this.assets.get(key, 'model'), [1, 1], { standIn: ITEM_STAND_IN });
    for (const g of t.glows ?? []) g.mesh.mesh.destroy();
    return { meshes: t.meshes, top: [0, 0, 0], failed: t.failed };
  }

  /**
   * Moving part `part` of model `model` on its own (a held instrument), in the part's own frame (its
   * node's origin and axes at rest), painted in `colors` as the object is; and its top (the highest
   * point: a mop's handle); null if the model has no such part. Without the file, a small block.
   */
  private async part(model: string, part: string, colors?: PartColors): Promise<{ meshes: ModelMesh[]; top: readonly [number, number, number]; failed?: string[] } | null> {
    const t: ModelTemplate = await buildModel(this.app, `prop:${model}#${part}`, painted(this.assets.get(model, 'model'), colors), [1, 1], { standIn: ITEM_STAND_IN });
    if (t.failed) return { meshes: t.meshes, top: [0, t.height, 0], failed: t.failed };
    for (const g of t.glows ?? []) g.mesh.mesh.destroy();
    const motion = t.motion;
    const j = motion?.names?.indexOf(part) ?? -1;
    const keep = t.meshes.filter((_, k) => j >= 0 && motion!.parts[k] === j);
    for (const m of t.meshes) if (!keep.includes(m)) m.mesh.destroy();
    if (j < 0 || !motion?.rests) {
      for (const m of keep) m.mesh.destroy();
      return null;
    }
    const local = invert(motion.rests[j], new Float32Array(16));
    const top: [number, number, number] = [0, -Infinity, 0];
    const meshes = keep.map((m) => {
      const data = bakeData(readData(m.mesh), local);
      const p = data.positions;
      for (let v = 1; v < p.length; v += 3) if (p[v] > top[1]) [top[0], top[1], top[2]] = [p[v - 1], p[v], p[v + 1]];
      m.mesh.destroy();
      return { ...m, mesh: uploadData(this.app.graphicsDevice, data) };
    });
    return { meshes, top };
  }
}

function destroyProp(p: Prop): void {
  for (const d of p.draws) d.destroy();
  for (const m of p.meshes) m.mesh.destroy();
}

/** A mesh's vertices back from it (as `uploadData` made it). */
function readData(mesh: pc.Mesh): VData {
  const n = mesh.vertexBuffer?.getNumVertices() ?? 0;
  const positions = new Float32Array(n * 3);
  mesh.getPositions(positions);
  const normals = new Float32Array(n * 3);
  const uvs = new Float32Array(n * 2);
  const colors = new Float32Array(n * 4);
  const tangents = new Float32Array(n * 4);
  const indices: number[] = [];
  mesh.getIndices(indices);
  return {
    positions,
    normals: mesh.getNormals(normals) ? normals : null,
    uvs: mesh.getUvs(0, uvs) ? uvs : null,
    colors: mesh.getColors(colors) ? colors : null,
    tangents: mesh.getVertexStream(pc.SEMANTIC_TANGENT, tangents) ? tangents : null,
    indices: Uint32Array.from(indices),
  };
}
