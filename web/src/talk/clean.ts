/**
 * What the language model wrote, made into a line a resident can say, or rejected. Small models
 * add labels ("Mia's line:"), quotes, stage directions and narration, or ramble; lines are
 * spoken and shown in speech bubbles, so only short, plain, harmless text gets through. Nothing
 * is ever spoken or shown that didn't pass here.
 *
 * Plain TypeScript without runtime imports (the Node check runs it by stripping the types).
 */

export interface CleanOptions {
  /** The speaker's first name (lines may start with it as a label). */
  me: string;
  /** Other names that may appear (who they talk to): "Tom smiles…" is narration. */
  names?: readonly string[];
  /** Words and phrases that reject a line (whole words, any case). */
  blocklist: readonly string[];
  /** Longest line kept, in characters. */
  maxLength?: number;
  /** The reply was cut short (its token limit or deadline): an unfinished last sentence goes (nothing finished, no line). */
  cut?: boolean;
}

/** Sentences a line keeps at most. */
export const MAX_SENTENCES = 5;

/** Characters a line may keep: Latin letters (with accents), digits, spaces and plain punctuation. */
const ALLOWED = /[^A-Za-zÀ-ÖØ-öø-ÿ0-9 .,!?'’\-—…:;]/g;
/** Verbs of narration after a name or pronoun ("Mia smiles", "She says"). */
const NARRATION_VERBS = 'says|said|smiles|smiled|laughs|laughed|nods|nodded|sighs|sighed|replies|replied|asks|asked|grins|grinned|looks|looked|whispers|whispered|shouts|shouted|thinks|thought|feels|felt|giggles|giggled|is|was|walks|turns|voice';

export function cleanLine(raw: string, o: CleanOptions): string | null {
  const max = o.maxLength ?? 360;
  let s = raw.replace(/<\|[^|]*\|>/g, ' ').trim();
  // The first line only (a reply should be one).
  s = s.split(/\n/)[0].trim();
  // Labels: "Line:", "Mia:", "Mia's line:", "Mia says,".
  const me = escapeRegExp(o.me);
  const who = ['he', 'she', 'they', me, ...(o.names ?? []).map(escapeRegExp)].join('|');
  const narration = new RegExp(`^(?:${who})(?:'s)?\\s+(?:${NARRATION_VERBS})\\b`, 'i');
  s = s.replace(new RegExp(`^(?:(?:${me}(?:'s)?|her|his|their|my)\\s+)?(?:line|reply|answer)\\s*[:\\-]\\s*`, 'i'), '');
  s = s.replace(new RegExp(`^${me}\\s*(?:(?:says|said|replies|replied|answers|asks|whispers|shouts)\\s*)?[:,\\-]\\s*`, 'i'), '');
  // Stage directions and asides.
  s = s.replace(/\*[^*]*\*/g, ' ').replace(/\([^)]*\)/g, ' ').replace(/\[[^\]]*\]/g, ' ');
  // Quoted speech: the words inside the first quote, if there's text around it (narration).
  const quoted = /["“”«»]([^"“”«»]{2,})["“”«»]/.exec(s);
  if (quoted && s.replace(quoted[0], '').trim().length > 0) {
    // A line that is mostly narration around a short quote isn't speech.
    if (narration.test(s.trim()) || /^(?:he|she|they)\b/i.test(s.trim())) return null;
    s = quoted[1];
  }
  s = s.replace(/["“”«»]/g, '');
  if (narration.test(s.trim())) return null;
  s = s.replace(ALLOWED, ' ').replace(/\s+/g, ' ').trim();
  // At most five sentences, within the length; cut short, only the finished ones.
  let sentences: string[] = s.match(/[^.!?…]+[.!?…]+|[^.!?…]+$/g) ?? [];
  if (o.cut && sentences.length && !/[.!?…]$/.test(sentences[sentences.length - 1].trim())) sentences = sentences.slice(0, -1);
  let out = '';
  for (const sentence of sentences.slice(0, MAX_SENTENCES)) {
    const next = (out + ' ' + sentence.trim()).trim();
    if (next.length > max) break;
    out = next;
  }
  if (!out) return null;
  out = out.replace(/^[,;:\-—\s]+/, '').trim();
  // Talking to themselves by name ("Mia, stop it!"): the name goes.
  out = out.replace(new RegExp(`^${me}\\s*[,!]\\s*`, 'i'), '');
  out = out.charAt(0).toUpperCase() + out.slice(1);
  if (!/[.!?…]$/.test(out)) out += '.';
  // Enough to say: two letters or more; not the speaker's own name alone.
  const letters = out.replace(/[^A-Za-zÀ-ÖØ-öø-ÿ]/g, '');
  if (letters.length < 2 || letters.toLowerCase() === o.me.toLowerCase()) return null;
  if (blocked(out, o.blocklist)) return null;
  return out;
}

function blocked(line: string, list: readonly string[]): boolean {
  const lower = line.toLowerCase();
  return list.some((w) => new RegExp(`(?:^|[^a-z])${escapeRegExp(w.toLowerCase())}(?:$|[^a-z])`).test(lower));
}

function escapeRegExp(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}
