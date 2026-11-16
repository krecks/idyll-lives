<script lang="ts">
  /**
   * Step 2's town editor, in the side panel under the town's options: the tools (`TOWN_TOOLS`, one in
   * hand at a time; Esc puts it down), undo, redo and reset (Ctrl/⌘+Z, Shift for redo), the card of
   * the tool in hand and of what is picked, a line on what the tool would do where the pointer is,
   * and the Problems list (those that block the way on first; a click goes to where one is).
   *
   * Edits show at once as ghosts over the town; the town is built again with them once they pause
   * (`session.ts`). Meanwhile a line says the town is being brought up to date.
   */
  import Icon from '../Icon.svelte';
  import type { TownEditorState } from './editor.svelte';
  import type { Issue } from './problems';
  import SelectCard from './SelectCard.svelte';
  import { TOWN_TOOLS } from './tools';

  const { editor, disabled = false }: { editor: TownEditorState; disabled?: boolean } = $props();

  const tool = $derived(editor.tool);
  const issues = $derived(editor.issues);
  const off = $derived(disabled || !editor.ready);
  let resetting = $state(false);

  function onkeydown(e: KeyboardEvent) {
    if (off) return;
    const t = e.target;
    if (t instanceof HTMLInputElement || t instanceof HTMLTextAreaElement || t instanceof HTMLSelectElement) return;
    // (The tool in hand first: Esc drops what it is drawing before it puts the tool down. A key the
    // town stage took already, Esc ending a drag, is done with.)
    if (e.defaultPrevented) return;
    if (!e.ctrlKey && !e.metaKey && !e.altKey && editor.stageTool?.key?.(e)) {
      e.preventDefault();
      return;
    }
    if (e.key === 'Escape') {
      if (!editor.toolId && !editor.selected) return;
      editor.take(null);
      editor.select(null);
      return;
    }
    if (!(e.ctrlKey || e.metaKey) || e.altKey) return;
    const key = e.key.toLowerCase();
    if (key === 'z') {
      e.preventDefault();
      if (e.shiftKey) editor.redo();
      else editor.undo();
    } else if (key === 'y' && e.ctrlKey) {
      e.preventDefault();
      editor.redo();
    }
  }

  const go = (i: Issue) => i.at && editor.glide(i);
</script>

<svelte:window {onkeydown} />

