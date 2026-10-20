/**
 * Main-thread side of the simulation. Owns the worker and turns the snapshot stream
 * into interpolatable frames for the renderer. Allocation-free per frame.
 */

import type { Catalog, Command, FromWorker, GameSource, ItemPlacement, SaveProblem, SimThreadStats, SocialEvent, SocialOption, ToWorker, UiUpdate, WorldStructure } from './protocol';
import { SharedSnapshotReader, type SnapshotLayout } from './snapshot';

/** The simulation refused a save (`problem` says why; the message is the simulation's own). */
export class SaveRefused extends Error {
  constructor(
    readonly problem: SaveProblem,
    message: string,
  ) {
    super(message);
  }
}

/** What the renderer needs each frame. Reused; do not keep references across frames. */
export interface FrameState {
  prev: Float32Array;
  curr: Float32Array;
  /** Progress from `prev` to `curr`: 0..1, slightly above 1 when a snapshot is late (extrapolates). */
  alpha: number;
  /** Sim steps from `prev` to `curr`: 1, more when frames missed snapshots (a long frame, under 20 fps). */
  steps: number;
  layout: SnapshotLayout;
  now: number;
}

type Listener<T> = (value: T) => void;

/** How quickly the frame clock follows snapshot arrivals (per arrival); small = smooth over jitter. */
const CLOCK_GAIN = 0.05;
/** Arrivals further off than this many steps (hitch, hidden tab, backlog) restart the clock. */
const CLOCK_RESET_STEPS = 2;
/** How far past the newest snapshot motion may continue while waiting for a late one. */
const MAX_EXTRAPOLATION = 1.3;
/** Longest wait for the worker's answer to a request (a save, the journal…). */
const REQUEST_LIMIT_MS = 15_000;
/**
 * Longest a start may go without progress before it gives up (a hang would stay on "Moving in…"):
 * the worker reports each phase, and the download as it arrives, so only a stuck one runs out.
 */
const START_STALL_MS = 30_000;
/** The worker sends something every 100 ms (UI updates); this long without a word, it has hung. */
const SILENCE_LIMIT_MS = 8000;
const WATCHDOG_MS = 1000;
const STOPPED = 'The game stopped because of an error.';

export class SimBridge {
  readonly layout: SnapshotLayout;
  private readonly worker: Worker;
  private readonly reader: SharedSnapshotReader | null;
  private readonly stepMs: number;
  /** The three newest snapshots, oldest first, and a buffer for the next one. */
  private prev2: Float32Array;
  private prev: Float32Array;
  private curr: Float32Array;
  private scratch: Float32Array;
  /** The sim steps they show (the worker counts them; snapshots a frame missed are skipped). */
  private step2 = 0;
  private step1 = 0;
  private step0 = 0;
  /**
   * Frame clock: the snapshot of sim step `n` is drawn as arriving at `clockBase + n * stepMs`.
   * It follows the average arrival rhythm, not each arrival, so timer jitter (e.g. browsers
   * coarsening timers for privacy) doesn't make motion stall and jump; and it counts the steps
   * the worker ran, so snapshots a long frame missed don't put it behind.
   */
  private clockBase = 0;
  private clocked = false;
  private hasData = false;
  private pending = false;
  /** Sim step of the snapshot posted last (without shared memory). */
  private pendingStep = 0;
  private readonly frameState: FrameState;

  private world: WorldStructure | null = null;
  /** A UI update arrived (late subscribers then ask for everything again). */
  private uiSeen = false;
  /** The worker's latest report on how it keeps up (null until the first, after a second). */
  threadStats: SimThreadStats | null = null;
  private worldListeners: Listener<WorldStructure>[] = [];
  /** Food and dishes, as last sent. */
  private items: ItemPlacement[] | null = null;
  private itemListeners: Listener<ItemPlacement[]>[] = [];
  /** The lamps switched off (object ids; sim-core `lights.rs`), as last sent. */
  private lampsOff: ReadonlySet<number> | null = null;
  private lampListeners: Listener<ReadonlySet<number>>[] = [];
  private uiListeners: Listener<UiUpdate>[] = [];
  private errorListeners: Listener<string>[] = [];
  private fatalListeners: Listener<string>[] = [];
  private nextRequest = 1;
  private readonly requests = new Map<number, { resolve: (value: never) => void; reject: (err: Error) => void; timer: ReturnType<typeof setTimeout> }>();
  /** Why the simulation is gone (it failed, or the game ended); null while it runs. */
  private gone: string | null = null;
  /** The watchdog: when the worker last said anything, and when it last looked. */
  private lastHeard = performance.now();
  private lastWatch = performance.now();
  private readonly watchdog: ReturnType<typeof setInterval>;

