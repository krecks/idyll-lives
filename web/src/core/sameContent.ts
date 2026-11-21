/**
 * Whether two values hold the same content (what `JSON.stringify` would write of them, in any key order): two drafts
 * of the menus' town, say (`menuScene.ts`). Parts they share (a draft made from another keeps its untouched parts)
 * are the same by identity, so the cost is what they don't share; the first difference ends it. Anything but plain
 * objects, arrays and typed arrays (a Map, a class) is the same only as itself.
 */
export function sameContent(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (typeof a !== 'object' || typeof b !== 'object' || a === null || b === null) return false;
  if (Array.isArray(a) || Array.isArray(b)) {
    if (!Array.isArray(a) || !Array.isArray(b) || a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) if (!sameContent(a[i], b[i])) return false;
    return true;
  }
  if (ArrayBuffer.isView(a) || ArrayBuffer.isView(b)) {
    if (!ArrayBuffer.isView(a) || !ArrayBuffer.isView(b) || a.constructor !== b.constructor) return false;
    const [x, y] = [a as unknown as ArrayLike<number>, b as unknown as ArrayLike<number>];
    if (x.length !== y.length) return false;
    for (let i = 0; i < x.length; i++) if (x[i] !== y[i]) return false;
    return true;
  }
  const plain = (o: object) => {
    const p = Object.getPrototypeOf(o);
    return p === Object.prototype || p === null;
  };
  if (!plain(a) || !plain(b)) return false;
  const [x, y] = [a as Record<string, unknown>, b as Record<string, unknown>];
  const keys = Object.keys(x).filter((k) => x[k] !== undefined);
  if (keys.length !== Object.keys(y).filter((k) => y[k] !== undefined).length) return false;
  for (const k of keys) if (!sameContent(x[k], y[k])) return false;
  return true;
}
