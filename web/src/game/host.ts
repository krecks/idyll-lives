/**
 * The game host owns everything that should outlive a single session — the canvas, the
 * renderer (engine, compiled shaders, cached meshes, landscape) — and prepares sessions
 * in the background, so "Continue" and "Move in" don't stall the page.
 *
 * Behind the menus it shows a town overview on the same renderer (`showTown`); a session can
 * be prepared meanwhile (its world builds hidden behind the overview). Starting a game then
 * flies the camera into the lot (same town) or fades through a veil (a saved game).
 */

import type { WorldStructure } from '../core/protocol';
import { BabylonRenderer } from '../render/babylon/BabylonRenderer';
import type { Renderer, RenderFailure } from '../render/types';
import { qualityFromSettings, RESTART_KEYS, settings as liveSettings, type Settings } from '../settings/settings.svelte';
import { app, type Progress } from '../ui/app.svelte';
import { perfTime, perfTimeAsync } from '../debug/perf';
import { game, toast } from '../ui/state.svelte';
import { basicReport, within } from '../debug/report';
import { plainState } from '../debug/snapshot.svelte';
import { webGpuFailed } from '../render/babylon/loaders';
import type { HouseholdDraft } from './household';
import { loadGameData, startSession, type GameSession, type StartRequest } from './session';

function requestKey(r: StartRequest): string {
  if (r.kind === 'load') return `load:${r.saveId}`;
  const members = (h: HouseholdDraft) => h.members.map((m) => m.uid).join(',');
  // The whole town counts: going back and changing the neighbours needs a new session.
  const town = `${r.town.name}:${r.town.world?.kind ?? ''}:${r.town.seed}:${r.town.slots.map((s) => `${s.template}@${s.lot ?? 'medium'}`).join(',')}:${r.town.households.map((h) => `${h.slot}=${members(h.household)}`).join(';')}`;
  // A household edited after its session was prepared (back to step 2) needs a new one too.
  return `${r.existing ? 'play' : 'new'}:${r.mode ?? 'living'}:${r.lifespan ?? 'normal'}:${town}:${r.slot}:${JSON.stringify(r.household)}`;
}

/** Seconds the camera takes to fly from the menu's view of a lot into the game. */
const FLY_IN = 1.3;
const VEIL_MS = 260;

interface Entry {
  key: string;
  /** Resolves once the session exists (before its scene is built). */
  started: Promise<GameSession>;
  /** Resolves once the session's scene is built and drawn. */
  session: Promise<GameSession>;
  /** Being started (Play, Continue): its loading shows. Prepared behind the menus, it loads unseen. */
  live: boolean;
  /** The session's scene is built and drawn (its progress reports end). */
  ready: boolean;
  /** Where its loading is (shown as the loading screen once it is live). */
  progress: Progress;
  /** The session's own look (and the mirror's clip-plane shaders) has been drawn once. */
  warmed: boolean;
  /** The session, once it exists. */
  instance?: GameSession;
}

class GameHost {
  /** The canvas the current renderer draws on. It is replaced when the graphics API changes. */
  private canvas: HTMLCanvasElement;
  readonly overlay: HTMLDivElement;
  private readonly root: HTMLDivElement;
  /** Fades the canvas out and in when the scene changes completely (menu town <-> a saved game). */
  private readonly veil: HTMLDivElement;
  private renderer: Promise<Renderer> | null = null;
  private rendererKey = '';
  /** The graphics API the current renderer was made with (a change needs a new canvas). */
  private rendererBackend: Settings['renderer'] | null = null;
  private current: Entry | null = null;
  /** The previous session letting go of the renderer; the next one starts after it. */
  private teardown: Promise<void> = Promise.resolve();
  /** The town the menus show (kept to show it again on a new renderer); null while in a game. */
  private town: { world: WorldStructure; minute: number } | null = null;
  /** The game being played (once it has started), for the crash screen's "Save and reload". */
  private played: GameSession | null = null;
  /** The player heard that WebGPU failed and WebGL2 draws instead. */
  private toldFallback = false;
  /** One failure handler for every renderer: it must not close over the renderer it replaces. */
  private readonly onRendererFailure = (why: RenderFailure, err?: unknown) => {
    app.crashed(why === 'device' ? { kind: 'graphics', detail: 'The graphics device was lost.' } : { kind: 'render', detail: err instanceof Error ? err.message : String(err) });
  };
  /** A full-screen menu covers the view (see `cover`). */
  private covered = false;

