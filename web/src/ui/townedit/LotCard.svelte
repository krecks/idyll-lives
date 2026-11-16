<script lang="ts">
  /**
   * The Lot tool's card: what it puts down (`lotChoice`). A home of a size, with a garden (or as the
   * world's homes have theirs); a farm; one of the public places a town's lot can be; an outdoor
   * spot; or a civic place the town hasn't got (placed by hand once streets can be built up).
   */
  import { publicChoices } from '../../game/edit/lots';
  import Segmented from '../kit/Segmented.svelte';
  import { civicLacking, lotChoice, outdoorChoices, type HomeSize, type LotGroup } from './lotChoice.svelte';
  import type { TownEditor } from './tools';

  const { editor }: { editor: TownEditor } = $props();

  const GROUPS: readonly { value: LotGroup; label: string }[] = [
    { value: 'home', label: 'Home' },
    { value: 'farm', label: 'Farm' },
    { value: 'public', label: 'Public' },
    { value: 'outdoor', label: 'Outdoor' },
    { value: 'civic', label: 'Civic' },
  ];
  const SIZES: readonly { value: HomeSize; label: string }[] = [
    { value: 'small', label: 'Small' },
    { value: 'medium', label: 'Medium' },
    { value: 'large', label: 'Large' },
  ];
  const GARDENS = [
    { value: 'any', label: 'Any' },
    { value: 'snug', label: 'Snug' },
    { value: 'roomy', label: 'Roomy' },
    { value: 'big', label: 'Big' },
  ] as const;

  const t = $derived(editor.t);
  const nameOf = (id: string) => t.parks.find((p) => p.id === id)?.name ?? id;
  const places = $derived(publicChoices(t));
  const outdoors = $derived(outdoorChoices(t));
  const civics = $derived((void editor.version, civicLacking(editor.draft)));
</script>

<div class="lot-card">
  <Segmented fill label="What to put down" bind:value={lotChoice.group} options={GROUPS} />
  {#if lotChoice.group === 'home'}
    <div class="option">
      <span class="eyebrow sub">Size</span>
      <Segmented fill label="Home size" bind:value={lotChoice.size} options={SIZES} />
    </div>
    <div class="option">
      <span class="eyebrow sub">Garden</span>
      <Segmented fill label="Garden" bind:value={lotChoice.garden} options={GARDENS} />
    </div>
  {:else if lotChoice.group === 'farm'}
    <p class="hint">A farmhouse with its fields and farmyard. It needs a wide stretch of road, out of the town centre.</p>
  {:else if lotChoice.group === 'public'}
    <div class="choices" role="radiogroup" aria-label="Public place">
      {#each places as id, i (id)}
        <button role="radio" class="chip" aria-checked={(lotChoice.place || places[0]) === id} class:on={(lotChoice.place || places[0]) === id} tabindex={i === 0 ? 0 : -1} onclick={() => (lotChoice.place = id)}>{nameOf(id)}</button>
      {/each}
    </div>
  {:else if lotChoice.group === 'outdoor'}
    <div class="choices" role="radiogroup" aria-label="Outdoor spot">
      {#each outdoors as id, i (id)}
        <button role="radio" class="chip" aria-checked={(lotChoice.outdoor || outdoors[0]) === id} class:on={(lotChoice.outdoor || outdoors[0]) === id} tabindex={i === 0 ? 0 : -1} onclick={() => (lotChoice.outdoor = id)}>{nameOf(id)}</button>
      {/each}
    </div>
    <p class="hint">A short footpath is laid to it from the road.</p>
  {:else}
    {#if civics.length}
      <div class="choices" aria-label="Civic places the town has not got">
        {#each civics as c (c.civic)}
          <button class="chip" disabled title="Coming with building streets up">{c.label}</button>
        {/each}
      </div>
    {:else}
      <p class="hint">The town has every civic place already.</p>
    {/if}
    <p class="hint">Coming soon: a civic place put down by hand, with building streets up.</p>
  {/if}
</div>

<style>
  .lot-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .option {
    display: flex;
    flex-direction: column;
    gap: 4px;
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
  .chip.on {
    background: var(--accent);
    color: #fff;
  }
  .chip:disabled {
    opacity: 0.55;
    cursor: default;
  }
  .hint {
    margin: 0;
  }
</style>
