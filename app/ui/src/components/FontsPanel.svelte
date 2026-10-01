<script lang="ts">
  import Icon from "./Icon.svelte";
  import { data, loadFonts } from "../lib/store.svelte";

  let query = $state("");
  $effect(() => {
    loadFonts();
  });

  const filtered = $derived(
    data.fonts.filter((f) => f.name.toLowerCase().includes(query.toLowerCase()))
  );
</script>

<div class="wrap">
  <div class="search">
    <Icon name="search" size={14} />
    <input placeholder="Filter fonts…" bind:value={query} />
  </div>
  <div class="count">{data.fonts.length} families available</div>

  {#if data.fonts.length === 0}
    <div class="none"><span class="spin"></span> loading font database…</div>
  {/if}

  {#each filtered as f (f.name)}
    <div class="font">
      <div class="ftop">
        <span class="fname" style="font-family: '{f.name}', serif">{f.name}</span>
        {#if f.variable}<span class="vf" title="Variable font">VF</span>{/if}
      </div>
      <div class="preview" style="font-family: '{f.name}', serif">Aa Bb Cc 123 — the quick brown fox</div>
      <div class="fmeta">
        <span>{f.variants} variant{f.variants === 1 ? "" : "s"}</span>
        {#each f.styles as s}<span class="style">{s}</span>{/each}
      </div>
    </div>
  {/each}
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
  .count {
    font-size: 10.5px;
    color: var(--fg-3);
    padding: 0 2px 2px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .font {
    padding: 9px 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg-2);
  }
  .font:hover {
    border-color: var(--border-strong);
  }
  .ftop {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .fname {
    font-size: 14px;
    color: var(--fg-0);
    font-weight: 600;
  }
  .vf {
    font-size: 9px;
    font-weight: 700;
    color: var(--accent);
    background: var(--accent-soft);
    padding: 1px 5px;
    border-radius: 4px;
    margin-left: auto;
  }
  .preview {
    font-size: 16px;
    color: var(--fg-1);
    margin: 6px 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .fmeta {
    display: flex;
    gap: 6px;
    align-items: center;
    flex-wrap: wrap;
    font-size: 11px;
    color: var(--fg-3);
  }
  .style {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 4px;
    background: var(--bg-4);
    color: var(--fg-2);
  }
  .none {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 20px 8px;
    color: var(--fg-3);
    font-size: 12.5px;
  }
  .spin {
    width: 13px;
    height: 13px;
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
