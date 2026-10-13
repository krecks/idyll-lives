/**
 * Main-thread side of the simulation. Owns the worker and turns the snapshot stream
 * into interpolatable frames for the renderer. Allocation-free per frame.
 */

import type { Catalog, Command, FromWorker, GameSource, ItemPlacement, SimThreadStats, SocialEvent, SocialOption, ToWorker, UiUpdate, WorldStructure } from './protocol';
import { SharedSnapshotReader, type SnapshotLayout } from './snapshot';

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
  private uiListeners: Listener<UiUpdate>[] = [];
  private errorListeners: Listener<string>[] = [];
  private nextRequest = 1;
  private readonly requests = new Map<number, (value: never) => void>();

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
    worker.onmessage = (e: MessageEvent<FromWorker>) => this.handle(e.data);
  }

  static start(content: string, source: GameSource): Promise<SimBridge> {
    const worker = new Worker(new URL('./sim.worker.ts', import.meta.url), { type: 'module' });
    return new Promise((resolve, reject) => {
      worker.onerror = (e) => reject(new Error(e.message || 'sim worker failed to start'));
      worker.onmessage = (e: MessageEvent<FromWorker>) => {
        const msg = e.data;
        if (msg.type === 'ready') resolve(new SimBridge(worker, msg.layout, msg.shared, msg.catalog));
        else if (msg.type === 'error') reject(new Error(msg.message));
      };
      worker.postMessage({ type: 'init', content, source } satisfies ToWorker);
    });
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

  private request<T>(message: (requestId: number) => ToWorker): Promise<T> {
    const requestId = this.nextRequest++;
    return new Promise<T>((resolve) => {
      this.requests.set(requestId, resolve as (value: never) => void);
      this.worker.postMessage(message(requestId));
    });
  }

  private resolve(requestId: number, value: unknown): void {
    this.requests.get(requestId)?.(value as never);
    this.requests.delete(requestId);
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
    this.worker.terminate();
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
      case 'error':
        for (const fn of this.errorListeners) fn(msg.message);
        break;
      case 'ready':
        break;
    }
  }
}
