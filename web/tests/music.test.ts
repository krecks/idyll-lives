// Which piece of music plays (`catalog.ts` `pieceAt`): it follows the hour and the dancing. (The pieces themselves are
// made in Rust: `crates/sound/src/music.rs` and its tests.)
import { describe, expect, test } from 'bun:test';
import { pieceAt, PLAYLIST } from '../src/audio/catalog';

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