  constructor() {
    this.root = document.createElement('div');
    this.root.className = 'game-host';
    this.canvas = sceneCanvas();
    this.overlay = document.createElement('div');
    this.overlay.className = 'world-overlay';
    this.veil = document.createElement('div');
    this.veil.className = 'scene-veil';
    this.root.append(this.canvas, this.veil, this.overlay);
  }

  /** Places the canvas in the page (once, behind every screen). */
  mount(container: HTMLElement): void {
    if (this.root.parentElement !== container) container.appendChild(this.root);
  }

  /**
   * A fresh canvas for a new graphics API: a canvas that has held a WebGPU context can't give a
   * WebGL2 one (and the other way round), so switching backend in Troubleshooting replaces it.
   */
  private freshCanvas(): void {
    const fresh = sceneCanvas();
    this.canvas.replaceWith(fresh);
    this.canvas = fresh;
  }

  /** The shared renderer (created on first use). */
  rendererNow(settings: Settings): Promise<Renderer> {
    return this.renderer ?? this.rendererFor(settings);
  }

  /**
   * Shows `world` as the town overview behind the menus. The renderer keeps drawing while it is
   * shown; `app.liveBackdrop` turns true once it is on screen.
   */
  async showTown(world: WorldStructure, minute: number, settings: Settings): Promise<void> {
    // A game started meanwhile (e.g. Continue before the menu town finished loading): it owns the view.
    if (this.current?.live) return;
    this.town = { world, minute };
    const renderer = await this.rendererNow(settings);
    if (this.town?.world !== world || this.current?.live) return;
    if (!renderer.townShown) renderer.configure(liveOptions(settings));
    await renderer.showTown(world, minute);
    if (this.current?.live) {
      // Too late: the game went live while the town was being built. Take it away again.
      this.town = null;
      if (renderer.townShown) await renderer.hideTown(0);
      return;
    }
    if (this.town?.world !== world) return;
    this.syncActive(renderer);
    await renderer.framesRendered(2);
    if (this.town?.world === world) app.liveBackdrop = true;
  }

  /** Live settings while only the menus use the renderer (a running game applies its own). */
  applyMenuSettings(settings: Settings): void {
    if (!this.town || this.current?.live) return;
    void this.renderer?.then((r) => r.configure(liveOptions(settings)));
  }

