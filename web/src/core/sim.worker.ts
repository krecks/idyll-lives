/**
 * Sim worker: owns the Rust/WASM simulation and runs it at a fixed rate,
 * independent of rendering. Publishes:
 * - a render snapshot every step (shared memory, zero allocations),
 * - a UI view at ~10 Hz,
 * - world structure (geometry, placements) only when it changes.
 */

import init, { Game, LoadError } from './wasm-pkg/sim_wasm.js';
import type { FromWorker, GameSource, LeanWorld, MeshArrays, SaveProblem, ToWorker, WorldStructure } from './protocol';
import { createSharedSnapshot, SharedSnapshotWriter, type SnapshotLayout } from './snapshot';

interface WorkerScope {
  onmessage: ((e: MessageEvent<ToWorker>) => void) | null;
  postMessage(message: FromWorker, transfer?: Transferable[]): void;
}
const scope = self as unknown as WorkerScope;

const UI_INTERVAL_MS = 100;
const STATS_INTERVAL_MS = 1000;
/**
 * Steps run back to back, at most, to catch up after one ran late. A longer backlog (a stall: a
 * slow step, the debugger, a busy machine) is dropped rather than run as a burst, which the
 * renderer would show as a jump; if steps keep taking longer than their share, the game runs
 * slower than its speed (the debug overlay's steps per second say so).
 */
const MAX_CATCH_UP = 2;

let game: Game | null = null;
let memory: WebAssembly.Memory;
let writer: SharedSnapshotWriter | null = null;
let lastStructure = -1;
let lastItems = -1;
let lastLamps = '';
/** Lot version and region the renderer last got the lot (walls, rooms, meshes) for. */
let lastLot = -1;
let lastRegion = '';
/** Tile rectangle whose walls and floors are sent to the renderer (null = whole town). */
let region: [number, number, number, number] | null = null;
/** What the simulation said as it panicked (its panic hook calls `__wasmPanic` just before the trap). */
let panicked: string | null = null;
/** The simulation failed for good (`stop`): nothing calls into it again. */
let stopped = false;
/** `__crash('panic')`: the next step panics. */
let panicNext = false;
/** Sim steps run so far: published with each snapshot (the renderer's clock counts steps, not snapshots). */
let simSteps = 0;

(self as unknown as { __wasmPanic: (message: string) => void }).__wasmPanic = (message) => {
  panicked = message;
  console.error(`[sim] ${message}`);
};

scope.onmessage = (e) => {
  const msg = e.data;
  if (msg.type === 'debugCrash') return debugCrash(msg.what);
  // One guard for every message: a trap ends the game; anything else fails just that request
  // (a command's error is told to the player).
  const requestId = 'requestId' in msg ? msg.requestId : undefined;
  try {
    if (msg.type === 'init') {
      void start(msg.content, msg.source).catch((err) => fail(err, true));
    } else if (msg.type === 'check') {
      void check(msg.content, msg.save).catch((err) => fail(err, true));
    } else if (requestId !== undefined && !game) {
      throw new Error(stopped ? 'The game stopped because of an error.' : "The game isn't running.");
    } else if (msg.type === 'save' && game) {
      scope.postMessage({ type: 'saved', requestId: msg.requestId, data: game.save() });
    } else if (msg.type === 'view') {
      region = msg.region;
      if (game) {
        watch(game);
        postWorld();
      }
    } else if (msg.type === 'inspect' && game) {
      // Their details at once, so the panel opens with them.
      game.inspect(msg.sim ?? undefined);
      postUi();
    } else if (msg.type === 'resync' && game) {
      game.resync_ui();
      postUi();
    } else if (msg.type === 'events' && game) {
      scope.postMessage({ type: 'events', requestId: msg.requestId, events: JSON.parse(game.events()) });
    } else if (msg.type === 'blueprint' && game) {
      try {
        scope.postMessage({ type: 'blueprint', requestId: msg.requestId, data: game.blueprint(msg.household) });
      } catch (err) {
        if (isTrap(err)) throw err;
        scope.postMessage({ type: 'blueprint', requestId: msg.requestId, data: null, error: err instanceof Error ? err.message : String(err) });
      }
    } else if (msg.type === 'socialOptions' && game) {
      const options = JSON.parse(game.social_options(msg.actor, msg.target));
      scope.postMessage({ type: 'socialOptions', requestId: msg.requestId, options });
    } else if (msg.type === 'command' && game) {
      game.command(JSON.stringify(msg.command));
      publish();
      postUi();
    }
  } catch (err) {
    if (isTrap(err)) stop(err);
    else if (requestId !== undefined) scope.postMessage({ type: 'failed', requestId, message: describe(err) });
    else fail(err, false);
  }
};