<section class="group editor" aria-labelledby="editor-head" class:off>
  <div class="top">
    <h2 class="eyebrow" id="editor-head">Change it</h2>
    {#if editor.settling && editor.ready}
      <span class="busy" role="status"><span class="spinner" aria-hidden="true"></span> Bringing the town up to date</span>
    {/if}
  </div>

  <div class="tools" role="radiogroup" aria-label="Town tools">
    {#each TOWN_TOOLS as t (t.id)}
      <button class="tool" role="radio" aria-checked={editor.toolId === t.id} class:on={editor.toolId === t.id} disabled={off} title={t.does} data-tool={t.id} onclick={() => editor.take(t.id)}>
        <Icon name={t.icon} size={16} />
        <span>{t.label}</span>
      </button>
    {/each}
  </div>

  <div class="history">
    <button class="btn ghost small" disabled={off || !editor.canUndo} title="Undo the last change (Ctrl+Z / ⌘Z)" onclick={() => editor.undo()}><Icon name="icon.ui.undo" size={14} /> Undo</button>
    <button class="btn ghost small" disabled={off || !editor.canRedo} title="Redo (Ctrl+Shift+Z / ⌘⇧Z)" onclick={() => editor.redo()}><span class="mirror"><Icon name="icon.ui.undo" size={14} /></span> Redo</button>
    {#if resetting}
      <span class="confirm" role="group" aria-label="Put the town back as it was laid?">
        <button class="btn small danger" onclick={() => ((resetting = false), editor.reset())}>Reset all</button>
        <button class="btn ghost small" onclick={() => (resetting = false)}>Keep</button>
      </span>
    {:else}
      <button class="btn ghost small reset" disabled={off || editor.edits === 0} title="Put the town back as it was laid" onclick={() => (resetting = true)}>Reset the town</button>
    {/if}
  </div>

  {#if !off}
    {#if tool}
      <p class="hint does"><b>{tool.label}</b> {tool.does} <kbd>Esc</kbd> puts it down.</p>
      <!-- (What the tool would do under the pointer, above its card: a tall card would push it out of view.) -->
      {#if editor.hintText}
        <p class="hint pointer" aria-live="polite">{editor.hintText}</p>
      {/if}
      {#if tool.card}
        {@const Card = tool.card}
        <Card {editor} />
      {/if}
    {:else}
      <p class="hint does">Take a tool, or click a lot to see what can be done to it.</p>
      {#if editor.hintText}
        <p class="hint pointer" aria-live="polite">{editor.hintText}</p>
      {/if}
    {/if}
    {#if editor.note}
      <p class="hint warn" role="alert">{editor.note}</p>
    {/if}
    {#if tool?.card !== SelectCard}
      <!-- (What is picked shows its card with any tool in hand: the Select tool's own card is it.) -->
      <SelectCard {editor} />
    {/if}
  {/if}
</section>

<section class="group problems" aria-labelledby="problems-head">
  <div class="top">
    <h2 class="eyebrow" id="problems-head">Problems</h2>
    {#if !editor.checked}
      <span class="busy"><span class="spinner" aria-hidden="true"></span> Checking</span>
    {/if}
  </div>
  {#if !issues.blocking.length && !issues.warnings.length}
    <p class="hint">None: the town is ready for its households.</p>
  {:else}
    <ul>
      {#each issues.blocking as i, k (k)}
        <li class="blocking"><button disabled={!i.at} onclick={() => go(i)}><span class="dot" aria-hidden="true"></span>{i.text}</button></li>
      {/each}
      {#each issues.warnings as i, k (k)}
        <li class="warning"><button disabled={!i.at} onclick={() => go(i)}><span class="dot" aria-hidden="true"></span>{i.text}</button></li>
      {/each}
    </ul>
    {#if issues.blocking.length}
      <p class="hint warn">Mend the problems in red (or undo what made them) to go on.</p>
    {/if}
  {/if}
</section>

<style>
  .editor.off .tools {
    opacity: 0.55;
  }
  .top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .busy {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 11.5px;
    color: var(--text-muted);
  }
  .spinner {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    border: 2px solid var(--hairline);
    border-top-color: var(--accent);
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .editor {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .tools {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(64px, 1fr));
    gap: 4px;
  }
  .tool {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
    padding: 6px 2px 5px;
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
    color: var(--text-muted);
    font-size: 11.5px;
    font-weight: 650;
  }
  .tool:hover:not(:disabled) {
    color: var(--text);
    background: rgba(255, 255, 255, 0.7);
  }
  .tool.on {
    background: #fff;
    color: var(--accent);
    border-color: color-mix(in srgb, var(--accent) 35%, transparent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  .tool:disabled {
    cursor: default;
  }
  .history {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px;
  }
  .reset,
  .confirm {
    margin-left: auto;
  }
  .confirm {
    display: inline-flex;
    gap: 4px;
  }
  .danger {
    background: var(--bad);
    color: #fff;
  }
  .mirror {
    display: inline-flex;
    transform: scaleX(-1);
  }
  .hint {
    margin: 0;
  }
  .hint b {
    margin-right: 3px;
    color: var(--text-strong);
  }
  .hint.pointer {
    color: var(--text);
    font-weight: 550;
  }
  .hint.warn {
    color: #a3552c;
  }
  kbd {
    padding: 0 5px;
    border-radius: 4px;
    background: var(--hairline);
    font-family: inherit;
    font-size: 11px;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  li button {
    display: flex;
    align-items: baseline;
    gap: 8px;
    width: 100%;
    padding: 4px 6px;
    border-radius: var(--radius-sm);
    text-align: left;
    font-size: 12.5px;
    line-height: 1.35;
    color: var(--text);
  }
  li button:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.7);
  }
  li button:disabled {
    cursor: default;
  }
  .dot {
    flex: none;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    transform: translateY(-1px);
  }
  .blocking .dot {
    background: var(--bad);
  }
  .warning .dot {
    background: var(--warn);
  }
</style>
