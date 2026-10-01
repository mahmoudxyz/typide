<script lang="ts">
  import Icon from "./Icon.svelte";
  import { ui, searchPackages, addPackage, fetchPackages, online } from "../lib/store.svelte";
  import type { PackageMeta } from "../lib/api";

  let query = $state("");
  let results = $state<PackageMeta[]>([]);
  let loading = $state(false);
  let error = $state("");
  let sel = $state(0);
  let added = $state<Set<string>>(new Set());
  let timer: ReturnType<typeof setTimeout>;

  async function run(q: string) {
    loading = true;
    error = "";
    try {
      results = await searchPackages(q);
      sel = 0;
    } catch (e) {
      error = String((e as any)?.message ?? e);
      results = [];
    } finally {
      loading = false;
    }
  }

  function onInput() {
    clearTimeout(timer);
    timer = setTimeout(() => run(query), 220);
  }

  $effect(() => {
    // initial load
    run("");
    return () => clearTimeout(timer);
  });

  function specOf(p: PackageMeta) {
    return `@preview/${p.name}:${p.version}`;
  }
  async function add(p: PackageMeta) {
    const spec = specOf(p);
    addPackage(spec);
    added = new Set([...added, spec]);
    fetchPackages();
  }
  function close() {
    ui.packageBrowser = false;
  }
  function keydown(e: KeyboardEvent) {
    if (e.key === "Escape") close();
    else if (e.key === "ArrowDown") {
      sel = Math.min(results.length - 1, sel + 1);
      e.preventDefault();
    } else if (e.key === "ArrowUp") {
      sel = Math.max(0, sel - 1);
      e.preventDefault();
    } else if (e.key === "Enter" && results[sel]) add(results[sel]);
  }
</script>

<div class="overlay" onclick={close} role="presentation">
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_interactive_supports_focus -->
  <div class="browser" onclick={(e) => e.stopPropagation()} role="dialog" aria-label="Package browser" tabindex="-1">
    <div class="head">
      <div class="input">
        <Icon name="package" size={18} />
        <!-- svelte-ignore a11y_autofocus -->
        <input autofocus bind:value={query} oninput={onInput} onkeydown={keydown} placeholder="Search Typst Universe…" />
        {#if loading}<span class="spin"></span>{/if}
      </div>
      <div class="src">
        {#if online()}
          <span class="dot ok"></span> Universe · live
        {:else}
          <span class="dot off"></span> offline · cached index
        {/if}
      </div>
    </div>

    <div class="list">
      {#if error}
        <div class="msg err"><Icon name="warn" size={16} /> {error}</div>
      {:else if results.length === 0 && !loading}
        <div class="msg"><Icon name="search" size={16} /> No packages found</div>
      {/if}
      {#each results as p, i (p.name)}
        <div class="row" class:sel={i === sel} onmousemove={() => (sel = i)} role="option" aria-selected={i === sel} tabindex="-1">
          <div class="rmain">
            <div class="rtop">
              <span class="rname mono">@preview/{p.name}</span>
              <span class="rver mono">{p.version}</span>
              {#each p.categories.slice(0, 2) as c}<span class="cat">{c}</span>{/each}
            </div>
            <div class="rdesc">{p.description}</div>
            <div class="rmeta">
              {#if p.authors.length}<span><Icon name="quote" size={11} /> {p.authors[0].split("<")[0].trim()}</span>{/if}
              {#if p.license}<span>{p.license}</span>{/if}
              {#each p.keywords.slice(0, 3) as k}<span class="kw">#{k}</span>{/each}
            </div>
          </div>
          {#if added.has(specOf(p))}
            <span class="badge added"><Icon name="check" size={13} /> Added</span>
          {:else}
            <button class="add" onclick={() => add(p)}><Icon name="download" size={13} /> Add</button>
          {/if}
        </div>
      {/each}
    </div>

    <footer>
      <span><kbd>↑</kbd><kbd>↓</kbd> navigate · <kbd>↵</kbd> add · <kbd>Esc</kbd> close</span>
      <span class="count">{results.length} shown · 4800+ on Universe</span>
    </footer>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    backdrop-filter: blur(2px);
    z-index: 220;
    display: flex;
    justify-content: center;
    padding-top: 9vh;
  }
  .browser {
    width: 720px;
    max-width: 94vw;
    max-height: 72vh;
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
  .head {
    border-bottom: 1px solid var(--border);
  }
  .input {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 0 16px;
    height: 52px;
    color: var(--fg-3);
  }
  .input input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    font-size: 16px;
    color: var(--fg-0);
  }
  .src {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 16px 8px;
    font-size: 11px;
    color: var(--fg-3);
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
  }
  .dot.ok {
    background: var(--ok);
    box-shadow: 0 0 0 3px var(--ok-soft);
  }
  .dot.off {
    background: var(--fg-3);
  }
  .list {
    overflow-y: auto;
    padding: 6px;
    flex: 1;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 11px 12px;
    border-radius: 9px;
  }
  .row.sel {
    background: var(--accent-soft);
  }
  .rmain {
    flex: 1;
    min-width: 0;
  }
  .rtop {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .rname {
    font-size: 13.5px;
    font-weight: 600;
    color: var(--fg-0);
  }
  .rver {
    font-size: 11px;
    color: var(--fg-2);
    background: var(--bg-4);
    padding: 0 6px;
    border-radius: 4px;
  }
  .cat {
    font-size: 9.5px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--accent-2);
    border: 1px solid color-mix(in srgb, var(--accent-2) 40%, transparent);
    padding: 1px 6px;
    border-radius: 4px;
  }
  .rdesc {
    font-size: 12.5px;
    color: var(--fg-1);
    margin-top: 3px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .rmeta {
    display: flex;
    gap: 12px;
    margin-top: 5px;
    font-size: 11px;
    color: var(--fg-3);
    align-items: center;
    flex-wrap: wrap;
  }
  .rmeta span {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
  .kw {
    color: var(--fg-3);
    opacity: 0.85;
  }
  .add {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 13px;
    border-radius: 7px;
    background: var(--accent);
    color: var(--accent-fg);
    font-size: 12px;
    font-weight: 600;
    flex: none;
  }
  .add:hover {
    filter: brightness(1.08);
  }
  .badge.added {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    color: var(--ok);
    flex: none;
  }
  .msg {
    display: flex;
    align-items: center;
    gap: 8px;
    justify-content: center;
    padding: 30px;
    color: var(--fg-3);
    font-size: 13px;
  }
  .msg.err {
    color: var(--error);
  }
  footer {
    height: 34px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    border-top: 1px solid var(--border);
    font-size: 11px;
    color: var(--fg-3);
  }
  .spin {
    width: 14px;
    height: 14px;
    border: 2px solid var(--border-strong);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
