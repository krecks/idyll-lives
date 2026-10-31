/**
 * A machine running on its own (sim-core `machines.rs`, the placement's `running`): its `motions.running`
 * clip goes round with nobody at it, and finishes its round once it stops (`ObjectMotion`).
 */

import { describe, expect, test } from 'bun:test';
import type { FrameState } from '../src/core/bridge';
import type { ObjectPlacement } from '../src/core/protocol';
import type { ModelClip } from '../src/render/shared/modelClips';
import { ObjectMotion, restPoses, type MovingObject } from '../src/render/shared/objectMotion';

/** A clip of one part, `duration` seconds, its poses all at rest (only its time matters here). */
function clip(duration: number): ModelClip {
  const samples = Math.round(duration * 30) + 1;
  const poses = new Float32Array(samples * 16);
  for (let k = 0; k < samples; k++) for (let i = 0; i < 4; i++) poses[k * 16 + i * 5] = 1;
  return { duration, tracks: [{ part: 0, poses, depth: 0 }] };
}

/** A frame at `now` ms, game speed 1, no residents. */
function frame(now: number): FrameState {
  const layout = { header: { minute: 0, speed: 1, simCount: 2 }, headerLen: 3, simStride: 0, sim: {} } as unknown as FrameState['layout'];
  const curr = new Float32Array([600, 1, 0]);
  return { prev: curr, curr, alpha: 1, steps: 1, layout, now };
}

function washer(running: boolean): MovingObject<number> {
  return {
    id: 7,
    target: { matrices: new Float32Array(16), index: 0, moving: { parts: [{ meshes: [1], matrices: new Float32Array(16) }], poses: restPoses(1) } },
    motion: { parts: [0], clips: new Map([['drum_spin', clip(1.2)], ['door_open', clip(1)]]) },
    motions: { uses: { laundry: [{ clip: 'door_open' }] }, loops: {}, running: 'drum_spin' },
    interactions: ['laundry', 'unload'],
    running,
  };
}

const placement = (running: boolean) => ({ id: 7, def: 'laundryWasher', x: 0, z: 0, rot: 0, w: 1, d: 1, quality: 0, sellValue: 0, running }) as ObjectPlacement;

describe('a machine running on its own', () => {
  test('turns its drum with nobody at it, and stops at rest once done', () => {
    const motion = new ObjectMotion<number>({ writeMoving: () => {}, upload: () => {} });
    const eye = { x: 0, y: 0, z: 0 };
    motion.set([washer(false)]);
    motion.update(frame(1000), [], eye);
    expect(motion.clipsOf(7)).toBeNull();

    // Loaded: it runs, round after round.
    motion.patch([placement(true)]);
    const times: number[] = [];
    for (let t = 1050; t <= 3000; t += 50) {
      motion.update(frame(t), [], eye);
      times.push(motion.clipsOf(7)?.get('drum_spin')?.time ?? -1);
    }
    expect(motion.clipsOf(7)?.get('drum_spin')?.kind).toBe('loop');
    expect(times.every((t) => t >= 0 && t < 1.2)).toBe(true);
    expect(times.some((t, k) => k > 0 && t < times[k - 1])).toBe(true); // came round again

    // Done: the drum finishes its round and rests; the door never moved.
    motion.patch([placement(false)]);
    for (let t = 3050; t <= 5000; t += 50) motion.update(frame(t), [], eye);
    expect(motion.clipsOf(7)?.get('drum_spin')).toBeUndefined();
    expect(motion.clipsOf(7)?.get('door_open')).toBeUndefined();
  });

  test('a world built while it runs shows it running', () => {
    const motion = new ObjectMotion<number>({ writeMoving: () => {}, upload: () => {} });
    motion.set([washer(true)]);
    motion.update(frame(1000), [], { x: 0, y: 0, z: 0 });
    motion.update(frame(1100), [], { x: 0, y: 0, z: 0 });
    expect(motion.clipsOf(7)?.get('drum_spin')?.kind).toBe('loop');
  });
});
