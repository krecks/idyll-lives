// The residents' engine-neutral code (`render/shared/characters/residents.ts`) driven through a recording
// `ResidentView`, the way an engine draws them: a scripted lot (sleeping, a cuddle at the crib, sitting at a table,
// a ride, talking and hugging, carrying, a guitar, a shower stepped into, walkers, the stress ghosts, a rebuild)
// gives the same pose texture, drawn rows, props, quilts and ring per frame as recorded (`bun test`; after a
// change meant to change how residents move, `bun test --update-snapshots`). The record was made when the code
// moved out of the Babylon renderer (Stage 7), where a side-by-side run against the Babylon version before the
// move showed no difference in any frame (bench-results/stage7b/equal.ts).
//
// Also: the Babylon matrix maths the residents' code carries (`rig.ts`) against Babylon's own, number for number.
import { describe, expect, test } from 'bun:test';
import { readFileSync, existsSync } from 'node:fs';
import { Matrix, Quaternion, Vector3 } from '@babylonjs/core/Maths/math.vector';

const WEB = new URL('../', import.meta.url).pathname;

/** A fixed sequence of numbers in 0..1 (the same every run). */
function numbers(seed: number): () => number {
  let s = seed >>> 0;
  return () => (s = (Math.imul(s, 1664525) + 1013904223) >>> 0) / 2 ** 32;
}

function hash(a: ArrayLike<number> | null | undefined, from = 0, to = a?.length ?? 0): string {
  if (!a) return '-';
  const f = new Float32Array(1);
  const u = new Uint32Array(f.buffer);
  let h = 2166136261;
  for (let i = from; i < to; i++) {
    f[0] = a[i];
    h = Math.imul(h ^ u[0], 16777619);
  }
  return (h >>> 0).toString(16);
}

describe('Babylon matrix maths in rig.ts', async () => {
  const rig = await import('../src/render/shared/characters/rig');
  const bits = (a: ArrayLike<number>) => Array.from(Float32Array.from(a));
  test('a placement carried by a ride, as Babylon composes, multiplies and decomposes it', () => {
    const r = numbers(5);
    const s = (k = 1) => (r() * 2 - 1) * k;
    for (let n = 0; n < 200; n++) {
      const q = Quaternion.RotationYawPitchRoll(s(3), s(1), s(0.5));
      const t = new Vector3(s(9), s(2), s(9));
      const ride = Matrix.Compose(new Vector3(1, 1, 1), Quaternion.RotationYawPitchRoll(s(0.3), s(0.3), s(0.3)), new Vector3(s(), s(), s()));
      const ours = new Float32Array(16);
      rig.compose(1, 1, 1, q.x, q.y, q.z, q.w, t.x, t.y, t.z, ours);
      const theirs = Matrix.Compose(new Vector3(1, 1, 1), q, t);
      expect(bits(ours)).toEqual(bits(theirs.m));
      rig.multiply(ours, ride.m, ours);
      theirs.multiplyToRef(ride, theirs);
      expect(bits(ours)).toEqual(bits(theirs.m));
      const out = { qx: 0, qy: 0, qz: 0, qw: 1 };
      rig.rotationOf(ours, out);
      const dq = new Quaternion();
      theirs.decompose(new Vector3(), dq, new Vector3());
      expect([out.qx, out.qy, out.qz, out.qw]).toEqual([dq.x, dq.y, dq.z, dq.w]);
    }
  });
  test("a bone's height at rest, as Babylon inverts the inverse bind matrix", () => {
    const r = numbers(9);
    const m = new Float32Array(32);
    for (let n = 0; n < 200; n++) {
      const a = Matrix.Compose(new Vector3(1, 1, 1), Quaternion.RotationYawPitchRoll(r() * 6, r() - 0.5, r() - 0.5), new Vector3(r() - 0.5, r() * 1.8, r() - 0.5)).invert();
      m.set(a.m, 16);
      expect(rig.bindY(m, 1)).toBe(Matrix.FromArray(m, 16).invert().m[13]);
    }
  });
});

