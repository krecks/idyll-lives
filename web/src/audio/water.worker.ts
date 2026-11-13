/**
 * Makes the sounds that take a moment off the main thread: the water sounds (`water.ts`), the
 * things' (`things.ts`) and the outdoors' (`outdoors.ts`), asked for by kind (`made.ts`), and the birds' songs
 * (`birdsong.ts`), asked for by bird and version, each at a sample rate. Sends back the samples (two
 * channels for a sound, or one for a sound of one channel; one for a song).
 */

import { songSamples, type Voice } from './birdsong';
import { makeSound, type WorkerKind } from './made';

export type SoundRequest = { kind: WorkerKind; rate: number } | { bird: Voice; version: number; rate: number };

export type SoundReply = { kind: WorkerKind; rate: number; left: Float32Array; right?: Float32Array } | { bird: Voice; version: number; rate: number; samples: Float32Array };

interface WorkerScope {
  onmessage: ((e: MessageEvent<SoundRequest>) => void) | null;
  postMessage(message: SoundReply, transfer?: Transferable[]): void;
}

const scope = self as WorkerScope;

scope.onmessage = (e) => {
  const q = e.data;
  if ('bird' in q) {
    const samples = songSamples(q.bird, q.version, q.rate);
    scope.postMessage({ ...q, samples }, [samples.buffer]);
    return;
  }
  const made = makeSound(q.kind, q.rate);
  if (made instanceof Float32Array) scope.postMessage({ kind: q.kind, rate: q.rate, left: made }, [made.buffer]);
  else scope.postMessage({ kind: q.kind, rate: q.rate, left: made.left, right: made.right }, [made.left.buffer, made.right.buffer]);
};
