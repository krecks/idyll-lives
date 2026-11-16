<script lang="ts">
  /**
   * What can be done to what is picked (`editor.selected`):
   * - a home: another house, its garden, remove it; a public place: another of its size; a farm or
   *   an outdoor spot: remove it;
   * - a street: rename it, remove it (what goes with it listed first), build it up with homes, the
   *   town's other buildings or both (`buildup.ts`: both in the main town, homes elsewhere, unless
   *   chosen; each press another seed), or clear the buildings fronting it;
   * - a building: a civic place made another the town hasn't got, remove it;
   * - a village: rename it (moving and removing it come with the Village tool);
   * - the main town: rename it (step 2's own name field, the two kept as one).
   */
  import { areaOfRoad, buildingsOn, buildUp, clearBuildings, notBuilt, roadKey, type BuildMode } from '../../game/edit/buildup';
  import { anotherHouse, civicSwaps, publicChoices, removeLotsEdit, setGarden, swapCivic, swapPlace } from '../../game/edit/lots';
  import { removeRoad, rename, whatGoes } from '../../game/edit/streets';
  import type { GardenSize } from '../../game/housegen';
  import { lengths } from '../../game/roads';
  import { generatedId, slotDims, type LotSize } from '../../game/town';
  import Icon from '../Icon.svelte';
  import Segmented from '../kit/Segmented.svelte';
  import RenameField from './RenameField.svelte';
  import type { TownEditor } from './tools';
  import { count, listed, lotKind, sentence, whatGoesLine } from './words';

  const { editor }: { editor: TownEditor } = $props();

  const picked = $derived(editor.selected);
  const draft = $derived(editor.draft);
  const t = $derived(editor.t);

  const GARDENS: readonly { value: GardenSize; label: string }[] = [
    { value: 'snug', label: 'Snug' },
    { value: 'roomy', label: 'Roomy' },
    { value: 'big', label: 'Big' },
  ];
  const ROAD_KIND: Record<string, string> = { street: 'Street', lane: 'Lane', country: 'Country road', trail: 'Footpath' };
  const USE: Record<string, string> = { office: 'Offices', shop: 'Shop', restaurant: 'Restaurant', school: 'School', church: 'Church', workshop: 'Workshop', home: 'Homes', flats: 'Flats', terrace: 'Terrace', barn: 'Barn' };

  // ---- a lot ----
  const lot = $derived(picked?.kind === 'lot' ? draft.slots[picked.slot] : undefined);
  /** A generated house's lot size and garden (`gen:<size>:<seed>[:<garden>]`); null: a fixed layout. */
  const gen = $derived.by(() => {
    if (!lot?.template.startsWith('gen:')) return null;
    const [, size, , garden] = lot.template.split(':');
    return { size: size as LotSize, garden: (garden ?? lot.garden ?? 'roomy') as GardenSize };
  });
  const homeLike = $derived(!!lot && lot.kind === 'house' && lot.lot !== 'farm' && lot.lot !== 'terrace');
  const swaps = $derived(lot && (lot.kind === 'park' || lot.kind === 'shop') ? publicChoices(t, slotDims(t, lot)) : []);
  const nameOf = (id: string) => t.parks.find((p) => p.id === id)?.name ?? id;

  /** Another generated house of the same size and garden on the lot: the first of a few new ones that fits. */
  function another() {
    if (!lot || !gen || picked?.kind !== 'lot') return;
    let why = '';
    for (let k = 0; k < 8; k++) {
      const e = anotherHouse(draft, t, picked.slot, generatedId(gen.size, editor.seed(), gen.garden));
      if ('draft' in e) return void editor.apply(e);
      why = e.refused;
    }
    editor.refuse(why || 'no other house fits this lot');
  }

  // ---- a road ----
  const road = $derived(picked?.kind === 'road' ? draft.roads[picked.road] : undefined);
  const goes = $derived(picked?.kind === 'road' ? whatGoes(draft, t, picked.road) : null);
  const goesLines = $derived(goes && !('refused' in goes) && picked?.kind === 'road' ? whatGoesLine(draft, goes, picked.road) : []);
  const roadLength = $derived(road ? Math.round(lengths(road.points).at(-1) ?? 0) : 0);
  const BUILD: readonly { value: BuildMode; label: string }[] = [
    { value: 'homes', label: 'Homes' },
    { value: 'buildings', label: 'Town buildings' },
    { value: 'both', label: 'Both' },
  ];
  /** What building it up puts along it: as chosen for this road, else both in the main town and homes elsewhere. */
  const roadId = $derived(road ? roadKey(road) : '');
  let buildChosen = $state<{ road: string; mode: BuildMode } | null>(null);
  const buildMode = $derived<BuildMode>(buildChosen?.road === roadId ? buildChosen.mode : picked?.kind === 'road' && areaOfRoad(draft, picked.road) === 0 ? 'both' : 'homes');
  const buildNo = $derived(picked?.kind === 'road' ? notBuilt(draft, picked.road) : null);
  const builtCount = $derived(picked?.kind === 'road' ? buildingsOn(draft, picked.road).length : 0);
  /** What the last build-up of this road left out where the ground is too steep. */
  let builtNote = $state<{ road: string; text: string } | null>(null);

  function buildIt() {
    if (picked?.kind !== 'road') return;
    const e = buildUp(draft, t, editor.content, { road: picked.road, mode: buildMode, seed: editor.seed() });
    const key = roadId;
    if (!editor.apply(e) || !('draft' in e)) return;
    builtNote = e.leftOut ? { road: key, text: `${count(e.leftOut, 'building was', 'buildings were')} left out where the ground is too steep.` } : null;
  }

  // ---- a building ----
  const building = $derived(picked?.kind === 'building' ? draft.open?.backdrop?.[picked.building] : undefined);
  const place = $derived(building?.place === undefined ? undefined : draft.open?.places?.[building.place]);
  const civics = $derived(picked?.kind === 'building' ? civicSwaps(draft, picked.building) : []);
  let civicPick = $state(-1);

  // ---- a village ----
  const area = $derived(picked?.kind === 'village' ? draft.open?.areas[picked.area] : undefined);
  const areaLots = $derived(picked?.kind === 'village' ? draft.slots.filter((s) => s.area === picked.area) : []);

  const renamed = (what: Parameters<typeof rename>[1]) => (name: string) => editor.apply(rename(draft, what, name));

  /** Svelte action: the card brought into the panel's view when something else is picked (a narrow screen's panel scrolls). */
  function shown(el: HTMLElement, key: string) {
    const bring = () => el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    bring();
    return { update: (next: string) => next !== key && ((key = next), bring()) };
  }
