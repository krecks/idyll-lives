/**
 * How far a foot on the floor travels while a resident starts or stops walking, measured on the runtime itself:
 * the residents' own code (`web/src/render/shared/characters/residents.ts`) posing residents frame by frame from a
 * scripted snapshot, as the game's renderer drives it, with no engine behind it (a view that draws nothing). So it
 * measures what is drawn: the crossfade, the gait's phase and rate, the last tick's glide, the hips' sway, the
 * procedural layers and whatever the runtime does about the feet (`stride.ts`).
 *
 * The script: on each body at each life stage but the baby's, `PHASES` residents stand for a second, walk straight
 * for two seconds and a little more (each a twelfth of a walk cycle longer than the one before, so their stops
 * fall on every part of the stride) and stand again. A foot touches the floor at its heel (`HEEL` behind the ankle,
 * on the sole) or at its ball (the toes' joint), whichever is lower, while that point is within `DOWN` of its
 * standing height; the foot's travel is the way that point goes over the floor between two frames on which it
 * is down (a foot rolling from its heel onto its ball, or lifting its heel, travels nothing). Measured from just
 * before a start to `SETTLE.start` s after it, and from just before a stop to `SETTLE.stop` s after it. A foot that steps (lifts, goes, comes down)
 * travels nothing; one that is dragged or pushed along the floor travels all of it.
 *
 * `measureTransitions()` gives every resident's numbers; `tools/characters/blend_analysis.ts` prints them (section
 * 6) and `tools/characters/check_clips.ts` holds them to `TRAVEL_LIMIT`.
 */

import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../../web/', import.meta.url).pathname;

/** Residents per body and stage: the stops' phases, evenly over a walk cycle. */
export const PHASES = 12;
/** A foot is down while its heel or its ball is within this of its standing height (m). */
export const DOWN = 0.004;
/** The heel's contact point: this far behind the ankle (m on the reference body, as clips/locomotion.ts `footAt`), on the sole. */
const HEEL = 0.05;
/** Seconds measured from a start (the fade and the first step) and from a stop (the fade's two steps, the body at rest). */
export const SETTLE = { start: 0.5, stop: 0.9 };
/** The walkers' speed (m/s): sim-core `world::WALK_SPEED` at game speed 1. */
export const PACE = 1.5;
/** Frames a second drawn, and how far into a tick each falls (the snapshots are the sim's 60 a second). */
const FPS = 60;
const INTO_TICK = 0.4;
/** The script (s): standing, the shortest walk, standing. */
const STAND = 1;
const WALK = 2;
const REST = 1.6;
/**
 * The most a foot on the floor may travel (m) over a start or a stop, on any body at any stage and at any part of
 * the stride (`check_clips.ts`): what is left once the feet step rather than slide (`stride.ts`: 31 to 56 mm over
 * a start, 13 to 41 mm over a stop, where it was 314 to 486 and 231 to 421), with some room.
 */
export const TRAVEL_LIMIT = { start: 0.07, stop: 0.06 };

export interface FootTravel {
  body: string;
  stage: string;
  /** Which of the `PHASES` residents: its stop falls `phase / PHASES` of a walk cycle later than the first's. */
  phase: number;
  /** The way each foot's ankle went over the floor while down (m): starting to walk, and stopping. */
  start: { l: number; r: number };
  stop: { l: number; r: number };
  /** The same over a second of plain walking between the two (what the gait itself slides at this pace). */
  walk: { l: number; r: number };
}

const STAGES = ['child', 'teen', 'youngAdult', 'adult', 'elder'];

/** A frame of one foot, for `measureTransitions`' `trace`: the time (s; the walk begins at `at.start` and ends at `at.stop`), the foot's contact point (0 heel, 1 ball), where it is (m, its height over its standing one) and whether it counts as down. */
export type Trace = (who: { body: string; stage: string; phase: number }, at: { t: number; start: number; stop: number }, side: 'l' | 'r', point: number, x: number, up: number, z: number, down: boolean) => void;

/**
 * Every resident's travel (`FootTravel`), on `bodies` (default: both) at `stages` (default: every one but the
 * baby's); `trace` is told every foot's contact point on every frame.
 */
