<script lang="ts">
  /** Static credits and licences. */
  const version = __APP_VERSION__;
  const commit = __APP_COMMIT__;
  const base = import.meta.env.BASE_URL;
</script>

<div class="credits">
  <p class="lead">Idyll Lives is a life simulation built for the browser, with performance first.</p>
  <dl>
    <dt>Simulation</dt>
    <dd>Rust compiled to WebAssembly, running in a Web Worker</dd>
    <dt>Rendering</dt>
    <dd>Babylon.js (Apache-2.0) on WebGPU, with a WebGL2 fallback</dd>
    <dt>Interface</dt>
    <dd>Svelte 5 (MIT) and plain CSS</dd>
    <dt>Typeface</dt>
    <dd>Inter by Rasmus Andersson (SIL Open Font License)</dd>
    <dt>Art</dt>
    <dd>Textures from Poly Haven and ambientCG (CC0); models, residents and their animations, and icons made in code for this project; all art is replaceable through the asset manifest</dd>
    <dt>Voices and language model</dt>
    <dd>
      Residents speak with Supertonic 3 by Supertone (BigScience OpenRAIL-M), their mouths follow the Misaki dictionary by hexgrad (Apache-2.0), and the optional language model for their lines is LFM2.5-350M by Liquid AI (LFM Open License v1.0); they run on ONNX Runtime Web (MIT) and Tokenizers.js (Apache-2.0).
      The voice model's licence binds everyone who plays: it and what it says may not be used to break the law, harm minors, spread harmful falsehoods or personal data, pass machine speech off as human, defame, harass, impersonate or discriminate against people, among the other uses <a href="{base}voice/supertonic-3/LICENSE.txt" target="_blank" rel="noopener">its licence</a> forbids.
    </dd>
    <dt>Open-source libraries</dt>
    <dd>
      Babylon.js, Svelte, serde, wasm-bindgen and the others the game is built with, each under its own licence: <a href="{base}THIRD-PARTY-LICENSES.txt" target="_blank" rel="noopener">the list and licence texts</a>; the art and models: <a href="{base}assets/CREDITS.md" target="_blank" rel="noopener">CREDITS.md</a>
    </dd>
    <dt>Licence</dt>
    <dd>Free software under the GNU General Public License v3.0 (or later): you may use, study, share and modify it; if you distribute it or a modified version, you must make the source available under the same licence. Third-party parts keep their own licences (above; the art is CC0).</dd>
  </dl>
  <p class="small">Version {version} (build {commit})</p>
</div>

<style>
  .lead {
    margin: 0 0 16px;
    font-size: 15px;
    line-height: 1.5;
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 10px 24px;
    margin: 0;
  }
  dt {
    font-weight: 650;
  }
  dd {
    margin: 0;
    color: var(--text-muted);
  }
  dd a {
    color: inherit;
  }
  .small {
    margin: 20px 0 0;
    font-size: 12px;
    color: var(--text-muted);
  }
</style>
