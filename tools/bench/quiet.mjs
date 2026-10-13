/**
 * The game at 1 % volume for the browser tools (`render.mjs`, `scene.mjs`, `../llm/devices.mjs`):
 * their windows open while someone works at the computer. Runs in the page before the game's own
 * scripts (`page.addInitScript(quiet)` or the context's), so Settings › Audio › Master starts at
 * 1 %; the other settings stay as they are.
 */
export function quiet() {
  // The settings' storage key (web/src/settings/settings.svelte.ts).
  const key = 'open-sims-wasm.settings';
  let settings = {};
  try {
    settings = JSON.parse(localStorage.getItem(key) ?? '{}');
  } catch {
    // Unreadable: the game starts from its defaults anyway.
  }
  localStorage.setItem(key, JSON.stringify({ ...settings, masterVolume: 0.01 }));
}
