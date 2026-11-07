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
    /**
     * The traffic lights' signal heads (sim-core `traffic::Signal`: one per arm of each lit junction, the same rows from
     * tick to tick): rows from `lightBase`, each laid out as `light` (missing in older layouts: none).
     */
    lightCount?: number;
  };
  /** Where the cars' rows start, and floats per car (missing in older layouts). */
  carBase?: number;
  carStride?: number;
  /**
   * A car's row: `shown` 0 out of sight, 1 parked, 2 driving; where its middle is and which way it faces; its open doors
   * (bit `d` for door `d`, `shared/cars.ts` `DOORS`); what it is (`kind`, one of `kinds`: a household's car, its look in the
   * structure's `cars`, or a work taxi, the renderer's own look; missing in older layouts: all households').
   */
  car?: { shown: number; x: number; z: number; yaw: number; doors?: number; kind?: number; kinds?: { household: number; taxi: number } };
  /** Where the signal heads' rows start, floats per head, and how many there's room for (missing in older layouts). */
  lightBase?: number;
  lightStride?: number;
  maxLights?: number;
  /**
   * A signal head's row: `shown` 1 to be seen; where its pole stands on the kerb and which way its head faces (towards
   * those coming in); `state` what it shows, one of `states` (red, amber, green, walk: red, those on foot crossing).
   */
  light?: {
    shown: number;
    x: number;
    z: number;
    yaw: number;
    state: number;
    states: { red: number; amber: number; green: number; walk: number };
  };
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
    /** Stable id of the thing they ride (a bike, an e-scooter), drawn under them and not at its place, or -1; left out by a place they're visiting, -2 less its stable id (not drawn at its place either). */
    ride?: number;
    /** That bike left out (`ride` -2 less its stable id): where its middle stands and which way it faces (rad, as a placement's turn); else 0 (missing in older layouts). */
    parkX?: number;
    parkZ?: number;
    parkYaw?: number;
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
 * The shared snapshot is a triple buffer: three slots, each the Int32 number of the sim step it
 * shows (padded) and then the snapshot's floats. The writer fills a slot of its own, the reader
 * copies out of a slot of its own, and the third is the newest one written, waiting: each side
 * swaps its slot for that one with an atomic exchange. Neither ever touches a slot the other has,
 * so the reader can't see one half written. (A seqlock, checking a counter before and after a
 * plain copy, isn't safe in JavaScript: nothing keeps the copy's reads between the two checks, and
 * on ARM they can move past the second.)
 */
const SLOT_HEADER_BYTES = 16;
/** Bytes before the slots: the Int32 state, the waiting slot (0..2) plus `FRESH` while it's newer than what the reader has. */
const SHARED_HEADER_BYTES = 16;
const FRESH = 4;
const SLOTS = 3;

function slotBytes(capacity: number): number {
  return SLOT_HEADER_BYTES + capacity * 4;
}

export function createSharedSnapshot(layout: SnapshotLayout): SharedArrayBuffer {
  return new SharedArrayBuffer(SHARED_HEADER_BYTES + SLOTS * slotBytes(layout.capacity));
}

/** A slot's step and floats. */
function slots(buffer: SharedArrayBuffer, capacity: number): { step: Int32Array; data: Float32Array }[] {
  return Array.from({ length: SLOTS }, (_, i) => {
    const at = SHARED_HEADER_BYTES + i * slotBytes(capacity);
    return { step: new Int32Array(buffer, at, 1), data: new Float32Array(buffer, at + SLOT_HEADER_BYTES, capacity) };
  });
}

/**
 * The writer's side. The buffer starts zeroed: slot 0 waiting (not fresh), the reader holding
 * slot 1 and the writer slot 2.
 */
export class SharedSnapshotWriter {
  private readonly state: Int32Array;
  private readonly slots: { step: Int32Array; data: Float32Array }[];
  private own = 2;

  constructor(buffer: SharedArrayBuffer, capacity: number) {
    this.state = new Int32Array(buffer, 0, 1);
    this.slots = slots(buffer, capacity);
  }

  /** `step`: the sim step it shows (snapshots published between steps, after a command, repeat it). */
  write(src: Float32Array, step: number): void {
    const slot = this.slots[this.own];
    slot.data.set(src);
    slot.step[0] = step;
    // (The exchange publishes the writes before it; the slot that was waiting is the writer's now.)
    this.own = Atomics.exchange(this.state, 0, this.own | FRESH) & ~FRESH;
  }
}

export class SharedSnapshotReader {
  private readonly state: Int32Array;
  private readonly slots: { step: Int32Array; data: Float32Array }[];
  private own = 1;
  /** The sim step of the snapshot last read. */
  step = 0;

  constructor(buffer: SharedArrayBuffer, capacity: number) {
    this.state = new Int32Array(buffer, 0, 1);
    this.slots = slots(buffer, capacity);
  }

  /** Copies the newest snapshot into `into`. Returns false if nothing new has been written since the last read. */
  read(into: Float32Array): boolean {
    if (!(Atomics.load(this.state, 0) & FRESH)) return false;
    // (The exchange sees everything written before the writer's: the slot is whole, and the writer's no more.)
    this.own = Atomics.exchange(this.state, 0, this.own) & ~FRESH;
    const slot = this.slots[this.own];
    into.set(slot.data);
    this.step = slot.step[0];
    return true;
  }
}
