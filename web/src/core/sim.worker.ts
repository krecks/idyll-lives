/**
 * Sim worker: owns the Rust/WASM simulation and runs it at a fixed rate,
 * independent of rendering. Publishes:
 * - a render snapshot every step (shared memory, zero allocations),
 * - a UI view at ~10 Hz,
 * - world structure (geometry, placements) when it changes: what changed, at most every
 *   `WORLD_INTERVAL_MS` (`view::StructureSync`).
 */

import init, { Game, LoadError } from './wasm-pkg/sim_wasm.js';
import { newestObjectUsed, type FromWorker, type GameSource, type MeshArrays, type SaveProblem, type ToWorker, type WorldUpdate } from './protocol';
import { createSharedSnapshot, readLayout, SharedSnapshotWriter, type SnapshotLayout } from './snapshot';

interface WorkerScope {
  onmessage: ((e: MessageEvent<ToWorker>) => void) | null;
  postMessage(message: FromWorker, transfer?: Transferable[]): void;
  /** Called by the wasm runtime just before a Rust panic traps; set below, where the runtime looks for it. */
  __wasmPanic?(message: string): void;
}
const scope = self as WorkerScope;

const UI_INTERVAL_MS = 100;
/**
 * The structure (and the lamps and food that go with it) is sent at most this often as time goes
 * by: a busy town changes it every few steps at speed 3 (a door opened, a crop grown, food put
 * down), and the main thread applies each message. A command's or a new view's is sent at once,
 * and so is one with a thing a resident already uses in the snapshots (`sentObject`).
 */
const WORLD_INTERVAL_MS = 100;
const STATS_INTERVAL_MS = 1000;
/** While the wasm downloads, how often the main thread hears that it's still coming. */
const START_PROGRESS_MS = 500;
/**
 * The most real time (ms) the steps catch up on, run back to back, after one ran late (a step's
 * worth at 20 steps a second: two steps at once at most). A longer backlog (a stall: a slow step,
 * the debugger, a busy machine) is dropped rather than run as a burst, which the renderer would
 * show as a jump; if steps keep taking longer than their share, the game runs slower than its
 * speed (the debug overlay's steps per second say so).
 */
const MAX_BACKLOG_MS = 50;

let game: Game | null = null;
let memory: WebAssembly.Memory;
let layout: SnapshotLayout;
let writer: SharedSnapshotWriter | null = null;
let lastStructure = -1;
let lastItems = -1;
let lastLights = -1;
/** Lot version and region the renderer last got the lot (walls, rooms, meshes) for. */
let lastLot = -1;
let lastRegion = '';
/** The view's `seq` (`view` message): echoed with each world, so the main thread knows which view it was built for. */
let viewSeq = 0;
/** When the structure, and the food, were last sent. */
let worldAt = -Infinity;
let itemsAt = -Infinity;
/**
 * The highest object id the main thread has had a world with (or was sent a world for at once):
 * a resident using a newer one (just bought) would otherwise use a thing that isn't there yet.
 */
let sentObject = -1;
/** Tile rectangle whose walls and floors are sent to the renderer (null = whole town). */
let region: [number, number, number, number] | null = null;
/**
 * The main thread has said which region it views (`view`, null too). Till then no meshes go: the
 * first world names the home it asks for next, and a big world's whole town is 34 MB of them.
 */
let viewed = false;
/** Tile rectangle the camera streams in: the sim's full-detail area (null: the view's). */
let watched: [number, number, number, number] | null = null;
/** What the simulation said as it panicked (its panic hook calls `__wasmPanic` just before the trap). */
let panicked: string | null = null;
/** The simulation failed for good (`stop`): nothing calls into it again. */
let stopped = false;
/** `__crash('panic')`: the next step panics. */
let panicNext = false;
/** Sim steps run so far: published with each snapshot (the renderer's clock counts steps, not snapshots). */
let simSteps = 0;

