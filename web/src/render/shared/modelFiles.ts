/**
 * A renderer's model files as they load, engine-neutral (each engine's `models.ts` keeps one per
 * scene or app, `F` being the file as it reads it): each file imported once, a failed import not
 * kept (so each build tries it again, none held back or failed at once because of an earlier
 * failure), and tried again in the background after it fails. Whoever drew something without it
 * (a box, or nothing) is told when it comes (`onArrived`), and builds it again.
 *
 * A file no built model uses any more (`use`, `release`) is kept a while for the next build of it,
 * among the last `idleLimit` such files; an older one is let go of (`evict`: its decoded geometry,
 * and whatever the engine made of it), and imported again if it's wanted again.
 */

import { background } from '../../assets/stream';

/**
 * After a file fails to load, how long (ms) until it's tried again in the background: each try that
 * fails waits for the next, and after the last nothing more is tried (until the browser comes back
 * online, when it starts over). Builds meanwhile still try it themselves, as ever.
 */
const RETRY_AFTER_MS = [5_000, 30_000, 120_000];

/** Files kept that no built model uses (the most recently wanted). */
const IDLE_FILES = 96;

export class ModelFiles<F> {
  /** Imported (or being imported), by URL. A failed one is dropped, so the next build tries it again. */
  private readonly files = new Map<string, Promise<F>>();
  /** Failed to load, by URL: how many background tries have been made, and the next one's timer. */
  private readonly failed = new Map<string, { tries: number; timer?: ReturnType<typeof setTimeout> }>();
  /** Told when a file that failed has come (`onArrived`). */
  private readonly arrived = new Set<(url: string) => void>();
  /** How many built models use each file (`use`, `release`). */
  private readonly users = new Map<string, number>();
  /** Files loaded that nothing uses, the least recently wanted first. */
  private readonly idle = new Set<string>();
  /** Imports under way (`settled`). */
  private readonly loading = new Set<Promise<F>>();
  /** A `trim` is due (after the builds waiting for files that just came have taken them). */
  private trimDue = false;

  /**
   * `importFile` imports a file (`now`: a build waits for it; else a background try, in the slot it
   * was given); `gone` says the scene or app it loads into has gone; `evict` lets go of a file nothing
   * has used for a while (none: files are kept for good).
   */
  constructor(
    private readonly importFile: (url: string, now: boolean) => Promise<F>,
    private readonly gone: () => boolean,
    private evict: ((file: F, url: string) => void) | null = null,
    private readonly idleLimit = IDLE_FILES,
  ) {
    if (typeof window !== 'undefined') window.addEventListener('online', this.online);
  }

  /** Back online: what failed (in an outage, most likely) is tried again from the start. */
  private readonly online = () => {
    for (const [url, f] of this.failed) {
      clearTimeout(f.timer);
      f.timer = undefined;
      f.tries = 0;
      this.scheduleRetry(url);
    }
  };

  /** The scene or app has gone: no more tries, nobody to tell. */
  dispose(): void {
    this.evict = null;
    if (typeof window !== 'undefined') window.removeEventListener('online', this.online);
    for (const f of this.failed.values()) clearTimeout(f.timer);
    this.failed.clear();
    this.arrived.clear();
  }

  /**
   * The file at `url`, imported once. Only the import counts as the file failing, not a build from
   * it: a file that loads but can't be built isn't fetched again. A background try imports in the
   * slot it was given (`now` false), not as a load someone waits for; a build that wants the file
   * meanwhile shares the download already under way.
   */
  load(url: string, now = true): Promise<F> {
    let file = this.files.get(url);
    if (!file) {
      const loading = this.importFile(url, now);
      this.files.set(url, (file = loading));
      this.loading.add(loading);
      // (Both outcomes are handled here; whoever asked handles the rejection it's given.)
      loading.then(
        () => {
          this.loading.delete(loading);
          if (this.files.get(url) === loading) this.fileArrived(url);
        },
        () => {
          this.loading.delete(loading);
          if (this.files.get(url) === loading) this.files.delete(url);
        },
      );
    }
    // Wanted again: the last of the idle files to go.
    if (this.idle.delete(url)) this.idle.add(url);
    file.catch(() => {
      if (!this.failed.has(url)) this.failed.set(url, { tries: 0 });
      this.scheduleRetry(url);
    });
    return file;
  }