async function start(content: string, source: GameSource): Promise<void> {
  memory = (await init()).memory;
  try {
    game = 'save' in source ? Game.fromSave(content, source.save) : new Game(content, source.lot, source.seed);
  } catch (err) {
    if (!('save' in source)) throw err;
    const { message, problem } = refusal(err);
    scope.postMessage({ type: 'error', message, fatal: true, problem: problem ?? undefined });
    return;
  }
  watch(game);
  const layout: SnapshotLayout = JSON.parse(game.snapshot_layout());

  const shared = typeof SharedArrayBuffer !== 'undefined' && self.crossOriginIsolated ? createSharedSnapshot(layout) : null;
  writer = shared ? new SharedSnapshotWriter(shared, layout.capacity) : null;
  scope.postMessage({ type: 'ready', layout, shared, catalog: JSON.parse(game.catalog()) });

  publish();
  postUi();
  runLoop(1000 / layout.ticksPerSecond);
}

/** Opens a save only to see whether it can be played (an imported file), then lets it go. */
async function check(content: string, save: string): Promise<void> {
  await init();
  try {
    Game.fromSave(content, save).free();
    scope.postMessage({ type: 'checked', problem: null, message: '' });
  } catch (err) {
    const { message, problem } = refusal(err);
    if (problem) scope.postMessage({ type: 'checked', problem, message });
    else scope.postMessage({ type: 'error', message, fatal: true });
  }
}

/**
 * Why `Game.fromSave` failed: what the simulation found wrong with the save, or a trap while it
 * loaded (`failed`: the save is kept, since a loader bug would otherwise remove good saves one by
 * one). No problem: the save isn't to blame.
 */
function refusal(err: unknown): { message: string; problem: SaveProblem | null } {
  if (err instanceof LoadError) {
    const out = { message: err.message, problem: (err.kind || null) as SaveProblem | null };
    err.free();
    return out;
  }
  return { message: err instanceof Error ? err.message : String(err), problem: err instanceof WebAssembly.RuntimeError ? 'failed' : null };
}

function runLoop(stepMs: number): void {
  let next = performance.now() + stepMs;
  let lastUi = 0;
  let lastStats = performance.now();
  let steps = 0;
  let busyMs = 0;
  let maxMs = 0;
  const step = () => {
    if (stopped) return;
    const start = performance.now();
    let now: number;
    try {
      if (panicNext) game!.debugPanic('a panic on purpose (__crash)');
      game!.advance();
      simSteps++;
      publish();
      now = performance.now();
      if (now - lastUi >= UI_INTERVAL_MS) {
        lastUi = now;
        postUi();
        now = performance.now();
      }
    } catch (err) {
      // The step loop is the game: whatever stopped a step, the game can't go on.
      return stop(err);
    }
    steps++;
    busyMs += now - start;
    maxMs = Math.max(maxMs, now - start);
    if (now - lastStats >= STATS_INTERVAL_MS) {
      const span = now - lastStats;
      scope.postMessage({
        type: 'stats',
        stats: {
          stepsPerSecond: (steps * 1000) / span,
          targetPerSecond: 1000 / stepMs,
          stepMs: busyMs / steps,
          stepMaxMs: maxMs,
          busy: busyMs / span,
          memoryBytes: memory.buffer.byteLength,
        },
      });
      lastStats = now;
      steps = busyMs = maxMs = 0;
    }
    next += stepMs;
    // (Steps due by now run at once: at most MAX_CATCH_UP of them.)
    next = Math.max(next, now - (MAX_CATCH_UP - 1) * stepMs);
    setTimeout(step, Math.max(0, next - now));
  };
  setTimeout(step, stepMs);
}

