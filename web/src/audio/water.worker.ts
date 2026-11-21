/**
 * Makes the sounds that take a moment off the main thread (`synth.ts`, in Rust): the water's, the things', the
 * outdoors' and the music, asked for by kind, and the birds' songs, asked for by bird and version, each at a sample
 * rate. Sends back the samples (two channels for a sound, or one for a sound of one channel; one for a song).
 */

import type { Voice, WorkerKind } from './kinds';
import { loadSynth, makeSound, songSamples } from './synth';

export type SoundRequest = { kind: WorkerKind; rate: number } | { bird: Voice; version: number; rate: number };

export type SoundReply = { kind: WorkerKind; rate: number; left: Float32Array; right?: Float32Array } | { bird: Voice; version: number; rate: number; samples: Float32Array };

interface WorkerScope {
  onmessage: ((e: MessageEvent<SoundRequest>) => void) | null;
  postMessage(message: SoundReply, transfer?: Transferable[]): void;
}

const scope = self as WorkerScope;
/** Asks wait for the synthesis to load, in the order they came. A load that fails throws in here, and the page's
 * `onerror` makes the sounds there instead. */
let ready = loadSynth();

scope.onmessage = (e) => {
  const q = e.data;
  ready = ready.then(() => {
    if ('bird' in q) {
      const samples = songSamples(q.bird, q.version, q.rate);
      scope.postMessage({ ...q, samples }, [samples.buffer]);
      return;
    }
    const made = makeSound(q.kind, q.rate);
    if (made instanceof Float32Array) scope.postMessage({ kind: q.kind, rate: q.rate, left: made }, [made.buffer]);
    else scope.postMessage({ kind: q.kind, rate: q.rate, left: made.left, right: made.right }, [made.left.buffer, made.right.buffer]);
  });
};
