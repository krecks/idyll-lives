/** Top-level navigation: which screen is shown, and which modal is open on top of it. */

import type { GameKind, Lifespan } from '../core/protocol';
import type { TownHistory } from '../game/edit/history';
import type { HouseholdDraft } from '../game/household';
import type { MenuStill } from '../game/menuStill';
import type { Pt } from '../game/roads';
import type { StartRequest } from '../game/session';
import { EMPTY_LOT, withLot, type NeighbourhoodDraft } from '../game/town';
import { moveInRequest, peopleSeedOf, type Chosen } from './newGame';

/**
 * New game, in four steps: land (step 1) → town (step 2) → household (step 3: choose one of the
 * town's, or make your own in Creation, `create`) → home (step 4, on the whole world: a household
 * of the town's has its own home chosen, and may move) → game.
 */
export type Screen = 'menu' | 'land' | 'town' | 'household' | 'create' | 'home' | 'game';

/** A step's draft and the key of what it was made from (`newGame.ts` `landKey`, `townKey`): a later step whose key no longer matches is made again. */
export interface Keyed {
  draft: NeighbourhoodDraft;
  key: string;
}

/** Step 1's land: no town on it yet. `waterKept`: its water is as generated (the town laid on it may use its plan's river and coast). */
export interface MadeLand extends Keyed {
  waterKept: boolean;
}

/**
 * Step 2's town, nobody living there yet: `draft` is the town settled from the player's edits (the
 * town the next steps read). `edited`: changed by hand (the editor's tools): what lays it out again
 * asks first. Once edited, the editor keeps its `plan` (the draft its edits made, which `draft` is
 * settled from: ahead of it until edits pause) and its undo `history` here, so going back and forth
 * through the steps keeps them.
 */
export interface LaidTown extends Keyed {
  edited: boolean;
  plan?: NeighbourhoodDraft;
  history?: TownHistory;
  /** What in `draft` is over a slope limit (the town job's, then each settle's `marks`): step 2's next settle's `before`. */
  marks?: readonly string[];
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
  /** Step 3's town with its neighbours, and what it was filled from: step 2's town and the people's seed (`newGame.ts` `peopleKey`). */
  town = $state.raw<NeighbourhoodDraft | null>(null);
  townOf = '';
  /** The households step 3 offers (their lots, in order); empty until step 3 offers a sample of the town's. */
  offer = $state.raw<readonly number[]>([]);
  /** How many times step 3's "Show others" was pressed for `offer` (`newGame.ts` `offerPage`). */
  offerPage = $state(0);
  household = $state.raw<HouseholdDraft | null>(null);
  /** Living (the residents earn the money) or Creative (building is free). */
  mode = $state<GameKind>('living');
  /** How fast residents age (the new game's choice). */
  lifespan = $state<Lifespan>('normal');
  /** The home is an empty lot (its house cleared away) rather than the house as it stands. */
  emptyLot = $state(false);
  /** House picked on the home screen (prepared in the background). */
  homeSlot = $state<number | null>(null);
  /**
   * The household chosen in step 3 from those living in town, by its home's slot (step 4 opens on
   * that home); null: the player's own from Creation (`household`). It wins over `household`: a
   * household made in Creation is kept while the player looks through the town's.
   */
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
    this.offerPage = 0;
    this.household = null;
    this.playSlot = null;
    this.homeSlot = null;
    this.emptyLot = false;
    this.screen = 'land';
  }

  /** The seed the town's people come from (the code's own, else drawn from the town's); undefined before step 1. */
  get peopleSeed(): number | undefined {
    return this.code ? peopleSeedOf(this.code) : undefined;
  }

  /** The town as the home screen shows it: the chosen lot cleared when starting on an empty lot. */
  homeTown(slot: number): NeighbourhoodDraft | null {
    const town = this.town;
    if (!town) return null;
    return this.emptyLot ? withLot(town, slot, EMPTY_LOT) : town;
  }

  /** The home of the town's household chosen to play (`playSlot`), while one still lives there; null: the household is Creation's. */
  get residentSlot(): number | null {
    const slot = this.playSlot;
    return slot !== null && !!this.town?.households.some((h) => h.slot === slot) ? slot : null;
  }

  /** The household moving in: the town's chosen in step 3, else the one made in Creation. */
  get mover(): HouseholdDraft | null {
    const from = this.residentSlot;
    return from === null ? this.household : (this.town?.households.find((h) => h.slot === from)?.household ?? null);
  }

  /**
   * The new game Move in starts on `slot` (`newGame.ts` `moveInRequest`): the household taking over
   * its own home, or moving into this one; the rest of the homes for sale filled on the way in.
   */
  homeRequest(slot: number): StartRequest | null {
    const town = this.town;
    const household = this.mover;
    if (!town || !household) return null;
    return moveInRequest({ town, slot, household, from: this.residentSlot, emptyLot: this.emptyLot, mode: this.mode, lifespan: this.lifespan, ...(this.code ? { peopleSeed: peopleSeedOf(this.code) } : {}) });
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
