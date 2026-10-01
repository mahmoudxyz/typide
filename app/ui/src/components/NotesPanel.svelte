<script lang="ts">
  import Icon from "./Icon.svelte";
  import { data, openFile } from "../lib/store.svelte";
</script>

<div class="wrap">
  <div class="search">
    <Icon name="search" size={14} />
    <input placeholder="Search notes (FTS5)…" />
  </div>
  {#if data.notes.length === 0}
    <div class="nnone"><Icon name="book" size={26} /><p>No notes yet</p><span>Add files under <code>notes/</code> or use <code>[[wiki-links]]</code>.</span></div>
  {/if}
  {#each data.notes as note (note.path)}
    <button class="note" onclick={() => openFile(note.path)}>
      <div class="top">
        <span class="ic"><Icon name="book" size={15} /></span>
        <span class="title">{note.title}</span>
        {#if note.citeKey}
          <span class="cite" title="Literature note"><Icon name="quote" size={11} /> {note.citeKey}</span>
        {/if}
      </div>
      <div class="meta">
        <span title="Backlinks">
          <Icon name="link" size={11} />
          {note.backlinks.length} backlink{note.backlinks.length === 1 ? "" : "s"}
        </span>
        {#if note.outgoing.length}
          <span class="out">→ {note.outgoing.join(", ")}</span>
        {/if}
      </div>
    </button>
  {/each}
  <div class="hint"><Icon name="info" size={12} /> Wiki links <code>[[note]]</code> · daily note · graph view (M5)</div>
</div>

<style>
  .wrap {
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .search {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 8px;
    height: 30px;
    background: var(--bg-inset);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--fg-3);
  }
  .search input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    font-size: 12.5px;
  }
  .note {
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px 9px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg-2);
    transition: border-color 0.12s, background 0.12s;
  }
  .note:hover {
    border-color: var(--border-strong);
    background: var(--bg-3);
  }
  .top {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .ic {
    color: var(--accent-2);
    display: inline-flex;
  }
  .title {
    font-weight: 600;
    font-size: 12.5px;
    color: var(--fg-0);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .cite {
    margin-left: auto;
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: 10px;
    font-family: "JetBrains Mono", monospace;
    color: var(--accent-2);
    background: var(--ok-soft);
    padding: 1px 6px;
    border-radius: 4px;
    flex: none;
  }
  .meta {
    display: flex;
    gap: 10px;
    font-size: 11px;
    color: var(--fg-2);
    align-items: center;
  }
  .meta span {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
  .out {
    color: var(--fg-3);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .hint {
    margin-top: 4px;
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--fg-3);
    padding: 6px 4px;
  }
  .nnone {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 5px;
    padding: 30px 16px;
    color: var(--fg-3);
  }
  .nnone p {
    font-size: 13px;
    color: var(--fg-1);
  }
  .nnone span {
    font-size: 11.5px;
  }
  code {
    font-family: "JetBrains Mono", monospace;
    background: var(--bg-4);
    padding: 0 4px;
    border-radius: 3px;
  }
</style>
