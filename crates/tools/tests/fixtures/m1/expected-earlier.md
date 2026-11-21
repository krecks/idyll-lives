# M1 estimate, 20261101-000000

The sim's ticks a second over 24 game hours, the camera's 320-tile square round the player's home at full detail at 10× (big_world's WATCH, simtick's --watch); best of 1 each.
10× needs 600; a preset gets a tier when the estimated wasm rate on its slowest intended machine is at least 600.

## Calibration

| | 7-Zip 1 thread (MIPS) | tickbench (µs a tick) |
|---|---:|---:|
| container (m3) | 4,999 | 11.0 |
| under the quota (m1, 72 %) | 3,600 | 17.1 |
| ratio | 0.72 | 0.65 |

The estimate scales this machine's numbers by 0.72 (an M1's single core against an M3 Pro's), with a band of 0.62 (sustained, fanless) to 0.78. The quota's own ratio above shows the container's "M1" runs at about that.

## Ticks a second

Wasm: ticks a second by the wall clock, and (in brackets) of the process's CPU time, which leaves out the waits for a core on a busy machine; the M1 wasm estimate scales the latter.

| tier | world | residents | M3 native | container | under quota | wasm Node | wasm Bun | M1 native est. | M1 wasm est. (band) | margin vs 600 | slowest hour (M1 wasm est.) |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| near | forest | – | – | – | – | – (–) | – (–) | – | – (–) | – | – (–:00) |
| near | river | – | – | – | – | – (–) | – (–) | – | – (–) | – | – (–:00) |
| half | forest | – | – | – | – | – (–) | – (–) | – | – (–) | – | – (–:00) |
| half | river | NaN | 12,000 | – | – | – (–) | – (–) | 8,640 | – (–) | – | – (–:00) |

## Per tier (the slowest world)

| tier | M3 wasm | M1 wasm est. | at the band's low end | ≥ 600 on an M1? | on an M3? |
|---|---:|---:|---:|---|---|

## What this cannot say

- **The GPU and the frame time.** Docker has no GPU: the renderer is not run, and the sim worker shares the M1 with it and the main thread in the game.
- **Unified-memory pressure.** The quota container is given 8 GB (as much as Docker Desktop has), but an M1 shares its 8 GB with the GPU, the browser and the system; swapping is not modelled.
- **Thermals.** A fanless M1 Air throttles under a sustained load; the band's low end stands for that, not a measurement.
- **The browser.** Node runs the same V8 as Chrome, but not Safari's or Firefox's engines; the worker's messaging and the UI updates are not counted.
- **A shared machine.** Other work on this machine slows every run; the best run of each is kept, but the numbers are as good as the quietest of them.
