// When the voices and the language model may work (`ml/schedule.svelte.ts`): one job at a time on the
// graphics card across both, in the order asked; jobs on the processor beside it; nothing while the
// game holds them off (and a forgotten hold ends by itself); models start only once allowed and
// nothing holds them; a job whose signal aborts while it waits gives up its place. And the
// processor's threads (`ml/ort.ts`): two at most, one on four cores.
import { describe, expect, test } from 'bun:test';
import { cpuThreads } from '../src/ml/ort';
import { heldSince, holdInference, inferenceHeld, inferenceSlot, startModels, whenModelsMayStart } from '../src/ml/schedule.svelte';

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

describe('inference schedule', () => {
  test('one job at a time on the graphics card, in order; the processor beside it', async () => {
    const order: string[] = [];
    const a = inferenceSlot('gpu').then((free) => (order.push('a'), free!));
    const b = inferenceSlot('gpu').then((free) => (order.push('b'), free!));
    const c = inferenceSlot('cpu').then((free) => (order.push('c'), free!));
    await tick();
    expect(order).toEqual(['a', 'c']);
    (await a)();
    await tick();
    expect(order).toEqual(['a', 'c', 'b']);
    (await b)();
    (await c)();
  });

  test('a hold keeps every job waiting until it is let go', async () => {
    const release = holdInference('test');
    expect(inferenceHeld()).toBe(true);
    let got = false;
    const job = inferenceSlot('cpu').then((free) => ((got = true), free!));
    await tick();
    expect(got).toBe(false);
    const since = performance.now();
    release();
    // (Again does nothing.)
    release();
    await tick();
    expect(got).toBe(true);
    expect(inferenceHeld()).toBe(false);
    expect(heldSince(since)).toBe(true);
    (await job)();
  });

  test('a forgotten hold ends by itself', async () => {
    holdInference('forgotten', 5);
    expect(inferenceHeld()).toBe(true);
    await new Promise((resolve) => setTimeout(resolve, 20));
    expect(inferenceHeld()).toBe(false);
  });

  test('a job aborted while it waits gives up its place', async () => {
    const first = (await inferenceSlot('gpu'))!;
    const control = new AbortController();
    const aborted = inferenceSlot('gpu', control.signal);
    let next = false;
    const after = inferenceSlot('gpu').then((free) => ((next = true), free!));
    control.abort();
    expect(await aborted).toBeNull();
    first();
    await tick();
    expect(next).toBe(true);
    (await after)();
  });

  test('models start once allowed, and not while held', async () => {
    let may = false;
    void whenModelsMayStart().then(() => (may = true));
    await tick();
    expect(may).toBe(false);
    const release = holdInference('loading');
    startModels();
    await tick();
    expect(may).toBe(false);
    release();
    await tick();
    expect(may).toBe(true);
  });

  test("the processor's threads leave the game three cores", () => {
    expect(cpuThreads(2)).toBe(1);
    expect(cpuThreads(4)).toBe(1);
    expect(cpuThreads(5)).toBe(2);
    expect(cpuThreads(16)).toBe(2);
  });
});