describe('residents', async () => {
  // --- the page's world in bun: content, assets and the character set from web/public ---
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
  const realFetch = globalThis.fetch;
  globalThis.fetch = (async (input: string | URL | Request) => {
    const url = new URL(typeof input === 'string' ? input : input instanceof URL ? input.href : input.url, 'http://localhost/');
    const path = `${WEB}public${decodeURIComponent(url.pathname)}`;
    return existsSync(path) ? new Response(readFileSync(path)) : new Response('missing', { status: 404 });
  }) as typeof fetch;
  const { Content } = await import('../src/content/content');
  const { AssetRegistry } = await import('../src/assets/registry');
  const { Characters, MAX_CHARACTERS } = await import('../src/render/shared/characters/residents');
  const { TemplateCache } = await import('../src/render/shared/templateCache');
  const { Pose } = await import('../src/core/snapshot');
  const { slotPlace } = await import('../src/render/shared/objectMotion');
  type View = import('../src/render/shared/characters/view').ResidentView<Part, Quilt, Prop>;
  type ResidentProp = import('../src/render/shared/characters/view').ResidentProp;
  type SnapshotLayout = import('../src/core/snapshot').SnapshotLayout;
  type FrameState = import('../src/core/bridge').FrameState;
  type ObjectPlacement = import('../src/core/protocol').ObjectPlacement;
  type SimInfo = import('../src/core/protocol').SimInfo;
  const content = await Content.load('http://localhost/content/base.json');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json');

  // --- the recording view ---
  interface Part {
    name: string;
    enabled: boolean;
    rows: Float32Array;
    count: number;
  }
  interface Quilt {
    name: string;
    enabled: boolean;
    count: number;
    matrices: Float32Array;
    ext: Float32Array;
    tint: Float32Array;
  }
  interface Prop extends ResidentProp {
    key: string;
    count: number;
  }
  const parts: Part[] = [];
  const quilts: Quilt[] = [];
  const props: Prop[] = [];
  const log: string[] = [];
  let ring = new Float32Array(16);
  let poses = new Float32Array(0);
  let width = 0;
  const cam = { x: 0, y: 6, z: 12 };
  const view: View = {
    init(_set, data, w) {
      poses = data;
      width = w;
    },
    uploadPoses: () => void log.push(`poses ${hash(poses)}`),
    uploadPoseRows: (from, count) => void log.push(`rows ${from} ${count} ${hash(poses, from * width * 4, (from + count) * width * 4)}`),
    createPart(body, part, shadow) {
      const p = { name: `${shadow ? 'shadow' : 'part'}:${body.name}:${part}`, enabled: true, rows: new Float32Array(0), count: 0 };
      parts.push(p);
      return p;
    },
    partEnabled: (p, on) => void (p.enabled = on),
    setPartRows(p, rows, perFrame) {
      p.rows = rows;
      p.count = rows.length / 4;
      log.push(`set ${p.name} ${hash(rows)} ${perFrame}`);
    },
    partRows(p, rows, count, change) {
      p.rows = rows;
      p.count = count;
      if (change === 'new') log.push(`new ${p.name} ${hash(rows, 0, count * 4)}`);
    },
    quiltMaterial: (toeZ, footZ) => void log.push(`quilts ${toeZ} ${footZ}`),
    createQuilt(name, geometry, matrices, ext, tint) {
      const q = { name, enabled: false, count: 0, matrices, ext, tint };
      quilts.push(q);
      log.push(`quilt ${name} ${hash(geometry.positions)} ${hash(geometry.uvs)} ${hash(geometry.indices)}`);
      return q;
    },
    quiltRows: (q, count) => void (q.count = count),
    quiltEnabled: (q, on) => void (q.enabled = on),
    createRing: (m) => void (ring = m),
    ringMoved: () => {},
    props: new TemplateCache<Prop | null>({ retell: () => {}, gone: () => false, hide: () => {}, dispose: () => {} }),
    async buildProp(key) {
      const p = { key, count: 0, matrices: new Float32Array(MAX_CHARACTERS * 16), top: key.includes('#') ? ([0, 0.95, 0.12] as const) : ([0, 0, 0] as const) };
      props.push(p);
      return p;
    },
    showProp: (p, count) => void (p.count = count),
    onModelFile: () => () => {},
    fileColour: async () => undefined,
    camera(out) {
      [out[0], out[1], out[2], out[3], out[4]] = [cam.x, cam.y, cam.z, 720, 0.8];
      return true;
    },
    onDispose: () => {},
  };

  // --- the lot: a station each, in a row, as the poses page has them (debug/poses.ts) ---
  const P = Pose;
  interface Station {
    def?: string;
    tag?: string;
    pose?: number;
    social?: number;
    script?: { seconds: number; pose: number; tag?: string }[];
    female?: boolean;
    stage?: string;
    held?: string;
    walk?: boolean;
    baby?: boolean;
    pair?: boolean;
  }
  const stations: Station[] = [
    { def: 'bedDouble', tag: 'sleep', pose: P.Lie, pair: true },
    {
      def: 'bed',
      script: [
        { seconds: 3, pose: P.Stand },
        { seconds: 6, pose: P.Lie, tag: 'sleep' },
      ],
    },
    { def: 'crib', baby: true, female: true, script: [{ seconds: 3, pose: P.Stand, tag: 'idle' }, { seconds: 15, pose: P.Stand, tag: 'cuddle' }, { seconds: 3, pose: P.Stand, tag: 'idle' }] },
    { def: 'diningChair', script: [{ seconds: 2, pose: P.Stand }, { seconds: 8, pose: P.Sit, tag: 'eat' }] },
    { def: 'rockingHorse', tag: 'play', pose: P.Sit, stage: 'child' },
    { def: 'sofaThree', tag: 'watch', pose: P.Sit, female: true },
    { def: 'guitarAcoustic', tag: 'music', female: true },
    { def: 'shower', tag: 'shower' },
    { def: 'counter', tag: 'chop', held: 'ingredients' },
    { def: 'chaise', tag: 'relax', pose: P.Sit, female: true, stage: 'elder' },
    { social: 1 },
    { social: 6 },
    { held: 'mealPlate', walk: true, female: true },
    { walk: true, stage: 'teen' },
  ];
  const objects: ObjectPlacement[] = [];
  interface Actor {
    station: number;
    x: number;
    z: number;
    yaw: number;
    object: number;
    partner: number;
    role: number;
    frontX?: number;
    frontZ?: number;
    baby?: boolean;
    female?: boolean;
  }
  const actors: Actor[] = [];
  stations.forEach((s, n) => {
    const cx = (n % 5) * 3.4 - 6.8;
    const cz = Math.floor(n / 5) * 3.4 - 3.4;
    if (s.social) {
      actors.push({ station: n, x: cx, z: cz - 0.5, yaw: 0, object: -1, partner: actors.length + 1, role: 1 });
      actors.push({ station: n, x: cx, z: cz + 0.5, yaw: Math.PI, object: -1, partner: actors.length - 1, role: 2, female: true });
    } else if (s.def) {
      const def = content.object(s.def);
      const [w, d] = def?.footprint ?? [1, 1];
      const ox = Math.round(cx - w / 2);
      const oz = Math.round(cz - d / 2);
      const id = objects.length;
      objects.push({ id, def: s.def, x: ox, z: oz, rot: 0, w, d, quality: 0, sellValue: null } as ObjectPlacement);
      const at = (k: number) => slotPlace(objects[id], def, k);
      const front = { frontX: ox + Math.floor((w - 1) / 2) + 0.5, frontZ: oz + d + 0.5 };
      actors.push({ station: n, ...at(0), object: id, partner: -1, role: 0, ...front, female: s.female });
      if (s.baby) actors.push({ station: n, ...at(0), object: id, partner: -1, role: 0, baby: true });
      if (s.pair) actors.push({ station: n, ...at(1), object: id, partner: -1, role: 0, female: true });
    } else actors.push({ station: n, x: cx, z: cz, yaw: 0, object: -1, partner: -1, role: 0, female: s.female });
  });
  const sims = actors.map(
    (a, i): SimInfo =>
      ({
        id: i,
        name: `sim${i}`,
        household: 0,
        gender: a.female ? 'female' : 'male',
        attractedTo: [],
        appearance: {
          body: a.female ? '#C9725F' : '#5F86C9',
          skin: ['#C08A64', '#8D5A3B', '#E8C0A0', '#6B4430'][i % 4],
          hair: ['#2B2320', '#7A4A2A', '#C9A15A', '#1A1A1A'][i % 4],
          hairStyle: a.female ? 'long' : 'short',
          height: 1,
          build: 1,
          top: 'top.tee',
          bottom: a.female ? 'bottom.skirt' : 'bottom.trousers',
          shoes: 'shoes.sneakers',
          ...(i % 5 === 3 ? { beard: true } : {}),
          ...(a.baby ? { stage: 'baby' } : stations[a.station].stage ? { stage: stations[a.station].stage } : {}),
        },
        traits: [],
        perks: [],
      }) as unknown as SimInfo,
  );
  const actions = [...content.animations];
  const items = content.items.map((it) => it.id);
  const FIELDS = ['id', 'x', 'z', 'yaw', 'pose', 'moving', 'social', 'outcome', 'partner', 'anim', 'emotion', 'role', 'away', 'object', 'action', 'mood', 'held'] as const;
  const layout: SnapshotLayout = {
    capacity: 6 + FIELDS.length * MAX_CHARACTERS,
    headerLen: 6,
    simStride: FIELDS.length,
    maxSims: MAX_CHARACTERS,
    ticksPerSecond: 20,
    header: { tick: 0, day: 1, minute: 2, speed: 3, simCount: 4, structureVersion: 5 },
    sim: Object.fromEntries(FIELDS.map((f, i) => [f, i])) as unknown as SnapshotLayout['sim'],
    actions,
    items,
  };
  function writeSnapshot(snap: Float32Array, t: number): void {
    const k = layout.sim;
    snap[layout.header.speed] = t < 14 ? 1 : t < 18 ? 3 : 1;
    snap[layout.header.simCount] = actors.length;
    actors.forEach((a, i) => {
      const s = stations[a.station];
      const o = layout.headerLen + i * layout.simStride;
      let pose = s.pose ?? P.Stand;
      let tag = s.tag ?? '';
      if (s.script) {
        let r = t % s.script.reduce((x, y) => x + y.seconds, 0);
        const step = s.script.find((x) => (r -= x.seconds) < 0)!;
        [pose, tag] = [step.pose, step.tag ?? ''];
      }
      if (a.baby) [pose, tag] = [P.Lie, ''];
      const standing = pose === P.Stand && a.object >= 0 && tag !== 'shower';
      snap[o + k.id] = i;
      snap[o + k.x] = standing ? a.frontX! : a.x;
      snap[o + k.z] = standing ? a.frontZ! : a.z;
      snap[o + k.yaw] = standing ? Math.PI : a.yaw;
      snap[o + k.pose] = pose;
      snap[o + k.moving] = 0;
      if (s.walk) {
        const u = (t * 0.45) % 2;
        snap[o + k.z] = a.z - 0.9 + 1.8 * (u > 1 ? 2 - u : u);
        snap[o + k.yaw] = u > 1 ? Math.PI : 0;
        snap[o + k.moving] = 1;
      }
      snap[o + k.held!] = s.held ? items.indexOf(s.held) + 1 : 0;
      snap[o + k.social] = s.social ? 1 : 0;
      snap[o + k.anim] = s.social ?? 0;
      snap[o + k.outcome] = s.social && t > 8 ? 1 : 0;
      snap[o + k.partner] = a.partner;
      snap[o + k.role] = a.role;
      snap[o + k.emotion] = (Math.floor(t / 3) + i) % (content.emotions.length + 1);
      snap[o + k.away] = 0;
      snap[o + k.object] = s.script && pose === P.Stand && !tag ? -1 : a.object;
      snap[o + k.action] = s.social ? actions.indexOf('talk') : tag ? actions.indexOf(tag) : -1;
      snap[o + k.mood!] = 0.3 + (i % 5) * 0.15;
    });
  }

  test('pose data, rows, props and quilts per frame, as recorded', async () => {
    const realNow = performance.now;
    let now = 0;
    performance.now = () => now;
    try {
      const characters = new Characters(view, assets, content, { castsShadows: true });
      expect(await characters.init()).toBe(true);
      characters.setEmotions(content.emotions.map((e) => e.id));
      await characters.build({ sims, objects });
      characters.setTables(new Map(objects.map((o) => [o.id, { host: o.id, top: 0.75, boxes: Float32Array.of(-0.5, 0.5, 0.7, 0.76, 0.35, 1.1) }])));
      const ride = new Float32Array(16);
      let t = 0;
      characters.setRide((id) => (objects[id]?.def === 'rockingHorse' ? Matrix.Compose(new Vector3(1, 1, 1), Quaternion.RotationYawPitchRoll(0, 0.2 * Math.sin(t * 3), 0), new Vector3(0, 0.03, 0)).copyToArray(ride) && ride : null));
      characters.setSun(() => ({ x: -0.45, z: -0.35 }));
      characters.setLoop(() => (t * 0.4) % 1);
      characters.select(1);
      const prev = new Float32Array(layout.capacity);
      const curr = new Float32Array(layout.capacity);
      const frame: FrameState = { prev, curr, alpha: 0, steps: 1, layout, now: 0 };
      const shadows = new Float32Array(MAX_CHARACTERS * 16);
      const track = { times: Float32Array.from([0, 0.12, 0.3, 0.45, 0.7, 0.9]), visemes: Uint8Array.from([10, 11, 13, 1, 4]), energy: Float32Array.from([0.4, 0.8, 1, 0.6, 0.3]) };
      let tick = 0;
      writeSnapshot(prev, 0);
      writeSnapshot(curr, 0.05);
      const record: string[] = [];
      let second: string[] = [];
      for (let f = 0; f < 30 * 24; f++) {
        t = f / 30;
        while ((tick + 1) * 0.05 <= t) {
          tick++;
          prev.set(curr);
          writeSnapshot(curr, (tick + 1) * 0.05);
        }
        frame.alpha = (t - tick * 0.05) / 0.05;
        frame.now = now = 1000 + t * 1000;
        // In and out (the faces' detail and the low-detail bodies) and round.
        [cam.x, cam.y, cam.z] = [Math.sin(t * 0.3) * (2 + t), 1.6 + t * 0.4, Math.cos(t * 0.3) * (2 + t)];
        if (f === 60) characters.setSpeech(actors.findIndex((a) => a.role === 1), track as never, now);
        if (f === 300) characters.stress(5, 0, 6);
        if (f === 450) await characters.build({ sims, objects }, 'residents');
        characters.update(frame, () => true, shadows);
        const drawn = parts.filter((p) => p.enabled && p.count).map((p) => `${p.name} ${hash(p.rows, 0, p.count * 4)}`);
        const covered = quilts.filter((q) => q.count).map((q) => `${q.name} ${q.count} ${hash(q.matrices, 0, q.count * 16)} ${hash(q.ext, 0, q.count * 2)} ${hash(q.tint, 0, q.count * 3)}`);
        const held = props.filter((p) => p.count).map((p) => `${p.key} ${hash(p.matrices)}`);
        second.push(hash(poses), hash(characters.heads), hash(shadows), hash(characters.visible), hash(ring), ...drawn, ...covered, ...held, ...log.splice(0), JSON.stringify([...characters.pulled]));
        if (f % 30 === 29) {
          record.push(`${(f + 1) / 30} s: ${hash(new TextEncoder().encode(second.join('\n')))}`);
          second = [];
        }
      }
      expect(record).toMatchSnapshot();
    } finally {
      performance.now = realNow;
      globalThis.fetch = realFetch;
    }
  });
});
