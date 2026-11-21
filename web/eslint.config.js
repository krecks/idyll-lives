// Bug-catching rules only, no style rules: svelte-check already covers types and unused code.
import svelte from 'eslint-plugin-svelte';
import ts from 'typescript-eslint';
import svelteConfig from './svelte.config.js';

export default ts.config(
  { ignores: ['dist/', 'src/**/wasm-pkg/'] },
  ...svelte.configs.recommended,
  {
    files: ['src/**/*.ts', 'src/**/*.svelte'],
    languageOptions: {
      parserOptions: {
        projectService: true,
        extraFileExtensions: ['.svelte'],
        tsconfigRootDir: import.meta.dirname,
      },
    },
    plugins: { '@typescript-eslint': ts.plugin },
    rules: {
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/no-misused-promises': 'error',
      // Noise here: flags plain Maps/Sets/Dates built inside $derived or helpers, and house style.
      'svelte/prefer-svelte-reactivity': 'off',
      'svelte/prefer-writable-derived': 'off',
      'svelte/no-useless-mustaches': 'off',
    },
  },
  // PlayCanvas only through render/pc/engine.ts: the bare package resolves to its debug build under
  // Vite's dev server (WebGPU validation around every draw). And no Babylon: the game draws with
  // PlayCanvas (the Babylon renderer was removed in Stage 11 of the port).
  {
    files: ['src/**/*.ts', 'src/**/*.svelte'],
    rules: {
      'no-restricted-imports': [
        'error',
        {
          paths: [{ name: 'playcanvas', message: 'Import PlayCanvas through render/pc/engine.ts (the release build).' }],
          patterns: [{ group: ['@babylonjs/*'], message: 'The game draws with PlayCanvas; Babylon is not a dependency.' }],
        },
      ],
    },
  },
  // What lays out a town uses game/pmath.ts, not Math.sin, Math.hypot, `**` and the like: those differ in the last
  // bits between machines and engines (Bun takes them from the system's libm), and a town must not.
  {
    files: ['src/game/**/*.ts', 'src/render/shared/builtSurface.ts', 'src/render/shared/junctions.ts', 'src/render/shared/chunks.ts', 'src/render/shared/geometry.ts'],
    ignores: ['src/game/pmath.ts'],
    rules: {
      'no-restricted-properties': [
        'error',
        ...['sin', 'cos', 'tan', 'asin', 'acos', 'atan', 'atan2', 'sinh', 'cosh', 'tanh', 'asinh', 'acosh', 'atanh', 'exp', 'expm1', 'log', 'log1p', 'log2', 'log10', 'pow', 'hypot', 'cbrt'].map((property) => ({
          object: 'Math',
          property,
          message: 'Not the same to the last bit on every machine and engine: use game/pmath.ts.',
        })),
      ],
      'no-restricted-syntax': [
        'error',
        { selector: "BinaryExpression[operator='**']:not([left.value=2])", message: 'Not the same to the last bit on every engine: use pmath.pow.' },
        { selector: "AssignmentExpression[operator='**=']", message: 'Not the same to the last bit on every engine: use pmath.pow.' },
      ],
    },
  },
  { files: ['src/**/*.ts'], languageOptions: { parser: ts.parser } },
  { files: ['src/**/*.svelte', 'src/**/*.svelte.ts'], languageOptions: { parserOptions: { parser: ts.parser, svelteConfig } } },
);
