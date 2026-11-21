/**
 * The built world's dump (`tools/built/samples.ts`, read by `cargo check-glb --built`): the samples are the same
 * every time they are made, they cover what the gate is there to hold (each roof, each covering and style, a porch,
 * a garage, diagonal walls, each building's use, each fence, each car), every far version names the sample it is the
 * far version of, and a sample's file is a `.glb` whose triangles wind as their normals say once the game's z is
 * mirrored.
 */
import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { glbOf, type DumpMesh } from '../../tools/built/glb';
import { index, samples, type Sample } from '../../tools/built/samples';
import { CARS } from '../src/render/shared/cars';

const content = JSON.parse(readFileSync(new URL('../public/content/base.json', import.meta.url), 'utf8')) as {
  wallCoverings: { finish: string }[];
  floorCoverings: { finish: string }[];
  fenceStyles: { id: string; kind: string }[];
};

describe('the built world as dumped', async () => {
  const list = await samples();
  const by = new Map(list.map((s) => [s.key, s]));
  const part = (s: Sample, name: string): DumpMesh[] => s.meshes.filter((m) => m.name === name);
  const triangles = (meshes: DumpMesh[]) => meshes.reduce((n, m) => n + m.indices.length / 3, 0);

  test('the same samples, to the byte, every time', async () => {
    const again = await samples();
    expect(again.map((s) => s.key)).toEqual(list.map((s) => s.key));
    for (const [i, s] of list.entries()) expect(Bun.hash(glbOf(again[i].meshes))).toBe(Bun.hash(glbOf(s.meshes)));
    expect(index(again)).toBe(index(list));
  });

  test('every key once, every far version of a sample there is', () => {
    expect(by.size).toBe(list.length);
    for (const s of list) {
      expect(s.meshes.length).toBeGreaterThan(0);
      if (s.kind === 'neighbour' || s.kind === 'far' || s.kind === 'town') expect(by.get(s.of!)?.kind).toBe('house');
      else if (s.kind === 'buildingFar') expect(by.get(s.of!)?.kind).toBe('building');
      else expect(s.of).toBeUndefined();
    }
    // A house is dumped four ways, a lot's building closed two.
    const houses = list.filter((s) => s.kind === 'house');
    for (const kind of ['neighbour', 'far', 'town'] as const) expect(list.filter((s) => s.kind === kind).map((s) => s.of)).toEqual(houses.map((s) => s.key));
    expect(list.filter((s) => s.kind === 'buildingFar').length).toBe(list.filter((s) => s.kind === 'building').length);
  });

  test('the houses cover the roofs, the storeys, a porch, a garage and diagonal walls', () => {
    const houses = list.filter((s) => s.kind === 'house');
    expect(houses.some((s) => s.storeys === 1)).toBe(true);
    expect(houses.some((s) => s.storeys === 2)).toBe(true);
    // Gable ends on some, none on others (hips and flat roofs); a roof on the lower storey round the upper one.
    expect(houses.some((s) => triangles(part(s, 'gables')) > 0)).toBe(true);
    expect(houses.some((s) => triangles(part(s, 'gables')) === 0)).toBe(true);
    for (const key of ['house.ring', 'house.wing', 'house.corner']) {
      const roofs = part(by.get(key)!, 'roof').map((m) => m.extras?.storey);
      expect(roofs).toEqual([0, 1]);
    }
    // The lot's own share of the street: a porch's and a garage's roofs and trim.
    const lot = (s: Sample) => s.meshes.filter((m) => m.extras?.lot);
    expect(lot(by.get('house.cottage')!).some((m) => m.name === 'silhouetteRoofs')).toBe(true);
    expect(lot(by.get('house.brick')!).some((m) => m.name === 'silhouetteBrick')).toBe(true);
    // Diagonal walls: faces that look neither along x nor along z, upright.
    const walls = part(by.get('house.finishes')!, 'wallsExterior')[0];
    let slanted = 0;
    for (let i = 0; i < walls.normals.length; i += 3) if (Math.abs(walls.normals[i]) > 0.5 && Math.abs(walls.normals[i + 2]) > 0.5) slanted++;
    expect(slanted).toBeGreaterThan(0);
  });

  test('one house has every wall and floor finish, and each kind of door leaf', () => {
    const s = by.get('house.finishes')!;
    for (const finish of new Set(content.wallCoverings.map((c) => c.finish))) expect([finish, triangles(part(s, `walls-${finish}`)) > 0]).toEqual([finish, true]);
    const floors: Record<string, string> = { wood: 'floors', whitewash: 'floorsWhitewashed', tile: 'floorsTiled', carpet: 'floorsCarpet', stone: 'floorsStone' };
    for (const finish of new Set(content.floorCoverings.map((c) => c.finish))) expect([finish, triangles(part(s, floors[finish] ?? `floors-${finish}`)) > 0]).toEqual([finish, true]);
    // Painted leaves, an oak one, glazed ones; clear glass in the windows.
    const leaves = part(s, 'doorLeaf');
    expect(leaves.some((m) => m.material === 'material.trim')).toBe(true);
    expect(leaves.some((m) => m.material === 'material.finish.wood')).toBe(true);
    expect(triangles(part(s, 'doorLeafGlass'))).toBeGreaterThan(0);
    expect(triangles(part(s, 'windows'))).toBeGreaterThan(0);
  });

  test('every building use, every fence style that is built, every car', () => {
    const uses = ['flats', 'shop', 'cafe', 'restaurant', 'office', 'school', 'church', 'hall', 'workshop', 'farm'];
    expect(list.filter((s) => s.kind === 'building').map((s) => s.key).sort()).toEqual(uses.map((u) => `building.${u}`).sort());
    expect(list.filter((s) => s.kind === 'fence').map((s) => s.key)).toEqual(content.fenceStyles.map((f) => `fence.${f.id}`));
    expect(list.filter((s) => s.kind === 'car').map((s) => s.key)).toEqual(CARS.map((b) => `car.${b.id}`));
    // The street: its road, its sidewalks, its markings, the neighbours' houses and what stands along it.
    const street = by.get('street.block')!;
    for (const material of ['material.road', 'material.sidewalk', 'built.roadPaint', 'material.roof', 'built.plain']) expect([material, street.meshes.some((m) => m.material === material)]).toEqual([material, true]);
  });

  test("a sample's file is a .glb whose triangles wind as their normals say", () => {
    for (const key of ['house.cottage', 'neighbour.cottage', 'far.cottage', 'town.cottage', 'building.cafe', 'buildingFar.cafe', 'fence.picket', 'car.saloon']) {
      const s = by.get(key)!;
      const glb = glbOf(s.meshes);
      const view = new DataView(glb.buffer);
      expect(view.getUint32(0, true)).toBe(0x46546c67);
      expect(view.getUint32(8, true)).toBe(glb.length);
      const gltf = JSON.parse(new TextDecoder().decode(glb.subarray(20, 20 + view.getUint32(12, true)))) as { nodes: { name: string }[]; meshes: unknown[]; buffers: { byteLength: number }[] };
      expect(gltf.nodes.map((n) => n.name)).toEqual(s.meshes.map((m) => m.name));
      expect(20 + view.getUint32(12, true) + 8 + gltf.buffers[0].byteLength).toBe(glb.length);
      let [turned, faces] = [0, 0];
      for (const m of s.meshes) {
        const at = (i: number) => [m.positions[i * 3], m.positions[i * 3 + 1], m.positions[i * 3 + 2]];
        for (let t = 0; t < m.indices.length; t += 3) {
          const [a, b, c] = [at(m.indices[t]), at(m.indices[t + 1]), at(m.indices[t + 2])];
          const [u, v] = [b.map((x, k) => x - a[k]), c.map((x, k) => x - a[k])];
          const n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
          const i = m.indices[t] * 3;
          const facing = n[0] * m.normals[i] + n[1] * m.normals[i + 1] + n[2] * m.normals[i + 2];
          // (A sliver has no winding to speak of.)
          if (Math.hypot(...n) < 1e-9) continue;
          faces++;
          if (facing < 0) turned++;
        }
      }
      expect([key, turned]).toEqual([key, 0]);
      expect(faces).toBeGreaterThan(0);
    }
  });
});
