/**
 * A game session: starts the sim worker for a new or saved game and wires the (long-lived)
 * renderer, input and HUD to it. Sessions start hidden and paused — they can be prepared
 * in the background while the player is still in a menu — and go live with `reveal()`.
 */

import { AssetRegistry } from '../assets/registry';
import { prefetch, prefetchNow, planStream, whenStreamSettled } from '../assets/stream';
import { Content } from '../content/content';
import { SaveRefused, SimBridge, type FrameState } from '../core/bridge';
import type { Command, GameKind, GameSource, ItemPlacement, Lifespan, Routine, SocialEvent, WorldStructure } from '../core/protocol';
import { lowestStorey } from '../core/protocol';
import { nextAutosaveId, readSave, refuseSave, UnloadableSave, writeSave, type SaveRecord } from '../persistence/saves';
import { BENCH } from '../debug/benchFlags';
import { recentLog } from '../debug/log';
import { deliverReport, gpuInfo, takePendingPose, within, type DebugReport } from '../debug/report';
import { plainState } from '../debug/snapshot.svelte';
import { liveSims } from '../debug/sims';
import { threadStats } from '../debug/threads';
import { createSimPreviews } from '../render/preview';
import { createItemPreviews } from '../render/preview/items';
import type { CameraPose, Renderer, WallMode } from '../render/types';
import { settings as liveSettings, type Settings } from '../settings/settings.svelte';
import { app } from '../ui/app.svelte';
import { services, type GameControls } from '../ui/services';
import { game, nextWallMode, toast } from '../ui/state.svelte';
import { saveDebugReport } from '../ui/debugReport';
import { ideaToasts } from '../ui/planner/ideaToasts';
import { aboutUs } from '../ui/story';
import { BubbleLayer } from './bubbles';
import { RoomOverlay } from './roomOverlay';
import { VoiceDirector } from '../voice';
import { endSoundOf, soundOf, WorldSound } from '../audio/world';
import { addBlueprint, blueprints } from '../persistence/blueprints.svelte';
import { Director } from './director';
import { addedObjects, BuildBuyInput, editFeedback, type Picked } from './buildmode';
import { ownSize } from './surfaces';
import { colorValues } from '../ui/buy/catalog';
import { play } from '../ui/sfx';
import { householdBonds, householdSpawns, lookAtStage, lookFromSeed, palette, type Appearance, type HouseholdDraft } from './household';
import { clockTime, openAt, staffed, staffResidents } from './staff';
import { PointerInput } from './input';
import { assembleTown, loadTemplates, useTownContent, type NeighbourhoodDraft } from './town';
import { gameAssetPlan } from './streamPlan';
import { perfEnd, perfPhase, perfStart, perfTime, perfTimeAsync } from '../debug/perf';
import { UiMirror } from './uiUpdates';

declare global {
  interface Window {
    /** What the benches drive the game through (`tools/bench/scene.mjs`); set only in development builds. */
    __osw?: Record<string, unknown>;
  }
}

/** Click radius (CSS px) around a Sim's projected centre. */
const SIM_PICK_RADIUS = 32;
const STATS_INTERVAL_MS = 500;
/** After the player picks a storey, the view stays on it this long before following residents again. */
const STOREY_STAY_MS = 8000;
const THUMBNAIL = { width: 320, height: 200 };
/** Longest wait for a save's picture (it needs a drawn frame); the save then keeps the last one. */
const THUMBNAIL_WAIT_MS = 1000;
/** Leaving the page (tab hidden, closed) autosaves at most this often. */
const LEAVE_AUTOSAVE_MS = 60_000;

export type StartRequest =
  | {
      kind: 'new';
      town: NeighbourhoodDraft;
      household: HouseholdDraft;
      slot: number;
      /** `household` already lives in `town` at `slot`: the player takes it over instead of moving a new one in. */
      existing?: boolean;
      /** Living (default) or Creative. */
      mode?: GameKind;
      /** How fast residents age (default normal). */
      lifespan?: Lifespan;
    }
  | { kind: 'load'; saveId: string };

/** How long a story-feed entry stays visible. */
const FEED_MS = 12_000;
/** Story-feed entries shown at once. */
const FEED_SIZE = 3;
/** Live: the camera starts watching this long after a game is revealed. */
const START_WATCH_MS = 2500;
/** Shift+R turns things that turn freely by this many degrees. */
const FINE_TURN = 15;
/** Fastest game speed (`clock::MAX_SPEED`). */
const MAX_SPEED = 5;
/** How long the interface stays after the mouse moves while it watches quietly. */
const HUD_AWAKE_MS = 3000;
/** How long a big moment plays at normal speed before the game speeds up again. */
const SLOW_MS = 25_000;
/** Frames drawn before a prepared session counts as ready (shaders compiled, textures up). */
const WARM_FRAMES = 3;
/** Longest wait for the household's portraits before a prepared session counts as ready anyway. */
const PORTRAIT_WAIT_MS = 8000;
/** Longest wait for the first snapshot (its residents' looks) before warming the game's draw. */
const FIRST_FRAME_WAIT_MS = 1500;
/** After this long in the game, the catalog's pictures are drawn in idle moments (the catalog then opens with them). */
const PREFETCH_PICTURES_MS = 4000;
/** Story events the journal keeps: as many as the simulation does (`EVENT_LOG_CAPACITY`). */
const JOURNAL_SIZE = 300;
/** Keys that act again while held (R turns, PageUp/PageDown change storey, Ctrl+Z/Y undo and redo); the rest act once per press. */
const REPEATING = new Set(['r', 'R', 'PageUp', 'PageDown', 'z', 'Z', 'y', 'Y']);
/** Longest wait for a debug report's picture or GPU details (it goes without them). */
const REPORT_PART_MS = 4000;

export interface GameSession {
  /** Saves into `slotId` (a new slot when omitted). Returns the slot id. */
  save(name: string, slotId?: string): Promise<string>;
  /** Saves into the older of the game's two autosaves (the crash screen's Save and reload). */
  saveAutosave(): Promise<string>;
  applySettings(settings: Settings): void;
  /** Pauses for menus; `false` restores the previous speed. */
  setMenuPause(paused: boolean): void;
  /** The player's household (its address until a family moves in). */
  readonly household: string;
  /** The simulation still runs (it hasn't stopped after an error): the game can be saved. */
  readonly alive: boolean;
  /**
   * Resolves when the world is built and drawn and the HUD's portraits exist: revealing is then
   * instant. Rejects if the simulation stops before that.
   */
  readonly ready: Promise<void>;
  /** Compiles and draws the game's own look (the mirror's clip-plane shaders too), if not done yet. */
  warm(): Promise<void>;
  /** Unpauses the game and attaches input. */
  reveal(): void;
  /** Ends the session; resolves once it has let go of the renderer, which is kept for the next one. */
  dispose(): Promise<void>;
}

/** Shared, immutable data: loaded once per page. */
let shared: Promise<{ content: Content; assets: AssetRegistry }> | null = null;
export function loadGameData(): Promise<{ content: Content; assets: AssetRegistry }> {
  const params = new URLSearchParams(location.search);
  const base = import.meta.env.BASE_URL;
  shared ??= Promise.all([
    perfTimeAsync('data: content', () => Content.load(`${base}content/base.json`)),
    perfTimeAsync('data: assets', () => AssetRegistry.load(`${base}assets/manifest.json`, params.getAll('pack'))),
  ]).then(([content, assets]) => {
    services.content = content;
    services.assets = assets;
    useTownContent(content);
    services.previews = createSimPreviews({ assets, emotions: content.emotions.map((e) => e.id), backend: () => liveSettings.renderer });
    services.items = createItemPreviews(assets, () => liveSettings.renderer);
    return { content, assets };
  });
  return shared;
}

/** Reports a loading step and the overall progress (0..1) while a session is prepared. */
export type ProgressReport = (label: string, value: number) => void;

