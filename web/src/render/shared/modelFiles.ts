/**
 * A renderer's model files as they load, engine-neutral (each engine's `models.ts` keeps one per
 * scene or app, `F` being the file as it reads it): each file imported once, a failed import not
 * kept (so each build tries it again, none held back or failed at once because of an earlier
 * failure), and tried again in the background after it fails. Whoever drew something without it
 * (a box, or nothing) is told when it comes (`onArrived`), and builds it again.
 */

import { background } from '../../assets/stream';

/**
 * After a file fails to load, how long (ms) until it's tried again in the background: each try that
 * fails waits for the next, and after the last nothing more is tried (until the browser comes back
 * online, when it starts over). Builds meanwhile still try it themselves, as ever.
 */
const RETRY_AFTER_MS = [5_000, 30_000, 120_000];

export class ModelFiles<F> {
  /** Imported (or being imported), by URL. A failed one is dropped, so the next build tries it again. */
  private readonly files = new Map<string, Promise<F>>();
  /** Failed to load, by URL: how many background tries have been made, and the next one's timer. */
  private readonly failed = new Map<string, { tries: number; timer?: ReturnType<typeof setTimeout> }>();
  /** Told when a file that failed has come (`onArrived`). */
  private readonly arrived = new Set<(url: string) => void>();

  /**
   * `importFile` imports a file (`now`: a build waits for it; else a background try, in the slot it
   * was given); `gone` says the scene or app it loads into has gone.
   */
  constructor(
    private readonly importFile: (url: string, now: boolean) => Promise<F>,
    private readonly gone: () => boolean,
  ) {
    window.addEventListener('online', this.online);
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
    window.removeEventListener('online', this.online);
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
      // (Both outcomes are handled here; whoever asked handles the rejection it's given.)
      loading.then(
        () => this.fileArrived(url),
        () => this.files.get(url) === loading && this.files.delete(url),
      );
    }
    file.catch(() => {
      if (!this.failed.has(url)) this.failed.set(url, { tries: 0 });
      this.scheduleRetry(url);
    });
    return file;
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
