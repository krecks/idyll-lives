/** Top-level navigation: which screen is shown, and which modal is open on top of it. */

import type { GameKind, Lifespan } from '../core/protocol';
import type { HouseholdDraft } from '../game/household';
import type { StartRequest } from '../game/session';
import { EMPTY_LOT, withLot, type NeighbourhoodDraft } from '../game/town';

/**
 * New game: world → neighbourhood (choose a household) → household (Creation) → home → game; or
 * world → neighbourhood → play (take over a household that already lives there) → game.
 */
export type Screen = 'menu' | 'world' | 'neighbourhood' | 'create' | 'home' | 'play' | 'game';
export type Overlay = 'settings' | 'load' | 'credits' | null;
/** A loading step and the overall progress, 0..1. */
export interface Progress {
  label: string;
  value: number;
}

/**
 * Why the game can't go on (the crash screen): the simulation stopped (`sim`), drawing broke
 * (`render`), the GPU was reset (`graphics`), the interface failed (`interface`), or the game's
 * data didn't load (`data`). `detail`: what went wrong, for the curious and for reports.
 */
export interface Crash {
  kind: 'sim' | 'render' | 'graphics' | 'interface' | 'data';
  detail: string;
}

class AppState {
  screen = $state<Screen>('menu');
  overlay = $state<Overlay>(null);
  /** What the game screen should start; bumping `sessionKey` restarts it. */
  request = $state.raw<StartRequest | null>(null);
  sessionKey = $state(0);
  dataReady = $state(false);
  fatal = $state('');
  /** The game can't go on: the crash screen shows (the first reason is kept). */
  crash = $state.raw<Crash | null>(null);
  /** A file of this version couldn't be fetched (most likely a new version was published meanwhile): a banner offers to reload. */
  newVersion = $state(false);
  /** Drafts kept while moving back and forth through the new-game steps: the world (nobody living there yet), then the town with its neighbours. */
  world = $state.raw<NeighbourhoodDraft | null>(null);
  town = $state.raw<NeighbourhoodDraft | null>(null);
  household = $state.raw<HouseholdDraft | null>(null);
  /** Living (the residents earn the money) or Creative (building is free). */
  mode = $state<GameKind>('living');
  /** How fast residents age (the new game's choice). */
  lifespan = $state<Lifespan>('normal');
  /** The home is an empty lot (its house cleared away) rather than the house as it stands. */
  emptyLot = $state(false);
  /** House picked on the home screen (prepared in the background). */
  homeSlot = $state<number | null>(null);
  /** Home of the neighbour household picked to play (prepared in the background). */
  playSlot = $state<number | null>(null);
  /** A live 3D scene is rendering behind the menus. */
  liveBackdrop = $state(false);
  /**
   * Start-up: everything the menus need is loaded first (game data, renderer, the menu's town)
   * behind a loading screen; the menu shows after (and prepares the latest save behind it).
   */
  booting = $state(true);
  boot = $state.raw<Progress>({ label: 'Loading', value: 0 });
  /** The loading screen over the menus or the game (null: none): a game being started (Play, Continue; one prepared unseen behind the menus shows none). */
  loading = $state.raw<Progress | null>(null);

  newGame(): void {
    this.world = null;
    this.town = null;
    this.household = null;
    this.playSlot = null;
    this.emptyLot = false;
    this.screen = 'world';
  }

  /** The town as the home screen shows it: the chosen lot cleared when starting on an empty lot. */
  homeTown(slot: number): NeighbourhoodDraft | null {
    const town = this.town;
    if (!town) return null;
    return this.emptyLot ? withLot(town, slot, EMPTY_LOT) : town;
  }

  /** The new game the home screen starts on `slot` (also prepared in the background). */
  homeRequest(slot: number): StartRequest | null {
    const town = this.homeTown(slot);
    const household = this.household;
    if (!town || !household) return null;
    return { kind: 'new', town, household, slot, mode: this.mode, lifespan: this.lifespan };
  }

  /** The new game that takes over the household already living on `slot` (also prepared in the background). */
  playRequest(slot: number): StartRequest | null {
    const town = this.town;
    const household = town?.households.find((h) => h.slot === slot)?.household;
    if (!town || !household) return null;
    return { kind: 'new', town, household, slot, existing: true, mode: this.mode, lifespan: this.lifespan };
  }

  start(request: StartRequest): void {
    this.request = request;
    this.overlay = null;
    this.sessionKey++;
    this.screen = 'game';
  }

  toMenu(): void {
    this.overlay = null;
    this.request = null;
    this.screen = 'menu';
  }

  /** Shows the crash screen (unless it already shows for an earlier reason). */
  crashed(crash: Crash): void {
    this.crash ??= crash;
  }
}

export const app = new AppState();