scope.__wasmPanic = (message) => {
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
      void start(msg.content, msg.source, msg.speed, msg.module).catch((err) => fail(err, true));
    } else if (msg.type === 'check') {
      void check(msg.content, msg.save).catch((err) => fail(err, true));
    } else if (requestId !== undefined && !game) {
      throw new Error(stopped ? 'The game stopped because of an error.' : "The game isn't running.");
    } else if (msg.type === 'save' && game) {
      scope.postMessage({ type: 'saved', requestId: msg.requestId, data: game.save() });
    } else if (msg.type === 'view') {
      region = msg.region;
      viewed = true;
      viewSeq = msg.seq;
      if (game) {
        watch(game);
        postWorld();
      }
    } else if (msg.type === 'watch') {
      watched = msg.region;
      if (game) watch(game);
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
      // (What the player did shows at once.)
      publish(true);
      postUi();
    }
  } catch (err) {
    if (isTrap(err)) stop(err);
    else if (requestId !== undefined) scope.postMessage({ type: 'failed', requestId, message: describe(err) });
    else fail(err, false);
  }
};

async function start(content: string, source: GameSource, speed: number | undefined, module?: WebAssembly.Module): Promise<void> {
  // Each phase after the download says it's under way (the main thread gives up on a start only
  // when it stalls; it started waiting when it sent `init`). A wasm the page compiled already is used as it is.
  memory = (await init({ module_or_path: module ?? fetchWasm() })).memory;
  starting();
  try {
    game = 'save' in source ? Game.fromSave(content, source.save) : new Game(content, source.lot, source.seed);
  } catch (err) {
    if (!('save' in source)) throw err;
    const { message, problem } = refusal(err);
    scope.postMessage({ type: 'error', message, fatal: true, problem: problem ?? undefined });
    return;
  }
  watch(game);
  // The first UI view is taken before the start's speed holds the game: it carries the game's own
  // speed (a save's), which play resumes at.
  const ui = JSON.parse(game.ui_update());
  // (Before the first step: a game that starts paused runs none.)
  if (speed !== undefined) game.command(JSON.stringify({ type: 'setSpeed', speed }));
  layout = readLayout(game.snapshot_layout());

  const shared = typeof SharedArrayBuffer !== 'undefined' && self.crossOriginIsolated ? createSharedSnapshot(layout) : null;
  writer = shared ? new SharedSnapshotWriter(shared, layout.capacity) : null;
  scope.postMessage({ type: 'ready', layout, shared, catalog: JSON.parse(game.catalog()), speed: ui.speed });

  publish(true);
  // (The next update says the speed it's held at.)
  scope.postMessage({ type: 'ui', ui });
  runLoop(1000 / layout.ticksPerSecond);
}

/** The simulation's wasm, saying `starting` as it arrives (it still compiles while it downloads). */
async function fetchWasm(): Promise<Response> {
  const response = await fetch(new URL('./wasm-pkg/sim_wasm_bg.wasm', import.meta.url));
  if (!response.ok || !response.body) return response;
  // A copy's arrival is the progress; the response itself goes to the compiler as it came, as the
  // browser keeps compiled code only for a response with its URL.
  void (async () => {
    const reader = response.clone().body!.getReader();
    let last = performance.now();
    for (;;) {
      const { done } = await reader.read();
      if (done) return;
      const now = performance.now();
      if (now - last >= START_PROGRESS_MS) {
        last = now;
        starting();
      }
    }
  })().catch(() => {});
  return response;
}

function starting(): void {
  scope.postMessage({ type: 'starting' });
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
    // (Steps due by now run at once: those of at most MAX_BACKLOG_MS.)
    next = Math.max(next, now - MAX_BACKLOG_MS);
    setTimeout(step, Math.max(0, next - now));
  };
  setTimeout(step, stepMs);
}

