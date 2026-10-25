// Crossfade continuity: the residents' blending (`residents.ts` `start`, `choose`, the per-frame blend; `pose.ts`
// `blendInto`) driven through a recording `ResidentView`, with five Sims scripted to churn — a chair sat on and
// stood up from, a walk cut short by a dance, a two-handed load carried and then eaten, a two-handed load swapped
// for a one-handed one, in and out of bed. Every frame, every bone's rotation is read out of the pose texture (the
// skinning matrix's rotation, what the viewer sees) and compared with the frame before: a crossfade must move each
// bone smoothly, so a single frame turning a bone further than a real fade does is a pop, not motion. `bun test`.
//
// (What this caught when written: `start` kept the old fading-out player only under `rig.w < 0.5`, so interrupting
// a fade jumped up to half the fade's whole delta — 83° of a hand here, 57° where lying meets standing; and the
// upper-body layer swapped clips at full weight — 79° of a hand for a two-handed load becoming one-handed. With
// `freezePlayer` and the upper layer's own fade, no bone turns more than 19° in a frame anywhere in the script,
// which is the fade itself: the widest handoff the clips have is 175° over a quarter second.)
import { afterAll, describe, expect, test } from 'bun:test';
import { readFileSync, existsSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('residents crossfade continuity', async () => {
  Object.assign(globalThis, {
    location: {
      href: 'http://localhost/',
      search: '',
      origin: 'http://localhost',
      pathname: '/',
    },
  });
  const realFetch = globalThis.fetch;
  globalThis.fetch = (async (input: string | URL | Request) => {
    const url = new URL(
      typeof input === 'string' ? input : input instanceof URL ? input.href : input.url,
      'http://localhost/',
    );
    const path = `${WEB}public${decodeURIComponent(url.pathname)}`;
    return existsSync(path) ? new Response(readFileSync(path)) : new Response('missing', { status: 404 });
  }) as typeof fetch;
  // (The shim is process-wide: put the real one back for whatever runs next.)
  afterAll(() => {
    globalThis.fetch = realFetch;
  });
  const { Content } = await import('../src/content/content');
  const { AssetRegistry } = await import('../src/assets/registry');
  const { Characters, MAX_CHARACTERS } = await import('../src/render/shared/characters/residents');
  const { TemplateCache } = await import('../src/render/shared/templateCache');
  const { Pose } = await import('../src/core/snapshot');
  const { slotPlace } = await import('../src/render/shared/objectMotion');
  const rig = await import('../src/render/shared/characters/rig');
  type View = import('../src/render/shared/characters/view').ResidentView<
    object,
    object,
    import('../src/render/shared/characters/view').ResidentProp
  >;
  type SnapshotLayout = import('../src/core/snapshot').SnapshotLayout;
  type FrameState = import('../src/core/bridge').FrameState;
  type ObjectPlacement = import('../src/core/protocol').ObjectPlacement;
  type SimInfo = import('../src/core/protocol').SimInfo;

  const content = await Content.load('http://localhost/content/base.json');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json');

  /** The pose texture and nothing else: this test reads the bones, it draws nothing. */
  let poses = new Float32Array(0);
  let width = 0;
  const view: View = {
    init(_set, data, w) {
      poses = data;
      width = w;
    },
    uploadPoses: () => {},
    uploadPoseRows: () => {},
    createPart: () => ({}),
    partEnabled: () => {},
    setPartRows: () => {},
    partRows: () => {},
    quiltMaterial: () => {},
    createQuilt: () => ({}),
    quiltRows: () => {},
    quiltEnabled: () => {},
    createRing: () => {},
    ringMoved: () => {},
    props: new TemplateCache<import('../src/render/shared/characters/view').ResidentProp | null>({
      retell: () => {},
      gone: () => false,
      hide: () => {},
      dispose: () => {},
    }),
    async buildProp(key) {
      return {
        key,
        matrices: new Float32Array(MAX_CHARACTERS * 16),
        top: [0, 0, 0] as const,
      };
    },
    showProp: () => {},
    onModelFile: () => () => {},
    fileColour: async () => undefined,
    camera: () => false,
    onDispose: () => {},
  };

  // --- the lot: a chair, a counter, a walker's tile, and a Sim carrying a load ------------------
  const P = Pose;
  const objects: ObjectPlacement[] = [];
  for (const def of ['diningChair', 'counter', 'bed']) {
    const d = content.object(def);
    const [w, dep] = d?.footprint ?? [1, 1];
    objects.push({
      id: objects.length,
      def,
      x: objects.length * 4,
      z: 0,
      rot: 0,
      w,
      d: dep,
      quality: 0,
      sellValue: null,
    } as ObjectPlacement);
  }
  const [chair, counter, bed] = objects;
  const seat = slotPlace(chair, content.object('diningChair'), 0);
  const atCounter = slotPlace(counter, content.object('counter'), 0);
  const onBed = slotPlace(bed, content.object('bed'), 0);
  /** Where a Sim stands in front of a thing (the tile in front of its middle). */
  const front = (o: ObjectPlacement) => ({
    x: o.x + Math.floor((o.w - 1) / 2) + 0.5,
    z: o.z + o.d + 0.5,
  });

  /** What a Sim does for `seconds`: the snapshot's pose, action tag, moving flag and held thing. */
  interface Step {
    seconds: number;
    pose?: number;
    tag?: string;
    moving?: number;
    held?: string;
  }
  interface Actor {
    /** The activities in a cycle, each `seconds` long (a fade is 0.25–0.3 s, so these interrupt each other). */
    script: Step[];
    /** 'cycle': round the script; 'carry': standing with a load, then sat at the counter eating it (for good);
     *  'swap': a two-handed load for a one-handed one, over and over. */
    kind: 'cycle' | 'carry' | 'swap';
    object: number;
    x: number;
    z: number;
  }
  const actors: Actor[] = [
    // A chair, sat on and stood up from, and walked to and away from: each fade cut short by a third clip.
    {
      kind: 'cycle',
      script: [
        { seconds: 0.4, pose: P.Sit, tag: 'eat' },
        { seconds: 0.4 },
        { seconds: 0.4, moving: 1 },
        { seconds: 0.4 },
      ],
      object: chair.id,
      ...seat,
    },
    // A walk, a stop, a dance, a stoop and back: a fade cut short at half its weight by a third clip, every time
    // (sim-core changing its mind 0.12 s later).
    {
      kind: 'cycle',
      script: [
        { seconds: 0.12 },
        { seconds: 0.12, moving: 1 },
        { seconds: 0.12, tag: 'dance' },
        { seconds: 0.12, tag: 'phoneScreen' },
      ],
      object: -1,
      x: 8.5,
      z: 0,
    },
    // Carrying a two-handed load, then sat at a counter eating it (`carry` -> `eat` at full weight).
    {
      kind: 'carry',
      script: [{ seconds: 2.4 }],
      object: counter.id,
      ...atCounter,
    },
    // A two-handed load for a one-handed one and back (`carry` <-> `carryOne` at full weight).
    { kind: 'swap', script: [{ seconds: 0.9 }], object: -1, x: 12.5, z: 0 },
    // Tucking in and out of bed every 0.45 s (lying takes a second to fade): the widest pair there is, cut short
    // at half its weight.
    {
      kind: 'cycle',
      script: [{ seconds: 0.45, pose: P.Lie, tag: 'sleep' }, { seconds: 0.45 }],
      object: bed.id,
      ...onBed,
    },
  ];
  const sims = actors.map((a, i): SimInfo => ({
    id: i,
    name: `sim${i}`,
    household: 0,
    gender: 'male',
    attractedTo: [],
    appearance: {
      body: '#5F86C9',
      skin: '#C08A64',
      hair: '#2B2320',
      hairStyle: 'short',
      height: 1,
      build: 1,
      top: 'top.tee',
      bottom: 'bottom.trousers',
      shoes: 'shoes.sneakers',
    },
    traits: [],
    perks: [],
  }));
  const items = content.items.map((it) => it.id);
  const actions = [...content.animations];
  /** An id's place in one of the snapshot's lists (`indexOf`, but a typo fails the test rather than reading as "none"). */
  const indexOf = (list: readonly string[], id: string): number => {
    const i = list.indexOf(id);
    if (i < 0) throw new Error(`blend.test: '${id}' is not in the snapshot's list`);
    return i;
  };
  const FIELDS = [
    'id',
    'x',
    'z',
    'yaw',
    'pose',
    'moving',
    'social',
    'outcome',
    'partner',
    'anim',
    'emotion',
    'role',
    'away',
    'object',
    'action',
    'mood',
    'held',
  ] as const;
  const layout: SnapshotLayout = {
    capacity: 6 + FIELDS.length * MAX_CHARACTERS,
    headerLen: 6,
    simStride: FIELDS.length,
    maxSims: MAX_CHARACTERS,
    ticksPerSecond: 20,
    header: {
      tick: 0,
      day: 1,
      minute: 2,
      speed: 3,
      simCount: 4,
      structureVersion: 5,
    },
    sim: Object.fromEntries(FIELDS.map((f, i) => [f, i])) as SnapshotLayout['sim'],
    actions,
    items,
  };
  /** The step of an actor's cycle at `t` s. */
  function stepAt(a: Actor, t: number): Step {
    const total = a.script.reduce((sum, x) => sum + x.seconds, 0);
    let r = t % total;
    for (const step of a.script) {
      r -= step.seconds;
      if (r < 0) return step;
    }
    return a.script[a.script.length - 1];
  }

  function writeSnapshot(snap: Float32Array, t: number): void {
    const k = layout.sim;
    snap[layout.header.speed] = 1;
    snap[layout.header.simCount] = actors.length;
    actors.forEach((a, i) => {
      const o = layout.headerLen + i * layout.simStride;
      const f = front(objects[a.object] ?? chair);
      let [x, z] = [a.x, a.z];
      let [pose, moving, held, tag] = [P.Stand, 0, '', ''];
      if (a.kind === 'cycle') {
        const s = stepAt(a, t);
        [pose, moving, held, tag] = [s.pose ?? P.Stand, s.moving ?? 0, s.held ?? '', s.tag ?? ''];
        // (On the seat's tile, or about it: the position drifts smoothly either way, so no step in the snapshot is
        // ever a jump — only the activity's.)
        const base = a.object >= 0 ? f : a;
        [x, z] = [base.x, base.z + 0.5 * Math.sin(t * 1.3)];
      } else if (a.kind === 'carry') {
        // Standing with a load, then sat at the counter eating it.
        const sat = t > a.script[0].seconds;
        [pose, tag, held] = sat ? [P.Sit, 'eat', 'ingredients'] : [P.Stand, '', 'ingredients'];
        [x, z] = [f.x, f.z];
      } else {
        // A two-handed load for a one-handed one, every `seconds`.
        held = Math.floor(t / a.script[0].seconds) % 2 === 0 ? 'ingredients' : 'mealPlate';
      }
      snap[o + k.id] = i;
      snap[o + k.x] = x;
      snap[o + k.z] = z;
      snap[o + k.yaw] = 0;
      snap[o + k.pose] = pose;
      snap[o + k.moving] = moving;
      snap[o + k.social] = 0;
      snap[o + k.anim] = 0;
      snap[o + k.outcome] = 0;
      snap[o + k.partner] = -1;
      snap[o + k.role] = 0;
      snap[o + k.emotion] = 1;
      snap[o + k.away] = 0;
      snap[o + k.object] = a.object;
      snap[o + k.action] = tag ? indexOf(actions, tag) : -1;
      snap[o + k.mood] = 0.5;
      snap[o + k.held] = held ? indexOf(items, held) + 1 : 0;
    });
  }

  test('every bone turns smoothly through the fades the activities script', async () => {
    const realNow = performance.now;
    let now = 0;
    performance.now = () => now;
    try {
      const characters = new Characters(view, assets, content, {
        castsShadows: true,
      });
      expect(await characters.init()).toBe(true);
      // (The view's `init` is what hands over the pose texture: without it every Sim would be read out of row 0,
      // and the comparison would measure one character five times.)
      expect(width).toBeGreaterThan(0);
      expect(poses.length).toBe(width * MAX_CHARACTERS * 4);
      characters.setEmotions(content.emotions.map((e) => e.id));
      await characters.build({ sims, objects });
      characters.setTables(
        new Map(
          objects.map((o) => [
            o.id,
            {
              host: o.id,
              top: 0.75,
              boxes: Float32Array.of(-0.5, 0.5, 0.7, 0.76, 0.35, 1.1),
            },
          ]),
        ),
      );
      characters.setSun(() => ({ x: -0.45, z: -0.35 }));
      const bones = (JSON.parse(readFileSync(`${WEB}public/assets/characters/rig.json`, 'utf8')) as { bones: string[] })
        .bones;
      const prev = new Float32Array(layout.capacity);
      const curr = new Float32Array(layout.capacity);
      const frame: FrameState = {
        prev,
        curr,
        alpha: 0,
        steps: 1,
        layout,
        now: 0,
      };
      const shadows = new Float32Array(MAX_CHARACTERS * 16);
      const FPS = 60;
      const SECONDS = 6;
      let tick = 0;
      writeSnapshot(prev, 0);
      writeSnapshot(curr, 0);
      // Every Sim's bone rotations the frame before, and the current frame's.
      const before = actors.map(() => new Float32Array(bones.length * 4));
      const after = new Float32Array(bones.length * 4);
      const q = { qx: 0, qy: 0, qz: 0, qw: 1 };
      // The worst turns seen, for the failure's message.
      const worst: { deg: number; sim: number; bone: string; at: number }[] = [];
      const note = (deg: number, sim: number, bone: string, at: number) => {
        if (deg < 16) return;
        worst.push({ deg, sim, bone, at });
        // Keep only the worst few: the message needs the top of the list, not the whole run.
        if (worst.length > 64) {
          worst.sort((x, y) => y.deg - x.deg);
          worst.length = 12;
        }
      };
      for (let f = 0; f < FPS * SECONDS; f++) {
        const t = f / FPS;
        while ((tick + 1) * 0.05 <= t) {
          tick++;
          prev.set(curr);
          writeSnapshot(curr, (tick + 1) * 0.05);
        }
        frame.alpha = (t - tick * 0.05) / 0.05;
        now = 1000 + t * 1000;
        frame.now = now;
        characters.update(frame, () => true, shadows);
        for (let i = 0; i < actors.length; i++) {
          // The skinning matrix of each bone, its rotation read out: what the viewer draws.
          const row = i * width * 4;
          for (let b = 0; b < bones.length; b++) {
            rig.rotationOf(poses.subarray(row + b * 16, row + b * 16 + 16), q);
            [after[b * 4], after[b * 4 + 1], after[b * 4 + 2], after[b * 4 + 3]] = [q.qx, q.qy, q.qz, q.qw];
          }
          // (The first half second is the build settling, not a fade.)
          if (t >= 0.5) {
            const prevQ = before[i];
            for (let b = 0; b < bones.length; b++) {
              note(quatAngle(prevQ, b * 4, after, b * 4), i, bones[b], t);
            }
          }
          before[i].set(after);
        }
      }
      // The widest handoff there is (175°, `blend_analysis`) faded over 0.25 s moves 175° * 1.5 / 4 / 60 = 17.5° a
      // frame at 60 fps (smoothstep's peak rate is 1.5x). 26° leaves room for that and still catches a dropped pose
      // (57° for the bed pair, 79° for a swapped upper clip, 136° for both at once).
      worst.sort((x, y) => y.deg - x.deg);
      const top = worst.slice(0, 6).map((w) => `sim${w.sim} ${w.bone} ${w.deg.toFixed(0)}° at ${w.at.toFixed(2)} s`);
      expect(worst[0]?.deg ?? 0, `single-frame turns: ${top.join('; ') || 'none'}`).toBeLessThan(26);
    } finally {
      performance.now = realNow;
    }
  });
});

/** The angle (deg) between two rotations in arrays `a` and `b`, at float offsets `oa` and `ob`. */
function quatAngle(a: Float32Array, oa: number, b: Float32Array, ob: number): number {
  const d = Math.abs(a[oa] * b[ob] + a[oa + 1] * b[ob + 1] + a[oa + 2] * b[ob + 2] + a[oa + 3] * b[ob + 3]);
  return (2 * Math.acos(Math.min(1, d)) * 180) / Math.PI;
}
