<script lang="ts">
  import type { SimView } from '../core/protocol';
  import Icon from './Icon.svelte';
  import SimPreview from './kit/SimPreview.svelte';
  import { friendLabel, kinLabel, romanceLabel } from './relationship';
  import { services } from './services';
  import { game } from './state.svelte';

  let { sim }: { sim: SimView } = $props();

  /** Phone actions with someone (text, call, invite over, ask to come over), and on their own (scrolling). */
  const actions = services.content.phoneActions.map((def, index) => ({ def, index }));
  const withSomeone = actions.filter((a) => a.def.contact);
  const alone = actions.filter((a) => !a.def.contact);

  const info = (id: number) => game.roster.find((r) => r.id === id);
  const closeness = (r: { friendship: number; romance: number; partners: boolean } | undefined) =>
    r ? r.friendship + Math.max(0, r.romance) * 0.5 + (r.partners ? 50 : 0) : -100;
  const phone = $derived(sim.phone);
  /** The people they're closest to first (partners and best friends at the top). */
  const contacts = $derived(
    (phone?.contacts ?? [])
      .map((c) => ({
        c,
        rel: game.relationships.find((r) => r.a === sim.id && r.b === c.id),
        back: game.relationships.find((r) => r.a === c.id && r.b === sim.id),
        who: info(c.id),
      }))
      .sort((x, y) => closeness(y.rel) - closeness(x.rel)),
  );

  /** Says what an action does, and why it's out if it is. */
  const describe = (action: number, contact: number) => {
    const name = info(contact)?.name ?? '';
    const label = `${services.content.phoneActions[action]?.label ?? ''} · ${name}`;
    const why = phone?.contacts.find((c) => c.id === contact)?.why[action] ?? null;
    return why ? `${label}: ${why}` : label;
  };
  /** The action under the pointer (its hint follows the contact's state as it changes). */
  let hovered = $state<{ action: number; contact: number } | null>(null);
  const hint = $derived(hovered ? describe(hovered.action, hovered.contact) : withSomeone.map((a) => a.def.label).join(' · '));
</script>

{#if !phone}
  <p class="idle">No phones in this town.</p>
{:else if phone.off}
  <p class="idle">{phone.off}.</p>
{:else}
  <div class="phone">
    {#if alone.length}
      <div class="alone">
        {#each alone as a (a.index)}
          <button class="btn small" onclick={() => services.controls.phone(a.index, null)}>
            <Icon name={a.def.icon ?? 'icon.ui.phone'} size={14} />{a.def.label}
          </button>
        {/each}
      </div>
    {/if}
    <ul class="contacts" aria-label="Contacts">
      {#each contacts as { c, rel, back, who } (c.id)}
        {@const family = kinLabel(rel?.kin, who?.gender)}
        {@const romance = romanceLabel(rel)}
        <li>
          <span class="mini">
            {#if who}<SimPreview appearance={who.appearance} gender={who.gender} id={who.id} size={30} />{/if}
          </span>
          <div class="person">
            <span class="name">
              <b>{who?.name}</b>
              <span class="state" class:out={!!c.status}><span class="dot"></span>{c.status ?? 'Free'}</span>
            </span>
            <span class="status">{family ? `${family} · ` : ''}{friendLabel(rel?.friendship ?? 0, back?.friendship ?? 0)}{romance ? ` · ${romance}` : ''}</span>
            <span class="track"><span class="mid"></span><span class="fill" class:neg={(rel?.friendship ?? 0) < 0} style="--v:{rel?.friendship ?? 0}"></span></span>
          </div>
          <div class="buttons">
            {#each withSomeone as a (a.index)}
              {@const why = c.why[a.index] ?? null}
              {@const text = describe(a.index, c.id)}
              <!-- (Disabled buttons get no pointer events: the wrapper tells why.) -->
              <span
                class="slot"
                role="presentation"
                title={text}
                onpointerenter={() => (hovered = { action: a.index, contact: c.id })}
                onpointerleave={() => (hovered = null)}
              >
                <button class="act" disabled={!!why} aria-label={text} onclick={() => services.controls.phone(a.index, c.id)}>
                  <Icon name={a.def.icon ?? 'icon.ui.phone'} size={15} />
                </button>
              </span>
            {/each}
          </div>
        </li>
      {:else}
        <li class="idle">{sim.name} hasn't met anyone with a phone yet.</li>
      {/each}
    </ul>
    <p class="hint">{hint}</p>
  </div>
{/if}

<style>
  .phone {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .alone {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .small {
    height: 30px;
    padding: 0 12px;
    gap: 6px;
    font-size: 12px;
  }
  .contacts {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: 220px;
    overflow: auto;
  }
  li {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .mini {
    display: grid;
    place-items: start start;
    width: 30px;
    height: 30px;
    flex: none;
    border-radius: 50%;
    overflow: hidden;
    background: var(--surface-muted);
  }
  .person {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .name {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 6px;
    font-size: 12.5px;
  }
  .name b {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .state {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    flex: none;
    font-size: 10.5px;
    font-weight: 600;
    color: var(--good);
  }
  .state.out {
    color: var(--text-muted);
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
  }
  .status {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-muted);
    font-size: 10.5px;
  }
  .track {
    position: relative;
    height: 4px;
    border-radius: var(--radius-pill);
    background: var(--hairline);
    overflow: hidden;
  }
  .mid {
    position: absolute;
    left: 50%;
    top: 0;
    bottom: 0;
    width: 1px;
    background: rgba(29, 34, 48, 0.25);
  }
  .fill {
    position: absolute;
    top: 0;
    bottom: 0;
    left: 50%;
    width: calc(max(var(--v), 0) * 0.5%);
    background: var(--good);
  }
  .fill.neg {
    left: calc(50% + var(--v) * 0.5%);
    width: calc(var(--v) * -0.5%);
    background: var(--bad);
  }
  .buttons {
    display: flex;
    gap: 2px;
    flex: none;
  }
  .slot {
    display: inline-flex;
  }
  .act {
    display: grid;
    place-items: center;
    width: 27px;
    height: 27px;
    border-radius: var(--radius-sm);
    color: var(--text);
    background: var(--hairline);
    transition: background var(--fast) var(--ease), color var(--fast) var(--ease);
  }
  .act:hover:not(:disabled) {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .act:disabled {
    opacity: 0.35;
    cursor: default;
  }
  .hint {
    margin: 0;
    min-height: 15px;
    font-size: 11px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .idle {
    margin: 0;
    padding: 4px 2px;
    color: var(--text-muted);
  }
</style>
