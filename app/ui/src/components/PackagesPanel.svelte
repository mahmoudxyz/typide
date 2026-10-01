<script lang="ts">
  import Icon from "./Icon.svelte";
  import { data, ui, fetchPackages, makeOfflineReady, mirrorPackages } from "../lib/store.svelte";

  const statusMeta: Record<string, { color: string; label: string }> = {
    vendored: { color: "var(--ok)", label: "vendored" },
    cached: { color: "var(--accent)", label: "cached" },
    missing: { color: "var(--error)", label: "missing" },
    mirror: { color: "var(--accent-2)", label: "mirror" },
  };
</script>

<div class="wrap">
  <button class="browse-btn" onclick={() => (ui.packageBrowser = true)}>
    <Icon name="search" size={14} /> Browse Universe
  </button>
  <div class="row2">
    <button class="mini" onclick={fetchPackages} disabled={ui.compiling} title="Download to cache">
      <Icon name="download" size={13} /> Fetch
    </button>
    <button class="mini" onclick={makeOfflineReady} disabled={ui.compiling} title="Vendor + write typide.lock">
      <Icon name="check-circle" size={13} /> Make offline-ready
    </button>
  </div>
  <button class="mini wide" onclick={mirrorPackages} disabled={ui.compiling} title="Build a Universe-layout mirror (USB / intranet)">
    <Icon name="layers" size={13} /> Create mirror…
  </button>
  {#if data.offlineReport}
    <div class="report">
      <div class="rline">
        <Icon name="check-circle" size={13} />
        {data.offlineReport.packages.filter((p) => p.vendored).length} vendored ·
        {data.offlineReport.packages.length} locked
      </div>
      {#if data.offlineReport.errors.length}
        <div class="rerr"><Icon name="warn" size={12} /> {data.offlineReport.errors.length} failed</div>
      {/if}
    </div>
  {/if}

  <div class="section-title">Imports ({data.packages.length})</div>
  {#each data.packages as p (p.spec)}
    <div class="pkg">
      <div class="row1">
        <span class="dot" style="background: {statusMeta[p.status].color}"></span>
        <span class="name mono">{p.name}</span>
        <span class="ver mono">{p.version}</span>
        {#if p.latest && p.latest !== p.version}
          <span class="update mono" title="Update available">↑ {p.latest}</span>
        {/if}
        <span class="status" style="color: {statusMeta[p.status].color}">
          {statusMeta[p.status].label}
        </span>
      </div>
      <div class="desc">{p.description}</div>
      {#if p.status === "missing"}
        <div class="fix">
          <button class="qf"><Icon name="download" size={11} /> Fetch package</button>
          <button class="qf">Install from file…</button>
        </div>
      {/if}
    </div>
  {/each}

  <div class="hint">
    <Icon name="info" size={12} /> Resolution: vendor → local → cache → mirrors → Universe
  </div>
</div>

<style>
  .wrap {
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .browse-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    height: 34px;
    border-radius: var(--radius-sm);
    background: var(--accent);
    color: var(--accent-fg);
    border: none;
    font-weight: 600;
    font-size: 12.5px;
  }
  .browse-btn:hover {
    filter: brightness(1.08);
  }
  .row2 {
    display: flex;
    gap: 6px;
  }
  .mini {
    flex: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    height: 30px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-strong);
    background: var(--bg-2);
    color: var(--fg-1);
    font-size: 11.5px;
    font-weight: 500;
  }
  .mini:hover:not(:disabled) {
    background: var(--bg-4);
  }
  .mini:disabled {
    opacity: 0.5;
  }
  .mini.wide {
    width: 100%;
  }
  .report {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    background: var(--ok-soft);
    border: 1px solid color-mix(in srgb, var(--ok) 30%, transparent);
  }
  .rline {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--ok);
  }
  .rerr {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--warn);
  }
  .section-title {
    font-size: 10.5px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--fg-3);
    margin: 2px 2px;
    font-weight: 600;
  }
  .pkg {
    padding: 8px 9px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg-2);
  }
  .row1 {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex: none;
  }
  .name {
    font-weight: 600;
    font-size: 12.5px;
    color: var(--fg-0);
  }
  .ver {
    font-size: 11px;
    color: var(--fg-2);
  }
  .update {
    font-size: 10.5px;
    color: var(--warn);
    background: var(--warn-soft);
    padding: 0 5px;
    border-radius: 4px;
  }
  .status {
    margin-left: auto;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-weight: 600;
  }
  .desc {
    font-size: 11.5px;
    color: var(--fg-2);
    margin-top: 4px;
  }
  .fix {
    display: flex;
    gap: 6px;
    margin-top: 8px;
  }
  .qf {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    padding: 3px 8px;
    border-radius: 4px;
    border: 1px solid var(--border-strong);
    color: var(--fg-1);
    background: var(--bg-3);
  }
  .qf:hover {
    background: var(--bg-4);
  }
  .hint {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--fg-3);
    padding: 4px 2px;
  }
</style>
