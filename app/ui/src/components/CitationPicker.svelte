<script lang="ts">
  import Icon from "./Icon.svelte";
  import { ui, data, insertAtCursor, importZotero } from "../lib/store.svelte";
  import type { Citation } from "../lib/types";

  let query = $state("");
  let sel = $state(0);

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    const list = data.citations;
    if (!q) return list;
    return list.filter(
      (c) =>
        c.key.toLowerCase().includes(q) ||
        c.title.toLowerCase().includes(q) ||
        c.authors.toLowerCase().includes(q) ||
        c.year.includes(q)
    );
  });

  function close() {
    ui.citationPicker = false;
    query = "";
    sel = 0;
  }
  function pick(c: Citation) {
    insertAtCursor("@" + c.key);
    close();
  }
  function pickCite(c: Citation) {
    insertAtCursor(`#cite(<${c.key}>)`);
    close();
  }
  function keydown(e: KeyboardEvent) {
    if (e.key === "Escape") close();
    else if (e.key === "ArrowDown") {
      sel = Math.min(filtered.length - 1, sel + 1);
      e.preventDefault();
    } else if (e.key === "ArrowUp") {
      sel = Math.max(0, sel - 1);
      e.preventDefault();
    } else if (e.key === "Enter" && filtered[sel]) {
      if (e.shiftKey) pickCite(filtered[sel]);
      else pick(filtered[sel]);
    }
  }
  $effect(() => {
    query;
    sel = 0;
  });
</script>

<div class="overlay" onclick={close} role="presentation">
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_interactive_supports_focus -->
  <div class="picker" onclick={(e) => e.stopPropagation()} role="dialog" aria-label="Insert citation" tabindex="-1">
    <div class="input">
      <Icon name="quote" size={17} />
      <!-- svelte-ignore a11y_autofocus -->
      <input autofocus bind:value={query} onkeydown={keydown} placeholder="Search citations by key, author, title, year…" />
      <button class="zotero" onclick={importZotero} title="Import from a local Zotero (Better BibTeX)">
        <Icon name="download" size={13} /> Zotero
      </button>
      <kbd>Esc</kbd>
    </div>
    <div class="list">
      {#if data.citations.length === 0}
        <div class="none">No citations — add a <code>.bib</code> file and <code>#bibliography()</code>.</div>
      {:else if filtered.length === 0}
        <div class="none">No matches</div>
      {/if}
      {#each filtered as c, i (c.key)}
        <button class="row" class:sel={i === sel} onclick={() => pick(c)} onmousemove={() => (sel = i)}>
          <span class="dot" class:cited={c.cited} title={c.cited ? "cited" : "uncited"}></span>
          <div class="main">
            <div class="top"><span class="key mono">@{c.key}</span><span class="src">{c.source}</span></div>
            <div class="title">{c.title}</div>
            <div class="meta">{c.authors}{c.year ? ` · ${c.year}` : ""}</div>
          </div>
          <span class="ins">insert</span>
        </button>
      {/each}
    </div>
    <footer>
      <span><kbd>↵</kbd> insert <span class="mono">@key</span></span>
      <span><kbd>⇧</kbd><kbd>↵</kbd> insert <span class="mono">#cite(&lt;key&gt;)</span></span>
    </footer>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(2px);
    z-index: 210;
    display: flex;
    justify-content: center;
    padding-top: 12vh;
  }
  .picker {
    width: 600px;
    max-width: 92vw;
    max-height: 62vh;
    background: var(--bg-3);
    border: 1px solid var(--border-strong);
    border-radius: 12px;
    box-shadow: var(--shadow);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: pop 0.12s ease;
  }
  @keyframes pop {
    from {
      transform: translateY(-6px);
      opacity: 0;
    }
  }
  .input {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 14px;
    height: 50px;
    border-bottom: 1px solid var(--border);
    color: var(--fg-3);
  }
  .input input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    font-size: 15px;
    color: var(--fg-0);
  }
  .zotero {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 11.5px;
    color: var(--accent-2);
    padding: 4px 9px;
    border-radius: 6px;
    border: 1px solid color-mix(in srgb, var(--accent-2) 35%, transparent);
    background: var(--bg-2);
  }
  .zotero:hover {
    background: var(--bg-4);
  }
  .list {
    overflow-y: auto;
    padding: 6px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 11px;
    width: 100%;
    padding: 9px 10px;
    border-radius: 8px;
    text-align: left;
  }
  .row.sel {
    background: var(--accent-soft);
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--fg-3);
    flex: none;
  }
  .dot.cited {
    background: var(--ok);
  }
  .main {
    flex: 1;
    min-width: 0;
  }
  .top {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .key {
    font-size: 13px;
    font-weight: 600;
    color: var(--accent);
  }
  .src {
    font-size: 9.5px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--fg-3);
  }
  .title {
    font-size: 12.5px;
    color: var(--fg-0);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-top: 2px;
  }
  .meta {
    font-size: 11.5px;
    color: var(--fg-3);
    margin-top: 1px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .ins {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--fg-3);
    flex: none;
  }
  .row.sel .ins {
    color: var(--accent);
  }
  .none {
    padding: 22px;
    text-align: center;
    color: var(--fg-3);
    font-size: 13px;
  }
  code {
    font-family: "JetBrains Mono", monospace;
    background: var(--bg-4);
    padding: 0 4px;
    border-radius: 3px;
  }
  footer {
    height: 32px;
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 0 14px;
    border-top: 1px solid var(--border);
    font-size: 11px;
    color: var(--fg-3);
  }
  footer span {
    display: flex;
    align-items: center;
    gap: 5px;
  }
</style>
