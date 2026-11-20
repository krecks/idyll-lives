/**
 * A game session: starts the sim worker for a new or saved game and wires the (long-lived)
 * renderer, input and HUD to it. Sessions start hidden and paused — they can be prepared
 * in the background while the player is still in a menu — and go live with `reveal()`.
 */
import type { GameKind, Lifespan } from '../core/protocol';
import { type Settings } from '../settings/settings.svelte';
import { type HouseholdDraft } from './household';
import { type NeighbourhoodDraft } from './town';
import { SUPER_SPEED } from './speed';
declare global {
  interface Window {
    /** What the benches drive the game through (`tools/bench/scene.mjs`); set only in development builds. */
    __osw?: Record<string, unknown>;
  }
}
/** Click radius (CSS px) around a Sim's projected centre. */
export const SIM_PICK_RADIUS = 32;
export const STATS_INTERVAL_MS = 500;
/** After the player picks a storey, the view stays on it this long before following residents again. */
export const STOREY_STAY_MS = 8000;
/** The storey in view follows residents only while the camera's target is within this many metres of the viewed lot. */
export const STOREY_NEAR = 40;
export const THUMBNAIL = { width: 320, height: 200 };
/** Longest wait for a save's picture (it needs a drawn frame); the save then keeps the last one. */
export const THUMBNAIL_WAIT_MS = 1000;
/** Leaving the page (tab hidden, closed) autosaves at most this often. */
export const LEAVE_AUTOSAVE_MS = 60_000;
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
      /**
       * The people seed the town's households were drawn from (`people.ts` `peopleStreams`): the
       * neighbours' bonds and the simulation's own dice come from it too (the page's `?seed=` still
       * wins for the dice). Missing: both are random.
       */
      peopleSeed?: number;
      /**
       * The homes still for sale are filled on the way in (`ui/newGame.ts` `townOnMoveIn`): all but a
       * few, from the people seed's streams (step 4's Move in).
       */
      fill?: boolean;
    }
  | { kind: 'load'; saveId: string };
/** How long a story-feed entry stays visible. */
export const FEED_MS = 12_000;
/** Story-feed entries shown at once. */
export const FEED_SIZE = 3;
/** Live: the camera starts watching this long after a game is revealed. */
export const START_WATCH_MS = 2500;
/** Shift+R turns things that turn freely by this many degrees. */
export const FINE_TURN = 15;
/** At super speed the sim's full-detail area is only the chunks round the camera's (`Renderer.setDetailTight`). */
export const TIGHT_SPEED = SUPER_SPEED;
/** How long the interface stays after the mouse moves while it watches quietly. */
export const HUD_AWAKE_MS = 3000;
/** How long a big moment plays at normal speed before the game speeds up again. */
export const SLOW_MS = 25_000;
/** Frames drawn before a prepared session counts as ready (shaders compiled, textures up). */
export const WARM_FRAMES = 3;
/** Longest wait for the household's portraits before a prepared session counts as ready anyway. */
export const PORTRAIT_WAIT_MS = 8000;
/** Longest wait for the first snapshot (its residents' looks) before warming the game's draw. */
export const FIRST_FRAME_WAIT_MS = 1500;
/** After this long in the game, the catalog's pictures are drawn in idle moments (the catalog then opens with them). */
export const PREFETCH_PICTURES_MS = 4000;
/** Longest wait for the simulation compiled at boot (`compileSim`) before a game's worker compiles its own. */
export const SIM_COMPILE_WAIT_MS = 15_000;
/** Longest wait for the street to build the chunks round the camera before the warm-up's frame (a big world). */
export const STREAM_WAIT_MS = 4000;
/** Longest wait for the textures asked for to load before the warm-up's frame (it then uploads them). */
export const TEXTURES_WAIT_MS = 5000;
/** Story events the journal keeps: as many as the simulation does (`EVENT_LOG_CAPACITY`). */
export const JOURNAL_SIZE = 300;
/** Keys that act again while held (R turns, PageUp/PageDown change storey, Ctrl+Z/Y undo and redo); the rest act once per press. */
export const REPEATING = new Set(['r', 'R', 'PageUp', 'PageDown', 'z', 'Z', 'y', 'Y']);
/** Longest wait for a debug report's picture or GPU details (it goes without them). */
export const REPORT_PART_MS = 4000;
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
  /** Compiles and draws the game's own look (the mirror's clip-plane shaders too), if not done yet; `progress` hears its steps (0..1). */
  warm(progress?: ProgressReport): Promise<void>;
  /** Unpauses the game and attaches input. */
  reveal(): void;
  /** Ends the session; resolves once it has let go of the renderer, which is kept for the next one. */
  dispose(): Promise<void>;
}
/** Reports a loading step and the overall progress (0..1) while a session is prepared. */
export type ProgressReport = (label: string, value: number) => void;
