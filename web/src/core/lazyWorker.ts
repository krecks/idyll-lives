/**
 * A worker started when it is first asked something and stopped once it has had nothing to do for a
 * while, its answers matched to its asks by id (`townJobs.ts`, `renderData.ts`). Where no worker can
 * start, or once one broke (its `onerror`, or an answer saying it is `broken`), every ask goes to the
 * fallback its caller gave (the page does the work itself): those waiting then, and all later ones.
 */

/** What the worker is sent: `message`, and the id its answer is to carry (0: no answer wanted, `post`). */
export interface WorkerAsk<In> {
  id: number;
  message: In;
}

/** What the worker answers: `answer`, or the error it met (`broken`: it can answer nothing, every ask goes to the page). */
export type WorkerAnswer<Out> = { id: number; answer: Out } | { id: number; error: string; broken?: boolean };

/** Raised for an ask whose signal aborted it (a newer one replaced it). */
export class Superseded extends Error {
  constructor() {
    super('superseded');
    this.name = 'Superseded';
  }
}

interface Pending<In, Out> {
  message: In;
  transfer: Transferable[];
  fallback: () => Promise<Out>;
  resolve: (answer: Out) => void;
  reject: (err: unknown) => void;
}

export interface LazyWorkerOptions {
  /** Starts the worker (`new Worker(new URL(…, import.meta.url))`, written where the bundler sees it). */
  start: () => Worker;
  /** It goes once it has had nothing to do (no ask waiting, nothing held) for this long. */
  idleMs: number;
  /** Told once, why it broke (for the console). */
  broke: (why: string) => void;
}

export class LazyWorker<In, Out> {
  private worker: Worker | null = null;
  /** No worker can run here (or one broke): asks go to their fallbacks. */
  private broken = false;
  /** Workers started so far: tells the one running from one before it (`run`). */
  private starts = 0;
  private nextId = 1;
  /** Asks sent and not yet answered, in the order they were sent. */
  private readonly pending = new Map<number, Pending<In, unknown>>();
  /** `hold`s not yet let go of. */
  private holds = 0;
  private idleTimer: ReturnType<typeof setTimeout> | undefined;

  constructor(private readonly options: LazyWorkerOptions) {}

  /** The worker running (a number of its own, from 1), or 0 when none is. */
  get run(): number {
    return this.worker ? this.starts : 0;
  }

  /**
   * `message`'s answer from the worker (started if need be), or `fallback`'s where no worker can
   * give it (none can run, or it broke first); rejected with the error the worker met. An aborted
   * `signal` rejects it with `Superseded`: for a worker that takes its asks in turn (the town
   * worker), one it is on stops it, and those waiting behind go to a fresh one.
   */
  ask<F = Out>(message: In, fallback: () => Promise<F>, { signal, transfer = [] }: { signal?: AbortSignal; transfer?: Transferable[] } = {}): Promise<Out | F> {
    if (signal?.aborted) return Promise.reject(new Superseded());
    return new Promise<Out | F>((resolve, reject) => {
      const id = this.nextId++;
      const p: Pending<In, Out | F> = { message, transfer, fallback, resolve, reject };
      this.pending.set(id, p as Pending<In, unknown>);
      signal?.addEventListener(
        'abort',
        () => {
          // (The worker takes asks in turn: the first waiting is the one it is on.)
          const running = this.pending.keys().next().value === id;
          if (!this.pending.delete(id)) return;
          reject(new Superseded());
          // Stopping the worker is the only way to stop the ask it is on (a fresh one starts over).
          // One still waiting is answered and thrown away.
          if (!running || !this.worker) return;
          this.stop();
          for (const [other, q] of [...this.pending]) this.send(other, q);
        },
        { once: true },
      );
      this.send(id, p as Pending<In, unknown>);
    });
  }

  /** Sends `message` with no answer wanted (the worker started if need be): the `run` it went to, 0 if none can run. */
  post(message: In, transfer: Transferable[] = []): number {
    const w = this.start();
    if (!w) return 0;
    clearTimeout(this.idleTimer);
    w.postMessage({ id: 0, message } satisfies WorkerAsk<In>, transfer);
    return this.starts;
  }

  /** Keeps the worker running (it is not stopped for being idle) until the returned function is called. */
  hold(): () => void {
    this.holds++;
    clearTimeout(this.idleTimer);
    let held = true;
    return () => {
      if (!held) return;
      held = false;
      this.holds--;
      this.idleSoon();
    };
  }

  /** Hands `p` to the worker, or to its fallback when there can be none. */
  private send(id: number, p: Pending<In, unknown>): void {
    const w = this.start();
    if (!w) {
      this.pending.delete(id);
      p.fallback().then(p.resolve, p.reject);
      return;
    }
    clearTimeout(this.idleTimer);
    w.postMessage({ id, message: p.message } satisfies WorkerAsk<In>, p.transfer);
  }

  private start(): Worker | null {
    if (this.worker || this.broken) return this.worker;
    let w: Worker;
    try {
      w = this.options.start();
    } catch {
      this.broken = true;
      return null;
    }
    this.worker = w;
    this.starts++;
    w.onmessage = (e: MessageEvent<WorkerAnswer<Out>>) => {
      const a = e.data;
      if ('error' in a && a.broken) return this.break(a.error);
      const p = this.pending.get(a.id);
      this.pending.delete(a.id);
      if (p) {
        if ('error' in a) p.reject(new Error(a.error));
        else p.resolve(a.answer);
      }
      this.idleSoon();
    };
    w.onerror = (e) => {
      e.preventDefault();
      this.break(e.message);
    };
    return w;
  }

  /** A worker that broke hands its asks to their fallbacks, and so do later ones. */
  private break(why: string): void {
    this.options.broke(why);
    this.broken = true;
    this.stop();
    for (const [id, p] of [...this.pending]) this.send(id, p);
  }

  private stop(): void {
    this.worker?.terminate();
    this.worker = null;
    clearTimeout(this.idleTimer);
  }

  private idleSoon(): void {
    if (this.pending.size || this.holds) return;
    clearTimeout(this.idleTimer);
    this.idleTimer = setTimeout(() => {
      if (!this.pending.size && !this.holds) this.stop();
    }, this.options.idleMs);
  }
}