</script>

{#if picked}
  <div class="card" aria-live="polite" use:shown={JSON.stringify(picked)}>
    <button class="close btn ghost" aria-label="Put it down" title="Put it down (Esc)" onclick={() => editor.select(null)}><Icon name="icon.ui.close" size={14} /></button>
    {#if lot && picked.kind === 'lot'}
      <span class="eyebrow sub">{lotKind(lot)}</span>
      <h3>{lot.name}</h3>
      {#if homeLike && gen}
        <div class="row">
          <button class="btn small" onclick={another}><Icon name="icon.ui.dice" size={14} /> Another house</button>
        </div>
        <div class="option">
          <span class="eyebrow sub">Garden</span>
          <Segmented fill label="Garden" value={gen.garden} options={GARDENS} onchange={(g) => editor.apply(setGarden(draft, t, picked.slot, g))} />
        </div>
      {/if}
      {#if swaps.length}
        <div class="option">
          <span class="eyebrow sub">Make it</span>
          <div class="choices" role="radiogroup" aria-label="Public place">
            {#each swaps as id (id)}
              <button role="radio" class="chip" aria-checked={lot.template === id} class:on={lot.template === id} onclick={() => lot.template !== id && editor.apply(swapPlace(draft, t, editor.content, picked.slot, id))}>{nameOf(id)}</button>
            {/each}
          </div>
        </div>
      {/if}
      <div class="row">
        <button class="btn small danger-soft" onclick={() => editor.apply(removeLotsEdit(draft, new Set([picked.slot])))}><Icon name="icon.ui.trash" size={14} /> Remove</button>
      </div>
    {:else if road && picked.kind === 'road'}
      <span class="eyebrow sub">{ROAD_KIND[road.kind ?? 'street'] ?? 'Road'}{road.bridge ? ', a bridge' : ''} · {roadLength} m</span>
      <RenameField label="Street name" value={road.name ?? ''} onname={renamed({ kind: 'road', road: picked.road })} />
      <div class="option">
        <span class="eyebrow sub">Remove it</span>
        {#if goes && 'refused' in goes}
          <p class="hint warn">{sentence(goes.refused)}</p>
        {:else}
          <p class="hint">{goesLines.length ? `It takes with it ${listed(goesLines)}.` : 'Nothing else goes with it.'}</p>
        {/if}
        <div class="row">
          <button class="btn small danger-soft" disabled={!goes || 'refused' in goes} onclick={() => editor.apply(removeRoad(draft, t, picked.road))}><Icon name="icon.ui.trash" size={14} /> Remove {road.name ?? 'it'}</button>
        </div>
      </div>
      <div class="option">
        <span class="eyebrow sub">Along it</span>
        {#if buildNo}
          <p class="hint warn">{sentence(buildNo)}</p>
        {:else}
          <Segmented fill label="Build it up with" value={buildMode} options={BUILD} onchange={(m) => (buildChosen = { road: roadId, mode: m })} />
        {/if}
        <div class="row">
          <button class="btn small" disabled={!!buildNo} title="Fills the free frontage; press again for another go" onclick={buildIt}><Icon name="icon.ui.home" size={14} /> Build up this street</button>
          <button class="btn small danger-soft" disabled={!builtCount} onclick={() => picked.kind === 'road' && editor.apply(clearBuildings(draft, picked.road))}><Icon name="icon.ui.trash" size={14} /> Clear its buildings</button>
        </div>
        <p class="hint">
          {builtCount ? `${count(builtCount, 'building')} front${builtCount === 1 ? 's' : ''} it.` : 'No buildings front it.'}
          {#if builtNote?.road === roadId}{builtNote.text}{/if}
        </p>
      </div>
    {:else if building && picked.kind === 'building'}
      <span class="eyebrow sub">{place ? 'Civic place' : 'Building'} · {USE[building.use] ?? building.use}, {count(building.storeys, 'storey')}</span>
      <h3>{place?.name ?? 'A building'}</h3>
      {#if civics.length}
        <div class="option">
          <span class="eyebrow sub">Make it</span>
          <div class="choices" aria-label="Another civic place">
            {#each civics as c (c.civic)}
              <button class="chip" class:on={civicPick === c.civic} onclick={() => ((civicPick = c.civic), editor.apply(swapCivic(draft, picked.building, c.civic, editor.content, editor.seed())))}>{c.label}</button>
            {/each}
          </div>
          <p class="hint">The places the town hasn't got. Its footprint stays.</p>
        </div>
      {/if}
      <div class="row">
        <button class="btn small danger-soft" onclick={() => editor.apply(removeLotsEdit(draft, new Set(), new Set([picked.building])))}><Icon name="icon.ui.trash" size={14} /> Remove</button>
      </div>
    {:else if area && picked.kind === 'village'}
      <span class="eyebrow sub">Village · {count(areaLots.filter((s) => s.kind === 'house').length, 'home')}</span>
      <RenameField label="Village name" value={area.name} onname={renamed({ kind: 'area', area: picked.area })} />
      <p class="hint">Moving and removing a village come with the Village tool.</p>
    {:else if picked.kind === 'town'}
      <span class="eyebrow sub">The town</span>
      <RenameField label="Town name" value={editor.townName} onname={(name) => ((editor.townName = name), true)} />
      <p class="hint">Also its name at the top.</p>
    {/if}
  </div>
{/if}

<style>
  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--surface) 70%, transparent);
    border: 1px solid var(--hairline);
  }
  h3 {
    margin: 0;
    padding-right: 28px;
    font-size: 16px;
    font-weight: 750;
    color: var(--text-strong);
  }
  .close {
    position: absolute;
    top: 6px;
    right: 6px;
    width: 28px;
    height: 28px;
    padding: 0;
    justify-content: center;
  }
  .option {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
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
  .chip.on {
    background: var(--accent);
    color: #fff;
  }
  .hint {
    margin: 0;
  }
  .hint.warn {
    color: #a3552c;
  }
  .danger-soft {
    color: var(--bad);
  }
</style>