  private constructor(
    worker: Worker,
    layout: SnapshotLayout,
    shared: SharedArrayBuffer | null,
    /** Careers and build/buy rules, as the simulation expanded them. */
    readonly catalog: Catalog,
  ) {
    this.worker = worker;
    this.layout = layout;
    this.stepMs = 1000 / layout.ticksPerSecond;
    this.reader = shared ? new SharedSnapshotReader(shared, layout.capacity) : null;
    this.prev2 = new Float32Array(layout.capacity);
    this.prev = new Float32Array(layout.capacity);
    this.curr = new Float32Array(layout.capacity);
    this.scratch = new Float32Array(layout.capacity);
    this.frameState = { prev: this.prev, curr: this.curr, alpha: 0, steps: 1, layout, now: 0 };
    worker.onmessage = (e: MessageEvent<FromWorker>) => {
      this.lastHeard = performance.now();
      this.handle(e.data);
    };
    // Anything the worker's own guard didn't catch: the game can't be trusted to go on.
    worker.onerror = (e) => {
      e.preventDefault();
      this.fail(`${e.message || 'The game stopped because of an error.'}${e.filename ? ` (${e.filename}:${e.lineno})` : ''}`);
    };
    worker.onmessageerror = () => this.fail("A message from the game couldn't be read.");
    this.watchdog = setInterval(this.watch, WATCHDOG_MS);
  }

  static start(content: string, source: GameSource): Promise<SimBridge> {
    const worker = new Worker(new URL('./sim.worker.ts', import.meta.url), { type: 'module' });
    return new Promise((resolve, reject) => {
      // A start that failed doesn't leave its worker (and the simulation's memory) behind.
      let timer: ReturnType<typeof setTimeout> | undefined;
      const arm = () => {
        clearTimeout(timer);
        timer = setTimeout(() => failed(new Error('The game took too long to start.')), START_STALL_MS);
      };
      const failed = (err: Error) => {
        clearTimeout(timer);
        worker.terminate();
        reject(err);
      };
      worker.onerror = (e) => failed(new Error(e.message || 'sim worker failed to start'));
      worker.onmessage = (e: MessageEvent<FromWorker>) => {
        const msg = e.data;
        if (msg.type === 'starting') arm();
        else if (msg.type === 'ready') {
          clearTimeout(timer);
          resolve(new SimBridge(worker, msg.layout, msg.shared, msg.catalog));
        } else if (msg.type === 'error') failed(msg.problem ? new SaveRefused(msg.problem, msg.message) : new Error(msg.message));
      };
      arm();
      worker.postMessage({ type: 'init', content, source } satisfies ToWorker);
    });
  }

  /** The simulation runs (it hasn't failed, and the game hasn't ended): it can answer and be saved. */
  get alive(): boolean {
    return this.gone === null;
  }

  /** Whether `save` opens (without starting a game): null if it does, else what's wrong with it. */
  static checkSave(content: string, save: string): Promise<SaveProblem | null> {
    const worker = new Worker(new URL('./sim.worker.ts', import.meta.url), { type: 'module' });
    return new Promise<SaveProblem | null>((resolve, reject) => {
      worker.onerror = (e) => reject(new Error(e.message || 'sim worker failed to start'));
      worker.onmessage = (e: MessageEvent<FromWorker>) => {
        const msg = e.data;
        if (msg.type === 'checked') resolve(msg.problem);
        else if (msg.type === 'error') reject(new Error(msg.message));
      };
      worker.postMessage({ type: 'check', content, save } satisfies ToWorker);
    }).finally(() => worker.terminate());
  }

  get sharedMemory(): boolean {
    return this.reader !== null;
  }

  send(command: Command): void {
    this.worker.postMessage({ type: 'command', command } satisfies ToWorker);
  }

  /** Limits wall/floor geometry to a tile rectangle (the lot being viewed); null = all. */
  setView(region: [number, number, number, number] | null): void {
    this.worker.postMessage({ type: 'view', region } satisfies ToWorker);
  }

  /** Serialized game state, taken between two sim steps. */
  requestSave(): Promise<string> {
    return this.request<string>((requestId) => ({ type: 'save', requestId }));
  }

  /** What `actor` can do with `target`, with success chances. */
  requestSocialOptions(actor: number, target: number): Promise<SocialOption[]> {
    return this.request<SocialOption[]>((requestId) => ({ type: 'socialOptions', requestId, actor, target }));
  }

