/**
 * Making a world and its town file off the main thread (`town.worker.ts`). Both are seeded and
 * pure, but a big world takes seconds (its layout, then a house for every lot) and assembling
 * its file most of a second: on the page that froze the screen. `runTownJob` is what the worker
 * runs (and the page itself, where workers can't start), so the two give the same result.
 */

import type { Content } from '../content/content';
import type { WorldStructure, WorldSize } from '../core/protocol';
import type { HouseholdDraft } from './household';
import { assembleTown, generatedHouses, generateWorld, loadTemplates, primeHouses, type AssembleOptions, type HouseTemplate, type NeighbourhoodDraft, type Templates } from './town';
import { previewWorld } from './townPreview';
import type { WorldKind } from './world';

export type TownJob =
  /** A new world (`generateWorld`), with the menu's preview of it (`previewWorld`). */
  | { kind: 'world'; world: WorldKind; seed: number; name?: string; size: WorldSize }
  /** The simulation's town file (`assembleTown`); `random`: the seed for its unseeded dice (neighbours' bonds). */
  | { kind: 'assemble'; town: NeighbourhoodDraft; player: HouseholdDraft; slot: number; options: AssembleOptions; random: number };

export type TownResult =
  | { kind: 'world'; draft: NeighbourhoodDraft; preview: WorldStructure; houses: [string, HouseTemplate | undefined][] }
  | { kind: 'assemble'; lot: string };

export type ToTownWorker = { id: number; job: TownJob };
/** `broken`: the worker couldn't load the content (every job fails there). */
export type FromTownWorker = { id: number; result: TownResult } | { id: number; error: string; broken?: boolean };

/** A small seeded generator (mulberry32) standing in for `Math.random` while a file is assembled. */
function seeded(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/**
 * Runs a job. `assembleTown` rolls some dice with `Math.random` (who are friends, rivals): they
 * come from the job's own seed, so the worker and the page give the same file for the same job.
 */
export function runTownJob(content: Content, t: Templates, job: TownJob): TownResult {
  if (job.kind === 'world') {
    const draft = generateWorld(content, t, job.world, job.seed, job.name, job.size);
    const preview = previewWorld(content, t, draft, null);
    return { kind: 'world', draft, preview, houses: generatedHouses(t, draft.slots.map((s) => s.template)) };
  }
  const real = Math.random;
  Math.random = seeded(job.random);
  try {
    return { kind: 'assemble', lot: assembleTown(content, t, job.town, job.player, job.slot, job.options) };
  } finally {
    Math.random = real;
  }
}

/** Raised for a job a newer one replaced (or whose signal aborted it). */
export class Superseded extends Error {
  constructor() {
    super('superseded');
    this.name = 'Superseded';
  }
}

/** The worker goes once it has had nothing to do for this long (its content and houses are freed). */
const IDLE_MS = 60_000;

interface Pending {
  job: TownJob;
  content: Content;
  resolve: (r: TownResult) => void;
  reject: (e: unknown) => void;
}

let worker: Worker | null = null;
/** Workers can't start here (or one failed): jobs run on the page. */
let inPage = false;
let nextId = 1;
const pending = new Map<number, Pending>();
let idleTimer: ReturnType<typeof setTimeout> | undefined;

function stopWorker(): void {
  worker?.terminate();
  worker = null;
  clearTimeout(idleTimer);
}

/** Hands `p` to the worker (started if need be), or runs it on the page when there can be none. */
function dispatch(id: number, p: Pending): void {
  const w = worker ?? (inPage ? null : startWorker());
  if (!w) {
    pending.delete(id);
    onPage(p.content, p.job).then(p.resolve, p.reject);
    return;
  }
  clearTimeout(idleTimer);
  w.postMessage({ id, job: p.job } satisfies ToTownWorker);
}

function startWorker(): Worker | null {
  try {
    worker = new Worker(new URL('./town.worker.ts', import.meta.url), { type: 'module' });
  } catch {
    inPage = true;
    return null;
  }
  worker.onmessage = (e: MessageEvent<FromTownWorker>) => {
    const msg = e.data;
    if ('broken' in msg && msg.broken) return broke(msg.error);
    const p = pending.get(msg.id);
    pending.delete(msg.id);
    if (p) {
      if ('error' in msg) p.reject(new Error(msg.error));
      else p.resolve(msg.result);
    }
    if (!pending.size) idle();
  };
  worker.onerror = (e) => {
    e.preventDefault();
    broke(e.message);
  };
  return worker;
}

/** A worker that can't load (or broke) hands its jobs to the page, and so do later ones. */
function broke(why: string): void {
  console.warn('[town] the town worker failed; making towns on the page:', why);
  inPage = true;
  stopWorker();
  for (const [id, p] of [...pending]) dispatch(id, p);
}

function idle(): void {
  clearTimeout(idleTimer);
  idleTimer = setTimeout(() => {
    if (!pending.size) stopWorker();
  }, IDLE_MS);
}

async function onPage(content: Content, job: TownJob): Promise<TownResult> {
  return runTownJob(content, await loadTemplates(), job);
}

/**
 * Runs `job` in the town worker (on the page if no worker can run; `content` is for that). An
 * aborted `signal` drops the job: one already running stops with its worker, and the jobs waiting
 * behind it go to a fresh one, so a player rolling world after world waits only for the last.
 */
export function townJob<J extends TownJob>(content: Content, job: J, signal?: AbortSignal): Promise<Extract<TownResult, { kind: J['kind'] }>>;
export function townJob(content: Content, job: TownJob, signal?: AbortSignal): Promise<TownResult> {
  if (signal?.aborted) return Promise.reject(new Superseded());
  return new Promise<TownResult>((resolve, reject) => {
    const id = nextId++;
    const p: Pending = { job, content, resolve, reject };
    pending.set(id, p);
    signal?.addEventListener(
      'abort',
      () => {
        // (The worker takes jobs in turn: the first waiting is the one it is on.)
        const running = pending.keys().next().value === id;
        if (!pending.delete(id)) return;
        reject(new Superseded());
        // Stopping the worker is the only way to stop the job it is on (a fresh one loads its
        // content again). One still waiting is made and thrown away.
        if (!running || !worker) return;
        stopWorker();
        for (const [other, q] of [...pending]) dispatch(other, q);
      },
      { once: true },
    );
    dispatch(id, p);
  });
}

/** A world's houses the worker made, kept on the page too (the menu and the screens after need them). */
export function keepHouses(result: Extract<TownResult, { kind: 'world' }>): void {
  primeHouses(result.houses);
}
