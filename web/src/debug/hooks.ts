/**
 * Whether the debug hooks a bench needs are in this build: always on the dev server, and in a production build made
 * for the benchmark (`VITE_BENCH=1 vite build`, `tools/bench/lib/build.mjs`), so pictures and timings can be taken of
 * what players get. A release has none of them (the flag is false there and what it guards is left out). The dev play
 * log and the dev server's `/__debug` endpoints stay with the dev server.
 */
export const DEBUG_HOOKS: boolean = import.meta.env.DEV || import.meta.env.VITE_BENCH === '1';