export async function startSession(
  renderer: Renderer,
  canvas: HTMLCanvasElement,
  overlay: HTMLElement,
  request: StartRequest,
  settings: Settings,
  progress: ProgressReport = () => {},
  warm = true,
): Promise<GameSession> {
  const params = new URLSearchParams(location.search);
  // Every loading step names the work in the perf report (a long task or a dropped frame then says which).
  const report = progress;
  progress = (label, value) => {
    perfPhase(`session: ${label}`);
    report(label, value);
  };
  const started = perfStart('session: start→ready');
  progress(request.kind === 'load' ? 'Reading your save' : 'Laying out the neighbourhood', 0.05);
  const { content, assets } = await perfTimeAsync('session: loadGameData', loadGameData);

  let source: GameSource;
  let household: string;
  /** The save being loaded (none for a new game). */
  let saved: SaveRecord | undefined;
  /** The game this is (its saves carry it): new at New game, the save's own when loading (a save from before games had one is a game of its own). */
  let gameId: string;
  if (request.kind === 'new') {
    const templates = await perfTimeAsync('session: loadTemplates', loadTemplates);
    const lot = perfTime('session: assembleTown', () =>
      assembleTown(content, templates, request.town, request.household, request.slot, { existing: request.existing, mode: request.mode, lifespan: request.lifespan }),
    );
    source = { lot, seed: Number(params.get('seed')) || (Math.random() * 2 ** 31) >>> 0 };
    household = request.household.name.trim();
    gameId = crypto.randomUUID();
  } else {
    saved = await perfTimeAsync('session: readSave', () => readSave(request.saveId));
    if (!saved) throw new Error('That save no longer exists.');
    source = { save: saved.data };
    household = saved.household;
    gameId = saved.game ?? saved.id;
  }

  progress('Starting the simulation', 0.15);
  const bridge = await perfTimeAsync('session: SimBridge.start', () => SimBridge.start(content.json, source)).catch(async (err: unknown) => {
    // A save the game can't open is set aside (kept if it's from a newer version, else removed).
    if (err instanceof SaveRefused && saved) throw new UnloadableSave(saved.id, err.problem, await refuseSave(saved, err.problem));
    throw err;
  });
  // The first snapshot carries the household's residents (their clothes, hair and skin), so the
  // warm-up draw must wait for it: without it the first live frame compiles their materials.
  const firstFrame = async (): Promise<FrameState> => {
    const until = performance.now() + FIRST_FRAME_WAIT_MS;
    while (!bridge.hasSnapshot && performance.now() < until) {
      bridge.sync(performance.now());
      await new Promise<void>((resolve) => setTimeout(resolve, 8));
    }
    bridge.sync(performance.now());
    return bridge.frame(performance.now());
  };
  progress('Building the world', 0.3);
  // Hold the clock until the player actually enters the game.
  bridge.send({ type: 'setSpeed', speed: 0 });
  let resumeSpeed: number | null = null;
  let revealed = false;
  let warmed = false;
  /** The world has been built once: later ones are patches or rare rebuilds, and need no warm-up. */
  let warmedFramesOnce = false;
  let disposed = false;
  /** The scene build in progress; disposing waits for it so it can't add to the next session's scene. */
  let building: Promise<unknown> = Promise.resolve();
  let markReady: () => void = () => {};
  let failReady: (err: Error) => void = () => {};
  /** The player household's portraits (requested with the first world). */
  let portraits: Promise<unknown> | null = null;
  const ready = new Promise<void>((resolve, reject) => {
    markReady = resolve;
    failReady = reject;
  });
  void ready.then(
    () => perfEnd(started),
    () => perfEnd(started),
  );
  /** `?debug`'s `__crash('render')`: every frame fails. */
  let failFrames = false;

  let world: WorldStructure | null = null;
  /** The lot the last world structure showed (edit feedback compares worlds of the same lot). */
  let worldView: number | null = null;
  let lastSpeed = 1;
  let focused = false;
  let viewPlot: number | null = null;
  let lastEvent = -1;
  /** Whether the first update's story (the history, empty in a new game) has been seen. */
  let storySeen = false;
  /** The HUD view as the worker's updates built it up. */
  const mirror = new UiMirror();
  /** The household routines last compared (they come again with every change to the household). */
  let routinesSeen: Routine[] | null = null;
  const bubbles = new BubbleLayer(overlay, renderer, content, assets);
  const roomOverlay = new RoomOverlay(overlay, renderer, content);
  /** Build mode's Home tab: the rooms at home tinted by their scores. */
  const roomOverlayOn = () => game.mode === 'build' && game.buildTab === 'home' && world !== null && worldView === homePlot();
  const refreshRoomOverlay = () => roomOverlay.set(roomOverlayOn() ? world : null, game.rooms, game.storey);
  /** When the player last chose a storey (the view then stays there a while rather than following residents). */
  let storeyChosenAt = -Infinity;
  let storeyCheckAt = 0;
  /** The storey in view follows the resident looked at as they go up or down the stairs. */
  const followStorey = (now: number) => {
    if (now < storeyCheckAt || game.storeys - game.lowestStorey < 2 || game.mode !== 'live' || now - storeyChosenAt < STOREY_STAY_MS) return;
    storeyCheckAt = now + 250;
    const id = game.inspected ?? game.selected;
    const snap = bridge.latest();
    const { layout } = bridge;
    if (id === null || layout.sim.height === undefined || id >= snap[layout.header.simCount]) return;
    const o = layout.headerLen + id * layout.simStride;
    if (snap[o + layout.sim.away] > 0 || game.sims.find((s) => s.id === id)?.plot !== viewPlot) return;
    const storey = Math.floor(snap[o + layout.sim.height] + 0.01);
    if (storey !== game.storey) setStorey(storey);
  };
  function setStorey(storey: number): void {
    const k = Math.max(game.lowestStorey, Math.min(game.storeys - 1, storey));
    if (k === game.storey) return;
    game.storey = k;
    renderer.setStorey(k);
    // The grid and previews move to the storey's rows.
    buildBuy.modeChanged();
    refreshRoomOverlay();
  }
  const voices = new VoiceDirector(content, renderer, { world: () => world, subtitles: bubbles });
  const worldSound = new WorldSound(renderer, content);
  const buildBuy = new BuildBuyInput(renderer, content, (command) => bridge.send(command));
  buildBuy.showStorey = (storey) => {
    storeyChosenAt = performance.now();
    setStorey(storey);
  };
  game.catalog = bridge.catalog;
  /** Sims the player controls. */
  const playerSims = () => game.roster.filter((s) => game.households[s.household]?.player).map((s) => s.id);
  /** Names of the player's household: a save's members (Load game lists them), not the whole town. */
  const householdNames = () => game.sims.filter((s) => game.households[s.household]?.player).map((s) => s.name);
  const homePlot = (): number | null => game.households.find((h) => h.player)?.plot ?? null;

  // --- Watching: after a while without input the director takes the camera ----------------
  const director = new Director({
    content,
    layout: bridge.layout,
    snapshot: () => bridge.latest(),
    pace: () => liveSettings.directorPace,
    reducedMotion: () => liveSettings.reducedMotion,
    shoot(shot, walls) {
      renderer.setGameShot(shot);
      if (game.wallMode !== walls) controls.setWallMode(walls);
    },
  });
  /** Last time the player touched anything (ms, performance clock). */
  let lastInput = performance.now();
  /** The wall mode to give back when the director lets go. */
  let playerWalls: WallMode | null = null;
  /** Speed before a big moment slowed the game down, and until when (restored unless the player picks a speed). */
  let slowedFrom: number | null = null;
  let slowUntil = 0;
  const canWatch = () =>
    revealed &&
    game.mode === 'live' &&
    game.follow === null &&
    !game.pauseMenu &&
    !game.menu &&
    !game.socialMenu &&
    !game.townOpen &&
    !game.jobBoardOpen &&
    !game.plannerOpen &&
    viewPlot !== null &&
    viewPlot === homePlot();
  function startWatching(now: number) {
    if (!world) return;
    playerWalls = game.wallMode;
    director.setWorld(world, homePlot());
    director.start(now);
    game.watching = true;
  }
  function stopWatching() {
    director.stop();
    renderer.setGameShot(null);
    game.watching = false;
    if (playerWalls !== null && game.wallMode !== playerWalls) controls.setWallMode(playerWalls);
    playerWalls = null;
  }
  /** Live: seconds after the player's last input before the camera watches on its own (Off: never). */
  const autoWatchDelay = () => (liveSettings.directorDelay > 0 ? liveSettings.directorDelay : Infinity);
  /** At the start of a game: when the camera starts watching (0: not pending). */
  let watchAt = 0;
  /** Any input: the camera is the player's again at once. */
  function takeBack() {
    lastInput = performance.now();
    watchAt = 0;
    if (game.watching) stopWatching();
  }
  const onInput = () => takeBack();
  /** The button last clicked: it keeps focus, but Space still pauses (unlike on a button reached with the keyboard). */
  let clicked: Element | null = null;
  const onPress = (e: PointerEvent) => (clicked = e.target instanceof Element ? e.target.closest('button') : null);
  let hudTimer: ReturnType<typeof setTimeout> | undefined;
  const onMove = () => {
    // Moving the mouse doesn't take the camera, but it does count as being here (and wakes a
    // quiet interface for a moment).
    if (!game.watching) lastInput = performance.now();
    game.hudAwake = true;
    clearTimeout(hudTimer);
    hudTimer = setTimeout(() => (game.hudAwake = false), HUD_AWAKE_MS);
  };
  let speedBeforeMenu: number | null = null;
  /** Build and Create pause the game; the speed to restore back in Live. */
  let speedBeforeBuy: number | null = null;
  let autosaveTimer: ReturnType<typeof setInterval> | undefined;
  let prefetchTimer: ReturnType<typeof setTimeout> | undefined;
  /** The viewed house's files (the second loading screen); made once, on the first built view. */
  let homeAssets: Promise<void> | null = null;
  /** Everything after the house: streamed once the game is revealed. */
  let restUrls: string[] = [];
  let autosaveMinutes = -1;
  /** The performance overlay setting last applied (F3 opens and closes the overlay in between). */
  let showFps: boolean | null = null;
  /**
   * The free will setting last sent: sent again only when it changes, not with every other setting (a debug `setAutonomy`
   * holds). A state the tests wrote (`?state=`) keeps its own free will as it opens (`check_states.rs`: off, so nobody
   * goes off on their own before a check turns it off); the setting is sent once it's changed.
   */
  let autonomy: boolean | null = params.has('state') ? settings.autonomy : null;

  const controls: GameControls = {
    setSpeed(speed) {
      // Time stands still in Build and Create.
      if (game.mode !== 'live') return;
      speed = Math.max(0, Math.min(MAX_SPEED, speed));
      if (speed > 0) lastSpeed = speed;
      slowedFrom = null;
      bridge.send({ type: 'setSpeed', speed });
    },
    togglePause() {
      controls.setSpeed(game.speed === 0 ? lastSpeed : 0);
    },
    selectSim(id) {
      if (!playerSims().includes(id)) return;
      game.selected = id;
      game.socialMenu = null;
      if (liveSettings.directControl === 'always') controls.inspect(id);
    },
    inspect(id) {
      game.inspected = id;
      game.socialMenu = null;
      // Their details and relationships come with UI updates while the panel is open.
      bridge.inspect(id);
      if (id !== null && playerSims().includes(id)) game.selected = id;
      renderer.setSelectedSim(id);
      if (id === null && game.follow !== null) controls.follow(null);
    },
    follow(id) {
      game.follow = id;
      renderer.followSim(id);
      if (id !== null) {
        takeBack();
        const plot = game.sims.find((s) => s.id === id)?.plot;
        if (plot != null && plot !== viewPlot) showPlot(plot);
      } else {
        const home = buildBuy.home();
        if (home && viewPlot !== home.id) showPlot(home.id);
      }
    },
    frameHouse() {
      takeBack();
      if (game.follow !== null) controls.follow(null);
      const plot = world && homePlot() !== null ? world.plots[homePlot()!] : null;
      if (!plot) return;
      const [x0, z0, x1, z1] = plot.house ?? [plot.x, plot.z, plot.x + plot.w, plot.z + plot.d];
      renderer.setGameShot({ target: { x: (x0 + x1) / 2, z: (z0 + z1) / 2 }, beta: 0.72, radius: Math.min(60, Math.max(x1 - x0, z1 - z0) * 1.25 + 8), duration: liveSettings.reducedMotion ? 0 : 1.6 });
      // The shot lands, then the camera is the player's again.
      setTimeout(() => {
        if (!director.active) renderer.setGameShot(null);
      }, liveSettings.reducedMotion ? 0 : 1700);
    },
    toggleJournal() {
      game.journalOpen = !game.journalOpen;
      if (game.journalOpen) game.townOpen = false;
      if (game.journalOpen) void bridge.requestEvents().then((events) => (game.journal = events));
    },
    setLifespan(lifespan) {
      bridge.send({ type: 'setLifespan', lifespan });
    },
    adopt(child) {
      bridge.send({ type: 'adopt', household: game.home, child });
    },
    setPlayerMoves(enabled) {
      bridge.send({ type: 'setPlayerMoves', enabled });
    },
    openPlanner(who) {
      if (!game.occupied) return;
      takeBack();
      const mine = playerSims();
      game.plannerFor = who ?? (game.inspected !== null && mine.includes(game.inspected) ? game.inspected : (mine[0] ?? 'household'));
      game.plannerOpen = true;
      game.menu = null;
      game.socialMenu = null;
    },
    setRoutines(sim, routines) {
      const anyone = playerSims()[0];
      if (sim === null) {
        if (anyone !== undefined) bridge.send({ type: 'setHouseholdRoutines', sim: anyone, routines });
      } else bridge.send({ type: 'setRoutines', sim, routines });
    },
    skipHouseholdRoutine(sim, routine, skip) {
      bridge.send({ type: 'skipHouseholdRoutine', sim, routine, skip });
    },
    addGoal(sim, goal) {
      bridge.send({ type: 'addGoal', sim, goal });
    },
    removeGoal(sim, index) {
      bridge.send({ type: 'removeGoal', sim, index });
    },
    moveGoal(sim, index, to) {
      bridge.send({ type: 'moveGoal', sim, index, to });
    },
    acceptSuggestion(sim, index) {
      bridge.send({ type: 'acceptSuggestion', sim, index });
    },
    dismissSuggestion(sim, index) {
      bridge.send({ type: 'dismissSuggestion', sim, index });
    },
    acceptPurchase(sim) {
      bridge.send({ type: 'acceptPurchase', sim });
    },
    declinePurchase(sim) {
      bridge.send({ type: 'declinePurchase', sim });
    },
    openCatalog(filter) {
      game.plannerOpen = false;
      controls.setBuildTab('furnish');
      game.buySelection = null;
      game.placing = null;
      game.buyFilter = filter;
    },
    showEvent(event) {
      if (game.mode !== 'live') return;
      const now = performance.now();
      if (!director.active) {
        playerWalls = game.wallMode;
        game.watching = true;
      }
      if (!director.show(event, now)) {
        if (!director.active) stopWatching();
        toast("They aren't at home right now.");
      }
    },
    joinCareer(career, level) {
      bridge.send({ type: 'joinCareer', sim: game.selected, career, level });
    },
    quitCareer() {
      bridge.send({ type: 'quitCareer', sim: game.selected });
    },
    visit(plot) {
      bridge.send({ type: 'visit', sim: game.selected, plot });
      game.townOpen = false;
    },
    goHome() {
      bridge.send({ type: 'goHome', sim: game.selected });
      game.townOpen = false;
    },
    socialize(target, social) {
      if (game.mode !== 'live') return;
      bridge.send({ type: 'social', sim: game.selected, target, social });
      voices.playerConversation(game.selected, target);
      game.socialMenu = null;
    },
    phone(action, contact) {
      if (game.mode !== 'live') return;
      bridge.send({ type: 'phone', sim: game.selected, action, contact: contact ?? undefined });
    },
    useObject(objectId, interaction) {
      if (game.mode !== 'live') return;
      bridge.send({ type: 'use', sim: game.selected, object: objectId, interaction });
      game.menu = null;
    },
    useItem(itemId, action) {
      if (game.mode !== 'live') return;
      bridge.send({ type: 'useItem', sim: game.selected, item: itemId, action });
      game.menu = null;
    },
    switchLamp(objectId) {
      if (game.mode !== 'live') return;
      bridge.send({ type: 'switchLamp', object: objectId, on: bridge.lampOff(objectId) });
      game.menu = null;
    },
    cancelAction(index) {
      bridge.send({ type: 'cancel', sim: game.selected, index });
    },
    setWallMode(mode) {
      game.wallMode = mode;
      renderer.setWallMode(mode);
    },
    closeMenu() {
      game.menu = null;
      game.socialMenu = null;
    },
    openPauseMenu() {
      game.menu = null;
      game.pauseMenu = true;
    },
    setMode(mode) {
      // Creation is the new game's: in a game, only a family moving into an empty home.
      if (game.mode === mode || (mode === 'create' && game.occupied)) return;
      takeBack();
      game.customising = null;
      if (mode !== 'live' && game.follow !== null) controls.follow(null);
      const fromLive = game.mode === 'live';
      buildBuy.cancel();
      game.eyedropper = false;
      game.pickHint = null;
      game.mode = mode;
      refreshRoomOverlay();
      play(mode === 'live' ? 'close' : fromLive ? 'open' : 'tab');
      game.menu = null;
      game.socialMenu = null;
      // Build and Create pause the game (and close Live's panels); back in Live the speed returns.
      if (mode !== 'live') {
        if (fromLive) {
          speedBeforeBuy = game.speed;
          bridge.send({ type: 'setSpeed', speed: 0 });
        }
        game.townOpen = false;
        game.jobBoardOpen = false;
        game.plannerOpen = false;
      } else if (speedBeforeBuy !== null) {
        if (speedBeforeBuy > 0) lastSpeed = speedBeforeBuy;
        bridge.send({ type: 'setSpeed', speed: speedBeforeBuy });
        speedBeforeBuy = null;
      }
      // Building happens at home, with the walls down so the floor plan is visible.
      const home = buildBuy.home();
      if (mode === 'build' && home) {
        if (viewPlot !== home.id) showPlot(home.id);
        controls.setWallMode('down');
      }
      buildBuy.modeChanged();
    },
    setBuildTab(tab) {
      if (tab === 'build') return controls.setBuildTool(game.buildTool);
      const wasBuilding = game.mode === 'build';
      controls.setMode('build');
      if (game.buildTab === tab) return;
      buildBuy.cancel();
      game.buildTab = tab;
      game.buildStart = null;
      buildBuy.clearPreviews();
      refreshRoomOverlay();
      if (wasBuilding) play('tab');
    },
    startPlacing(def) {
      // Stairs are built with the Build tab's Stairs tool.
      if (content.object(def)?.stairs) return controls.setBuildTool('stairs');
      controls.setBuildTab('furnish');
      game.buySelection = null;
      const turns = !!content.object(def) && content.turns(content.object(def)!);
      game.placing = { def, rot: game.placing?.rot ?? 0, turn: turns ? (game.placing?.turn ?? 0) : 0, objectId: null };
    },
    startMoving(objectId) {
      const obj = game.objects.find((o) => o.id === objectId);
      if (!obj) return;
      if (content.object(obj.def)?.stairs) {
        controls.setBuildTool('stairs');
        buildBuy.cancel();
      } else {
        controls.setBuildTab('furnish');
        game.buySelection = null;
      }
      game.placing = { def: obj.def, rot: obj.rot, turn: obj.turn ?? 0, objectId, size: ownSize(content, obj), tall: obj.tall ?? 0 };
    },
    rotatePlacing(fine = false) {
      if (!game.furnishing && !buildBuy.stairsTool()) return;
      // Facing and turn as one angle: a quarter turn, or 15° for things that turn freely.
      const turned = (def: string, rot: number, turn: number) => {
        const d = content.object(def);
        const step = fine && d && content.turns(d) ? FINE_TURN : 90;
        const angle = (rot * 90 + turn + step) % 360;
        return { rot: Math.floor(angle / 90), turn: angle % 90 };
      };
      if (game.placing) {
        game.placing = { ...game.placing, ...turned(game.placing.def, game.placing.rot, game.placing.turn) };
        buildBuy.rehover();
        play('rotate');
      } else if (game.buySelection !== null) {
        const obj = game.objects.find((o) => o.id === game.buySelection);
        if (obj) bridge.send({ type: 'moveObject', household: game.home, object: obj.id, x: obj.x, z: obj.z, ...turned(obj.def, obj.rot, obj.turn ?? 0) });
      }
    },
    cancelPlacing() {
      buildBuy.cancel();
    },
    sell(objectId) {
      if (!game.furnishing && !buildBuy.stairsTool()) return;
      // Selling what's in hand puts it down first.
      if (game.placing?.objectId === objectId) buildBuy.cancel();
      bridge.send({ type: 'sell', household: game.home, object: objectId });
      game.buySelection = null;
      game.menu = null;
    },
    repair(objectId) {
      bridge.send({ type: 'repair', household: game.home, object: objectId });
    },
    undo() {
      // The step count follows a little later; don't send more undos than there are steps.
      if (game.mode !== 'build' || game.undoSteps <= 0) return;
      // (It may move or remove things: something carried goes back first.)
      buildBuy.dropOwned();
      game.undoSteps--;
      game.buildStart = null;
      game.buySelection = null;
      buildBuy.clearPreviews();
      bridge.send({ type: 'undo', household: game.home });
    },
    redo() {
      if (game.mode !== 'build' || game.redoSteps <= 0) return;
      buildBuy.dropOwned();
      game.redoSteps--;
      game.buildStart = null;
      game.buySelection = null;
      buildBuy.clearPreviews();
      bridge.send({ type: 'redo', household: game.home });
    },
    recolor(objectId, colors) {
      bridge.send({ type: 'recolor', household: game.home, object: objectId, colors: colorValues(colors) });
    },
    colorPlacing(colors) {
      if (!game.placing || game.placing.objectId !== null) return;
      game.placing = { ...game.placing, colors };
      buildBuy.rehover();
    },
    resetSize(objectId) {
      const obj = game.objects.find((o) => o.id === objectId);
      const def = obj && content.object(obj.def);
      if (!obj || !def) return;
      // Back to its kind's footprint from the same corner (the simulation refuses it if that doesn't fit).
      const [w, d] = def.footprint ?? [1, 1];
      bridge.send({ type: 'moveObject', household: game.home, object: obj.id, x: obj.x, z: obj.z, rot: obj.rot, offset: [0, 0], size: [w * 100, d * 100], tall: 0 });
    },
    /** The catalog: buy the next quality level (instant). */
    upgrade(objectId) {
      if (!game.furnishing) return;
      bridge.send({ type: 'upgrade', household: game.home, object: objectId });
      game.menu = null;
    },
    toggleEyedropper() {
      if (game.mode !== 'build') return;
      game.eyedropper = !game.eyedropper;
      if (!game.eyedropper) game.pickHint = null;
      else {
        game.placing = null;
        game.buildStart = null;
        buildBuy.clearPreviews();
      }
      play('tab');
    },
    setRoof(style, color) {
      bridge.send({ type: 'setRoof', household: game.home, style, color });
    },
    setStorey(storey) {
      storeyChosenAt = performance.now();
      setStorey(storey);
    },
    async saveBlueprint(name) {
      try {
        const [data, thumbnail] = await Promise.all([bridge.requestBlueprint(game.home), renderer.captureThumbnail(THUMBNAIL.width, THUMBNAIL.height)]);
        if (addBlueprint(name, data, thumbnail)) toast(`Saved “${name}” as a blueprint`);
        else toast('There is no room left in this browser for another blueprint');
      } catch (err) {
        toast(err instanceof Error ? err.message : String(err));
      }
    },
    buildBlueprint(id) {
      const saved = blueprints.list.find((b) => b.id === id);
      if (!saved) return;
      try {
        bridge.send({ type: 'buildBlueprint', household: game.home, blueprint: JSON.parse(saved.data) });
      } catch (err) {
        toast(err instanceof Error ? err.message : String(err));
      }
    },
    setBuildTool(tool) {
      const wasBuilding = game.mode === 'build';
      controls.setMode('build');
      // Changing tab or tool, or (re)entering the Stairs tool, drops what's in hand.
      const fromTab = game.buildTab;
      if (fromTab !== 'build' || tool !== game.buildTool || tool === 'stairs') buildBuy.cancel();
      game.buildTab = 'build';
      game.buildTool = tool;
      if (fromTab !== 'build') {
        refreshRoomOverlay();
        if (wasBuilding) play('tab');
      }
      // Painting needs the walls standing to see them; the roof shows with walls up.
      if (tool === 'paint' && game.wallMode === 'down') controls.setWallMode('cutaway');
      if (tool === 'roof') controls.setWallMode('up');
      game.buildStart = null;
      buildBuy.clearPreviews();
      // Stairs: one in hand straight away.
      const stairs = tool === 'stairs' ? buildBuy.stairsDef() : undefined;
      if (stairs) game.placing = { def: stairs.id, rot: 0, turn: 0, objectId: null };
    },
    build(edits) {
      bridge.send({ type: 'build', household: game.home, edits });
    },
    paint(faces) {
      bridge.send({ type: 'paint', household: game.home, faces });
    },
    moveIn(draft) {
      const home = buildBuy.home();
      if (!home) return;
      const [x, z] = home.entry ?? [home.x + home.w / 2, home.z + 1.5];
      // Everyone arrives at the front of the lot, side by side within the entry tile.
      const n = draft.members.length;
      const spawns = draft.members.map((_, i): [number, number] => [x + (n > 1 ? (i / (n - 1) - 0.5) * 0.6 : 0), z]);
      bridge.send({
        type: 'moveIn',
        household: game.home,
        name: draft.name.trim(),
        sims: householdSpawns(draft, game.home, spawns),
        bonds: householdBonds(draft, 0),
      });
      // Back to Live (its speed returns) to see them arrive.
      controls.setMode('live');
      play('open');
    },
    currentState() {
      return bridge.requestSave();
    },
    async debugReport(note) {
      const scale = Math.min(window.devicePixelRatio || 1, 2);
      // A game that stopped has no state to save: the report still carries the log, the settings
      // and the picture. Nothing here waits for long (drawing may have stopped too).
      const [save, screenshot, gpu] = await Promise.all([
        bridge.alive ? bridge.requestSave().catch(() => null) : null,
        within(renderer.captureThumbnail(Math.round(canvas.clientWidth * scale), Math.round(canvas.clientHeight * scale)), REPORT_PART_MS, null),
        within(gpuInfo(), REPORT_PART_MS, null),
      ]);
      // Everything the HUD shows, minus the (large, static) catalog and the threads (reported on their own).
      const ui = plainState(game, ['catalog', 'threads']);
      const report: DebugReport = {
        note,
        version: __APP_VERSION__,
        commit: __APP_COMMIT__,
        createdAt: new Date().toISOString(),
        url: location.href,
        userAgent: navigator.userAgent,
        screen: { width: window.innerWidth, height: window.innerHeight, pixelRatio: window.devicePixelRatio || 1 },
        gpu,
        renderer: renderer.stats(),
        threads: threadStats(renderer.stats(), bridge.threadStats, voices.stats()),
        settings: plainState(liveSettings),
        camera: renderer.cameraPose(),
        ui: { ...ui, viewPlot },
        log: recentLog(),
        saveMeta: { id: 'debug', name: 'Debug report', household, members: householdNames(), day: game.day, minute: game.minute, savedAt: Date.now(), autosave: false },
      };
      return deliverReport({ report, save, screenshot });
    },
  };
  services.controls = controls;
  // `?debug`: drive the game from the console / test harness (and look at any lot: `__showPlot(i)`, `__world().plots`;
  // `__send(command)` sends any simulation command, `__items()` the food and dishes about). `__world().objects` carry
  // a lamp's (or a fire's) `off`, which the world itself doesn't (the snapshot has it: `bridge.lampOff`).
  if (params.has('debug')) {
    const withLamps = (w: WorldStructure | null) =>
      w && { ...w, objects: w.objects.map((o) => (content.object(o.def)?.light ? { ...o, off: bridge.lampOff(o.id) } : o)) };
    Object.assign(window, { __controls: controls, __showPlot: showPlot, __world: () => withLamps(world), __send: (cmd: Command) => bridge.send(cmd), __items: () => itemsSeen });
    // `__sims()`: every resident as the newest snapshot shows them (where, pose, the object and use,
    // needs; `debug/sims.ts`), to wait on; `__content` the game's content (`__content.object(id)`).
    // `__addFunds(amount)`: money for the player's household. `__customise(id)`: the colour studio
    // of the object `id` at home (null: closed).
    // `__camera(pose?)`: the camera's pose, and with a (partial) pose, puts it there and stops the
    // camera watching on its own. `__controls.follow(id, view?)` also sets the radius, beta, alpha
    // or target height (`y`) it follows with; `follow(null)` stops following and stays on the lot in view.
    const placeCamera = (pose: Partial<CameraPose> & { y?: number }) => {
      takeBack();
      const now = renderer.cameraPose();
      const target = pose.target ?? [now.target[0], pose.y ?? now.target[1], now.target[2]];
      renderer.setCameraPose({ ...now, ...pose, target });
    };
    Object.assign(window, {
      __sims: () => liveSims(bridge.latest(), bridge.layout, content, game.sims),
      __content: content,
      __addFunds: (amount: number, household = game.home) => bridge.send({ type: 'addFunds', household, amount }),
      __customise: (id: number | null) => {
        const def = game.objects.find((o) => o.id === id)?.def;
        game.customising = def && id !== null ? { def, objectId: id } : null;
      },
      __camera: (pose?: Partial<CameraPose> & { y?: number }) => {
        if (pose) placeCamera(pose);
        return renderer.cameraPose();
      },
      __controls: Object.assign(Object.create(controls) as GameControls, {
        // (Stopping, the view stays on the lot it shows: the game's own goes back home, which a check looking at
        // another lot doesn't want.)
        follow(id: number | null, view?: Partial<Omit<CameraPose, 'target'>> & { y?: number }) {
          if (id === null) {
            game.follow = null;
            renderer.followSim(null);
          } else controls.follow(id);
          if (view) placeCamera(view);
        },
      }),
    });
    // `__crash(what)`: fails on purpose, to see how the game copes. 'panic': the simulation panics;
    // 'throw': an error its worker doesn't catch; 'hang': its worker hangs; 'render': every frame fails.
    Object.assign(window, { __crash: (what: 'panic' | 'throw' | 'hang' | 'render') => (what === 'render' ? (failFrames = true) : bridge.debugCrash(what)) });
  }
  game.sharedMemory = bridge.sharedMemory;
  game.household = household;
  // Portraits of new faces in play come from a studio already warmed up.
  services.previews.keep(true);

  // Goal ideas and wishes as they come up, to accept or decline.
  const ideas = ideaToasts();
  bridge.onError((message) => {
    if (disposed) return;
    toast(message);
    if (game.mode === 'build') play('error');
  });
  // The simulation stopped for good. In play: the crash screen. Still being prepared: the start
  // fails (the host lets go of it).
  let stoppedWith: string | null = null;
  bridge.onFatal((message) => {
    if (disposed) return;
    stoppedWith = message;
    if (revealed) app.crashed({ kind: 'sim', detail: message });
    else failReady(new Error(`The game stopped because of an error: ${message}`));
  });
  bridge.onUi((ui) => {
    if (disposed) return;
    // Updates carry changes only: lists are replaced when something in them changed, keeping
    // the objects of everything that didn't.
    const changed = mirror.apply(ui);
    if (!changed) return;
    // The first update carries the speed the game was saved at.
    resumeSpeed ??= ui.speed > 0 ? ui.speed : 1;
    game.day = ui.day;
    game.minute = ui.minute;
    game.speed = ui.speed;
    game.lifespan = ui.lifespan;
    game.playerMoves = ui.playerMoves;
    if (changed.sims) game.sims = changed.sims;
    game.weekday = ui.weekday;
    ideas.update(game.sims);
    const mine = [...mirror.households.values()].find((h) => game.households[h.id]?.player);
    game.funds = mine?.funds ?? 0;
    game.rent = mine?.rent ?? null;
    game.bills = mine?.bills ?? null;
    game.expecting = mine?.expecting ?? null;
    game.undoSteps = mine?.undo ?? 0;
    game.redoSteps = mine?.redo ?? 0;
    if (changed.rooms) {
      game.rooms = changed.rooms;
      refreshRoomOverlay();
    }
    if (mine?.routines && mine.routines !== routinesSeen) {
      routinesSeen = mine.routines;
      if (JSON.stringify(mine.routines) !== JSON.stringify(game.householdRoutines)) game.householdRoutines = mine.routines;
    }
    if (changed.relationships) game.relationships = changed.relationships;
    // The view stays home; it goes along to other lots only with a resident the player follows.
    const followed = game.follow === null ? null : game.sims.find((s) => s.id === game.follow);
    if (game.follow !== null && !followed) controls.follow(null);
    else if (followed?.plot != null && followed.plot !== viewPlot) showPlot(followed.plot);
    // New story events: the journal, the director, and the feed (only the first update's story is
    // history; a new game's is empty, and its first event is news).
    const events = ui.events ?? [];
    const fresh = events.filter((e) => e.id > lastEvent);
    if (storySeen && fresh.length) onStory(fresh);
    storySeen = true;
    if (events.length) lastEvent = events[events.length - 1].id;
  });
  /** The newest picture of the game (a save while nothing is drawn, e.g. in a hidden tab, keeps it). */
  let thumbnail = saved?.thumbnail ?? null;
  /**
   * Saves into slot `id`. Its picture needs a drawn frame: it's skipped in a hidden tab and waited
   * for only so long, so saving works while nothing is drawn.
   */
  const store = async (name: string, id: string, autosave: boolean): Promise<string> => {
    const picture = document.hidden
      ? Promise.resolve(null)
      : Promise.race([perfTimeAsync('save: picture', () => renderer.captureThumbnail(THUMBNAIL.width, THUMBNAIL.height)), new Promise<null>((resolve) => setTimeout(resolve, THUMBNAIL_WAIT_MS, null))]);
    const [data, taken] = await Promise.all([perfTimeAsync('save: data', () => bridge.requestSave()), picture]);
    thumbnail = taken ?? thumbnail;
    const record: SaveRecord = {
      id,
      name,
      household,
      members: householdNames(),
      day: game.day,
      minute: game.minute,
      savedAt: Date.now(),
      thumbnail,
      autosave,
      game: gameId,
      data,
    };
    await perfTimeAsync('save: write', () => writeSave(record));
    return id;
  };
  /** Saves into the older of the game's two autosaves. */
  const storeAutosave = async () => store(`${household} · Autosave`, await nextAutosaveId(gameId), true);
  /** When the game last autosaved (or started). */
  let autosavedAt = -Infinity;
  let autosaveFailed = false;
  /**
   * Autosaves the game being played (`storeAutosave`); a failure is told once.
   * Not once the simulation stopped: there's nothing to save, and the crash screen says so.
   */
  const autosave = async () => {
    if (!revealed || disposed || stoppedWith !== null) return;
    autosavedAt = performance.now();
    try {
      await storeAutosave();
    } catch (err) {
      console.warn('[save] autosave failed', err);
      if (autosaveFailed) return;
      autosaveFailed = true;
      toast(`Autosave failed: ${err instanceof Error ? err.message : err}. Save from the pause menu to keep your progress.`, 8000);
    }
  };
  /** The tab is hidden or the page goes away (it may not come back): autosave, at most once a minute. */
  const onLeave = () => {
    if (autosaveMinutes > 0 && performance.now() - autosavedAt >= LEAVE_AUTOSAVE_MS) void autosave();
  };
  // Back from a hidden tab (throttled, maybe suspended): start the HUD over from everything.
  const onVisibility = () => {
    if (!document.hidden) bridge.resyncUi();
    else onLeave();
  };
  document.addEventListener('visibilitychange', onVisibility);
  window.addEventListener('pagehide', onLeave);
  let itemsSeen: ItemPlacement[] = [];
  bridge.onItems((items) => {
    itemsSeen = items;
    if (!disposed) renderer.setItems(items);
  });
  bridge.onLamps((off) => {
    if (!disposed) renderer.setLampsOff(off);
  });
  bridge.onWorld((w) => {
    if (disposed) return;
    // Newcomers and babies the simulation made up carry an appearance seed: give them their
    // look (babies from their parents', so parents first). Residents with none at all (a game
    // state the simulation's tests wrote) get one from their id and name.
    type Seeded = { seed?: number; parents?: number[]; voice?: Appearance['voice'] } | null;
    const nameSeed = (id: number, name: string) => [...name].reduce((h, c) => Math.imul(h ^ c.charCodeAt(0), 0x01000193) >>> 0, 0x811c9dc5 ^ id);
    for (const pass of [false, true]) {
      for (const s of w.sims) {
        const { seed, parents, voice } = (s.appearance as Seeded) ?? { seed: nameSeed(s.id, s.name) };
        if (typeof seed !== 'number' || Boolean(parents?.length) !== pass) continue;
        const looks = (parents ?? []).map((p) => w.sims[p]?.appearance).filter((a) => a && (a as Seeded)?.seed === undefined);
        s.appearance = lookFromSeed(assets, s.gender, seed, looks);
        // Only the look comes from the seed: a voice stored with it is kept.
        if (voice) s.appearance.voice = voice;
      }
    }
    for (const s of w.sims) {
      // Life stage: body and head size, and age greys the hair (as the menus show them).
      const stage = content.lifeStages.find((st) => st.id === s.stage);
      if (s.appearance) s.appearance = lookAtStage(s.appearance, stage, palette(assets, 'palette.hair'));
    }
    // The town's vendors and shopkeepers, drawn after the residents (`game/staff.ts`).
    w.staffSims = staffResidents(content, assets, w);
    // Build mode: show what the simulation just accepted (furniture popping in, dust, money).
    const feedback = game.mode === 'build' && world && worldView === viewPlot ? editFeedback(world, w) : null;
    if (feedback?.sound) play(feedback.sound);
    // Things the player just placed: the director looks out for their first use.
    if (game.mode === 'build' && world) {
      const added = addedObjects(world, w);
      if (added.length) director.noteBuilt(added.map((o) => o.id), performance.now());
    }
    world = w;
    worldView = viewPlot;
    game.storeys = w.storeys ?? 1;
    game.lowestStorey = lowestStorey(w);
    if (game.storey >= game.storeys || game.storey < game.lowestStorey) setStorey(Math.max(game.lowestStorey, Math.min(game.storeys - 1, game.storey)));
    buildBuy.setStructure(w);
    refreshRoomOverlay();
    const mine = w.households.find((h) => h.player);
    director.setWorld(w, mine?.plot ?? null);
    game.creative = w.mode === 'creative';
    if (mine) {
      game.home = mine.id;
      household = game.household = mine.name;
    }
    game.objects = w.objects;
    // Something recoloured while in hand: the ghost takes the new colour.
    if (game.placing?.objectId != null) buildBuy.rehover();
    if (game.buySelection !== null && !w.objects.some((o) => o.id === game.buySelection)) game.buySelection = null;
    buildBuy.objectsChanged();
    game.roster = w.sims.filter((s) => !s.gone);
    game.everyone = w.sims;
    game.former = w.former ?? [];
    game.households = w.households;
    game.plots = w.plots;
    // Portraits for the HUD, the household first (drawn once, cached by look); the game is
    // ready once the household's are.
    const mineFirst = [...game.roster].sort((a, b) => Number(!!w.households[b.household]?.player) - Number(!!w.households[a.household]?.player));
    const looks = mineFirst.map((s) => ({ gender: s.gender, appearance: s.appearance, id: s.id }));
    portraits ??= Promise.all(looks.filter((_, i) => w.households[mineFirst[i].household]?.player).map((look) => services.previews.portrait(look)));
    services.previews.prefetch(looks);
    if (!focused) {
      focused = true;
      const first = playerSims()[0];
      // Nobody lives at home: nobody to give orders to.
      game.selected = first ?? -1;
      if (first !== undefined && liveSettings.directControl === 'always') controls.inspect(first);
      // Start on the home lot; its geometry arrives in the next world message.
      const home = w.households.find((h) => h.player)?.plot;
      if (home != null && w.plots[home]) return showPlot(home);
    } else if (!playerSims().includes(game.selected)) {
      // A family just moved in: the first of them is the one to look at.
      const first = playerSims()[0];
      if (first !== undefined) {
        game.selected = first;
        controls.inspect(first);
      }
    }
    const plot = viewPlot === null ? null : w.plots[viewPlot];
    const view = plot ? { x: plot.x, z: plot.z, w: plot.w, d: plot.d } : null;
    worldSound.setWorld(w, view);
    if (!homeAssets) {
      // First the house (the loading screen waits for it), then the rest of the town (streamed
      // after Play). Planning once keeps a later world update from restarting the indicator.
      const plan = gameAssetPlan(content, assets, w, view);
      planStream([...plan.home, ...plan.rest], 'Game data');
      restUrls = plan.rest;
      homeAssets = prefetchNow(plan.home, (done, total) => {
        progress(`Fetching your home (${done}/${total})`, 0.3 + 0.35 * (done / Math.max(1, total)));
      }).then(() => progress('Building the world', 0.65));
    }
    const build = homeAssets.then(async () => {
      const buildSpan = perfStart('session: renderer setWorld');
      try {
        return await renderer.setWorld(w, view);
      } finally {
        perfEnd(buildSpan);
      }
    });
    building = build.catch(() => {});
    if (feedback?.effects.length) void build.then(() => feedback.effects.forEach((fx) => renderer.buildEffect(fx)), () => {});
    build
      .then(async () => {
        progress('Warming up', 0.75);
        // Compile the game's own look (its lighting and cutaway) while the menu still covers it.
        // (When the game is only prepared in the background, it waits until it is started: the
        // mirror's clip-plane shaders take a long frame, which must not land on a visible menu.)
        if (warm && !warmed) {
          warmed = true;
          const frame = await perfTimeAsync('session: first frame', firstFrame);
          if (frame) await perfTimeAsync('session: warm game', () => renderer.warmGame(frame));
          else perfPhase('session: warm game (no frame)');
        } else perfPhase('session: warm game (deferred)');
        // Only the first world is drawn a few times to compile what it shows: a later one is
        // patched in place (an object opened, a crop grown), and warms nothing new.
        const frames = warmedFramesOnce ? 1 : WARM_FRAMES;
        warmedFramesOnce = true;
        return perfTimeAsync('session: warm frames', () => renderer.framesRendered(frames));
      })
      .then(() => {
        progress('Drawing portraits', 0.9);
        return perfTimeAsync('session: portraits', () => Promise.race([portraits, new Promise((resolve) => setTimeout(resolve, PORTRAIT_WAIT_MS))]));
      })
      .then(() => {
        progress('Ready', 1);
        markReady();
      })
      .catch((err) => toast(`Rendering failed: ${err}`));
  });

  function onStory(fresh: SocialEvent[]) {
    const now = Date.now();
    // The open journal keeps up (it's fetched again when it opens).
    if (game.journalOpen && game.journal.length) game.journal = game.journal.concat(fresh).slice(-JOURNAL_SIZE);
    director.noteEvents(fresh.filter(aboutUs), performance.now());
    // The feed: what happens to the household, and the town's bigger news.
    const news = fresh.filter((e) => (aboutUs(e) && e.importance >= 1) || e.importance >= 2 || (aboutUs(e) && e.kind === 'skillUp'));
    if (news.length) game.feed = [...game.feed.filter((f) => now - f.at < FEED_MS), ...news.map((event) => ({ event, at: now }))].slice(-FEED_SIZE);
    // Big moments at home are worth seeing at normal speed.
    const big = fresh.find((e) => e.importance >= 2 && aboutUs(e));
    if (big && liveSettings.autoSlow && game.mode === 'live' && game.speed >= 3) {
      const from = game.speed;
      controls.setSpeed(1);
      slowedFrom = from;
      slowUntil = performance.now() + SLOW_MS;
      toast('Slowed down for a moment');
    }
  }

  /**
   * Switches the view to a lot: the worker sends that lot's geometry, then the scene is rebuilt.
   * The camera moves to its middle, unless it is going along with someone the player follows.
   */
  function showPlot(plot: number) {
    const p = world?.plots[plot];
    if (!p) return;
    viewPlot = plot;
    game.viewPlot = plot;
    if (game.follow === null) renderer.focus(p.x + p.w / 2, p.z + p.d / 2);
    bridge.setView([p.x, p.z, p.x + p.w, p.z + p.d]);
  }
  controls.setWallMode(game.wallMode);

  const inLot = (x: number, z: number) => !!world && x >= 0 && z >= 0 && x < world.width && z < world.depth;

  const screen = { x: 0, y: 0 };
  const head = { x: 0, y: 0, z: 0 };
  /** Nearest Sim whose projected body centre is within the pick radius. */
  const simAt = (px: number, py: number): number | null => {
    const snap = bridge.latest();
    const { layout } = bridge;
    let best: number | null = null;
    let bestDist = SIM_PICK_RADIUS;
    for (let i = 0; i < snap[layout.header.simCount]; i++) {
      const o = layout.headerLen + i * layout.simStride;
      // Where the body is drawn (on its storey; residents not drawn can't be clicked).
      const dist = renderer.simHead(i, head) && renderer.project(head.x, head.y - 0.8, head.z, screen)
        ? Math.hypot(screen.x - px, screen.y - py)
        : Infinity;
      if (dist < bestDist) {
        bestDist = dist;
        best = snap[o + layout.sim.id];
      }
    }
    return best;
  };

  const openMenu = (objectId: number, x: number, y: number) => {
    const placement = world?.objects.find((o) => o.id === objectId);
    const def = placement && content.object(placement.def);
    if (!placement || !def) return;
    // Uses the resident's age isn't for are greyed out (the kids' things for a grown-up, wine for a child).
    const stage = game.sims.find((s) => s.id === game.selected)?.stage;
    // Paid interactions show their price; the pie menu greys them out when funds run short.
    // Dining at a table only at a chair pulled up to one; folding down only what's folded up.
    // (Trying for a baby is asked of a partner, not offered by the bed.)
    const atTable = def.interactions.some((it) => it.needsTable) && buildBuy.atTable(objectId);
    const state = placement.open ? 'open' : 'closed';
    // A crop is harvested only ripe (growth stage 2).
    const offered = (when: string | undefined) => !when || (when === 'ripe' ? placement.growth === undefined || placement.growth === 2 : when === state);
    // A coffee needs a working coffee machine on the same lot (content `requires` / `provides`).
    const plot = world?.plots.find((p) => placement.x >= p.x && placement.z >= p.z && placement.x < p.x + p.w && placement.z < p.z + p.d);
    const provided = new Set<string>();
    if (plot) {
      for (const o of world?.objects ?? []) {
        if (o.x < plot.x || o.z < plot.z || o.x >= plot.x + plot.w || o.z >= plot.z + plot.d || (o.wear ?? 0) >= 1) continue;
        for (const thing of content.object(o.def)?.provides ?? []) provided.add(thing);
      }
    }
    const supplied = (requires: string[] | undefined) => !requires?.length || requires.every((r) => provided.has(r));
    // Sold by a vendor or shopkeeper where someone serves (`game/staff.ts`): greyed out while they're off
    // (the post's hours, or the café's machine a table's coffee is fetched from); helped to elsewhere.
    const post = staffed(world, objectId);
    const closedAt = (it: (typeof def.interactions)[number]): number | null => {
      if (it.selfServe && post) return openAt(def, game.minute) ? null : (def.staff?.hours[0] ?? null);
      if (!it.fetch || !plot) return null;
      const sources = (world?.objects ?? []).filter((o) => {
        const d = content.object(o.def);
        return o.id !== objectId && o.x >= plot.x && o.z >= plot.z && o.x < plot.x + plot.w && o.z < plot.z + plot.d && (o.wear ?? 0) < 1 && (it.requires ?? []).every((r) => d?.provides?.includes(r)) && d?.interactions.some((i) => i.id === it.fetch);
      });
      const shut = sources.filter((o) => staffed(world, o.id) && !openAt(content.object(o.def)!, game.minute));
      return sources.length && shut.length === sources.length ? (content.object(shut[0].def)?.staff?.hours[0] ?? null) : null;
    };
    const items: { label: string; index: number; disabled?: boolean; cost?: number }[] = def.interactions.flatMap((it, index) => {
      if ((it.needsTable && !atTable) || !supplied(it.requires) || !offered(it.when) || it.conceive) return [];
      const opens = closedAt(it);
      const label = it.selfServe && !post ? it.selfServe.label : it.label;
      return [{ label: opens === null ? label : `${label} (opens at ${clockTime(opens)})`, index, ...(it.cost ? { cost: it.cost } : {}), ...(content.mayUse(def, it, stage) && opens === null ? {} : { disabled: true }) }];
    });
    // A lamp: its switch; a fire is lit and put out.
    const off = bridge.lampOff(objectId);
    if (def.light && def.light.switch !== false) items.unshift({ label: def.light.fire ? (off ? 'Light the fire' : 'Put out the fire') : off ? 'Turn on' : 'Turn off', index: -1 });
    // Upgrades are bought in the catalog, not from here.
    if (items.length === 0) return;
    game.menu = { objectId, title: placement.quality ? `${def.name} ${'★'.repeat(placement.quality)}` : def.name, x, y, items };
  };

  /** Food or a dish: eat a serving, or clear it away (to the sink). */
  const openItemMenu = (itemId: number, x: number, y: number) => {
    const it = bridge.item(itemId);
    const def = it && content.item(it.def);
    if (!it || !def) return;
    const edible = it.state === 'fresh' && it.servings > 0;
    const servings = it.servings > 1 ? ` (${it.servings} servings)` : '';
    const items = [
      { label: 'Eat', index: 0, disabled: !edible },
      { label: 'Clear away', index: 1 },
    ];
    game.menu = { objectId: -1, itemId, title: `${def.label}${it.state === 'spoiled' ? ' (gone off)' : servings}`, x, y, items };
  };

  let pointer: PointerInput | null = null;
  const attachPointer = () => new PointerInput(canvas, {
    click(x, y) {
      if (game.pauseMenu) return;
      game.menu = null;
      game.socialMenu = null;
      game.townOpen = false;
      if (game.mode === 'create') return;
      if (game.mode === 'build') {
        const hit = renderer.pick(x, y);
        const ground = hit.ground && inLot(hit.ground.x, hit.ground.z) ? hit.ground : null;
        if (buildBuy.picking()) return usePicked(buildBuy.pick(ground, hit.objectId));
        const held = game.placing;
        const refused = game.placeRefused;
        buildBuy.click(hit.ground, hit.objectId);
        if (!held && game.placing) play('pick');
        if (game.placeRefused !== refused) play('error');
        return;
      }
      // Watching first: a click looks at someone; orders come from their panel (or, with the
      // classic setting, straight away).
      const direct = liveSettings.directControl === 'always';
      const actor = direct ? game.selected : game.inspected !== null && playerSims().includes(game.inspected) ? game.inspected : null;
      const sim = simAt(x, y);
      if (sim !== null && actor !== null && sim !== actor) {
        game.socialMenu = { target: sim, x, y, options: null };
        bridge.requestSocialOptions(actor, sim).then(
          (options) => {
            if (game.socialMenu?.target === sim) game.socialMenu = { ...game.socialMenu, options };
          },
          (err) => {
            console.warn('[social] options failed', err);
            if (game.socialMenu?.target === sim) game.socialMenu = null;
          },
        );
        return;
      }
      if (sim !== null) {
        if (!direct) controls.inspect(sim === game.inspected ? null : sim);
        return;
      }
      const hit = renderer.pick(x, y);
      if (hit.itemId != null) return openItemMenu(hit.itemId, x, y);
      if (hit.objectId !== null) return openMenu(hit.objectId, x, y);
      if (actor !== null && hit.ground && inLot(hit.ground.x, hit.ground.z)) {
        bridge.send({ type: 'moveTo', sim: actor, x: hit.ground.x, z: hit.ground.z });
      } else if (!direct) controls.inspect(null);
    },
    hover(x, y) {
      const hit = renderer.pick(x, y);
      const g = hit.ground;
      renderer.setHoverTile(g && inLot(g.x, g.z) && !game.pauseMenu ? { x: Math.floor(g.x), z: Math.floor(g.z) } : null);
      if (game.mode === 'build') buildBuy.hover(g && inLot(g.x, g.z) ? g : null, hit.objectId, { x, y });
    },
    leave() {
      renderer.setHoverTile(null);
      buildBuy.clearPreviews();
    },
    // Wall and Remove tools: press, drag, release (see `BuildBuyInput.press`).
    press(x, y) {
      // The eyedropper picks with a plain click.
      if (game.pauseMenu || game.mode !== 'build' || buildBuy.picking()) return false;
      const hit = renderer.pick(x, y);
      const g = hit.ground;
      return buildBuy.press(g && inLot(g.x, g.z) ? g : null, { x, y });
    },
    release(x, y, moved) {
      const g = renderer.pick(x, y).ground;
      buildBuy.release(g && inLot(g.x, g.z) ? g : null, moved);
    },
    cancel: () => buildBuy.cancelDrawing(),
    abort: () => buildBuy.abortPress(),
  });

  /** Puts what the eyedropper picked up in hand: a build tool with its look, or another object. */
  function usePicked(p: Picked | null) {
    game.eyedropper = false;
    game.pickHint = null;
    if (!p) {
      play('error');
      return toast('Nothing to pick up there. Point at a wall, a floor or something you can buy.');
    }
    play('pick');
    if (p.kind === 'tool') return controls.setBuildTool(p.tool);
    controls.startPlacing(p.def);
    if (game.placing) game.placing = { ...game.placing, rot: p.rot, turn: p.turn, colors: p.colors, size: p.size, tall: p.tall };
  }

  const onKey = (e: KeyboardEvent) => {
    const t = e.target;
    if (t instanceof HTMLInputElement || t instanceof HTMLTextAreaElement || t instanceof HTMLSelectElement || (t instanceof HTMLElement && t.isContentEditable)) return;
    // Behind the crash screen only a debug report (F8) is taken.
    if (app.crash && e.key !== 'F8') return;
    // A held key repeats only what's worth repeating (turning, storeys, undo and redo); toggles happen once.
    if (e.repeat && !REPEATING.has(e.key)) {
      if (e.key === ' ' || e.key === 'Tab' || e.key === 'F3' || e.key === 'F8') e.preventDefault();
      return;
    }
    // The colour studio: its keys are its own (Escape closes it, as Cancel).
    if (game.customising) {
      if (e.key === 'Escape' && !game.pauseMenu) game.customising = null;
      return;
    }
    // Creation: its keys are its own (Escape goes back to Live).
    if (game.mode === 'create') {
      if (e.key === 'Escape' && !game.pauseMenu) controls.setMode('live');
      return;
    }
    if (e.key === 'Escape') {
      if (game.townOpen) return void (game.townOpen = false);
      if (game.jobBoardOpen) return void (game.jobBoardOpen = false);
      if (game.eyedropper) return controls.toggleEyedropper();
      // Step back: drop what's in hand, then leave Build mode.
      if (buildBuy.cancel()) return;
      if (game.mode !== 'live') return controls.setMode('live');
      if (game.menu || game.socialMenu) return controls.closeMenu();
      if (game.journalOpen) return void (game.journalOpen = false);
      if (game.follow !== null) return controls.follow(null);
      if (game.inspected !== null && liveSettings.directControl !== 'always') return controls.inspect(null);
      game.pauseMenu = !game.pauseMenu;
      return;
    }
    if (game.pauseMenu) return;
    if ((e.ctrlKey || e.metaKey) && (e.key === 'z' || e.key === 'Z')) {
      e.preventDefault();
      return e.shiftKey ? controls.redo() : controls.undo();
    }
    if (e.ctrlKey && (e.key === 'y' || e.key === 'Y')) {
      e.preventDefault();
      return controls.redo();
    }
    switch (e.key) {
      case ' ':
        // A button reached with the keyboard (a menu's arrow keys) is pressed with Space.
        if (t instanceof HTMLButtonElement && t !== clicked) return;
        e.preventDefault();
        return controls.togglePause();
      case '0':
      case '1':
      case '2':
      case '3':
      case '4':
      case '5':
        return controls.setSpeed(Number(e.key));
      case 'PageUp':
      case 'PageDown':
        e.preventDefault();
        return controls.setStorey(game.storey + (e.key === 'PageUp' ? 1 : -1));
      case 'f':
      case 'F':
        if (game.mode !== 'live') return;
        return controls.follow(game.follow === null ? (game.inspected ?? game.selected) : null);
      case 'h':
      case 'H':
        if (game.mode === 'live') controls.frameHouse();
        return;
      case 'j':
      case 'J':
        if (game.mode === 'live') controls.toggleJournal();
        return;
      case 'c':
      case 'C':
        // Construct: Build mode's walls, rooms and paint (again: back to Live). Not Ctrl/⌘+C: that copies.
        if (e.ctrlKey || e.metaKey) return;
        if (game.mode === 'build' && game.buildTab === 'build') return controls.setMode('live');
        return controls.setBuildTab('build');
      case 'o':
      case 'O':
        // Our home: Build mode's Home tab (again: back to Live).
        if (game.mode === 'build' && game.buildTab === 'home') return controls.setMode('live');
        return controls.setBuildTab('home');
      case 'p':
      case 'P':
        if (game.mode === 'live') {
          if (game.plannerOpen) game.plannerOpen = false;
          else controls.openPlanner();
        }
        return;
      case 'm':
      case 'M':
        if (game.mode === 'live') {
          game.townOpen = !game.townOpen;
          if (game.townOpen) game.journalOpen = false;
        }
        return;
      case 'w':
      case 'W':
        return controls.setWallMode(nextWallMode(game.wallMode));
      case 'F8':
        e.preventDefault();
        void saveDebugReport();
        return;
      case 'F3':
        e.preventDefault();
        game.perfOpen = !game.perfOpen;
        return;
      case 'r':
      case 'R':
        return controls.rotatePlacing(e.shiftKey);
      case 'e':
      case 'E':
        return controls.toggleEyedropper();
      case 'Delete':
      case 'Backspace':
        if (game.furnishing || buildBuy.stairsTool()) {
          const target = game.placing?.objectId ?? game.buySelection;
          if (target !== null) controls.sell(target);
        }
        return;
      // B toggles Build mode (V: straight to its catalog), L back to Live.
      case 'b':
      case 'B':
        return controls.setMode(game.mode === 'build' ? 'live' : 'build');
      case 'v':
      case 'V':
        game.buyFilter = null;
        return game.furnishing ? controls.setMode('live') : controls.setBuildTab('furnish');
      case 'l':
      case 'L':
        return controls.setMode('live');
      case 'Tab': {
        e.preventDefault();
        const ids = playerSims();
        if (!ids.length) return;
        const next = ids[(ids.indexOf(game.inspected ?? game.selected) + 1) % ids.length];
        controls.selectSim(next);
        controls.inspect(next);
        return;
      }
    }
  };

  const statsTimer = setInterval(() => {
    // GPU timing costs a little, so it runs only while the overlay is open.
    renderer.measureGpu(game.perfOpen);
    if (!game.perfOpen) return;
    game.stats = renderer.stats();
    game.threads = threadStats(game.stats, bridge.threadStats, voices.stats());
  }, STATS_INTERVAL_MS);

  /** This frame's interpolation state (for what is placed over the view once the camera has moved). */
  let shown: FrameState | null = null;
  renderer.run((now) => {
    if (failFrames) throw new Error('a frame failed on purpose (__crash)');
    bridge.sync(now);
    const frame = (shown = bridge.frame(now));
    renderer.update(frame);
    renderer.setHeldObject(game.placing?.objectId ?? buildBuy.heldId());
    buildBuy.syncHandles();
    if (!revealed) return;
    voices.update(frame);
    worldSound.update(frame);
    followStorey(now);
    if (game.watching) {
      // Menus and other modes end watching (opening them is input, but not all of it is ours).
      if (!canWatch() && !director.active) stopWatching();
      else director.update(now);
    } else if (canWatch() && ((watchAt > 0 && now >= watchAt) || now - lastInput > autoWatchDelay() * 1000)) {
      watchAt = 0;
      startWatching(now);
    }
    // After a big moment, back to the speed the player had (unless they picked another).
    if (slowedFrom !== null && now > slowUntil) {
      if (game.speed === 1 && game.mode === 'live') bridge.send({ type: 'setSpeed', speed: slowedFrom });
      slowedFrom = null;
    }
  }, () => {
    // Bubbles, subtitles and room tags: where the camera, moved for this frame, shows the heads and rooms.
    if (!revealed || !shown) return;
    bubbles.update(shown);
    if (roomOverlayOn()) roomOverlay.update();
  });

  const session: GameSession = {
    get household() {
      return household;
    },
    get alive() {
      return bridge.alive;
    },
    ready,
    async warm() {
      if (warmed) return;
      warmed = true;
      const frame = await perfTimeAsync('session: first frame', firstFrame);
      if (frame) await perfTimeAsync('session: warm game', () => renderer.warmGame(frame));
      else perfPhase('session: warm game (no frame)');
      await perfTimeAsync('session: warm frames', () => renderer.framesRendered(WARM_FRAMES));
      // A save's picture is drawn into its own target: compile its shaders now, not at the first autosave.
      await perfTimeAsync('session: warm thumbnail', () => renderer.captureThumbnail(THUMBNAIL.width, THUMBNAIL.height));
    },
    reveal() {
      if (revealed) return;
      revealed = true;
      // Prepared, then the simulation stopped before the game was shown.
      if (stoppedWith !== null) app.crashed({ kind: 'sim', detail: stoppedWith });
      // Nothing to lose yet: leaving the page in the first minute doesn't autosave.
      autosavedAt = performance.now();
      // A loaded debug report shows the view it was captured from.
      const pose = takePendingPose();
      if (pose) renderer.setCameraPose(pose);
      pointer = attachPointer();
      window.addEventListener('keydown', onKey);
      window.addEventListener('pointerdown', onInput, true);
      window.addEventListener('pointerdown', onPress, true);
      canvas.addEventListener('wheel', onInput, { capture: true, passive: true });
      window.addEventListener('keydown', onInput, true);
      window.addEventListener('pointermove', onMove, { passive: true });
      lastInput = performance.now();
      // The camera starts watching once it has flown in (unless watching on its own is off).
      if (liveSettings.directorDelay > 0) watchAt = lastInput + START_WATCH_MS;
      // (A benchmark runs at its own speed.)
      bridge.send({ type: 'setSpeed', speed: BENCH?.speed ?? resumeSpeed ?? 1 });
      // Nobody lives here yet, or there's no house (an empty lot): the game starts with
      // building (time stands still until Live).
      const home = homePlot();
      if (!game.occupied || (home !== null && !game.plots[home]?.house)) {
        game.speed = resumeSpeed ?? 1;
        // No house yet: the Build tab; a house nobody lives in: the catalog to furnish it.
        if (home !== null && !game.plots[home]?.house) controls.setBuildTool(game.buildTool);
        else controls.setBuildTab('furnish');
      }
      perfPhase(`session: reveal ${game.mode} ${game.buildTab} storey ${game.storey} walls ${game.wallMode}`);
      // The house is cached by now: stream the rest of the town while the player is in it. Its
      // models are built into the world once the stream is over (each settle used to rebuild it).
      void (async () => {
        await homeAssets;
        if (disposed) return;
        renderer.holdModelRefresh(true);
        prefetch(restUrls);
        // The catalog's pictures are built only once the stream is quiet, in idle moments.
        await whenStreamSettled();
        renderer.holdModelRefresh(false);
        if (disposed || BENCH) return;
        prefetchTimer = setTimeout(() => {
          // (Not while benchmarking: drawing the pictures would show in the measurements.)
          if (disposed || BENCH) return;
          services.items.prefetch(content.shop.map((def) => ({ model: def.model, footprint: def.footprint ?? [1, 1] })));
        }, PREFETCH_PICTURES_MS);
      })();
    },
    save(name, slotId) {
      return store(name, slotId ?? crypto.randomUUID(), false);
    },
    saveAutosave: storeAutosave,
    applySettings(s) {
      renderer.configure({
        resolution: s.resolution,
        cameraSensitivity: s.cameraSensitivity,
        fpsCap: s.fpsCap,
        shading: s.shading,
        reducedMotion: s.reducedMotion || matchMedia('(prefers-reduced-motion: reduce)').matches,
      });
      if (s.autonomy !== autonomy) bridge.send({ type: 'setAutonomy', enabled: (autonomy = s.autonomy) });
      bridge.send({ type: 'setAutoFast', enabled: s.skipQuietHours });
      if (s.showFps !== showFps) game.perfOpen = showFps = s.showFps;
      if (s.autosaveMinutes !== autosaveMinutes) {
        autosaveMinutes = s.autosaveMinutes;
        clearInterval(autosaveTimer);
        if (autosaveMinutes > 0) {
          autosaveTimer = setInterval(() => {
            if (!game.pauseMenu) void autosave();
          }, autosaveMinutes * 60_000);
        }
      }
    },
    setMenuPause(paused) {
      if (paused && speedBeforeMenu === null) {
        speedBeforeMenu = game.speed;
        bridge.send({ type: 'setSpeed', speed: 0 });
      } else if (!paused && speedBeforeMenu !== null) {
        bridge.send({ type: 'setSpeed', speed: speedBeforeMenu });
        speedBeforeMenu = null;
      }
    },
    async dispose() {
      disposed = true;
      ideas.clear();
      clearInterval(statsTimer);
      clearInterval(autosaveTimer);
      clearTimeout(prefetchTimer);
      window.removeEventListener('keydown', onKey);
      document.removeEventListener('visibilitychange', onVisibility);
      window.removeEventListener('pagehide', onLeave);
      window.removeEventListener('pointerdown', onInput, true);
      window.removeEventListener('pointerdown', onPress, true);
      canvas.removeEventListener('wheel', onInput, true);
      window.removeEventListener('keydown', onInput, true);
      window.removeEventListener('pointermove', onMove);
      director.stop();
      clearTimeout(hudTimer);
      renderer.setGameShot(null);
      renderer.followSim(null);
      services.previews.keep(false);
      pointer?.dispose();
      bubbles.dispose();
      roomOverlay.dispose();
      voices.dispose();
      worldSound.dispose();
      buildBuy.dispose();
      await building;
      renderer.clear();
      bridge.dispose();
    },
  };
  session.applySettings(settings);
  if (import.meta.env.DEV) {
    // Test hook for automated browser checks; not present in production builds.
    window.__osw = {
      simScreen(id: number) {
        const head = { x: 0, y: 0, z: 0 };
        const out = { x: 0, y: 0 };
        return renderer.simHead(id, head) && renderer.project(head.x, 0.9, head.z, out) ? out : null;
      },
      focusSim(id: number) {
        const head = { x: 0, y: 0, z: 0 };
        if (renderer.simHead(id, head)) renderer.focus(head.x, head.z);
      },
      stats: () => renderer.stats(),
      view: () => viewPlot,
      project(x: number, y: number, z: number) {
        const out = { x: 0, y: 0 };
        return renderer.project(x, y, z, out) ? out : null;
      },
      home: () => buildBuy.home(),
      /** Sim-clock minute of the newest snapshot (changes when one arrives). */
      snapshotMinute: () => bridge.latest()[bridge.layout.header.minute],
      renderer,
      structure: () => ({ fences: world?.fences?.map((f) => `${f.kind}:${f.axis}:${f.x}:${f.z}:${f.style ?? 0}`) ?? [], walls: world?.walls?.length ?? 0, openings: world?.openings?.map((o) => o.kind) ?? [], diagonals: world?.diagonals?.map((d) => `${d.axis}:${d.x}:${d.z}:${d.kind}`) ?? [], funds: game.funds }),
      townFile: () => ('lot' in source ? source.lot : null),
      /** Sends a simulation command (benchmarks and checks). */
      send: (command: Parameters<typeof bridge.send>[0]) => bridge.send(command),
      /** The world sounds heard on the last update (loops by residents and things, sounds played once). */
      sound: () => worldSound.playing(),
      /** What a use sounds like (`soundOf`), and what it plays as it ends: by object and use id; `anim` and `recipe` (an id) for a cooking step at it. */
      soundOf(object: string, use: string | null, anim?: string, recipe?: string) {
        const def = content.objectList.findIndex((o) => o.id === object);
        const it = def >= 0 && use !== null ? content.objectList[def].interactions.findIndex((i) => i.id === use) : -1;
        const tag = anim ?? (it >= 0 ? (content.objectList[def].interactions[it].anim ?? '') : '');
        const cooking = recipe === undefined ? 0 : content.recipes.findIndex((r) => r.id === recipe) + 1;
        return { loop: soundOf(content, tag, def, it, cooking) ?? null, end: endSoundOf(content, def, it) ?? null, ambient: (def >= 0 && content.objectList[def].ambient) || null };
      },
      save: (id: string) => session.save(id, id),
      autosave: () => autosave(),
      plan: (id: number) => game.sims.find((x) => x.id === id)?.plan ?? null,
      objects: () => game.objects.map((o) => ({ id: o.id, def: o.def, x: o.x, z: o.z, rot: o.rot, turn: o.turn ?? 0, offset: o.offset ?? [0, 0] })),
      steps: () => ({ undo: game.undoSteps, redo: game.redoSteps }),
      watch: () => ({ mode: game.mode, watching: game.watching }),
      /** Doors, windows and wall looks (for checks that point at them). */
      openings: () => world?.openings ?? [],
      wallLooks: () => (world?.walls ?? []).filter((w) => w.faces || w.form),
      floors: () => world?.floors ?? [],
      build: () => ({ mode: game.mode, tab: game.buildTab, tool: game.buildTool, look: { ...game.buildLook }, placing: game.placing, eyedropper: game.eyedropper, hint: game.pickHint }),
      sims: () =>
        game.sims.map((x) => {
          const snap = bridge.latest();
          const o = bridge.layout.headerLen + x.id * bridge.layout.simStride;
          const pos = [snap[o + bridge.layout.sim.x].toFixed(1), snap[o + bridge.layout.sim.z].toFixed(1)];
          return { id: x.id, name: x.name, plot: x.plot, pos, away: x.awayUntil, visiting: x.visiting, job: x.job?.title, doing: x.actions[0]?.label };
        }),
    };
  }
  return session;
}
