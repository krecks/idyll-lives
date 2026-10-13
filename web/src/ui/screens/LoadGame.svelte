<script lang="ts">
  import { onMount } from 'svelte';
  import { SimBridge } from '../../core/bridge';
  import { loadGameData } from '../../game/session';
  import { deleteSave, exportSave, importProblemText, listSaves, onSavesChanged, parseSaveFile, playable, writeSave, type SaveMeta } from '../../persistence/saves';
  import { settings } from '../../settings/settings.svelte';
  import { app } from '../app.svelte';
  import { clock, timeAgo } from '../format';
  import Icon from '../Icon.svelte';
  import { toast } from '../state.svelte';

  let saves = $state<SaveMeta[] | null>(null);
  let confirmDelete = $state<string | null>(null);
  let error = $state('');
  let importing = $state(false);
  let fileInput = $state<HTMLInputElement>();

  async function refresh() {
    try {
      saves = await listSaves();
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
      saves = [];
    }
  }

  async function remove(id: string) {
    await deleteSave(id);
    confirmDelete = null;
  }

  /** Downloads the save as a file (to keep a copy, or to play it in another browser). */
  async function download(id: string) {
    try {
      const { name, text } = await exportSave(id);
      const url = URL.createObjectURL(new Blob([text], { type: 'application/json' }));
      const a = document.createElement('a');
      a.href = url;
      a.download = name;
      a.click();
      setTimeout(() => URL.revokeObjectURL(url), 10_000);
    } catch (err) {
      error = `Exporting failed: ${err instanceof Error ? err.message : err}`;
    }
  }

  /** Adds a save file to the list, if this version of the game can open it. */
  async function upload(file: File | undefined) {
    if (!file) return;
    importing = true;
    error = '';
    try {
      const record = parseSaveFile(await file.text());
      if (!record) {
        error = "That file isn't an Idyll Lives save.";
        return;
      }
      const problem = await SimBridge.checkSave((await loadGameData()).content.json, record.data);
      if (problem) {
        error = importProblemText(problem);
        return;
      }
      await writeSave(record);
      toast(`Imported “${record.name}”`);
    } catch (err) {
      error = `Importing failed: ${err instanceof Error ? err.message : err}`;
    } finally {
      importing = false;
      if (fileInput) fileInput.value = '';
    }
  }

  onMount(() => {
    void refresh();
    return onSavesChanged(() => void refresh());
  });
</script>

{#if app.screen === 'game'}
  <p class="notice">Loading replaces the current game. Unsaved progress will be lost.</p>
{/if}
{#if error}<p class="notice bad">{error}</p>{/if}

{#if saves === null}
  <p class="empty">Loading saves…</p>
{:else if saves.length === 0}
  <p class="empty">No saved games yet. Saves appear here after you save from the pause menu, or after an autosave.</p>
{:else}
  <ul>
    {#each saves as s (s.id)}
      <li>
        <div class="thumb">
          {#if s.thumbnail}<img src={s.thumbnail} alt="" />{/if}
        </div>
        <div class="info">
          <b>{s.name}</b>
          <span>{s.members.join(', ')}</span>
          <span class="when">Day {s.day} · {clock(s.minute, settings.clock24h)} · saved {timeAgo(s.savedAt)}</span>
          {#if !playable(s)}<span class="newer">{s.failedIn ? "This version couldn't open it" : 'Needs a newer version of Idyll Lives'}</span>{/if}
        </div>
        <div class="actions">
          {#if confirmDelete === s.id}
            <button class="btn danger solid" onclick={() => remove(s.id)}>Delete</button>
            <button class="btn ghost" onclick={() => (confirmDelete = null)}>Keep</button>
          {:else}
            <button class="btn ghost" aria-label="Delete {s.name}" onclick={() => (confirmDelete = s.id)}>
              <Icon name="icon.ui.trash" size={16} />
            </button>
            <button class="btn ghost" title="Download this save as a file" onclick={() => download(s.id)}>Export</button>
            <button class="btn primary" disabled={!playable(s)} onclick={() => app.start({ kind: 'load', saveId: s.id })}>Load</button>
          {/if}
        </div>
      </li>
    {/each}
  </ul>
{/if}

<div class="files">
  <span>Saves are kept in this browser. Export one to keep a copy, or to play it in another browser.</span>
  <button class="btn ghost" disabled={importing} onclick={() => fileInput?.click()}>{importing ? 'Importing…' : 'Import a save…'}</button>
  <input type="file" accept=".json,application/json" hidden bind:this={fileInput} onchange={() => upload(fileInput?.files?.[0])} />
</div>

<style>
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  li {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 10px;
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: var(--shadow-sm);
  }
  .thumb {
    width: 112px;
    height: 70px;
    flex: none;
    border-radius: var(--radius-sm);
    overflow: hidden;
    background: linear-gradient(135deg, #dfe7f6, #f3e6da);
  }
  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .info span {
    color: var(--text-muted);
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .when {
    font-size: 12px !important;
  }
  .info .newer {
    font-size: 12px;
    color: #8a5e0c;
    font-weight: 550;
  }
  .files {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 14px;
    color: var(--text-muted);
    font-size: 13px;
  }
  .files span {
    flex: 1;
  }
  .actions {
    display: flex;
    gap: 6px;
  }
  .empty {
    color: var(--text-muted);
    padding: 24px 0;
    text-align: center;
  }
  .notice {
    margin: 0 0 12px;
    padding: 10px 14px;
    border-radius: var(--radius-md);
    background: rgba(240, 181, 74, 0.16);
    color: #8a5e0c;
    font-weight: 550;
  }
  .notice.bad {
    background: rgba(236, 106, 92, 0.12);
    color: var(--bad);
  }
</style>