/** `now`: send what changed at once (a command's effect, the first world), not waiting for `WORLD_INTERVAL_MS`. */
function publish(now = false): void {
  const g = game!;
  const version = g.structure_version();
  // Re-create the view each time: WASM memory growth detaches old views.
  const view = new Float32Array(memory.buffer, g.snapshot_ptr(), g.snapshot_len());
  // A thing the main thread hasn't heard of, used already (one just bought, walked to at once):
  // its world goes now, not up to `WORLD_INTERVAL_MS` after the snapshots that name it. (Read
  // here: a later call into the simulation may grow its memory, which detaches `view`.)
  const newest = version !== lastStructure ? newestObjectUsed(view, layout) : -1;
  if (writer) {
    writer.write(view, simSteps);
  } else {
    const copy = view.slice();
    scope.postMessage({ type: 'snapshot', data: copy, step: simSteps }, [copy.buffer]);
  }
  const at = performance.now();
  const due = now || at - worldAt >= WORLD_INTERVAL_MS || newest > sentObject;
  // (Before the world, which the renderer builds with them.)
  const lights = g.lights_version();
  // (A light that follows a use, a hood over a hob, bumps the version as it comes on or goes off.)
  if (due && (version !== lastStructure || lights !== lastLights)) {
    lastLights = lights;
    worldAt = at;
    scope.postMessage({ type: 'lamps', off: JSON.parse(g.lamps_off()) });
  }
  if (due && version !== lastStructure) {
    postWorld();
    // (Once for each: an id the world doesn't list doesn't send every change at once.)
    sentObject = Math.max(sentObject, newest);
  }
  const items = g.items_version();
  if (items !== lastItems && (now || at - itemsAt >= WORLD_INTERVAL_MS)) {
    lastItems = items;
    itemsAt = at;
    scope.postMessage({ type: 'items', items: JSON.parse(g.items()) });
  }
}

/**
 * What changed of the structure since the last world sent: the lot (walls, openings, floors, and
 * the rooms as a typed array) only when it changed, and of it only what did; its meshes when it or
 * the viewed region did (once the main thread has said which it views).
 * Most changes are a door opened or a thing moved: the main thread keeps the rest.
 */
function postWorld(): void {
  const g = game!;
  lastStructure = g.structure_version();
  worldAt = performance.now();
  const lot = g.lot_version();
  const area = JSON.stringify(region);
  const lotChanged = lot !== lastLot;
  const meshesDue = viewed && (lotChanged || area !== lastRegion);
  lastLot = lot;
  const world: WorldUpdate = JSON.parse(g.structure_update(lotChanged));
  for (const o of world.objects ?? world.objectSplice?.[2] ?? []) sentObject = Math.max(sentObject, o.id);
  const message: Extract<FromWorker, { type: 'world' }> = { type: 'world', view: viewSeq, world };
  const transfer: Transferable[] = [];
  // (Only when they changed: a big world's are megabytes, and a fence or a floor leaves them.)
  const rooms = lotChanged ? g.rooms_changed() : undefined;
  if (rooms) {
    message.rooms = rooms;
    transfer.push(rooms.buffer);
  }
  if (meshesDue) {
    lastRegion = area;
    const meshes = { walls: mesh(g, 0), wallsLow: mesh(g, 1), floors: mesh(g, 2) };
    message.meshes = meshes;
    transfer.push(...Object.values(meshes).flatMap((m) => [m.positions.buffer, m.normals.buffer, m.uvs.buffer, m.indices.buffer]));
  }
  scope.postMessage(message, transfer);
}

/**
 * What the camera streams in (`watched`; until the renderer says: the viewed lot) is simulated in
 * full detail, lots nobody watches in less (`sim_core::lod`). A sim with `set_watch` (open-world
 * sim-grid) keeps the view (the focus lot's structure) and the watch apart; an older one has only
 * `set_view`, which is its full-detail area.
 */
function watch(g: Game): void {
  const split = g as Game & { set_watch?(x0: number, z0: number, x1: number, z1: number): void; clear_watch?(): void };
  if (typeof split.set_watch === 'function') {
    if (region) g.set_view(region[0], region[1], region[2], region[3]);
    else g.clear_view();
    // (Only the camera's: it's also what's drawn, and cars out of it drive through each other. Till
    // then the viewed lot is the full-detail area, `set_view`, and every car is in sight.)
    if (watched) split.set_watch(watched[0], watched[1], watched[2], watched[3]);
    else split.clear_watch?.();
    return;
  }
  const area = watched ?? region;
  if (area) g.set_view(area[0], area[1], area[2], area[3]);
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

/** The error's message, or what the simulation said as it panicked (said once: a later failure is its own). */
function describe(err: unknown): string {
  const said = panicked;
  panicked = null;
  return said ?? (err instanceof Error ? err.message : String(err));
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
