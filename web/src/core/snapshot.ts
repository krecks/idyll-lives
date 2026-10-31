/**
 * Snapshot layout and the shared-memory channel between the sim worker and the renderer.
 * The layout itself is defined in Rust (`sim-core/src/snapshot.rs`) and fetched at startup.
 */

export interface SnapshotLayout {
  capacity: number;
  headerLen: number;
  simStride: number;
  maxSims: number;
  /** Most servers' rows after the residents' (missing in older layouts). */
  maxStaff?: number;
  ticksPerSecond: number;
  header: {
    tick: number;
    day: number;
    minute: number;
    speed: number;
    simCount: number;
    structureVersion: number;
    /** 1 while time skips ahead because it's quiet at home (missing in older layouts). */
    calm?: number;
    /** Id of the latest milestone event modulo 2^24 (missing in older layouts). */
    majorEvent?: number;
    /** Food and dishes changed (`Items::version` modulo 2^24; missing in older layouts). */
    itemsVersion?: number;
    /**
     * The town's vendors and shopkeepers (sim-core `staff.rs`): as many rows as this right after the residents',
     * each laid out as a resident's, `away` while off work (missing in older layouts: none).
     */
    staffCount?: number;
    /** Households' cars (sim-core `cars.rs`): rows from `carBase`, each laid out as `car` (missing in older layouts: none). */
    carCount?: number;
  };
  /** Where the cars' rows start, and floats per car (missing in older layouts). */
  carBase?: number;
  carStride?: number;
  /** A car's row: `shown` 0 out of sight, 1 parked, 2 driving; where its middle is and which way it faces; its open doors (bit `d` for door `d`, `shared/cars.ts` `DOORS`). */
  car?: { shown: number; x: number; z: number; yaw: number; doors?: number };
  sim: {
    id: number;
    x: number;
    z: number;
    yaw: number;
    pose: number;
    moving: number;
    social: number;
    outcome: number;
    partner: number;
    /** Conversation animation: index into sim-core's `social::ANIMATIONS` + 1 ('talk', 'laugh', ...), 0 = none. */
    anim: number;
    emotion: number;
    /** 1 = started the conversation, 2 = was approached, 3 = joined in and listens (`partner`: the one who started it), 0 = not talking. */
    role: number;
    away: number;
    /** Stable object id (as in `WorldStructure.objects`) the Sim is using or walking to use, -1 = none. */
    object: number;
    /** Index into `actions` while using an object (not while walking to it) or talking, -1 = none. */
    action: number;
    /** Mood 0..1 (missing in older layouts). */
    mood?: number;
    /** Thought bubble: 0 none, 1 skipping a planned block, 2 wants a place for one, 3 kept one, 4 reached a goal. */
    thought?: number;
    /** Activity index (1–3) or goal definition (4). */
    thoughtSubject?: number;
    /** Height in storeys (0 the ground; fractions on the stairs); `z` is in lot rows (see `WorldStructure.storeys`). */
    height?: number;
    /** Object definition (index into the content's objects) the Sim is using or walking to use, -1 = none (missing in older layouts). */
    objectDef?: number;
    /** Index into that definition's interactions, -1 = none (missing in older layouts). */
    interaction?: number;
    /** What they carry: index into `items` + 1, 0 = nothing (missing in older layouts). */
    held?: number;
    /** The recipe they're cooking: index into `recipes` + 1, 0 = none. While cooking, `object` is the station and `action` the step's animation. */
    recipe?: number;
    /** Calling out someone's name (to talk): their id + 1, 0 = nobody (missing in older layouts). */
    call?: number;
    /** While walking, the waypoint they walk to, and where their way is 1.5 m on (`snapshot::AHEAD`); else where they stand (missing in older layouts). */
    wayX?: number;
    wayZ?: number;
    aheadX?: number;
    aheadZ?: number;
    /** The baby they hold in the arms (sim-core `carry::carrying`: lifting them out of the crib, carrying them): their id + 1, 0 = none (missing in older layouts). */
    carry?: number;
    /** How far through what they do at `object` they are, 0..1 (a use's minutes, a recipe step's), while at it; else -1 (missing in older layouts). */
    progress?: number;
    /** What they eat while eating (on the table in front of them or in their hands): index into `items` + 1, 0 = nothing (missing in older layouts). */
    plate?: number;
    /**
     * 1 while the room they're in is kept shut on their account (a bathroom they're in, the bedroom
     * they try for a baby in; sim-core `privacy.rs`), else 0: its doors stay shut to anyone but
     * whoever goes through them (missing in older layouts).
     */
    shuts?: number;
  };
  /** Animation tags from content (`animations` in base.json), named by `sim.action`. */
  actions: string[];
  /** Content item ids (named by `sim.held` - 1). */
  items?: string[];
  /** Content recipe ids (named by `sim.recipe` - 1). */
  recipes?: string[];
}

export const Pose = { Stand: 0, Sit: 1, Lie: 2 } as const;

/**
 * A layout's `sim` indices for the fields it writes: `simFields(['id', 'x'])` gives `{ id: 0, x: 1 }`. The names
 * are checked against the layout, so a field that does not exist is a compile error rather than a NaN write.
 */
export function simFields(fields: readonly (keyof SnapshotLayout['sim'])[]): SnapshotLayout['sim'] {
  return Object.fromEntries(fields.map((f, i) => [f, i])) as SnapshotLayout['sim'];
}

/**
 * Bytes reserved before the snapshot floats: the Int32 sequence counter, then the Int32 number of
 * the sim step the snapshot shows (padded).
 */
const SHARED_HEADER_BYTES = 16;

export function createSharedSnapshot(layout: SnapshotLayout): SharedArrayBuffer {
  return new SharedArrayBuffer(SHARED_HEADER_BYTES + layout.capacity * 4);
}

/**
 * Seqlock writer: the sequence is odd while writing and even when the data is consistent.
 * Lock-free: the reader never blocks the writer.
 */
export class SharedSnapshotWriter {
  /** [sequence, step]. */
  private readonly seq: Int32Array;
  private readonly data: Float32Array;

  constructor(buffer: SharedArrayBuffer, capacity: number) {
    this.seq = new Int32Array(buffer, 0, 2);
    this.data = new Float32Array(buffer, SHARED_HEADER_BYTES, capacity);
  }

  /** `step`: the sim step it shows (snapshots published between steps, after a command, repeat it). */
  write(src: Float32Array, step: number): void {
    Atomics.add(this.seq, 0, 1);
    this.data.set(src);
    Atomics.store(this.seq, 1, step);
    Atomics.add(this.seq, 0, 1);
  }
}

export class SharedSnapshotReader {
  private readonly seq: Int32Array;
  private readonly data: Float32Array;
  private lastSeq = 0;
  /** The sim step of the snapshot last read. */
  step = 0;

  constructor(buffer: SharedArrayBuffer, capacity: number) {
    this.seq = new Int32Array(buffer, 0, 2);
    this.data = new Float32Array(buffer, SHARED_HEADER_BYTES, capacity);
  }

  /** Copies a new consistent snapshot into `into`. Returns false if nothing new (or torn; retried next frame). */
  read(into: Float32Array): boolean {
    const before = Atomics.load(this.seq, 0);
    if (before === this.lastSeq || before & 1) return false;
    into.set(this.data);
    const step = Atomics.load(this.seq, 1);
    if (Atomics.load(this.seq, 0) !== before) return false;
    this.lastSeq = before;
    this.step = step;
    return true;
  }
}
