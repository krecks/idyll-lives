/** The median of the finite numbers in `xs` (the mean of the middle two for an even count), to one decimal; null for none. */
export function median(xs) {
  const s = xs.filter(Number.isFinite).sort((a, b) => a - b);
  const n = s.length;
  if (!n) return null;
  return +(n % 2 ? s[(n - 1) / 2] : (s[n / 2 - 1] + s[n / 2]) / 2).toFixed(1);
}
