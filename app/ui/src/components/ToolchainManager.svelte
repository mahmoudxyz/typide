<script lang="ts">
  import Icon from "./Icon.svelte";
  import { ui, data, online, loadToolchains, installToolchain, removeToolchain, checkToolchainsOnline, installToolchainFromArchive } from "../lib/store.svelte";

  $effect(() => {
    loadToolchains();
  });

  const installed = $derived(new Set(data.toolchains.map((t) => t.version)));
  const available = $derived(data.toolchainsAvailable.filter((v) => !installed.has(v)));

  const srcColor: Record<string, string> = {
    bundled: "var(--accent)",
    system: "var(--accent-2)",
    installed: "var(--ok)",
  };
  function close() {
    ui.toolchainManager = false;
  }
</script>

<div class="overlay" onclick={close} role="presentation">
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_interactive_supports_focus -->
  <div class="tm" onclick={(e) => e.stopPropagation()} role="dialog" aria-label="Toolchains" tabindex="-1">
    <header>
      <span class="ttl"><Icon name="layers" size={17} /> Typst Toolchains</span>
      <button class="fromfile" onclick={installToolchainFromArchive} title="Install from a local .tar.xz (offline)">
        <Icon name="download" size={13} /> Install from file…
      </button>
      <span class="net">{online() ? "online" : "offline"}</span>
      <button class="x" onclick={close}><Icon name="x" size={16} /></button>
    </header>

    <div class="body">
      <div class="section">Installed</div>
      {#each data.toolchains as t (t.version + t.source)}
        <div class="row">
          <span class="dot" style="background: {srcColor[t.source] ?? 'var(--fg-2)'}"></span>
          <span class="ver mono">Typst {t.version}</span>
          <span class="src" style="color: {srcColor[t.source] ?? 'var(--fg-2)'}">{t.source}</span>
          <span class="path mono">{t.path}</span>
          {#if t.source === "installed"}
            <button class="rm" onclick={() => removeToolchain(t.version)} title="Remove"><Icon name="x" size={13} /></button>
          {/if}
        </div>
      {/each}

      <div class="section">
        Available to install
        {#if online() && data.toolchainsAvailable.length === 0}
          <button class="check" onclick={checkToolchainsOnline}>Check online</button>
        {/if}
      </div>
      {#if !online()}
        <div class="hint"><Icon name="wifi-off" size={13} /> Go online to list downloadable versions.</div>
      {:else if data.toolchainsAvailable.length === 0}
        <div class="hint"><Icon name="info" size={13} /> Click "Check online" to list GitHub releases.</div>
      {:else if available.length === 0}
        <div class="hint"><Icon name="check-circle" size={13} /> All available versions are installed.</div>
      {/if}
      {#each available as v (v)}
        <div class="row">
          <span class="dot" style="background: var(--border-strong)"></span>
          <span class="ver mono">Typst {v}</span>
          <span class="spacer"></span>
          <button class="install" onclick={() => installToolchain(v)} disabled={!online()}>
            <Icon name="download" size={13} /> Install
          </button>
        </div>
      {/each}
    </div>

    <footer>
      <Icon name="info" size={13} />
      <span>The bundled Typst {"0.15.1"} is linked into the app; other versions run as CLI subprocesses.</span>
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
  .tm {
    width: 640px;
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
  header {
    height: 50px;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 16px;
    border-bottom: 1px solid var(--border);
  }
  .ttl {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 15px;
    font-weight: 700;
  }
  .fromfile {
    margin-left: auto;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 11.5px;
    color: var(--fg-1);
    padding: 5px 10px;
    border-radius: 7px;
    border: 1px solid var(--border-strong);
    background: var(--bg-2);
  }
  .fromfile:hover {
    background: var(--bg-4);
  }
  .net {
    font-size: 11px;
    color: var(--fg-3);
  }
  .x {
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--fg-3);
    border-radius: 7px;
  }
  .x:hover {
    background: var(--bg-4);
    color: var(--fg-0);
  }
  .body {
    overflow-y: auto;
    padding: 8px;
    flex: 1;
  }
  .section {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 10.5px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--fg-3);
    font-weight: 700;
    padding: 12px 8px 6px;
  }
  .check {
    text-transform: none;
    letter-spacing: 0;
    font-size: 11px;
    color: var(--accent);
    padding: 2px 8px;
    border-radius: 5px;
    background: var(--accent-soft);
  }
  .check:hover {
    filter: brightness(1.1);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 10px;
    border-radius: 8px;
  }
  .row:hover {
    background: var(--bg-4);
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex: none;
  }
  .ver {
    font-size: 13px;
    font-weight: 600;
    color: var(--fg-0);
  }
  .src {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-weight: 700;
  }
  .path {
    font-size: 11px;
    color: var(--fg-3);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-left: auto;
    max-width: 260px;
  }
  .spacer {
    flex: 1;
  }
  .rm {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--fg-3);
    border-radius: 5px;
    flex: none;
  }
  .rm:hover {
    background: var(--error-soft);
    color: var(--error);
  }
  .install {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: 7px;
    background: var(--accent);
    color: var(--accent-fg);
    font-size: 12px;
    font-weight: 600;
    flex: none;
  }
  .install:hover:not(:disabled) {
    filter: brightness(1.08);
  }
  .install:disabled {
    opacity: 0.5;
  }
  .hint {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 10px;
    font-size: 12px;
    color: var(--fg-3);
  }
  footer {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    border-top: 1px solid var(--border);
    font-size: 11.5px;
    color: var(--fg-3);
  }
</style>
