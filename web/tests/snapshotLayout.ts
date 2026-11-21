/**
 * The snapshot's layout as sim-core writes it (`snapshot.rs` `layout_json`), worked out from its
 * source for the tests that read car and light rows (no wasm build needed): the header's slots,
 * where the cars' and the lights' rows start and how long each is, and their fields. Where a base
 * is worked out here, the test checks snapshot.rs still says so (`BASES`), so a layout that moves
 * fails here rather than reading the wrong floats.
 */

import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import type { SnapshotLayout } from '../src/core/snapshot';

const SIM = join(import.meta.dir, '../../crates/sim-core/src');
const source = (file: string) => readFileSync(join(SIM, file), 'utf8');
export const SNAPSHOT_RS = source('snapshot.rs');

/** Every `.rs` file under sim-core's `src/` (its path there, `/`-separated), read once. */
const SOURCES = (readdirSync(SIM, { recursive: true }) as string[])
  .filter((f) => f.endsWith('.rs'))
  .map((f) => ({ file: f.split('\\').join('/'), src: source(f) }));

/**
 * The value of sim-core's constant `name` as its source writes it (the text between `=` and `;`), wherever it is
 * declared under `src/`, so a test names only the constant, not its file. A const of that name of any visibility
 * counts; where more than one module declares one, the test says which with `file` (its path under `src/`). None, or
 * more than one, fails, so a const of the same name added elsewhere can't quietly become the one a test reads.
 */
export function rustConst(name: string, file?: string): string {
  const found = SOURCES.filter((s) => !file || s.file === file).flatMap((s) =>
    [...s.src.matchAll(new RegExp(`^\\s*(?:pub(?:\\([^)]*\\))? )?const ${name}: [^=]+= ([^;]+);`, 'gm'))].map((m) => m[1].trim()),
  );
  if (found.length !== 1) throw new Error(`${found.length ? 'more than one' : 'no'} const ${name} in sim-core${file ? `'s ${file}` : ''}`);
  return found[0];
}

/** sim-core's constant `name` (`rustConst`, in `file` where given; in snapshot.rs's `pub mod module`, when given), as a number. */
export function constant(name: string, module?: string, file?: string): number {
  let value: string;
  if (module) {
    const from = SNAPSHOT_RS.indexOf(`pub mod ${module} {`);
    if (from < 0) throw new Error(`no module ${module}`);
    const m = new RegExp(`pub const ${name}: \\w+ = ([0-9._]+);`).exec(SNAPSHOT_RS.slice(from));
    if (!m) throw new Error(`no ${module}::${name}`);
    value = m[1];
  } else value = rustConst(name, file);
  const n = Number(value.replace(/_/g, ''));
  if (Number.isNaN(n)) throw new Error(`const ${name} is not a number: ${value}`);
  return n;
}

/**
 * The sim's rates (sim-core `lib.rs`): ticks a real second at speed 1, and ticks a game minute. Tests that play
 * snapshots out in time (a snapshot every 1 / `TICKS_PER_SECOND` s) and the layouts they write them in keep to these,
 * so they go on holding whatever sim-core makes them.
 */
export const TICKS_PER_SECOND = constant('TICKS_PER_SECOND');
export const TICKS_PER_MINUTE = constant('TICKS_PER_MINUTE');
export const DRIVE_SCALE = constant('DRIVE_SCALE');
/** A layout's `ticksPerSecond`, `ticksPerMinute` and `driveScale`, as the sim's. */
export const TICK_RATES = { ticksPerSecond: TICKS_PER_SECOND, ticksPerMinute: TICKS_PER_MINUTE, driveScale: DRIVE_SCALE };

/** The bases as snapshot.rs works them out (checked against its source by the tests). */
export const BASES = {
  car: 'pub const CAR_BASE: usize = HEADER_LEN + (MAX_SIMS + crate::staff::MAX_STAFF) * SIM_STRIDE;',
  light: 'pub const LIGHT_BASE: usize = CAR_BASE + crate::cars::MAX_CARS * CAR_STRIDE;',
  capacity: 'pub const CAPACITY: usize = LIGHT_BASE + MAX_LIGHTS * LIGHT_STRIDE;',
};

const HEADER_LEN = constant('HEADER_LEN');
const SIM_STRIDE = constant('SIM_STRIDE');
const MAX_SIMS = constant('MAX_SIMS');
const MAX_STAFF = constant('MAX_STAFF');
const MAX_CARS = constant('MAX_CARS');
const CAR_STRIDE = constant('CAR_STRIDE');
const LIGHT_STRIDE = constant('LIGHT_STRIDE');
const MAX_LIGHTS = constant('MAX_LIGHTS');
const CAR_BASE = HEADER_LEN + (MAX_SIMS + MAX_STAFF) * SIM_STRIDE;
const LIGHT_BASE = CAR_BASE + MAX_CARS * CAR_STRIDE;

/** The layout's car and light parts (and the header's counts), as `layout_json` has them. */
export const LAYOUT = {
  capacity: LIGHT_BASE + MAX_LIGHTS * LIGHT_STRIDE,
  headerLen: HEADER_LEN,
  simStride: SIM_STRIDE,
  maxSims: MAX_SIMS,
  maxStaff: MAX_STAFF,
  ...TICK_RATES,
  header: {
    tick: constant('TICK', 'header'),
    day: constant('DAY', 'header'),
    minute: constant('MINUTE', 'header'),
    speed: constant('SPEED', 'header'),
    simCount: constant('SIM_COUNT', 'header'),
    structureVersion: constant('STRUCTURE_VERSION', 'header'),
    carCount: constant('CAR_COUNT', 'header'),
    lightCount: constant('LIGHT_COUNT', 'header'),
  },
  carBase: CAR_BASE,
  carStride: CAR_STRIDE,
  maxCars: MAX_CARS,
  car: {
    shown: constant('SHOWN', 'car'),
    x: constant('X', 'car'),
    z: constant('Z', 'car'),
    yaw: constant('YAW', 'car'),
    doors: constant('DOORS', 'car'),
    kind: constant('KIND', 'car'),
    kinds: { household: constant('HOUSEHOLD', 'car'), taxi: constant('TAXI', 'car') },
  },
  lightBase: LIGHT_BASE,
  lightStride: LIGHT_STRIDE,
  maxLights: MAX_LIGHTS,
  light: {
    shown: constant('SHOWN', 'light'),
    x: constant('X', 'light'),
    z: constant('Z', 'light'),
    yaw: constant('YAW', 'light'),
    state: constant('STATE', 'light'),
    states: { red: constant('RED', 'light'), amber: constant('AMBER', 'light'), green: constant('GREEN', 'light'), walk: constant('WALK', 'light') },
  },
  sim: {},
  actions: [],
  items: [],
};

export const layout = LAYOUT as unknown as SnapshotLayout;
