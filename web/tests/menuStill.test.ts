// The main menu's picture of the last world made (`game/menuStill.ts`), as Cache Storage keeps it: read
// back, it is the same picture and the same town's name (a name of any letters); an entry that isn't a
// picture with a name is none of ours.
import { describe, expect, test } from 'bun:test';
import { readStill, stillEntry } from '../src/game/menuStill';

const picture = new Blob([new Uint8Array([0xff, 0xd8, 0xff, 0xe0, 1, 2, 3])], { type: 'image/jpeg' });

describe("the menu's picture kept between visits", () => {
  test('read back: the same picture and town', async () => {
    const back = await readStill(stillEntry(picture, 'Ålby-on-Sea · Nord'));
    if (!back) throw new Error('not read back');
    expect(back.name).toBe('Ålby-on-Sea · Nord');
    expect(new Uint8Array(await back.picture.arrayBuffer())).toEqual(new Uint8Array(await picture.arrayBuffer()));
  });

  test('an entry without a name, not a picture, or empty is none', async () => {
    expect(await readStill(new Response(picture, { headers: { 'Content-Type': 'image/jpeg' } }))).toBeNull();
    expect(await readStill(new Response('{}', { headers: { 'Content-Type': 'application/json', 'X-Town-Name': 'Elmview' } }))).toBeNull();
    expect(await readStill(stillEntry(new Blob([], { type: 'image/jpeg' }), 'Elmview'))).toBeNull();
  });
});
