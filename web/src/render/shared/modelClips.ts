/**
 * A model's clips as the game plays them (not the engines' animation systems): each moving part's
 * pose sampled `CLIP_RATE` times a second when the file is read (`models.ts` `readClip`), and
 * blended here between samples. Engine-neutral: the poses are 16-float matrices in game space,
 * Babylon's row-vector layout (`shared/math.ts`): a vertex times the pose.
 */

export interface ModelClip {
  /** Seconds. */
  duration: number;
  /**
   * Per part it moves (or carries: a part inside one it moves), its pose in model space, sampled
   * `CLIP_RATE` times a second (16 floats each), and `depth`, how deep in the file's nodes the
   * outermost node it moves of that part's is (0: the root's children), so a part posed by several
   * clips at once takes the inner ones' poses first (`partPose`).
   */
  tracks: { part: number; poses: Float32Array; depth: number }[];
}

/** Samples per second of a clip's poses. */
export const CLIP_RATE = 30;

/**
 * Where `clip` puts `part` at `seconds` (clamped to the clip) into `out`, relative to rest: its
 * vertices at rest times `out` are where they are then. False (and `out` untouched) if the clip
 * doesn't move that part.
 */
export function clipPose(clip: ModelClip, part: number, seconds: number, out: Float32Array, offset = 0): boolean {
  const track = clip.tracks.find((t) => t.part === part);
  if (!track) return false;
  const last = track.poses.length / 16 - 1;
  const at = clamp(seconds * CLIP_RATE, 0, last);
  const i = Math.min(Math.floor(at), Math.max(0, last - 1));
  const f = last > 0 ? at - i : 0;
  const a = i * 16;
  const b = Math.min(i + 1, last) * 16;
  // Samples are close together: blending the matrices is as good as turning between them.
  for (let k = 0; k < 16; k++) out[offset + k] = track.poses[a + k] * (1 - f) + track.poses[b + k] * f;
  return true;
}

/**
 * Where the clips playing (`playing`, each at its seconds) put `part` together, into `out` (as
 * `clipPose`): one clip's pose as it is; several multiply, the clips that move the part itself
 * before those that move a part it is inside (a record spinning on an arm that swings up: the spin
 * and then the swing), so each clip of a nested part plays as if the others did too. On one node, a
 * crop's growth clip (`kind` `growth`) comes first and the others play on top of it, about the part's
 * rest origin: a veggie bed's plants sway as they stand at their stage, seedlings by their own small
 * height. False (`out` untouched) if none moves it.
 */
export function partPose(playing: readonly { clip: ModelClip; time: number; kind?: string }[], part: number, out: Float32Array, offset = 0): boolean {
  let found = 0;
  let first: { clip: ModelClip; time: number } | undefined;
  for (const p of playing) {
    if (!p.clip.tracks.some((t) => t.part === part)) continue;
    if (!found++) first = p;
  }
  if (found <= 1) return found === 1 && clipPose(first!.clip, part, first!.time, out, offset);
  const posed: { depth: number; growth: number; pose: Float32Array }[] = [];
  for (const p of playing) {
    const track = p.clip.tracks.find((t) => t.part === part);
    if (!track) continue;
    const pose = new Float32Array(16);
    clipPose(p.clip, part, p.time, pose);
    posed.push({ depth: track.depth, growth: p.kind === 'growth' ? 1 : 0, pose });
  }
  // Deepest first, and on one node the growth stage first (row vectors: a vertex times the first, then the next).
  posed.sort((a, b) => b.depth - a.depth || b.growth - a.growth);
  const acc = posed[0].pose;
  const tmp = new Float32Array(16);
  for (let n = 1; n < posed.length; n++) {
    const m = posed[n].pose;
    for (let i = 0; i < 4; i++) for (let j = 0; j < 4; j++) tmp[i * 4 + j] = acc[i * 4] * m[j] + acc[i * 4 + 1] * m[4 + j] + acc[i * 4 + 2] * m[8 + j] + acc[i * 4 + 3] * m[12 + j];
    acc.set(tmp);
  }
  out.set(acc, offset);
  return true;
}

function clamp(v: number, lo: number, hi: number): number {
  return v < lo ? lo : v > hi ? hi : v;
}