function publish(): void {
  const g = game!;
  // Re-create the view each time: WASM memory growth detaches old views.
  const view = new Float32Array(memory.buffer, g.snapshot_ptr(), g.snapshot_len());
  if (writer) {
    writer.write(view, simSteps);
  } else {
    const copy = view.slice();
    scope.postMessage({ type: 'snapshot', data: copy, step: simSteps }, [copy.buffer]);
  }
  const version = g.structure_version();
  // (Before the world, which the renderer builds with them; ids shift as objects go.)
  const lamps = `${version}:${g.lights_version()}`;
  if (lamps !== lastLamps) {
    lastLamps = lamps;
    scope.postMessage({ type: 'lamps', off: JSON.parse(g.lamps_off()) });
  }
  if (version !== lastStructure) {
    lastStructure = version;
    postWorld();
  }
  const items = g.items_version();
  if (items !== lastItems) {
    lastItems = items;
    scope.postMessage({ type: 'items', items: JSON.parse(g.items()) });
  }
}

function postWorld(): void {
  const g = game!;
  // The lot (walls, rooms, floors and their meshes) only when it or the viewed region changed:
  // most edits move furniture, and the main thread keeps the lot it has.
  const lot = g.lot_version();
  const area = JSON.stringify(region);
  if (lot === lastLot && area === lastRegion) {
    const world: LeanWorld = JSON.parse(g.structure(false));
    scope.postMessage({ type: 'world', world, full: false });
    return;
  }
  lastLot = lot;
  lastRegion = area;
  const meshes = { walls: mesh(g, 0), wallsLow: mesh(g, 1), floors: mesh(g, 2) };
  const world: WorldStructure = { ...JSON.parse(g.structure(true)), meshes };
  const transfer = Object.values(meshes).flatMap((m) => [m.positions.buffer, m.normals.buffer, m.uvs.buffer, m.indices.buffer]);
  scope.postMessage({ type: 'world', world, full: true }, transfer);
}

/** The viewed lot is simulated in full detail; lots nobody watches in less (`sim_core::lod`). */
function watch(g: Game): void {
  if (region) g.set_view(region[0], region[1], region[2], region[3]);
  else g.clear_view();
}

function mesh(g: Game, kind: number): MeshArrays {
  const [x0, z0, x1, z1] = region ?? [0, 0, 1 << 16, 1 << 16];
  const m = g.build_mesh(kind, x0, z0, x1, z1);
  const out = { positions: m.positions, normals: m.normals, uvs: m.uvs, indices: m.indices };
  m.free();
  return out;
}

function postUi(): void {
  scope.postMessage({ type: 'ui', ui: JSON.parse(game!.ui_update()) });
}

function fail(err: unknown, fatal: boolean): void {
  scope.postMessage({ type: 'error', message: describe(err), fatal });
}

/**
 * A trap: a Rust panic (the build aborts on panic) or another WebAssembly fault, or
 * wasm-bindgen's "recursive use" error (a call that trapped still holds the game). The instance
 * is broken after any of them: its stack and allocator are wherever the call stopped.
 */
function isTrap(err: unknown): boolean {
  return err instanceof WebAssembly.RuntimeError || (err instanceof Error && /recursive use of an object|Maximum call stack/.test(err.message));
}

/** The game failed for good: the loop stops, nothing calls into the instance again, and the main thread is told. */
function stop(err: unknown): void {
  if (stopped) return;
  stopped = true;
  game = null;
  fail(err, true);
}

/** The error's message, or what the simulation said as it panicked. */
function describe(err: unknown): string {
  if (panicked !== null) return panicked;
  return err instanceof Error ? err.message : String(err);
}

/** `?debug`'s `__crash` (game/session.ts): fails on purpose, to see how the game copes. */
function debugCrash(what: 'panic' | 'throw' | 'hang'): void {
  if (what === 'panic') panicNext = true;
  else if (what === 'throw') {
    setTimeout(() => {
      throw new Error('an error nothing caught, on purpose (__crash)');
    });
  } else for (;;);
}
