// The close-up body (`hd.body`, `data.ts` `loadCloseUp`): in files of its own, which only a view that asks for it
// fetches (the creator's stage), once each; drawn in the body's place for the residents flagged for it and for no
// one else; its morphs after the set's own, which stay as they are; and a file that doesn't come leaves the body
// drawn. `bun test`.
//
// (Why: as a spike the part sat in every body's file, 1.29 MB gzipped of the 5.14 MB a player fetched, and was
// thrown away at load.)
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('the close-up body', async () => {
  const realLocation = globalThis.location;
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
  const realFetch = globalThis.fetch;
  /** Every file asked for, by name; and the names answered with a 404 though they exist. */
  const fetched: string[] = [];
  const missing = new Set<string>();
  globalThis.fetch = (async (input: string | URL | Request) => {
    const url = new URL(typeof input === 'string' ? input : input instanceof URL ? input.href : input.url, 'http://localhost/');
    const name = decodeURIComponent(url.pathname).split('/').pop()!;
    fetched.push(name);
    const path = `${WEB}public${decodeURIComponent(url.pathname)}`;
    return existsSync(path) && !missing.has(name) ? new Response(readFileSync(path)) : new Response('missing', { status: 404 });
  }) as typeof fetch;
  afterAll(() => {
    globalThis.fetch = realFetch;
    Object.assign(globalThis, { location: realLocation });
  });
  const { Content } = await import('../src/content/content');
  const { AssetRegistry } = await import('../src/assets/registry');
  const { Characters, MAX_CHARACTERS } = await import('../src/render/shared/characters/residents');
  const { CLOSE_UP, loadCharacterSet } = await import('../src/render/shared/characters/data');
  const { TemplateCache } = await import('../src/render/shared/templateCache');
  type ResidentProp = import('../src/render/shared/characters/view').ResidentProp;
  type View = import('../src/render/shared/characters/view').ResidentView<object, object, ResidentProp>;
  type MorphSet = import('../src/render/shared/characters/data').MorphSet;
  type SimInfo = import('../src/core/protocol').SimInfo;

  const content = await Content.load('http://localhost/content/base.json');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json');
  const set = await loadCharacterSet(assets.get('model.sim', 'character')!.url);

  /** A view that keeps the names of the parts made and the morphs it was last given. */
  function viewOf(): { view: View; parts: string[]; morphs: MorphSet[] } {
    const parts: string[] = [];
    const morphs: MorphSet[] = [];
    const view: View = {
      init: () => {},
      setMorphs: (m) => void morphs.push(m),
      uploadPoses: () => {},
      uploadPoseRows: () => {},
      createPart(body, part, shadow) {
        if (!shadow) parts.push(`${body.name}:${part}`);
        return {};
      },
      partEnabled: () => {},
      setPartRows: () => {},
      partRows: () => {},
      quiltMaterial: () => {},
      createQuilt: () => ({}),
      quiltRows: () => {},
      quiltEnabled: () => {},
      createRing: () => {},
      ringMoved: () => {},
      props: new TemplateCache<ResidentProp | null>({ retell: () => {}, gone: () => false, hide: () => {}, dispose: () => {} }),
      async buildProp(key) {
        return { key, matrices: new Float32Array(MAX_CHARACTERS * 16), top: [0, 0, 0] as const };
      },
      showProp: () => {},
      onModelFile: () => () => {},
      fileColour: async () => undefined,
      camera: () => false,
      onDispose: () => {},
    };
    return { view, parts, morphs };
  }
  const sim = (id: number, gender: string, stage: string | undefined, closeUp: boolean): SimInfo => ({
    id,
    name: `sim${id}`,
    household: 0,
    gender,
    attractedTo: [],
    appearance: { body: '#5F86C9', skin: '#C08A64', hair: '#2B2320', hairStyle: 'short', height: 1, build: 1, top: 'top.tee', bottom: 'bottom.trousers', shoes: 'shoes.sneakers', ...(stage ? { stage } : {}) },
    traits: [],
    perks: [],
    ...(closeUp ? { closeUp } : {}),
  });
  const hd = () => fetched.filter((f) => f.includes('.hd.'));

  test('is fetched by no one who does not ask: the residents of every life stage are built without it', async () => {
    const { view, parts } = viewOf();
    const characters = new Characters(view, assets, content, {});
    expect(await characters.init()).toBe(true);
    const stages = ['baby', 'child', 'teen', undefined, 'adult', 'elder'];
    // (Flagged, too: the flag draws it only once someone has fetched it.)
    await characters.build({ sims: ['male', 'female'].flatMap((g, i) => stages.map((s, j) => sim(i * 6 + j, g, s, true))), objects: [] });
    expect(fetched.filter((f) => f.endsWith('.bin')).length).toBe(12);
    expect(hd()).toEqual([]);
    expect(parts.filter((p) => p.endsWith(`:${CLOSE_UP}`))).toEqual([]);
    expect(parts).toContain('male.child:body');
    expect(characters.hasCloseUp('male', 'child')).toBe(false);
  });

  test("comes when asked for, once: the family's file and the stage's, and the residents flagged for it draw it", async () => {
    const { view, parts, morphs } = viewOf();
    const characters = new Characters(view, assets, content, {});
    expect(await characters.init()).toBe(true);
    const own = set.morphs!;
    const [data, height] = [own.data, own.height];
    const before = data.slice();
    /** The texels the set's own morphs use (the rest of their last row is spare). */
    const used = set.morphLayout!.texels;
    expect(await Promise.all([characters.closeUp('male', 'child'), characters.closeUp('male', 'child')])).toEqual([true, true]);
    expect(await characters.closeUp('male', 'child')).toBe(true);
    expect(hd()).toEqual(['male.hd.bin', 'male.child.hd.bin']);
    expect(characters.hasCloseUp('male', 'child')).toBe(true);
    expect(characters.hasCloseUp('male')).toBe(true);
    expect(characters.hasCloseUp('female')).toBe(false);
    // One flagged for it, one not, and a woman flagged whose close-up body nobody fetched.
    await characters.build({ sims: [sim(0, 'male', 'child', true), sim(1, 'male', undefined, false), sim(2, 'female', undefined, true)], objects: [] });
    expect(parts).toContain(`male.child:${CLOSE_UP}`);
    expect(parts).not.toContain('male.child:body');
    expect(parts).toContain('male:body');
    expect(parts).not.toContain(`male:${CLOSE_UP}`);
    expect(parts).toContain('female:body');
    // Its morphs: after the set's own, in a larger copy the view was given; the set's own untouched.
    expect(set.morphs).toBe(own);
    expect(own.data).toBe(data);
    expect(own.height).toBe(height);
    expect(data).toEqual(before);
    const union = set.closeUpMorphs!;
    expect(morphs).toEqual([union]);
    expect(union.height).toBeGreaterThan(height);
    expect(union.data.subarray(0, used * 4)).toEqual(data.subarray(0, used * 4));
    expect(set.morphLayout!.texels).toBeGreaterThan(used);
    // The close-up body's vertices that morphs move point past the set's own texels.
    const part = set.bodies.get('male.child')!.parts.get(CLOSE_UP)!;
    const slots = part.morph!;
    let moved = 0;
    for (let v = 0; v < slots.length / 4; v++) {
      if (slots[v * 4] < 0) continue;
      moved++;
      expect(slots[v * 4]).toBe(used);
    }
    expect(moved).toBeGreaterThan(1000);
    // A life stage's shares the base's morph slots and has its own shape.
    const base = set.bodies.get('male')!.parts.get(CLOSE_UP)!;
    expect(part.morph).toBe(base.morph);
    expect(part.positions).not.toEqual(base.positions);
    expect(part.positions.length).toBe(base.positions.length);
  });

  test('that does not come leaves the body drawn, and is asked for again the next time', async () => {
    const { view, parts, morphs } = viewOf();
    const characters = new Characters(view, assets, content, {});
    expect(await characters.init()).toBe(true);
    const warn = console.warn;
    console.warn = () => {};
    missing.add('female.hd.bin');
    const from = fetched.length;
    try {
      expect(await characters.closeUp('female')).toBe(false);
    } finally {
      console.warn = warn;
    }
    expect(characters.hasCloseUp('female')).toBe(false);
    await characters.build({ sims: [sim(0, 'female', undefined, true)], objects: [] });
    expect(parts).toContain('female:body');
    expect(parts).not.toContain(`female:${CLOSE_UP}`);
    expect(morphs).toEqual([]);
    missing.delete('female.hd.bin');
    expect(await characters.closeUp('female')).toBe(true);
    expect(fetched.slice(from).filter((f) => f === 'female.hd.bin').length).toBe(2);
    expect(characters.hasCloseUp('female')).toBe(true);
  });
});
