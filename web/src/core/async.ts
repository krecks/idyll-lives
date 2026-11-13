/** Small helpers for waiting on promises. */

/** `promise`, or `fallback` if it takes longer than `ms` (the timer is cleared either way). */
export function within<T, F>(promise: Promise<T>, ms: number, fallback: F): Promise<T | F> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const late = new Promise<F>((resolve) => (timer = setTimeout(() => resolve(fallback), ms)));
  return Promise.race([promise, late]).finally(() => clearTimeout(timer));
}

/**
 * Both results, once both have settled; the first one's error if either failed. Unlike
 * `Promise.all`, a failure doesn't leave the other running on behind the caller's back (two builds
 * side by side: the next build must not start while one is still going).
 */
export async function both<A, B>(a: Promise<A>, b: Promise<B>): Promise<[A, B]> {
  const [ra, rb] = await Promise.allSettled([a, b]);
  if (ra.status === 'rejected') throw ra.reason;
  if (rb.status === 'rejected') throw rb.reason;
  return [ra.value, rb.value];
}
