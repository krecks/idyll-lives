/**
 * `SimPreviews`: the household-creator stage and Sim portraits, from one small PlayCanvas app
 * (`studio.ts`, beside the game's) that runs only while it has work: a stage attached, or portraits
 * to draw. It is destroyed a few seconds after the last of either, except while a game keeps it
 * (`keep`): then it only stops drawing, and new faces in play find its shaders compiled (portraits
 * are cached for the page).
 *
 * Portraits asked for ahead of time (`prefetch`: a whole town's) are drawn the first few at once
 * (the household's, who the caller puts first: behind the loading screen), the rest a few at a time
 * in idle moments, not a town's worth in the first seconds of play.
 */

import type { AssetRegistry } from '../../assets/registry';
import { loadCharacterSet } from '../shared/characters/data';
import type { SimLook, SimPreviews, SimStage, Wardrobe } from '../types';
import { cached, lookKey, Portraits } from './portraits';
import { StageController } from './stage';
import { Studio, type StudioDeps } from './studio';

/** Idle time (ms) before the app is released. */
const RELEASE_AFTER = 4000;
/** Of the looks `prefetch` is given, the first this many are drawn at once; the rest in idle moments. */
const PREFETCH_NOW = 8;
/** Portraits asked for per idle moment (one small batch, drawn before the next is asked for). */
const PER_IDLE = 4;
/** Longest wait (ms) for an idle moment: a page that never idles still gets its portraits, slowly. */
const IDLE_TIMEOUT = 2000;

export function createSimPreviews(deps: StudioDeps): SimPreviews {
  return new Previews(deps);
}

class Previews implements SimPreviews {
  private studio: Promise<Studio | null> | null = null;
  private live: Studio | null = null;
  private unavailable = false;
  private stageCtl: StageController | null = null;
  private readonly portraits = new Portraits();
  private looping = false;
  private releaseTimer = 0;
  private kept = false;
  /** Looks to draw in idle moments (`prefetch`), by look key, in the order asked. */
  private readonly ahead = new Map<string, SimLook>();
  private drawingAhead = false;

  constructor(private readonly deps: StudioDeps) {
    // `?debug`: inspect from the console / test harness (`__previews.live` is the studio, if any).
    if (new URLSearchParams(location.search).has('debug')) Object.assign(window, { __previews: this });
  }

  async stage(host: HTMLElement): Promise<SimStage | null> {
    const studio = await this.open();
    if (!studio) return null;
    if (!host.isConnected) {
      this.idleCheck();
      return null;
    }
    this.stageCtl?.detach();
    const ctl = new StageController(studio, host, () => {
      if (this.stageCtl === ctl) this.stageCtl = null;
      this.idleCheck();
    });
    this.stageCtl = ctl;
    this.loop(studio);
    return ctl;
  }

  portrait(look: SimLook): Promise<string | null> {
    const key = lookKey(look);
    const url = cached(key);
    if (url) return Promise.resolve(url);
    if (this.unavailable) return Promise.resolve(null);
    const p = this.portraits.request(key, look);
    void this.open().then((studio) => {
      if (studio) this.loop(studio);
      else this.portraits.failAll();
    });
    return p;
  }

  cachedPortrait(look: SimLook): string | null {
    return cached(lookKey(look));
  }

  prefetch(looks: readonly SimLook[]): void {
    looks.forEach((look, i) => {
      const key = lookKey(look);
      if (cached(key)) return;
      if (i < PREFETCH_NOW) {
        this.ahead.delete(key);
        void this.portrait(look);
      } else if (!this.ahead.has(key)) this.ahead.set(key, look);
    });
    void this.drawAhead();
  }

  /** Draws the looks `prefetch` left for later: a few per idle moment, once the portraits asked for before are drawn. */
  private async drawAhead(): Promise<void> {
    if (this.drawingAhead) return;
    this.drawingAhead = true;
    try {
      while (this.ahead.size && !this.unavailable) {
        await idle();
        if (this.portraits.busy) continue;
        const next = [...this.ahead].slice(0, PER_IDLE);
        for (const [key] of next) this.ahead.delete(key);
        await Promise.all(next.map(([, look]) => this.portrait(look)));
      }
    } finally {
      this.drawingAhead = false;
    }
  }

  keep(on: boolean): void {
    this.kept = on;
    if (!on) this.idleCheck();
  }

  async wardrobe(gender: string): Promise<Wardrobe | null> {
    const set = await loadSet(this.deps.assets);
    const body = set?.bodies.get(gender === 'female' ? 'female' : 'male') ?? set?.bodies.values().next().value;
    if (!body) return null;
    const parts = Object.keys(body.garments);
    return {
      tops: parts.filter((p) => p.startsWith('top.')),
      bottoms: parts.filter((p) => p.startsWith('bottom.')),
      shoes: parts.filter((p) => p.startsWith('shoes.')),
      beard: body.parts.has('hair.beard'),
    };
  }

  private open(): Promise<Studio | null> {
    clearTimeout(this.releaseTimer);
    if (this.unavailable) return Promise.resolve(null);
    this.studio ??= Studio.create(this.deps).then(
      (studio) => (this.live = studio),
      (err: unknown) => {
        console.warn('[preview] 3D Sim previews unavailable', err);
        this.unavailable = true;
        this.studio = null;
        return null;
      },
    );
    return this.studio;
  }

  private loop(studio: Studio): void {
    clearTimeout(this.releaseTimer);
    if (this.looping) return;
    this.looping = true;
    studio.run((now) => {
      const stage = this.stageCtl;
      stage?.direct(now);
      this.portraits.direct(studio, now);
      studio.pose(now);
      // One frame draws the stage (onto the canvas) and the portraits ready (into their targets).
      const portraits = this.portraits.prepare(studio, now);
      if (stage || portraits) studio.draw(now);
      this.portraits.collect(studio);
      if (!stage && !this.portraits.busy) {
        studio.stop();
        this.looping = false;
        this.idleCheck();
      }
    });
  }

  /** Releases the app once neither the stage nor portraits have needed it for a while. */
  private idleCheck(): void {
    clearTimeout(this.releaseTimer);
    this.releaseTimer = window.setTimeout(() => {
      const studio = this.live;
      if (!studio || this.kept || this.stageCtl || this.portraits.busy) return;
      this.live = null;
      this.studio = null;
      this.looping = false;
      this.portraits.dispose();
      studio.dispose();
    }, RELEASE_AFTER);
  }
}

/** The next idle moment (or a short wait where there are none). */
function idle(): Promise<void> {
  return new Promise((resolve) => {
    if (typeof requestIdleCallback === 'function') requestIdleCallback(() => resolve(), { timeout: IDLE_TIMEOUT });
    else setTimeout(resolve, 100);
  });
}

function loadSet(assets: AssetRegistry) {
  const entry = assets.get('model.sim', 'character');
  return entry ? loadCharacterSet(entry.url).catch(() => null) : Promise.resolve(null);
}
