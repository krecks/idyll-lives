<script lang="ts">
  /**
   * The Street tool's card: what it draws (`streetChoice`). The kind, which is a street of the town
   * or village it runs in or a country road outside them unless a lane or a footpath is asked for;
   * whether what is in its way is cleared (else the street is refused there); and what is put along
   * it: homes, the town's other buildings, both, or nothing; by where it runs unless chosen (both in
   * the main town, homes elsewhere).
   */
  import Segmented from '../kit/Segmented.svelte';
  import Toggle from '../kit/Toggle.svelte';
  import { alongFor, streetChoice, type Along, type StreetKindChoice } from './streetChoice.svelte';
  import type { TownEditor } from './tools';

  const { editor }: { editor: TownEditor } = $props();

  const KINDS: readonly { value: StreetKindChoice; label: string }[] = [
    { value: 'street', label: 'Street' },
    { value: 'lane', label: 'Lane' },
    { value: 'country', label: 'Country road' },
    { value: 'trail', label: 'Footpath' },
  ];
  const ALONG: readonly { value: Along; label: string }[] = [
    { value: 'homes', label: 'Homes' },
    { value: 'buildings', label: 'Town buildings' },
    { value: 'both', label: 'Both' },
    { value: 'none', label: 'Nothing' },
  ];

  /** `auto` shows as what the street under the pointer would be. */
  const kind = $derived<StreetKindChoice>(streetChoice.kind === 'auto' ? (streetChoice.shown === 'country' ? 'country' : 'street') : streetChoice.kind);
  const trail = $derived(streetChoice.kind === 'trail');
  /** `auto` shows as what goes along a street where the pointer is (both in the main town, homes elsewhere). */
  const along = $derived(alongFor(streetChoice.along, streetChoice.shownArea));
</script>

<div class="street-card">
  <div class="option">
    <span class="eyebrow sub">Kind</span>
    <Segmented fill label="Kind of road" value={kind} options={KINDS} onchange={(v) => (streetChoice.kind = v)} />
    <p class="hint">
      {#if streetChoice.kind === 'trail'}
        No cars and nothing along it; it does not cross the water.
      {:else if streetChoice.kind === 'lane'}
        A narrow lane in {editor.townName || 'the town'} or a village; a country road outside them.
      {:else}
        A street in {editor.townName || 'the town'} or a village, a country road outside them.
        {#if streetChoice.kind !== 'auto'}<button class="link" onclick={() => (streetChoice.kind = 'auto')}>By where it runs</button>{/if}
      {/if}
    </p>
  </div>
  <label class="row">
    <span>Clear what's in the way{#if !streetChoice.clear}<span class="sub-note"> (else it is refused there)</span>{/if}</span>
    <Toggle label="Clear what's in the way" bind:checked={streetChoice.clear} />
  </label>
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
  <p class="hint keys"><kbd>Enter</kbd> or a double click ends it as a dead end, <kbd>Backspace</kbd> takes a point back, <kbd>Esc</kbd> drops it.</p>
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
