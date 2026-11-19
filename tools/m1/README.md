# The M1 estimate

The game's floor is an 8 GB Apple M1 (2020). This estimates how fast the simulation runs on one,
from a faster Mac (an M3 Pro), and whether it keeps up with 10×, the fastest fixed speed: 10
ticks a step, 60 steps a second, so 600 ticks a second. (Super speed has no rate to keep up with:
it runs as fast as the machine can, up to 30×, and up to 60× in quiet hours.)

    tools/m1/run.sh

It runs the three city worlds (seaside, river, forest at `town=city,villages=many`) at each fill
tier (`web/src/game/people.ts` `FillTier`), each over a game day with the camera's 320-tile square
at full detail at 10×, as the game runs (`crates/sim-core/tests/big_world.rs` WATCH,
`tools/bench/simtick.mjs --watch`):

- **natively** on this machine (`tools/bench/tiers.sh`);
- as **wasm** under Node and Bun (`tools/bench/simtick.mjs`, the build the game ships);
- in a **container** (`compose.yml` `m3`: Linux on arm64, the game's own toolchain, `Dockerfile`);
- in the container **under a CPU quota** (`m1`: 72 % of a core's time, `M1_QUOTA`, and 8 GB).

`calibrate.sh` checks the quota: 7-Zip's single-threaded benchmark and the sim's tickbench, best
of three, in both containers. Their ratio should come out near 0.72, the M1's single-core speed
against the M3 Pro's.

The report (`bench-results/m1/<time>/report.md`) scales this machine's numbers by 0.72, with a band
of 0.62 (a fanless Air under a sustained load) to 0.78, and sets the wasm estimate against 600,
which is also the floor a graphics preset's tier must clear on its slowest intended machine.

`SKIP="native wasm container"` (any of them) reuses the last run's numbers for those parts;
`HOURS`, `ROUNDS`, `PARALLEL`, `TIERS`, `WORLDS_AT` and `WATCH` are `tools/bench/tiers.sh`'s.

## What it cannot say

- **The GPU and the frame time.** Docker has no GPU, so the renderer isn't measured at all.
- **Unified-memory pressure.** The container gets at most what Docker Desktop's VM has; an M1
  shares its 8 GB with the GPU, the browser and the system.
- **Thermals.** The band's low end stands in for a throttled Air; nothing here heats up like one.
- **Other browsers' engines.** Node is Chrome's V8; Safari and Firefox are not measured.
