<script lang="ts">
  import Icon from "./Icon.svelte";
  import { ui, completeFirstRun, applyTheme, networkLocked } from "../lib/store.svelte";

  const detected = [
    { name: "Typst 0.15.1", detail: "bundled toolchain", ok: true },
    { name: "Tinymist 0.13.10", detail: "bundled language server", ok: true },
    { name: "Git", detail: "found on PATH · user.name set", ok: true },
    { name: "Zotero local API", detail: "not reachable on :23119", ok: false },
  ];

  function setTheme(t: "dark" | "light") {
    ui.theme = t;
    applyTheme();
  }
</script>

<div class="overlay" role="presentation">
  <div class="card" role="dialog" aria-label="Welcome to Typide">
    <div class="hero">
      <svg width="46" height="46" viewBox="0 0 24 24" aria-hidden="true">
        <rect x="2" y="2" width="20" height="20" rx="6" fill="var(--accent)" />
        <path d="M7 8h10M12 8v9" stroke="#fff" stroke-width="2.4" stroke-linecap="round" />
      </svg>
      <h1>Welcome to Typide</h1>
      <p>An offline-first IDE for Typst. Everything works with no account and no internet — let's set a few defaults.</p>
    </div>

    <div class="section">
      <span class="slabel">Appearance</span>
      <div class="themes">
        <button class="themecard" class:on={ui.theme === "dark"} onclick={() => setTheme("dark")}>
          <div class="swatch dark"><span></span><span></span><span></span></div>
          Dark
        </button>
        <button class="themecard" class:on={ui.theme === "light"} onclick={() => setTheme("light")}>
          <div class="swatch light"><span></span><span></span><span></span></div>
          Light
        </button>
      </div>
    </div>

    <div class="section">
      <span class="slabel">Network mode {#if networkLocked()}· <span class="locked"><Icon name="settings" size={11} /> managed by your organization</span>{/if}</span>
      <div class="netmodes">
        {#each [["offline", "wifi-off", "Offline", "Never use the network. Fully air-gapped."], ["ask", "wifi", "Ask each time", "Prompt before any download."], ["online", "wifi", "Online", "Allow downloads & update checks."]] as [m, ic, t, d]}
          <button class="netmode" class:on={ui.networkMode === m} disabled={networkLocked()} onclick={() => !networkLocked() && (ui.networkMode = m as any)}>
            <span class="nic"><Icon name={ic} size={16} /></span>
            <div><div class="nt">{t}</div><div class="nd">{d}</div></div>
            {#if ui.networkMode === m}<span class="chk"><Icon name="check" size={14} /></span>{/if}
          </button>
        {/each}
      </div>
    </div>

    <div class="section">
      <span class="slabel">Detected on this machine</span>
      <div class="detect">
        {#each detected as d}
          <div class="drow">
            <span class="dic" class:ok={d.ok}>
              <Icon name={d.ok ? "check-circle" : "info"} size={15} />
            </span>
            <span class="dn">{d.name}</span>
            <span class="dd">{d.detail}</span>
          </div>
        {/each}
      </div>
    </div>

    <button class="start" onclick={completeFirstRun}>Get started <Icon name="chevron" size={16} /></button>
    <p class="foot">No telemetry. Ever. · Settings can be changed anytime in Customize.</p>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: var(--bg-0);
    z-index: 400;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow-y: auto;
    padding: 30px;
  }
  .card {
    width: 460px;
    max-width: 100%;
    display: flex;
    flex-direction: column;
    gap: 22px;
  }
  .hero {
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }
  .hero h1 {
    font-size: 24px;
    font-weight: 700;
    margin-top: 6px;
  }
  .hero p {
    font-size: 13px;
    color: var(--fg-2);
    line-height: 1.55;
    max-width: 380px;
  }
  .section {
    display: flex;
    flex-direction: column;
    gap: 9px;
  }
  .slabel {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--fg-3);
  }
  .locked {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    text-transform: none;
    letter-spacing: 0;
    color: var(--warn);
    font-weight: 600;
  }
  .netmode:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
  .themes {
    display: flex;
    gap: 10px;
  }
  .themecard {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 12px;
    border: 1px solid var(--border-strong);
    border-radius: 11px;
    background: var(--bg-2);
    font-size: 13px;
    color: var(--fg-1);
  }
  .themecard.on {
    border-color: var(--accent);
    background: var(--accent-soft);
    color: var(--fg-0);
  }
  .swatch {
    width: 100%;
    height: 44px;
    border-radius: 7px;
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 8px;
    border: 1px solid var(--border);
  }
  .swatch.dark {
    background: #1c1f26;
  }
  .swatch.light {
    background: #f4f5f8;
  }
  .swatch span {
    height: 6px;
    border-radius: 3px;
  }
  .swatch span:nth-child(1) {
    width: 30%;
    background: #5b8def;
  }
  .swatch span:nth-child(2) {
    width: 22%;
    background: #33c2b0;
  }
  .swatch span:nth-child(3) {
    width: 26%;
    background: #ffcb6b;
  }
  .netmodes {
    display: flex;
    flex-direction: column;
    gap: 7px;
  }
  .netmode {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 11px 13px;
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    background: var(--bg-2);
    text-align: left;
  }
  .netmode.on {
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .nic {
    color: var(--fg-2);
    display: inline-flex;
  }
  .netmode.on .nic {
    color: var(--accent);
  }
  .netmode > div {
    flex: 1;
  }
  .nt {
    font-size: 13px;
    font-weight: 600;
    color: var(--fg-0);
  }
  .nd {
    font-size: 11.5px;
    color: var(--fg-3);
    margin-top: 1px;
  }
  .chk {
    color: var(--accent);
    display: inline-flex;
  }
  .detect {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    border-radius: 10px;
    overflow: hidden;
  }
  .drow {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 13px;
    background: var(--bg-2);
    border-bottom: 1px solid var(--border);
    font-size: 12.5px;
  }
  .drow:last-child {
    border-bottom: none;
  }
  .dic {
    color: var(--fg-3);
    display: inline-flex;
  }
  .dic.ok {
    color: var(--ok);
  }
  .dn {
    font-weight: 500;
    color: var(--fg-0);
  }
  .dd {
    margin-left: auto;
    color: var(--fg-3);
    font-size: 11.5px;
  }
  .start {
    height: 46px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    background: var(--accent);
    color: var(--accent-fg);
    border-radius: 11px;
    font-size: 14.5px;
    font-weight: 600;
    margin-top: 4px;
  }
  .start:hover {
    filter: brightness(1.08);
  }
  .start :global(svg) {
    transform: rotate(90deg);
  }
  .foot {
    text-align: center;
    font-size: 11px;
    color: var(--fg-3);
  }
</style>
