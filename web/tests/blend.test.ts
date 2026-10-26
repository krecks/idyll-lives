// Crossfade continuity: the residents' blending (`residents.ts` `start`, `choose`, the per-frame blend; `pose.ts`
// `blendInto`) driven through a recording `ResidentView`, with six Sims scripted to churn — a chair sat on and
// stood up from, a walk cut short by a dance, a two-handed load carried and then eaten, a two-handed load swapped
// for a one-handed one (now and then, and every 0.1 s), in and out of bed. Every frame, every bone's rotation and
// the pelvis's position are read out of the pose texture (the skinning matrices, what the viewer sees) and compared
// with the frame before: a crossfade must move the body smoothly, so a single frame turning a bone further than a
// real fade does, or moving the body further than a Sim can, is a pop, not motion. `bun test`.
//
// (What this caught when written: `start` kept the old fading-out player only under `rig.w < 0.5`, so interrupting
// a fade jumped up to half the fade's whole delta — 83° of a hand here, 57° where lying meets standing; and the
// upper-body layer swapped clips at full weight — 79° of a hand for a two-handed load becoming one-handed. With
// `freezePlayer` and the upper layer's own fade, no bone turns more than 19° in a frame anywhere in the script,
// which is the fade itself: the widest handoff the clips have is 175° over a quarter second. Later: an upper-layer
// fade cut short by the next swap dropped the clip it was fading from, 61° of a hand; and a rise off a seat cut
// short by a walk put the body on the tile in one frame, 43 cm; and, hidden before it had caught up, came back into
// view on its way off the seat it had left, 62 cm from its place.)
import { afterAll, describe, expect, test } from 'bun:test';
import { readFileSync, existsSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('residents crossfade continuity', async () => {
  const realLocation = globalThis.location;
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
  // (Both are process-wide: put the real ones back for whatever runs next.)
  afterAll(() => {
    globalThis.fetch = realFetch;
    Object.assign(globalThis, { location: realLocation });
  });
  const { Content } = await import('../src/content/content');
  const { AssetRegistry } = await import('../src/assets/registry');
  const { Characters, MAX_CHARACTERS } = await import('../src/render/shared/characters/residents');
  const { TemplateCache } = await import('../src/render/shared/templateCache');
  const { Pose, simFields } = await import('../src/core/snapshot');
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
  let poses: Float32Array = new Float32Array(0);
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
    // The same, every 0.1 s: each upper-layer fade cut short by the next swap.
    { kind: 'swap', script: [{ seconds: 0.1 }], object: -1, x: 14.5, z: 0 },
    // Tucking in and out of bed every 0.45 s (lying takes a second to fade): the widest pair there is, cut short
    // at half its weight.
    {
      kind: 'cycle',
      script: [{ seconds: 0.45, pose: P.Lie, tag: 'sleep' }, { seconds: 0.45 }],
      object: bed.id,
      ...onBed,
    },
  ];
  // One Sim per actor, alike in everything but their name: the actors script the snapshot, not them.
  const sims = Array.from({ length: actors.length }, (_, i): SimInfo => ({
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
    sim: simFields(FIELDS),
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
    // (The layout's own type leaves its fields optional; this test's layout has every one it writes.)
    const k = layout.sim as Record<(typeof FIELDS)[number], number>;
    snap[layout.header.speed] = 1;
    snap[layout.header.simCount] = actors.length;
    actors.forEach((a, i) => {
      const o = layout.headerLen + i * layout.simStride;
      let [x, z] = [a.x, a.z];
      let [pose, moving, held, tag]: [number, number, string, string] = [P.Stand, 0, '', ''];
      if (a.kind === 'cycle') {
        const s = stepAt(a, t);
        [pose, moving, held, tag] = [s.pose ?? P.Stand, s.moving ?? 0, s.held ?? '', s.tag ?? ''];
        // (On the seat's tile, or about it: the position drifts smoothly either way, so no step in the snapshot is
        // ever a jump — only the activity's. A walker's `object` is -1: it has no tile in front of anything.)
        const base = a.object >= 0 ? front(objects[a.object]) : a;
        [x, z] = [base.x, base.z + 0.5 * Math.sin(t * 1.3)];
      } else if (a.kind === 'carry') {
        // Standing with a load, then sat at the counter eating it.
        const sat = t > a.script[0].seconds;
        [pose, tag, held] = sat ? [P.Sit, 'eat', 'ingredients'] : [P.Stand, '', 'ingredients'];
        const spot = front(objects[a.object]);
        [x, z] = [spot.x, spot.z];
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
      const { bones, sourcePelvis } = JSON.parse(readFileSync(`${WEB}public/assets/characters/rig.json`, 'utf8')) as {
        bones: string[];
        sourcePelvis: [number, number, number];
      };
      const pelvis = bones.indexOf('pelvis');
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
      // Every Sim's pelvis the frame before (world, m): where the root's placement puts the body. And the farthest
      // one moved in a frame, for the failure's message.
      const hips = actors.map(() => [NaN, NaN, NaN]);
      let farthest = { cm: 0, sim: -1, at: 0 };
      // When the chair's Sim is out of view (s), whether it has been since it was last drawn, and how far (cm) its
      // pelvis was from where the snapshot put it on the frame it came back.
      const HIDDEN = [2.37, 3] as const;
      let wasHidden = false;
      let reappeared = NaN;
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
        // The chair's Sim out of view just after a walk cut its rise short (`hidden`), and back in view later.
        const hiding = t >= HIDDEN[0] && t < HIDDEN[1];
        characters.update(frame, (x) => !(hiding && x < 2), shadows);
        for (let i = 0; i < actors.length; i++) {
          if (hiding && i === 0) {
            wasHidden = true;
            continue;
          }
          // The skinning matrix of each bone, its rotation read out: what the viewer draws.
          const row = i * width * 4;
          for (let b = 0; b < bones.length; b++) {
            rig.rotationOf(poses.subarray(row + b * 16, row + b * 16 + 16), q);
            [after[b * 4], after[b * 4 + 1], after[b * 4 + 2], after[b * 4 + 3]] = [q.qx, q.qy, q.qz, q.qw];
          }
          // The pelvis: its rest position (the clips' `sourcePelvis`, near enough on any body) through its skinning
          // matrix, so a root offset that jumps (`rootY` / `rootFwd`, a `frame`) moves it, as it moves the body.
          const m = poses.subarray(row + pelvis * 16, row + pelvis * 16 + 16);
          const [sx, sy, sz] = sourcePelvis;
          const hip = [0, 1, 2].map((k) => m[k] * sx + m[4 + k] * sy + m[8 + k] * sz + m[12 + k]);
          // (The first half second is the build settling, not a fade.)
          if (i === 0 && wasHidden) {
            // Back in view: drawn where the snapshot puts them, not on the way off the seat they left before.
            wasHidden = false;
            const o = layout.headerLen;
            const k = layout.sim as Record<(typeof FIELDS)[number], number>;
            const at = (f: number) => prev[o + f] + (curr[o + f] - prev[o + f]) * frame.alpha;
            reappeared = Math.hypot(hip[0] - at(k.x), hip[2] - at(k.z)) * 100;
          } else if (t >= 0.5) {
            const prevQ = before[i];
            for (let b = 0; b < bones.length; b++) {
              note(quatAngle(prevQ, b * 4, after, b * 4), i, bones[b], t);
            }
            const cm = Math.hypot(hip[0] - hips[i][0], hip[1] - hips[i][1], hip[2] - hips[i][2]) * 100;
            if (cm > farthest.cm) farthest = { cm, sim: i, at: t };
          }
          before[i].set(after);
          hips[i] = hip;
        }
      }
      // The widest handoff there is (175°, `blend_analysis`) faded over 0.25 s moves 175° * 1.5 / 4 / 60 = 17.5° a
      // frame at 60 fps (smoothstep's peak rate is 1.5x). 26° leaves room for that and still catches a dropped pose
      // (57° for the bed pair, 79° for a swapped upper clip, 136° for both at once).
      worst.sort((x, y) => y.deg - x.deg);
      const top = worst.slice(0, 6).map((w) => `sim${w.sim} ${w.bone} ${w.deg.toFixed(0)}° at ${w.at.toFixed(2)} s`);
      expect(worst[0]?.deg ?? 0, `single-frame turns: ${top.join('; ') || 'none'}`).toBeLessThan(26);
      // And the body as a whole: 20 cm a frame is 12 m/s, faster than any Sim moves (a run is a third of that, the
      // widest settle here 12 cm), and well under a placement that jumps (43 cm for a rise cut short by a walk).
      const hipsAt = `sim${farthest.sim} ${farthest.cm.toFixed(0)} cm at ${farthest.at.toFixed(2)} s`;
      expect(farthest.cm, `single-frame pelvis move: ${hipsAt}`).toBeLessThan(20);
      // (Standing, the pelvis is within a few cm of the Sim's place; the seat they rose from is 43 cm away.)
      expect(reappeared, 'pelvis from its place on coming back into view (cm)').toBeLessThan(15);
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