export async function measureTransitions(bodies?: string[], stages: string[] = STAGES, trace?: Trace): Promise<FootTravel[]> {
  // The page's world: the files under web/public by `fetch`, as `web/tests/residents.test.ts` has them.
  const world = globalThis as unknown as { location?: unknown; fetch: typeof fetch };
  const [realFetch, realLocation] = [world.fetch, world.location];
  world.location = { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' };
  world.fetch = (async (input: string | URL | Request) => {
    const url = new URL(typeof input === 'string' ? input : input instanceof URL ? input.href : input.url, 'http://localhost/');
    const path = `${WEB}public${decodeURIComponent(url.pathname)}`;
    return existsSync(path) ? new Response(readFileSync(path)) : new Response('missing', { status: 404 });
  }) as typeof fetch;
  try {
    const { AssetRegistry } = await import('../../web/src/assets/registry.ts');
    const { Characters, MAX_CHARACTERS } = await import('../../web/src/render/shared/characters/residents.ts');
    const { TemplateCache } = await import('../../web/src/render/shared/templateCache.ts');
    const { Pose, simFields, tickRates } = await import('../../web/src/core/snapshot.ts');
    const { BONE_FLOATS } = await import('../../web/src/render/shared/characters/layout.ts');
    const rig = JSON.parse(readFileSync(`${WEB}public/assets/characters/rig.json`, 'utf8'));
    const assets = await AssetRegistry.load('http://localhost/assets/manifest.json');

    // A view that draws nothing: the pose texture's data is all that is read.
    let poses = new Float32Array(0);
    let width = 0;
    const view = {
      init(_set: unknown, data: Float32Array, w: number) {
        poses = data;
        width = w;
      },
      setMorphs() {},
      uploadPoses() {},
      uploadPoseRows() {},
      createPart: () => ({}),
      partEnabled() {},
      setPartRows() {},
      partRows() {},
      quiltMaterial() {},
      createQuilt: () => ({}),
      quiltRows() {},
      quiltEnabled() {},
      createRing() {},
      ringMoved() {},
      props: new TemplateCache<null>({ retell: () => {}, gone: () => false, hide: () => {}, dispose: () => {} }),
      buildProp: async () => null,
      showProp() {},
      onModelFile: () => () => {},
      fileColour: async () => undefined,
      camera(out: Float32Array) {
        [out[0], out[1], out[2], out[3], out[4]] = [0, 1.6, 6, 1080, 0.8];
        return true;
      },
      onDispose() {},
    };

    const who: { body: string; stage: string; phase: number }[] = [];
    for (const body of bodies ?? Object.keys(rig.bodies)) for (const stage of stages) for (let phase = 0; phase < PHASES; phase++) who.push({ body, stage, phase });
    if (who.length > MAX_CHARACTERS) throw new Error(`transitions: ${who.length} residents (max ${MAX_CHARACTERS})`);
    const sims = who.map((w, id) => ({
      id,
      name: `${w.body} ${w.stage} ${w.phase}`,
      household: 0,
      gender: w.body,
      attractedTo: [],
      appearance: { body: '#5F86C9', skin: '#C08A64', hair: '#4A3020', hairStyle: 'short', eyes: '#4E7BA6', height: 1, top: 'top.tee', bottom: 'bottom.trousers', shoes: 'shoes.sneakers', stage: w.stage, face: {} },
      traits: [],
      perks: [],
    }));
    const characters = new Characters(view as never, assets);
    if (!(await characters.init())) throw new Error('transitions: no character set');
    // (Everyone posed in full every frame, however small on screen.)
    characters.everyFrame = true;
    await characters.build({ sims: sims as never, objects: [] });

    const FIELDS = ['id', 'x', 'z', 'yaw', 'pose', 'moving', 'social', 'outcome', 'partner', 'anim', 'emotion', 'role', 'away', 'object', 'action', 'mood'] as const;
    const layout = {
      capacity: 6 + FIELDS.length * MAX_CHARACTERS,
      headerLen: 6,
      simStride: FIELDS.length,
      maxSims: MAX_CHARACTERS,
      ...tickRates,
      header: { tick: 0, day: 1, minute: 2, speed: 3, simCount: 4, structureVersion: 5 },
      sim: simFields(FIELDS),
      actions: [],
    };
    const tick = 1 / layout.ticksPerSecond;
    // The walk's cycle at this pace on the reference legs (clips/locomotion.ts: 1.1 m/s over 1.1 s): the stops a twelfth of it apart.
    const cycle = (1.1 * 1.1) / PACE;
    const walkFor = (phase: number) => Math.round((WALK + (cycle * phase) / PHASES) / tick) * tick;
    /** The snapshot of tick `n`: where everyone is and whether they are on their way (sim-core moves, then says so). */
    function snapshot(snap: Float32Array, n: number): void {
      const k = layout.sim;
      const t = n * tick;
      snap[layout.header.speed] = 1;
      snap[layout.header.simCount] = who.length;
      who.forEach((w, i) => {
        const o = layout.headerLen + i * layout.simStride;
        const walked = Math.min(Math.max(t - STAND, 0), walkFor(w.phase));
        snap[o + k.id] = i;
        snap[o + k.x] = (i % 24) * 1.5;
        snap[o + k.z] = Math.floor(i / 24) * 12 + PACE * walked;
        snap[o + k.yaw] = 0;
        snap[o + k.pose] = Pose.Stand;
        snap[o + k.moving] = t > STAND && t - STAND < walkFor(w.phase) ? 1 : 0;
        snap[o + k.partner] = -1;
        snap[o + k.object] = -1;
        snap[o + k.action] = -1;
        snap[o + k.mood!] = 0.8;
      });
    }

    // Each foot's two contact points at rest in its body's frame, the heel and the ball: the foot bone's skinning
    // matrix takes them to the world.
    const feet = who.map((w) => {
      const b = rig.bodies[w.body];
      const s = w.stage === b.stage ? b : b.stages[w.stage];
      const joint = (name: string) => {
        const at = [0, 0, 0];
        for (let j = rig.bones.indexOf(name); j >= 0; j = rig.parents[j]) for (let c = 0; c < 3; c++) at[c] += s.restT[j * 3 + c];
        return at;
      };
      return ['l', 'r'].map((side) => {
        const [ankle, ball] = [joint(`foot_${side}`), joint(`ball_${side}`)];
        return { bone: rig.bones.indexOf(`foot_${side}`) as number, points: [[ankle[0], ball[1], ankle[2] - HEEL * s.pelvisScale], ball] };
      });
    });
    const contact = (i: number, side: number, point: number, out: number[]): void => {
      const { bone, points } = feet[i][side];
      const at = points[point];
      const d = i * width * 4 + bone * BONE_FLOATS;
      for (let r = 0; r < 3; r++) out[r] = poses[d + r * 4] * at[0] + poses[d + r * 4 + 1] * at[1] + poses[d + r * 4 + 2] * at[2] + poses[d + r * 4 + 3];
    };

    const prev = new Float32Array(layout.capacity);
    const curr = new Float32Array(layout.capacity);
    const frame = { prev, curr, alpha: 0, steps: 1, layout, now: 0 };
    const shadows = new Float32Array(MAX_CHARACTERS * 16);
    const realNow = performance.now;
    let now = 0;
    performance.now = () => now;
    const out: FootTravel[] = who.map((w) => ({ ...w, start: { l: 0, r: 0 }, stop: { l: 0, r: 0 }, walk: { l: 0, r: 0 } }));
    try {
      const seconds = STAND + WALK + cycle + REST;
      // Per resident, foot and contact point: where it was last frame, and its standing height.
      const last = who.map(() => [0, 1].map(() => [0, 1].map(() => [NaN, NaN, NaN])));
      const floor = who.map(() => [0, 1].map(() => [Infinity, Infinity]));
      const p = [[0, 0, 0], [0, 0, 0]];
      let n = -1;
      for (let f = 0; f <= seconds * FPS; f++) {
        const t = f / FPS + INTO_TICK * tick;
        const at = Math.floor(t / tick);
        if (at !== n) {
          n = at;
          snapshot(prev, n);
          snapshot(curr, n + 1);
        }
        frame.alpha = t / tick - n;
        frame.now = now = 1000 + t * 1000;
        characters.update(frame as never, () => true, shadows);
        who.forEach((w, i) => {
          const stopAt = STAND + walkFor(w.phase);
          for (let side = 0; side < 2; side++) {
            const [was, low] = [last[i][side], floor[i][side]];
            for (let k = 0; k < 2; k++) contact(i, side, k, p[k]);
            // (Their standing heights: the lowest they are while standing at the start.)
            if (t < STAND) for (let k = 0; k < 2; k++) low[k] = Math.min(low[k], p[k][1]);
            else {
              // The lower of the two now, if it is down and was.
              const k = p[0][1] - low[0] < p[1][1] - low[1] ? 0 : 1;
              const down = p[k][1] < low[k] + DOWN && was[k][1] < low[k] + DOWN;
              trace?.(w, { t, start: STAND, stop: stopAt }, side ? 'r' : 'l', k, p[k][0], p[k][1] - low[k], p[k][2], down);
              if (down) {
                const way = Math.hypot(p[k][0] - was[k][0], p[k][2] - was[k][2]);
                const key = side ? 'r' : 'l';
                // (A tick's lead: the frame the snapshot says so is the one it starts or stops on.)
                if (t >= STAND - tick && t < STAND + SETTLE.start) out[i].start[key] += way;
                else if (t >= stopAt - tick && t < stopAt + SETTLE.stop) out[i].stop[key] += way;
                else if (t >= STAND + 1 && t < STAND + 2) out[i].walk[key] += way;
              }
            }
            for (let k = 0; k < 2; k++) [was[k][0], was[k][1], was[k][2]] = p[k];
          }
        });
      }
    } finally {
      performance.now = realNow;
      characters.dispose();
    }
    return out;
  } finally {
    world.fetch = realFetch;
    world.location = realLocation;
  }
}

/** The worst foot of each resident (m), per body and stage: the mean and the most over the stride's phases. */
export function summarise(travel: FootTravel[]): { body: string; stage: string; start: { mean: number; max: number }; stop: { mean: number; max: number }; walk: number }[] {
  const groups = new Map<string, FootTravel[]>();
  for (const t of travel) groups.set(`${t.body} ${t.stage}`, [...(groups.get(`${t.body} ${t.stage}`) ?? []), t]);
  return [...groups.values()].map((g) => {
    const of = (kind: 'start' | 'stop') => {
      const worst = g.map((t) => Math.max(t[kind].l, t[kind].r));
      return { mean: worst.reduce((a, b) => a + b, 0) / worst.length, max: Math.max(...worst) };
    };
    return { body: g[0].body, stage: g[0].stage, start: of('start'), stop: of('stop'), walk: Math.max(...g.map((t) => Math.max(t.walk.l, t.walk.r))) };
  });
}