  /** The whole story log, oldest first. */
  requestEvents(): Promise<SocialEvent[]> {
    return this.request<SocialEvent[]>((requestId) => ({ type: 'events', requestId }));
  }

  /** The household's home as a blueprint (JSON); rejects if it has no home. */
  async requestBlueprint(household: number): Promise<string> {
    const reply = await this.request<{ data: string | null; error?: string }>((requestId) => ({ type: 'blueprint', requestId, household }));
    if (reply.data === null) throw new Error(reply.error ?? 'no blueprint');
    return reply.data;
  }

  /** `?debug`'s `__crash` (game/session.ts): the simulation fails on purpose. */
  debugCrash(what: 'panic' | 'throw' | 'hang'): void {
    this.worker.postMessage({ type: 'debugCrash', what } satisfies ToWorker);
  }

  /** Answers come back by id; none is waited for forever (a failed or hung game rejects them). */
  private request<T>(message: (requestId: number) => ToWorker): Promise<T> {
    if (this.gone !== null) return Promise.reject(new Error(this.gone));
    const requestId = this.nextRequest++;
    return new Promise<T>((resolve, reject) => {
      const timer = setTimeout(() => this.reject(requestId, new Error("The game didn't answer.")), REQUEST_LIMIT_MS);
      this.requests.set(requestId, { resolve: resolve as (value: never) => void, reject, timer });
      this.worker.postMessage(message(requestId));
    });
  }

  private resolve(requestId: number, value: unknown): void {
    const request = this.requests.get(requestId);
    if (!request) return;
    this.requests.delete(requestId);
    clearTimeout(request.timer);
    request.resolve(value as never);
  }

  private reject(requestId: number, err: Error): void {
    const request = this.requests.get(requestId);
    if (!request) return;
    this.requests.delete(requestId);
    clearTimeout(request.timer);
    request.reject(err);
  }

  /** No word from the worker for too long while the page runs normally: it has hung. */
  private readonly watch = () => {
    const now = performance.now();
    // A late look (a hidden tab's throttled timers, the main thread busy): the worker's messages
    // may still be queued behind it, so start counting again.
    if (document.hidden || now - this.lastWatch > 2 * WATCHDOG_MS) this.lastHeard = now;
    this.lastWatch = now;
    if (now - this.lastHeard > SILENCE_LIMIT_MS) this.fail('The game stopped responding.');
  };

  /** The simulation failed for good: its worker goes, every request fails, and the game is told (`onFatal`). */
  private fail(message: string): void {
    if (this.gone !== null) return;
    console.error('[sim] the game stopped:', message);
    this.end(STOPPED);
    for (const fn of this.fatalListeners) fn(message);
  }

  private end(message: string): void {
    this.gone = message;
    clearInterval(this.watchdog);
    this.worker.terminate();
    for (const id of [...this.requests.keys()]) this.reject(id, new Error(message));
  }

  /** Latest world structure is replayed to late subscribers. */
  onWorld(fn: Listener<WorldStructure>): void {
    this.worldListeners.push(fn);
    if (this.world) fn(this.world);
  }

  /** Food and dishes whenever they change; the latest are replayed to late subscribers. */
  onItems(fn: Listener<ItemPlacement[]>): void {
    this.itemListeners.push(fn);
    if (this.items) fn(this.items);
  }

  /** The lamps switched off (object ids) whenever that changes; the latest replayed to late subscribers. */
  onLamps(fn: Listener<ReadonlySet<number>>): void {
    this.lampListeners.push(fn);
    if (this.lampsOff) fn(this.lampsOff);
  }

  /** Whether lamp `id` is switched off, as last sent. */
  lampOff(id: number): boolean {
    return !!this.lampsOff?.has(id);
  }

  /** The item `id` as last sent. */
  item(id: number): ItemPlacement | undefined {
    return this.items?.find((it) => it.id === id);
  }

  /** UI updates carry changes only; a late subscriber gets a full one (`resyncUi`). */
  onUi(fn: Listener<UiUpdate>): void {
    this.uiListeners.push(fn);
    if (this.uiSeen) this.resyncUi();
  }

  /** The next UI update carries everything (the main thread lost track, or starts over). */
  resyncUi(): void {
    this.worker.postMessage({ type: 'resync' } satisfies ToWorker);
  }

  /** The resident whose panel is open: their details and relationships come with UI updates. */
  inspect(sim: number | null): void {
    this.worker.postMessage({ type: 'inspect', sim } satisfies ToWorker);
  }

