// The sounds' synthesis (`audio/synth.ts`: Rust, as WebAssembly), loaded for the tests that make sounds: import
// this first. Needs `bun run sound` (`tools/ci-local.sh web` runs it).
import { readFileSync } from 'node:fs';
import { loadSynth } from '../../src/audio/synth';

await loadSynth(readFileSync(new URL('../../src/audio/wasm-pkg/sound_wasm_bg.wasm', import.meta.url)));
