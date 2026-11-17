# M1 estimate, 20261102-000000

The sim's ticks a second over 6 game hours, the view on the player's lot; best of 3 each.
10× needs 600; a preset gets a tier when the estimated wasm rate on its slowest intended machine is at least 600.

## Calibration

| | 7-Zip 1 thread (MIPS) | tickbench (µs a tick) |
|---|---:|---:|
| container (m3) | 5,000 | 12.3 |
| under the quota (m1, 65 %) | 3,600 | 17.1 |
| ratio | 0.72 | 0.72 |

The estimate scales this machine's numbers by 0.72 (an M1's single core against an M3 Pro's), with a band of 0.62 (sustained, fanless) to 0.78. The quota's own ratio above shows the container's "M1" runs at about that.
The m1 container: memory limit 8589934592 bytes, cpu.max 7200 10000, 10 cpus.

## Ticks a second

Wasm: ticks a second by the wall clock, and (in brackets) of the process's CPU time, which leaves out the waits for a core on a busy machine; the M1 wasm estimate scales the latter.

| tier | world | residents | M3 native | container | under quota | wasm Node | wasm Bun | M1 native est. | M1 wasm est. (band) | margin vs 600 | slowest hour (M1 wasm est.) |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| full | seaside | – | – | – | – | 4,100 (4,300) | 3,800 (3,950) | – | 3,096 (2,666–3,354) | +416 % | 1,586 (08:00) |
| full | forest | – | – | – | – | – (–) | – (–) | – | – (–) | – | – (–:00) |
| near | seaside | – | – | – | – | – (–) | – (–) | – | – (–) | – | – (–:00) |
| near | forest | – | – | – | – | 690 (695) | 650 (660) | – | 500 (431–542) | -17 % | 290 (18:00) |

## Per tier (the slowest world)

| tier | M3 wasm | M1 wasm est. | at the band's low end | ≥ 600 on an M1? | on an M3? |
|---|---:|---:|---:|---|---|
| full | 4,300 | 3,096 | 2,666 | yes | yes |
| near | 695 | 500 | 431 | no | yes |

## What this cannot say

- **The GPU and the frame time.** Docker has no GPU: the renderer is not run, and the sim worker shares the M1 with it and the main thread in the game.
- **Unified-memory pressure.** The quota container is given 8 GB (as much as Docker Desktop has), but an M1 shares its 8 GB with the GPU, the browser and the system; swapping is not modelled.
- **Thermals.** A fanless M1 Air throttles under a sustained load; the band's low end stands for that, not a measurement.
- **The browser.** Node runs the same V8 as Chrome, but not Safari's or Firefox's engines; the worker's messaging and the UI updates are not counted.
- **A shared machine.** Other work on this machine slows every run; the best run of each is kept, but the numbers are as good as the quietest of them.
