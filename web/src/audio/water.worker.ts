/** Makes the water sounds (`water.ts`) off the main thread: asked for a kind at a sample rate, sends back its two channels. */

import { WATER, type WaterKind } from './water';

export interface WaterRequest {
  kind: WaterKind;
  rate: number;
}

export interface WaterReply extends WaterRequest {
  left: Float32Array;
  right: Float32Array;
}

interface WorkerScope {
  onmessage: ((e: MessageEvent<WaterRequest>) => void) | null;
  postMessage(message: WaterReply, transfer?: Transferable[]): void;
}

const scope = self as unknown as WorkerScope;

scope.onmessage = (e) => {
  const { kind, rate } = e.data;
  const { left, right } = WATER[kind](rate);
  scope.postMessage({ kind, rate, left, right }, [left.buffer, right.buffer]);
};