  /** Resolves once no import is under way (those started meanwhile too); failures count as done. */
  async settled(): Promise<void> {
    while (this.loading.size) await Promise.allSettled([...this.loading]);
  }

  /** Whether the file at `url` is imported (or being imported) and kept. */
  has(url: string): boolean {
    return this.files.has(url);
  }

  /** A model built from the file at `url` is in use (until `release`). */
  use(url: string): void {
    this.users.set(url, (this.users.get(url) ?? 0) + 1);
    this.idle.delete(url);
  }

  /** A model built from the file at `url` has gone: the file is kept a while, then let go of once nothing uses it. */
  release(url: string): void {
    const left = (this.users.get(url) ?? 0) - 1;
    if (left > 0) {
      this.users.set(url, left);
      return;
    }
    this.users.delete(url);
    if (this.files.has(url)) this.idle.add(url);
    this.trimSoon();
  }

  /**
   * Lets go of the least recently wanted idle files past the limit, in a task of its own: a build
   * whose file just came takes it (`use`) in the promise callbacks before that, so it's never let go
   * of between coming and being used.
   */
  private trimSoon(): void {
    if (!this.evict || this.trimDue) return;
    this.trimDue = true;
    setTimeout(() => {
      this.trimDue = false;
      this.trim();
    }, 0);
  }

  private trim(): void {
    while (this.idle.size > this.idleLimit) {
      const url = this.idle.values().next().value!;
      this.idle.delete(url);
      const file = this.files.get(url);
      this.files.delete(url);
      // (Its import has settled: only one that loaded is idle.)
      void file?.then(
        (f) => this.evict?.(f, url),
        () => {},
      );
    }
  }

  /**
   * Calls `fn` with a file's URL when it loads after failing to (in a background try, or for any
   * build): whatever was built without it (a box, or nothing) can be built again. Returns what stops
   * it. Files nobody listens for aren't tried in the background.
   */
  onArrived(fn: (url: string) => void): () => void {
    this.arrived.add(fn);
    return () => this.arrived.delete(fn);
  }

  /** Tells the listeners again that the file at `url` has come (a template still being built when it came turned out to be built without it). */
  retell(url: string): void {
    this.tell(url);
  }

  /** A file loaded: one that had failed isn't tried in the background any more, and the listeners are told. */
  private fileArrived(url: string): void {
    // Nothing built from it yet (imported ahead of time, or for a build still to come): idle until something is.
    if (!this.users.has(url)) {
      this.idle.add(url);
      this.trimSoon();
    }
    const f = this.failed.get(url);
    if (!f) return;
    clearTimeout(f.timer);
    this.failed.delete(url);
    this.tell(url);
  }

  private tell(url: string): void {
    for (const fn of [...this.arrived]) {
      try {
        fn(url);
      } catch (err) {
        console.error(`[render] building again with ${url} failed`, err);
      }
    }
  }

  /** Sets the next background try of a failed file, unless one is set already or none are left. */
  private scheduleRetry(url: string): void {
    const f = this.failed.get(url);
    if (!f || f.timer !== undefined || f.tries >= RETRY_AFTER_MS.length || !this.arrived.size || this.gone()) return;
    // A load under way (a build's) is this try; if it fails, the next is set from here. Otherwise the
    // try loads the file itself.
    const sharedTry = () => {
      const loading = this.files.get(url);
      if (loading) loading.catch(() => this.scheduleRetry(url));
      return !!loading;
    };
    f.timer = setTimeout(() => {
      f.timer = undefined;
      f.tries++;
      if (this.gone() || !this.failed.has(url) || sharedTry()) return;
      // It waits its turn behind what's on screen; by then the file may have come, or a build be loading it.
      void background(async () => {
        if (this.gone() || !this.failed.has(url) || sharedTry()) return;
        // `load` notes the outcome (and sets the next try); the rejection itself is nobody's to handle.
        await this.load(url, false).catch(() => {});
      }).catch(() => {});
    }, RETRY_AFTER_MS[f.tries]);
  }
}
