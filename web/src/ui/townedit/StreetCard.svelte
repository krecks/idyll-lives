<script lang="ts">
  /**
   * The Street tool's card: what it draws (`streetChoice`). The kind, which is a street of the town
   * or village it runs in or a country road outside them unless a street, a lane, an avenue or a
   * footpath is asked for (a street, a lane or an avenue asked for stays one out of town, given an
   * area when it is built); whether it is one-way, and which way (arrows along it as it is drawn);
   * whether what is in its way is cleared (else the street is refused there); and what is put along
   * it: homes, the town's other buildings, both, or nothing; by where it runs unless chosen (both in
   * the main town, homes elsewhere). A street just drawn and split at its bridges lists its stretches,
   * each picked by a click (the tool picks the one nearest where the street was finished).
   */
  import Segmented from '../kit/Segmented.svelte';
  import Toggle from '../kit/Toggle.svelte';
  import { streetWidth } from '../../game/edit/streets';
  import Icon from '../Icon.svelte';
  import { alongFor, streetChoice, type Along, type StreetKindChoice } from './streetChoice.svelte';
  import { drawnStretches } from './streetLive';
  import type { TownEditor } from './tools';

  const { editor }: { editor: TownEditor } = $props();

  const KINDS: readonly { value: StreetKindChoice; label: string }[] = [
    { value: 'auto', label: 'Auto' },
    { value: 'street', label: 'Street' },
    { value: 'lane', label: 'Lane' },
    { value: 'avenue', label: 'Avenue' },
    { value: 'country', label: 'Country' },
    { value: 'trail', label: 'Footpath' },
  ];
  /** A lane is narrow only where its kind is narrower than a street (`streetWidth`). */
  const ALONG: readonly { value: Along; label: string }[] = [
    { value: 'homes', label: 'Homes' },
    { value: 'buildings', label: 'Town buildings' },
    { value: 'both', label: 'Both' },
    { value: 'none', label: 'Nothing' },
  ];

  const trail = $derived(streetChoice.kind === 'trail');
  /** `auto` shows as what goes along a street where the pointer is (both in the main town, homes elsewhere). */
  const along = $derived(alongFor(streetChoice.along, streetChoice.shownArea));
  /** The street just drawn, split at its bridges: its stretches while the town has them. */
  const stretches = $derived(drawnStretches(editor.draft, streetChoice.drawn));
  const picked = $derived(editor.selected?.kind === 'road' ? editor.selected.road : -1);
</script>

<div class="street-card">
  {#if stretches.length}
    <div class="option">
      <span class="eyebrow sub">The street drawn</span>
      <div class="choices" role="radiogroup" aria-label="The street drawn, a stretch at a time">
        {#each stretches as s (s.road)}
          <button role="radio" class="chip" class:deck={s.deck} aria-checked={picked === s.road} class:on={picked === s.road} onclick={() => editor.select({ kind: 'road', road: s.road })}>{s.name || (s.deck ? 'The bridge' : 'A stretch')}</button>
        {/each}
      </div>
      <p class="hint">Split at its {stretches.filter((s) => s.deck).length === 1 ? 'bridge' : 'bridges'}: each stretch is a road of its own, with its own card.</p>
    </div>
  {/if}
  <div class="option">
    <span class="eyebrow sub">Kind</span>
    <div class="kinds"><Segmented fill label="Kind of road" value={streetChoice.kind} options={KINDS} onchange={(v) => (streetChoice.kind = v)} /></div>
    <p class="hint">
      {#if streetChoice.kind === 'trail'}
        No cars and nothing along it; it does not cross the water.
      {:else if streetChoice.kind === 'lane'}
        A narrow lane, {streetWidth('lane')} m to a street's {streetWidth('street')}: two cars meeting on it pull over to pass. In {editor.townName || 'the town'}, a village or out of them (given to the nearest when the town is built).
      {:else if streetChoice.kind === 'street'}
        A street, {streetWidth('street')} m wide, in {editor.townName || 'the town'}, a village or out of them (given to the nearest when the town is built).
      {:else if streetChoice.kind === 'avenue'}
        A main road, {streetWidth('avenue')} m wide: two lanes each way and trees down the middle. It keeps well clear of other roads.
      {:else if streetChoice.kind === 'country'}
        A country road, {streetWidth('country')} m wide, with no sidewalks.
      {:else}
        A street in {editor.townName || 'the town'} or a village, a country road outside them{streetChoice.shown === 'country' ? ' (a country road here)' : ''}.
      {/if}
    </p>
  </div>
  {#if !trail}
    <div class="row">
      <span>One-way{#if streetChoice.oneway}{' '}<span class="sub-note">({streetChoice.flip ? 'against the way you draw it' : 'the way you draw it'})</span>{/if}</span>
      <span class="way">
        {#if streetChoice.oneway}
          <button class="btn small ghost" title="Flip which way cars go" aria-label="Flip which way cars go" onclick={() => (streetChoice.flip = !streetChoice.flip)}><Icon name="icon.ui.rotate" size={14} /> Flip</button>
        {/if}
        <Toggle label="One-way" bind:checked={streetChoice.oneway} />
      </span>
    </div>
  {/if}
  <label class="row">
    <span>Clear what's in the way{#if !streetChoice.clear}<span class="sub-note"> (else it is refused there)</span>{/if}</span>
    <Toggle label="Clear what's in the way" bind:checked={streetChoice.clear} />
  </label>
  <!-- (Step 2 has no "Along it": the zones beside a street decide what Build puts along it.) -->
  {#if editor.step !== 'layout'}
  <div class="option" class:off={trail}>
    <span class="eyebrow sub">Along it</span>
    <Segmented fill label="Along it" value={along} options={ALONG} onchange={(v) => (streetChoice.along = v)} />
    <p class="hint">
      {#if along === 'none'}
        Nothing along it; a street card's "Build up this street" can add it later.
      {:else if along === 'homes'}
        Homes from the town's mix of lots.
      {:else if along === 'buildings'}
        Shops, offices, terraces and flats as the town has them; none on a country road out of town.
      {:else}
        Homes, and the town's other buildings in the gaps; none on a country road out of town.
      {/if}
      {#if streetChoice.along !== 'auto'}<button class="link" onclick={() => (streetChoice.along = 'auto')}>By where it runs</button>{/if}
    </p>
  </div>
  {/if}
  <p class="hint keys"><kbd>Enter</kbd> or a double click ends it (on a road just ahead, else as a dead end), <kbd>Backspace</kbd> takes a point back, <kbd>Esc</kbd> drops it.</p>
</div>

<style>
  .street-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .option {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .option.off {
    opacity: 0.55;
    pointer-events: none;
  }
  .kinds :global(.segmented button) {
    padding: 0 6px;
    font-size: 12px;
  }
  .way {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .choices {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .chip {
    height: 28px;
    padding: 0 10px;
    border-radius: var(--radius-pill);
    background: var(--hairline);
    color: var(--text);
    font-size: 12px;
    font-weight: 600;
  }
  .chip.deck {
    font-style: italic;
  }
  .chip.on {
    background: var(--accent);
    color: #fff;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text);
  }
  .hint {
    margin: 0;
  }
  .sub-note {
    font-weight: 500;
    color: var(--text-muted);
  }
  .link {
    padding: 0;
    color: var(--accent);
    font: inherit;
    font-weight: 600;
    text-decoration: underline;
  }
  kbd {
    padding: 0 5px;
    border-radius: 4px;
    background: var(--hairline);
    font-family: inherit;
    font-size: 11px;
  }
</style>
