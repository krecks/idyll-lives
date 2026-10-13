/**
 * Debug reports: a screenshot, the full game state (a regular save) and the context needed to
 * reproduce a problem. With the dev server they are written to `debug-reports/<name>/` in the
 * project (see the `debugReports` plugin in vite.config.ts); otherwise they download as one file.
 * `?debugReport=<name>` (dev server only) loads a report's game state with its camera.
 */

import type { CameraPose } from '../render/types';
import { writeSave, type SaveRecord } from '../persistence/saves';
import { recentLog, type LogEntry } from './log';

export interface DebugReport {
  /** What the report is about, typed by the player (may be empty). */
  note: string;
  /** Game version the report was taken with. */
  version: string;
  createdAt: string;
  url: string;
  userAgent: string;
  screen: { width: number; height: number; pixelRatio: number };
  gpu: Record<string, string> | null;
  renderer: unknown;
  /** Main, sim and voice threads (see `debug/threads.ts`). */
  threads: unknown;
  settings: unknown;
  /** The game camera (null: no game was running). */
  camera: CameraPose | null;
  /** UI state (mode, selection, panels, needs…) at capture time. */
  ui: unknown;
  log: LogEntry[];
  /** Save metadata; the save data itself travels as `save`. */
  saveMeta: Omit<SaveRecord, 'data' | 'thumbnail'>;
}

export interface DebugPayload {
  report: DebugReport;
  /** The simulation's save data (null: there was no game, or it had stopped). */
  save: string | null;
  /** JPEG data URL of the 3D view, or null if capture failed. */
  screenshot: string | null;
}

/** `promise`, or `fallback` if it takes longer than `ms`. */
export function within<T, F>(promise: Promise<T>, ms: number, fallback: F): Promise<T | F> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const late = new Promise<F>((resolve) => (timer = setTimeout(() => resolve(fallback), ms)));
  return Promise.race([promise, late]).finally(() => clearTimeout(timer));
}

/**
 * A report without a game to ask (the crash screen when no game runs, or game data that didn't
 * load): the log, the settings, the interface state and a picture if one could be taken.
 */
export async function basicReport(note: string, parts: { settings: unknown; ui: unknown; renderer: unknown; screenshot: string | null }): Promise<string> {
  const report: DebugReport = {
    note,
    version: __APP_VERSION__,
    createdAt: new Date().toISOString(),
    url: location.href,
    userAgent: navigator.userAgent,
    screen: { width: window.innerWidth, height: window.innerHeight, pixelRatio: window.devicePixelRatio || 1 },
    gpu: await within(gpuInfo(), 4000, null),
    renderer: parts.renderer,
    threads: null,
    settings: parts.settings,
    camera: null,
    ui: parts.ui,
    log: recentLog(),
    saveMeta: { id: 'debug', name: 'Debug report', household: '', members: [], day: 0, minute: 0, savedAt: Date.now(), autosave: false },
  };
  return deliverReport({ report, save: null, screenshot: parts.screenshot });
}

/** Adapter details for the report (WebGPU only; empty when unavailable). */
export async function gpuInfo(): Promise<Record<string, string> | null> {
  try {
    const gpu = (navigator as unknown as { gpu?: { requestAdapter(): Promise<{ info?: Record<string, string> } | null> } }).gpu;
    const adapter = await gpu?.requestAdapter();
    const info = adapter?.info;
    if (!info) return null;
    return { vendor: info.vendor, architecture: info.architecture, device: info.device, description: info.description };
  } catch {
    return null;
  }
}

/** Stores the report: in the project folder via the dev server, else as a download. Returns where it went. */
export async function deliverReport(payload: DebugPayload): Promise<string> {
  if (import.meta.env.DEV) {
    const res = await fetch('/__debug/report', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`dev server answered ${res.status}`);
    const { path } = (await res.json()) as { path: string };
    return path;
  }
  const stamp = payload.report.createdAt.replace(/[:.]/g, '-');
  const name = `idyll-lives-debug-${stamp}.json`;
  const url = URL.createObjectURL(new Blob([JSON.stringify(payload)], { type: 'application/json' }));
  const a = document.createElement('a');
  a.href = url;
  a.download = name;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 10_000);
  return `Downloads/${name}`;
}

/** Camera pose to restore once a debug report's game is shown. */
let pendingPose: CameraPose | null = null;

export function takePendingPose(): CameraPose | null {
  const pose = pendingPose;
  pendingPose = null;
  return pose;
}

/**
 * Loads report `name` from the dev server into a save slot and returns its id.
 * The camera pose is applied when the game is revealed (`takePendingPose`).
 */
export async function loadDebugReport(name: string): Promise<string> {
  const base = `/__debug/reports/${encodeURIComponent(name)}`;
  const [reportRes, saveRes] = await Promise.all([fetch(`${base}/report.json`), fetch(`${base}/save.txt`)]);
  if (!reportRes.ok || !saveRes.ok) throw new Error(`Debug report "${name}" not found`);
  const report = (await reportRes.json()) as DebugReport;
  const record: SaveRecord = { ...report.saveMeta, id: `debug-${name}`, name: `Debug: ${name}`, autosave: false, thumbnail: null, data: await saveRes.text() };
  await writeSave(record);
  pendingPose = report.camera;
  return record.id;
}

/** Game minutes per simulation tick (sim-core `MINUTES_PER_TICK`). */
const MINUTES_PER_TICK = 1 / 20;

/**
 * Puts a pasted game state into a save slot and returns its id: the simulation's own save
 * (`World::save_json`, what the tests write with `DUMP_STATES=1`) or a whole debug report.
 */
export async function loadState(text: string, name = 'Pasted state'): Promise<string> {
  let parsed: { tick?: number; households?: { name: string; player: boolean }[]; sims?: { name: string; household: number }[]; save?: string };
  try {
    parsed = JSON.parse(text);
  } catch {
    throw new Error("That isn't a game state (it isn't JSON).");
  }
  // A debug report carries the state as `save`.
  const data = typeof parsed.save === 'string' ? parsed.save : text;
  const state = data === text ? parsed : JSON.parse(data);
  if (typeof state.tick !== 'number' || !Array.isArray(state.households) || !Array.isArray(state.sims)) {
    throw new Error("That isn't a game state (no tick, households and residents).");
  }
  const player = state.households.findIndex((h: { player: boolean }) => h.player);
  const minutes = Math.floor(state.tick * MINUTES_PER_TICK);
  const record: SaveRecord = {
    id: 'debug-state',
    name: `Debug: ${name}`,
    household: state.households[player]?.name ?? '',
    members: state.sims.filter((s: { household: number }) => s.household === player).map((s: { name: string }) => s.name),
    day: Math.floor(minutes / 1440) + 1,
    minute: minutes % 1440,
    savedAt: Date.now(),
    thumbnail: null,
    autosave: false,
    data,
  };
  await writeSave(record);
  return record.id;
}

/** `?state=<name>` (dev server): a state the simulation's tests wrote to `debug-states/`. */
export async function fetchState(name: string): Promise<string> {
  const res = await fetch(`/__debug/states/${encodeURIComponent(name)}`);
  if (!res.ok) throw new Error(`Game state "${name}" not found (run the tests with DUMP_STATES=1)`);
  return loadState(await res.text(), name);
}
