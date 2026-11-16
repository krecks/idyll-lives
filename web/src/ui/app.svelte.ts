/** Top-level navigation: which screen is shown, and which modal is open on top of it. */

import type { GameKind, Lifespan } from '../core/protocol';
import type { HouseholdDraft } from '../game/household';
import type { MenuStill } from '../game/menuStill';
import type { Pt } from '../game/roads';
import type { StartRequest } from '../game/session';
import { EMPTY_LOT, withLot, type NeighbourhoodDraft } from '../game/town';
import type { Chosen } from './newGame';

/**
 * New game, in four steps: land (step 1) → town (step 2) → household (step 3: choose one of the
 * town's, or make your own in Creation, `create`) → home (step 4) → game. A household chosen from
 * the town can be met first (`play`; it goes once step 4 offers their home, U4).
 */
export type Screen = 'menu' | 'land' | 'town' | 'household' | 'create' | 'home' | 'play' | 'game';

/** A step's draft and the key of what it was made from (`newGame.ts` `landKey`, `townKey`): a later step whose key no longer matches is made again. */
export interface Keyed {
  draft: NeighbourhoodDraft;
  key: string;
}

/** Step 1's land: no town on it yet. `waterKept`: its water is as generated (the town laid on it may use its plan's river and coast). */
export interface MadeLand extends Keyed {
  waterKept: boolean;
}

/** Step 2's town, nobody living there yet. `edited`: changed by hand (the editor's tools): what lays it out again asks first. */
export interface LaidTown extends Keyed {
  edited: boolean;
}
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
  // What the new game's steps made, kept while the player goes back and forth through them. Each is
  // made again only when what it was made from changed (its key): a land shaped or chosen again
  // lays the town again; a town laid again is lived in again; a household made in Creation is kept.

  /** The world's code as chosen so far: step 1's place, land and seed, step 2's town, villages and town seed. Null until step 1 opens. */
  code = $state.raw<Chosen | null>(null);
  /** Step 1's land (with the player's shaping). */
  land = $state.raw<MadeLand | null>(null);
  /** Where the player moved the town (its box's corner, m; step 2's "Move the town"); null: where the land has it. */
  site = $state.raw<Pt | null>(null);
  /** The town's name as typed or rolled in step 2; null: the one it was laid with. */
  townName = $state<string | null>(null);
  /** Step 2's town (nobody living there yet). */
  world = $state.raw<LaidTown | null>(null);
  /** Step 3's town with its neighbours, and the key of step 2's town it was filled from (`world.key`). */
  town = $state.raw<NeighbourhoodDraft | null>(null);
  townOf = '';
  /** The households step 3 offers (their lots, in order); empty until step 3 offers a sample of the town's. */
  offer = $state.raw<readonly number[]>([]);
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
  /** The main menu's picture (the last world made, `menuStill.ts`); null: the painted backdrop only. */
  menuStill = $state.raw<MenuStill | null>(null);
  /**
   * Start-up: everything the menus need is loaded first (game data, renderer, the menu's picture)
   * behind a loading screen; the menu shows after (and prepares the latest save behind it).
   */
  booting = $state(true);
  boot = $state.raw<Progress>({ label: 'Loading', value: 0 });
  /** The loading screen over the menus or the game (null: none): a game being started (Play, Continue; one prepared unseen behind the menus shows none). */
  loading = $state.raw<Progress | null>(null);

  newGame(): void {
    this.code = null;
    this.land = null;
    this.site = null;
    this.townName = null;
    this.world = null;
    this.town = null;
    this.townOf = '';
    this.offer = [];
    this.household = null;
    this.playSlot = null;
    this.homeSlot = null;
    this.emptyLot = false;
    this.screen = 'land';
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

  /** The main menu's picture from now on (the one it replaces let go once the menu has swapped it). */
  setStill(still: MenuStill): void {
    const old = this.menuStill;
    this.menuStill = still;
    if (old?.url.startsWith('blob:') && old.url !== still.url) setTimeout(() => URL.revokeObjectURL(old.url), 2000);
  }

  /** Shows the crash screen (unless it already shows for an earlier reason). */
  crashed(crash: Crash): void {
    this.crash ??= crash;
  }
}

export const app = new AppState();
