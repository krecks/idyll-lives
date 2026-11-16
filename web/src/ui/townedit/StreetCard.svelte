<script lang="ts">
  /**
   * The Street tool's card: what it draws (`streetChoice`). The kind, which is a street of the town
   * or village it runs in or a country road outside them unless a lane or a footpath is asked for;
   * whether what is in its way is cleared (else the street is refused there); and what is put along
   * it: homes now, the town's other buildings once streets can be built up.
   */
  import Segmented from '../kit/Segmented.svelte';
  import Toggle from '../kit/Toggle.svelte';
  import { streetChoice, type Along, type StreetKindChoice } from './streetChoice.svelte';
  import type { TownEditor } from './tools';

  const { editor }: { editor: TownEditor } = $props();

  const KINDS: readonly { value: StreetKindChoice; label: string }[] = [
    { value: 'street', label: 'Street' },
    { value: 'lane', label: 'Lane' },
    { value: 'country', label: 'Country road' },
    { value: 'trail', label: 'Footpath' },
  ];
  const ALONG: readonly { value: Along; label: string; disabled?: boolean }[] = [
    { value: 'homes', label: 'Homes' },
    { value: 'buildings', label: 'Town buildings', disabled: true },
    { value: 'both', label: 'Both', disabled: true },
    { value: 'none', label: 'Nothing' },
  ];

  /** `auto` shows as what the street under the pointer would be. */
  const kind = $derived<StreetKindChoice>(streetChoice.kind === 'auto' ? (streetChoice.shown === 'country' ? 'country' : 'street') : streetChoice.kind);
  const trail = $derived(streetChoice.kind === 'trail');
</script>

<div class="street-card">
  <div class="option">
    <span class="eyebrow sub">Kind</span>
    <Segmented fill label="Kind of road" value={kind} options={KINDS} onchange={(v) => (streetChoice.kind = v)} />
    <p class="hint">
      {#if streetChoice.kind === 'auto'}
        A street where it runs in {editor.townName || 'the town'} or a village, a country road outside them.
      {:else if streetChoice.kind === 'street' || streetChoice.kind === 'country'}
        A street in the town or a village, a country road outside them, whichever is picked.
        <button class="link" onclick={() => (streetChoice.kind = 'auto')}>By where it runs</button>
      {:else if streetChoice.kind === 'lane'}
        A narrow lane of the town or village it runs in; a country road outside them.
      {:else}
        A footpath: no cars, no homes along it, and it does not cross the water.
      {/if}
    </p>
  </div>
  <label class="row">
    <span>Clear what's in the way</span>
    <Toggle label="Clear what's in the way" bind:checked={streetChoice.clear} />
  </label>
  {#if !streetChoice.clear}
    <p class="hint">A street over a lot or a building is refused.</p>
  {/if}
  <div class="option" class:off={trail}>
    <span class="eyebrow sub">Along it</span>
    <Segmented fill label="Along it" bind:value={streetChoice.along} options={ALONG} />
    <p class="hint">
      {#if trail}
        Nothing is put along a footpath.
      {:else}
        Town buildings along it are coming, with building streets up. A country road outside the town and villages gets nothing.
      {/if}
    </p>
  </div>
  <p class="hint keys">
    Click to set points, or drag to draw. It ends on the road it reaches; <kbd>Enter</kbd> or a double click ends it as a dead end. <kbd>Backspace</kbd> takes the last point back, <kbd>Esc</kbd> drops it.
  </p>
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
