/**
 * Makes the sounds that take a moment off the main thread: the water sounds (`water.ts`) and the
 * things' (`things.ts`), asked for by kind, and the birds' songs (`birdsong.ts`), asked for by bird
 * and version, each at a sample rate. Sends back the samples (two channels for a sound, one for a
 * song).
 */

import { songSamples, type Voice } from './birdsong';
import { MADE, type MadeKind } from './things';

export type SoundRequest = { kind: MadeKind; rate: number } | { bird: Voice; version: number; rate: number };

export type SoundReply = { kind: MadeKind; rate: number; left: Float32Array; right: Float32Array } | { bird: Voice; version: number; rate: number; samples: Float32Array };

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
  const { left, right } = MADE[q.kind](q.rate);
  scope.postMessage({ kind: q.kind, rate: q.rate, left, right }, [left.buffer, right.buffer]);
};
