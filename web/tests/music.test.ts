// The music (`audio/music.ts`): a piece is the same for the same seed and another for another; it is sixteen bars
// whose first phrase comes back, its tune in its key's five-note scale and ending on the key's own note, every note
// on the beat's grid; played, it is a loop of its written length; and which piece plays follows the hour and the
// dancing (`catalog.ts` `pieceAt`).
import { describe, expect, test } from 'bun:test';
import { pieceAt, PLAYLIST } from '../src/audio/catalog';
import { MUSIC, perform, PIECES, score, seconds, type MusicKind, type Style } from '../src/audio/music';

const RATE = 48000;
const STYLES: Style[] = ['folk', 'dance', 'slow'];
const tuneOf = (style: Style, seed: number) => score(style, seed).notes.filter((n) => n.voice === 'pluck');

describe('a piece as written', () => {
  test('the same seed, the same piece; another seed, another tune', () => {
    for (const style of STYLES) {
      expect(score(style, 4)).toEqual(score(style, 4));
      const tunes = new Set([1, 2, 3, 4, 5, 6].map((seed) => tuneOf(style, seed).map((n) => `${n.at}:${n.midi}`).join(' ')));
      expect(tunes.size).toBeGreaterThanOrEqual(5);
    }
  });

  test('sixteen bars, a chord a bar, every note on the grid and inside the piece', () => {
    for (const style of STYLES) {
      for (const seed of [1, 2, 3]) {
        const s = score(style, seed);
        expect(s.bars).toBe(16);
        expect(s.chords.length).toBe(16);
        const beats = s.bars * s.meter;
        for (const n of s.notes) {
          expect(n.at).toBeGreaterThanOrEqual(0);
          expect(n.at).toBeLessThan(beats);
          expect(n.len).toBeGreaterThan(0);
          expect(n.level).toBeGreaterThan(0);
          expect(n.level).toBeLessThanOrEqual(1);
          expect(Math.abs(n.pan)).toBeLessThanOrEqual(1);
          // Sixteenths at the finest (a strummed chord's strings a hair apart).
          if (n.voice !== 'strum') expect(Math.abs(n.at * 4 - Math.round(n.at * 4))).toBeLessThan(1e-9);
        }
        // The first phrase's chords come back for the second and the last.
        expect(s.chords.slice(4, 8)).toEqual(s.chords.slice(0, 4));
        expect(s.chords.slice(12, 16)).toEqual(s.chords.slice(0, 4));
        expect(s.chords[0]).toBe(0);
      }
    }
  });

  test('the tune keeps to the key\'s five notes, comes back, and ends on the key\'s own note', () => {
    for (const style of STYLES) {
      for (const seed of [1, 2, 3, 4]) {
        const s = score(style, seed);
        const tune = s.notes.filter((n) => n.voice === 'pluck');
        expect(tune.length).toBeGreaterThan(16);
        const five = s.minor ? [0, 3, 5, 7, 10] : [0, 2, 4, 7, 9];
        for (const n of tune) expect(five).toContain((((n.midi - s.key) % 12) + 12) % 12);
        // Within an octave and a half of where it starts: singable.
        const pitches = tune.map((n) => n.midi);
        expect(Math.max(...pitches) - Math.min(...pitches)).toBeLessThanOrEqual(24);
        // Bars 5-8 are bars 1-4 again.
        const phrase = (from: number) => tune.filter((n) => n.at >= from * s.meter && n.at < (from + 4) * s.meter).map((n) => `${n.at - from * s.meter}:${n.midi}`);
        expect(phrase(4)).toEqual(phrase(0));
        expect(phrase(8)).not.toEqual(phrase(0));
        expect((tune[tune.length - 1].midi - s.key) % 12).toBe(0);
      }
    }
  });

  test('under the tune: a dance has its kick on every beat and its clap on two and four, a waltz three beats a bar', () => {
    const dance = score('dance', 1);
    const kicks = dance.notes.filter((n) => n.voice === 'kick').map((n) => n.at);
    expect(kicks).toEqual(Array.from({ length: 64 }, (_, i) => i));
    expect(dance.notes.filter((n) => n.voice === 'clap').every((n) => n.at % 2 === 1)).toBe(true);
    expect(score('slow', 1).meter).toBe(3);
    expect(score('folk', 1).notes.some((n) => n.voice === 'kick')).toBe(false);
    // The bass plays the chord's own note at the start of each bar of a folk tune.
    const folk = score('folk', 2);
    const scale = [0, 2, 4, 5, 7, 9, 11];
    for (let bar = 0; bar < folk.bars; bar++) {
      const bass = folk.notes.find((n) => n.voice === 'bass' && n.at === bar * folk.meter)!;
      expect((((bass.midi - folk.key) % 12) + 12) % 12).toBe(scale[folk.chords[bar] % 7]);
    }
  });
});

describe('a piece played', () => {
  test('a loop of its written length, both sides, whole, the same every time', () => {
    const piece = score('slow', 2);
    const a = perform(piece, RATE);
    expect(a.left.length).toBe(Math.round(seconds(piece) * RATE));
    expect(a.right.length).toBe(a.left.length);
    expect(seconds(piece)).toBeCloseTo((16 * 3 * 60) / 72, 9);
    expect(perform(piece, RATE).left).toEqual(a.left);
    let sum = 0;
    let differ = 0;
    for (let i = 0; i < a.left.length; i++) {
      sum += a.left[i] ** 2;
      differ += (a.left[i] - a.right[i]) ** 2;
    }
    expect(Number.isFinite(sum)).toBe(true);
    expect(sum).toBeGreaterThan(0);
    // The voices sit a little left and right: the two sides are not the same.
    expect(differ / sum).toBeGreaterThan(0.001);
  });

  test('the pieces the game plays are half a minute and more, each in its style', () => {
    for (const kind of Object.keys(PIECES) as MusicKind[]) {
      const p = PIECES[kind];
      expect(seconds(score(p.style, p.seed))).toBeGreaterThanOrEqual(30);
      expect(typeof MUSIC[kind]).toBe('function');
    }
    expect(new Set(Object.values(PIECES).map((p) => p.style)).size).toBe(3);
  });
});

describe('which piece plays', () => {
  test('a dance while someone dances, a slow one in the evening and the night, a folk tune by day', () => {
    for (const hour of [0, 8, 13, 22]) expect(pieceAt(hour, true)).toBe('musicDance');
    expect(pieceAt(8, false)).toBe('musicFolk');
    expect(pieceAt(PLAYLIST.evening - 0.1, false)).toBe('musicFolk');
    expect(pieceAt(PLAYLIST.evening, false)).toBe('musicSlow');
    expect(pieceAt(2, false)).toBe('musicSlow');
    expect(pieceAt(PLAYLIST.morning, false)).toBe('musicFolk');
  });
});
