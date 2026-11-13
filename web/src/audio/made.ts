/**
 * The sounds that take a moment to make and are made by kind, off the main thread where there is a worker
 * (`water.worker.ts`; `world.ts` `WorkerSounds`): the water's and the things' (`things.ts` `MADE`) and the
 * outdoors' (`outdoors.ts`). No DOM or audio context here.
 */

import { OUTDOORS, type OutdoorKind } from './outdoors';
import { MADE, type MadeKind } from './things';
import type { Stereo } from './water';

export type WorkerKind = MadeKind | OutdoorKind;

/** The sound of a kind at a sample rate: two channels, or the samples of one. */
export function makeSound(kind: WorkerKind, rate: number): Stereo | Float32Array {
  return kind in OUTDOORS ? OUTDOORS[kind as OutdoorKind](rate) : MADE[kind as MadeKind](rate);
}