  onError(fn: Listener<string>): void {
    this.errorListeners.push(fn);
  }

  /** The simulation stopped for good (a trap, a failed step, an uncaught error, or it hung): what went wrong. */
  onFatal(fn: Listener<string>): void {
    this.fatalListeners.push(fn);
  }

  /** Whether a snapshot has arrived (the first one can be later than `start`). */
  get hasSnapshot(): boolean {
    return this.hasData;
  }

  /** Pulls the newest snapshot. Call once per rendered frame, before `frame()`. */
  sync(now: number): void {
    const fresh = this.reader ? this.reader.read(this.scratch) : this.pending;
    if (!fresh) return;
    this.pending = false;
    const step = this.reader ? this.reader.step : this.pendingStep;
    if (this.hasData && step === this.step0) {
      // Published again without a step (a command took effect): the same moment, changed.
      const old = this.curr;
      this.curr = this.scratch;
      this.scratch = old;
      return;
    }
    const old = this.prev2;
    this.prev2 = this.prev;
    this.prev = this.curr;
    this.curr = this.scratch;
    this.scratch = old;
    this.step2 = this.step1;
    this.step1 = this.step0;
    this.step0 = step;
    if (!this.hasData) {
      this.prev.set(this.curr);
      this.prev2.set(this.curr);
      this.step1 = step - 1;
      this.step2 = step - 2;
      this.hasData = true;
    }
    const late = now - (this.clockBase + step * this.stepMs);
    if (!this.clocked || Math.abs(late) > CLOCK_RESET_STEPS * this.stepMs) {
      this.clockBase = now - step * this.stepMs;
      this.clocked = true;
    } else {
      this.clockBase += late * CLOCK_GAIN;
    }
  }

  /** Interpolation state for rendering: draws one step behind the sim for smooth motion. */
  frame(now: number): FrameState {
    const f = this.frameState;
    // The (fractional) sim step shown now.
    const at = (now - this.clockBase) / this.stepMs - 1;
    if (at < this.step1) {
      // The newest snapshot came early: finish the step(s) before it first.
      f.prev = this.prev2;
      f.curr = this.prev;
      f.steps = Math.max(1, this.step1 - this.step2);
      f.alpha = Math.max(0, (at - this.step2) / f.steps);
    } else {
      f.prev = this.prev;
      f.curr = this.curr;
      f.steps = Math.max(1, this.step0 - this.step1);
      f.alpha = Math.min((at - this.step1) / f.steps, MAX_EXTRAPOLATION);
    }
    f.now = now;
    return f;
  }

  /** Most recent snapshot (not interpolated). */
  latest(): Float32Array {
    return this.curr;
  }

  dispose(): void {
    if (this.gone === null) this.end('The game has ended.');
  }

  private handle(msg: FromWorker): void {
    switch (msg.type) {
      case 'snapshot':
        this.scratch.set(msg.data);
        this.pendingStep = msg.step;
        this.pending = true;
        break;
      case 'world': {
        // A lean world leaves out the lot: keep the one we have (same arrays, so consumers
        // can tell by reference that walls, floors and meshes didn't change).
        const world: WorldStructure = msg.full || !this.world ? (msg.world as WorldStructure) : { ...this.world, ...msg.world };
        this.world = world;
        for (const fn of this.worldListeners) fn(world);
        break;
      }
      case 'items':
        this.items = msg.items;
        for (const fn of this.itemListeners) fn(msg.items);
        break;
      case 'lamps': {
        const off = new Set(msg.off);
        this.lampsOff = off;
        for (const fn of this.lampListeners) fn(off);
        break;
      }
      case 'ui':
        this.uiSeen = true;
        for (const fn of this.uiListeners) fn(msg.ui);
        break;
      case 'saved':
        this.resolve(msg.requestId, msg.data);
        break;
      case 'socialOptions':
        this.resolve(msg.requestId, msg.options);
        break;
      case 'events':
        this.resolve(msg.requestId, msg.events);
        break;
      case 'blueprint':
        this.resolve(msg.requestId, { data: msg.data, error: msg.error });
        break;
      case 'stats':
        this.threadStats = msg.stats;
        break;
      case 'failed':
        this.reject(msg.requestId, new Error(msg.message));
        break;
      case 'error':
        if (msg.fatal) this.fail(msg.message);
        else for (const fn of this.errorListeners) fn(msg.message);
        break;
      case 'starting':
      case 'ready':
        break;
    }
  }
}