  /**
   * Starts a session in the background (paused, no input). Calling again with the same
   * request returns the same session; a different request replaces it.
   */
  prepare(request: StartRequest, settings: Settings, warm = true): Promise<GameSession> {
    const key = requestKey(request);
    // (Unless its simulation stopped while it waited behind the menus: then a new one.)
    if (this.current?.key === key && this.current.instance?.alive !== false) {
      const entry = this.current;
      const session = entry.session;
      if (!warm || entry.warmed) return session;
      // Prepared in the background without its own look drawn: draw it now, behind a loading screen.
      return session.then(async (s) => {
        if (entry.warmed) return s;
        entry.warmed = true;
        if (entry.live) app.loading = { label: 'Warming up', value: 0.95 };
        // Let the loading screen paint before the warm's long frame.
        await new Promise<void>((resolve) => requestAnimationFrame(() => setTimeout(resolve, 0)));
        await perfTimeAsync('host: warm', () => s.warm());
        if (this.current === entry) app.loading = null;
        return s;
      });
    }
    this.release();
    const previous = this.teardown;
    const entry = { key, live: false, ready: false, warmed: warm, progress: { label: 'Getting ready', value: 0 } } as Entry;
    entry.started = (async () => {
      const renderer = await this.rendererFor(settings);
      // One session at a time on the shared renderer: a replaced one must finish clearing it first.
      await previous;
      if (this.current !== entry) throw new Error('Replaced by another game.');
      return startSession(
        renderer,
        this.canvas,
        this.overlay,
        request,
        settings,
        (label, value) => {
          if (this.current !== entry || entry.ready) return;
          entry.progress = { label, value };
          if (entry.live) app.loading = entry.progress;
        },
        warm,
      );
    })();
    entry.session = entry.started.then(async (s) => {
      entry.instance = s;
      await s.ready;
      entry.ready = true;
      if (this.current === entry) app.loading = null;
      // Warmed up: stop drawing unless it's shown (as the game, or the menus' town is).
      if (this.current === entry) this.syncActive(await this.renderer);
      return s;
    });
    const session = entry.session;
    this.current = entry;
    session.catch(() => {
      // A start that failed (or whose simulation stopped before it was shown): let go of what it set up.
      if (this.current?.session === session) this.release();
    });
    return session;
  }

  /** Prepares (or reuses) the session and makes it live: flies (or fades) from the menus into it. */
  async start(request: StartRequest, settings: Settings): Promise<GameSession> {
    const pending = this.prepare(request, settings, true);
    const entry = this.current;
    if (entry) {
      entry.live = true;
      // Still loading (prepared behind the menus, or not at all): from now on, the loading screen says so.
      if (!entry.ready) app.loading = entry.progress;
    }
    const session = await perfTimeAsync('host: prepare (start)', () => pending);
    const renderer = await this.renderer;
    if (!renderer || this.current !== entry) return session;
    this.played = session;
    renderer.setActive(true);
    let veiled = false;
    if (renderer.townShown) {
      // The menus showed this very town (choosing a home or a household to play): fly into the lot. Otherwise fade.
      const meta = this.town?.world.meta;
      const same = request.kind === 'new' && !!meta && request.town.seed === meta.seed && request.town.width === this.town?.world.width;
      if (same && !reducedMotion()) {
        await perfTimeAsync('host: fly in', () => renderer.hideTown(FLY_IN));
      } else {
        veiled = true;
        await perfTimeAsync('host: veil', () => this.setVeil(true));
        await perfTimeAsync('host: hide town', () => renderer.hideTown(0));
        await perfTimeAsync('host: hide frames', () => renderer.framesRendered(3));
      }
    }
    this.town = null;
    app.liveBackdrop = false;
    if (this.current !== entry) return session;
    perfTime('host: reveal', () => session.reveal());
    if (veiled) void this.setVeil(false);
    return session;
  }

  /** Ends the current session (if any); the renderer stays for the next one. */
  release(): void {
    const current = this.current;
    this.current = null;
    this.played = null;
    app.loading = null;
    if (!this.town) app.liveBackdrop = false;
    if (!current) return;
    const renderer = this.renderer;
    // Don't wait for the scene to be ready: a replaced session may never get there.
    this.teardown = current.started
      .then((s) => s.dispose())
      .catch(() => {})
      .then(() => renderer?.then((r) => this.syncActive(r)))
      .catch(() => {});
    game.reset();
  }

  /** The game being played can still be saved (its simulation runs). */
  get canSave(): boolean {
    return !!this.played?.alive;
  }

  /** Saves the game being played as a new save (if it still runs), then loads the page again. */
  async saveAndReload(): Promise<void> {
    const session = this.played;
    // The game's own autosaves, not a new manual save every time the crash screen is used.
    if (session?.alive) await session.saveAutosave();
    location.reload();
  }

