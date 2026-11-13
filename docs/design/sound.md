# Sound

Every sound in the game is made in code. There are no audio files: a sound is a function that
returns samples at a sample rate, the same every time, so it can be rendered, measured and tested
off the browser. Voices are the one exception in kind (a speech model's output, `docs/design/voices.md`),
and they go through the same mix.

## The mix

One `AudioContext` for the whole game (`web/src/audio/mixer.ts`), started by the first click or key
press, stopped while the tab is hidden.

```
interface blips ───────────────────────────────→ effects bus (0.5) ──┐
voice clip → place ─→ voices' space ───────────→ voices bus ─────────┤
thing, step, car → place ─→ world's space ──┐                        ├→ limiter → master → out
bed, crickets, birds, town, leaves ─────────┴──→ world bus → duck ───┘
```

- **Buses.** `effects` (the interface, at a fixed 0.5), `voices`, `world`. Each bus's gain is its
  setting.
- **Duck.** While any voice clip plays, the world is at half (`DUCK`: about -6 dB, down in 80 ms,
  back over half a second after the last voice ends).
- **Limiter.** What the buses add up to is held at -3 dBFS (`LIMIT`, a `DynamicsCompressorNode`).
  No sound reaches it on its own; it works when many sound at once.
- `mixNow()` says what the mix is doing (voices speaking, the duck's level, dB being limited).

## How a sound is made and levelled

| what | made in | played by |
|---|---|---|
| water, machines, games, tools, instruments (two channels, loops and sounds played once) | `water.ts`, `things.ts` | `world.ts` |
| a pan, a knife, the TV, drums, the outdoor bed, crickets, the knock (one channel) | `mono.ts` | `world.ts` |
| music: three pieces of 32 to 40 s, written from a seed (`score`) and played (`perform`) | `music.ts` | `world.ts` (`catalog.ts` `pieceAt`: a dance while someone dances, a slow piece from 20 to 6 o'clock, a folk tune by day; a thing can name a piece) |
| footsteps: five surfaces, shoes or barefoot, four versions each | `steps.ts` | `footsteps.ts` (what is underfoot: `surfaces.ts`) |
| a car's engine and tyres, the town from above, the wind in the leaves | `outdoors.ts` | `outside.ts` |
| birds' songs | `birdsong.ts` | `world.ts` |
| the interface's blips | `blips.ts` | `ui/sfx.ts` |
| voices | `crates/voice`, `web/src/voice/` | `voice/player.ts` |

What takes a moment to make (the water's, the things', the outdoors', the music, the songs) is made
in a worker the first time it is wanted (`water.worker.ts`, `made.ts`); a sound not ready yet starts
when it is.

**Levels are in one place**, `web/src/audio/catalog.ts`: `LOOPS` and `ONCE_LEVEL` (things, by the key
the content names them with), `AMBIENCE`, `STEPS`, `OUTSIDE`, `EFFECTS_LEVEL`. A level is a plain gain
on the sound as made (the water's, the things', the outdoors' and the music are made to an average
level; the rest to a peak of 1). `everySound()` lists every sound as it comes out in the middle of the
picture at its level: what the render writes and the tests measure.

**Loudness** is LUFS (ITU-R BS.1770, gated: `loudness.ts`), not peak or RMS: what one sound is against
another to the ear. Each group has a range the tests hold it to (`GUARD`):

| group | range (LUFS) | where they are |
|---|---|---|
| things in use (and a car, the music) | -40 to -18 | a shower -19.8, a blender -22.9, the TV -33.5, a pottery wheel -35.4 |
| one footstep | -52 to -34 | shoes on tiles -36.8, bare feet on carpet -47.0; a resident's walk -41 to -50 |
| the outdoors | -46 to -28 | crickets -31.8, the town -34.0, leaves at a full gust -35.9, the bed -36.1, a bird -34 to -40 |
| the interface | -50 to -24 | a wall's thump -28.2, a tab's tick -45.1 |

Voices are levelled where they are made: every line is brought to -25 LUFS by one gain
(`crates/voice/src/dsp.rs` `level`).

## Space

A thing's sound, a step, a car and a voice are placed on their way into their bus
(`web/src/audio/space.ts`; the rules are plain functions in `acoustics.ts`):

```
sound → [wall: low-pass, level] → panner ─┬──────────────────────────────→ [distance: low-pass, level] → bus
                                          └→ send → room reverb (by size) ─┘
```

- **Left to right.** A `StereoPannerNode` by where the thing is on screen, 0.7 of the way to either
  side at the screen's edges.
- **Rooms.** The world's `rooms` grid says which room a resident or a thing stands in and how big it
  is. A sound made indoors goes to the reverb of a room its size: small (to 9 m²), medium (to 28 m²),
  large. The reverbs are `ConvolverNode`s whose responses are generated (`rooms.ts`: a few early
  echoes, then a tail of 0.3 to 0.6 s whose highs die first), 13 to 15 dB under the sound. Outdoors a
  sound is dry. (Each water and thing sound also has a little room of its own baked in, as it had.)
- **Walls.** With the camera closed in on a room (within 10 m), a sound from another room, another
  storey or the other side of an outside wall is low-passed at 1.5 kHz and 4 dB down. This eases to
  nothing by 24 m: from the game's own zoom the whole house is in view and nothing is muffled.
- **Distance** (the world only). Up to 24 m, the distance a game opens at, a sound is as made. Further
  out it falls as 24 m over the camera's distance and loses its highs (4 kHz at the play camera's
  95 m, where the level reaches its floor of -12 dB); out in the town view it sinks to -20 dB.
  Nothing goes silent. Voices are not made quieter by distance.
- The outdoors (the bed, crickets, the unseen birds, the town, the leaves) is all round: no place, no
  room, no distance. Birds in view have a place and their own fall with distance.

Footsteps start as the foot lands: the renderer says how far through its stride a resident is and
how fast it goes (`Renderer.simStride`), and the step is scheduled on the audio clock for the heel's
strike, the sound card's delay taken off. The wind's level follows the gust the grass shows
(`Renderer.gust`). A car's level follows its speed on screen and how near it passes the middle of
the picture.

## Settings

| setting | what it does |
|---|---|
| Master volume | the gain after the limiter: everything |
| Sound effects | the interface's blips on or off (the effects bus has no slider: it is at 0.5) |
| Sounds of the world | the world bus on or off: things in use, footsteps, cars, the wind, the town, birds, crickets. Off, nothing of it is made or scheduled |
| World volume | the world bus's gain |
| Voice volume | the voices bus's gain (a voice's room reverb with it) |
| Mute in background | the master fades out while another window has focus |

## Checking a sound

Nobody has to hear a sound to know it is whole and where it sits:

- `cd web && bun run sounds` (`tools/audio/listen.ts`, `--out <folder>`) renders every sound to
  `bench-results/sounds/<name>.wav` at the level the game plays it, with `index.md`: length, peak, RMS,
  LUFS, spectral centroid, who sounds it; then the same sounds in each room, through a wall and from
  far off, a walk on every surface, cars passing, gusts. Play the WAVs to judge what numbers can't:
  whether a microwave sounds like a microwave.
- `bun test tests/sounds.test.ts` holds every sound to its group's range, to no NaN, a peak under full
  scale and, for a loop, a seam without a click. `space.test.ts`, `footsteps.test.ts`,
  `outside.test.ts` and `music.test.ts` hold the rules (the distance law, a room's response, a walk's
  steps against its heel strikes, a piece's key and form).
- `web/src/audio/offline.ts` does on samples what the space's nodes do (the node's filter, an FFT
  convolution), so a placed sound can be measured off the browser. The graph itself takes a
  `BaseAudioContext`: rendered in Chromium's `OfflineAudioContext` it matches `offline.ts` to 3e-8.
- In the dev game, `window.__osw.sound()` says what is sounding: the loops and where each is placed,
  the piece of music, the steps started, the cars heard, the gust, the camera's distance.
- A new sound: write the function, give it a level in `catalog.ts` and an entry in `everySound()`,
  run `bun run sounds`, and set the level so its LUFS sits where its kind does.

## Not there

A bike's freewheel; rain or weather; sounds from lots other than the one in view (their residents
are not drawn); a listener that moves through the house (the camera looks from above: place is by
the screen, not by a 3D panner).