  /** A debug report without asking a game (none runs, or it stopped): the log, settings, interface state and a picture. */
  async basicReport(note: string): Promise<string> {
    const renderer = await this.renderer?.catch(() => null);
    const scale = Math.min(window.devicePixelRatio || 1, 2);
    const picture = renderer?.captureThumbnail(Math.round(this.canvas.clientWidth * scale), Math.round(this.canvas.clientHeight * scale));
    return basicReport(note, {
      settings: plainState(liveSettings),
      ui: { screen: app.screen, crash: app.crash, fatal: app.fatal, ...plainState(game, ['catalog', 'threads']) },
      renderer: renderer?.stats() ?? null,
      screenshot: picture ? await within(picture, 4000, null) : null,
    });
  }

  /**
   * A full-screen menu (the pause menu, Settings, Load game, Credits, the planner, the crash
   * screen) covers the view, or no longer does: nothing is drawn behind it (`Renderer.hold`).
   */
  cover(on: boolean): void {
    if (on === this.covered) return;
    this.covered = on;
    void this.renderer?.then((r) => r.hold(on)).catch(() => {});
  }

  /** Draws only while something is on screen: the game, or the menus' town. */
  private syncActive(renderer: Renderer | undefined | null): void {
    if (!renderer) return;
    renderer.setActive(!!this.current?.live || renderer.townShown);
  }

  private setVeil(on: boolean): Promise<void> {
    this.veil.classList.toggle('on', on);
    return new Promise((resolve) => setTimeout(resolve, reducedMotion() ? 0 : VEIL_MS));
  }

  /** The shared renderer; recreated only when a restart-only setting changed. */
  private async rendererFor(settings: Settings): Promise<Renderer> {
    const key = JSON.stringify(RESTART_KEYS.map((k) => settings[k]));
    if (this.renderer && key === this.rendererKey) return this.renderer;
    const old = this.renderer;
    const backendChanged = old !== null && this.rendererBackend !== settings.renderer;
    this.rendererKey = key;
    this.renderer = (async () => {
      (await old?.catch(() => null))?.dispose();
      if (backendChanged) this.freshCanvas();
      const { content, assets } = await loadGameData();
      const params = new URLSearchParams(location.search);
      const renderer = new BabylonRenderer({ assets, content, backend: settings.renderer }, qualityFromSettings(settings, params));
      // Drawing that broke for good can't be mended in place: the crash screen (or, after a GPU
      // reset, "Graphics were reset") offers to save and reload.
      renderer.onFailure = this.onRendererFailure;
      await renderer.init(this.canvas);
      this.rendererBackend = settings.renderer;
      renderer.hold(this.covered);
      if (webGpuFailed() && settings.renderer !== 'webgl' && !this.toldFallback) {
        this.toldFallback = true;
        toast("WebGPU didn't start on this computer, so the game draws with WebGL instead.", 6000);
      }
      return renderer;
    })();
    // A new renderer while the menus show a town: show it there too.
    if (old && this.town) {
      const { world, minute } = this.town;
      void this.renderer.then(async (r) => {
        if (this.town?.world !== world) return;
        await r.showTown(world, minute);
        this.syncActive(r);
      });
    }
    return this.renderer;
  }
}

/** The canvas the world is drawn on. */
function sceneCanvas(): HTMLCanvasElement {
  const canvas = document.createElement('canvas');
  canvas.className = 'scene';
  canvas.tabIndex = -1;
  return canvas;
}

function liveOptions(s: Settings) {
  return { resolution: s.resolution, cameraSensitivity: s.cameraSensitivity, fpsCap: s.fpsCap, shading: s.shading, reducedMotion: s.reducedMotion || matchMedia('(prefers-reduced-motion: reduce)').matches };
}

/** Reduce motion: the setting (as applied to the page) or the system's. */
export function reducedMotion(): boolean {
  return document.documentElement.classList.contains('reduce-motion') || matchMedia('(prefers-reduced-motion: reduce)').matches;
}

export const host = new GameHost();
