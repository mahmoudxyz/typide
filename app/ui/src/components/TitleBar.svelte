<script lang="ts">
  import Icon from "./Icon.svelte";
  import { ui, data, toggleTheme, closeProject, openFromPicker, exportActiveProfile, networkLocked } from "../lib/store.svelte";

  let profileOpen = $state(false);
  let netOpen = $state(false);
  let projOpen = $state(false);

  async function runCompile() {
    await exportActiveProfile();
  }

  const netModes: ("offline" | "ask" | "online")[] = ["offline", "ask", "online"];
</script>

<header class="titlebar">
  <div class="left">
    <div class="logo">
      <svg width="20" height="20" viewBox="0 0 24 24" aria-hidden="true">
        <rect x="2" y="2" width="20" height="20" rx="5" fill="var(--accent)" />
        <path d="M7 8h10M12 8v9" stroke="#fff" stroke-width="2.2" stroke-linecap="round" />
      </svg>
      <span class="brand">Typide</span>
    </div>
    <div class="divider"></div>
    <div class="proj-wrap">
      <button class="project" onclick={() => (projOpen = !projOpen)}>
        <span class="pkind">{data.project?.kind ?? "—"}</span>
        <span class="pname">{data.project?.name ?? "No project"}</span>
        <Icon name="chevron" size={12} />
      </button>
      {#if projOpen}
        <div class="menu proj-menu">
          <button class="mi" onclick={() => { ui.wizardOpen = true; projOpen = false; }}>
            <Icon name="file-code" size={14} /> <span class="mname">New Project…</span>
          </button>
          <button class="mi" onclick={() => { openFromPicker(); projOpen = false; }}>
            <Icon name="folder-open" size={14} /> <span class="mname">Open Folder…</span>
          </button>
          <div class="mdiv"></div>
          <button class="mi" onclick={() => { closeProject(); projOpen = false; }}>
            <Icon name="compass" size={14} /> <span class="mname">Close Project</span>
            <span class="mout mono">back to Welcome</span>
          </button>
        </div>
      {/if}
    </div>
  </div>

  <div class="center">
    <div class="runbar">
      <button class="runcfg" onclick={() => (profileOpen = !profileOpen)}>
        <Icon name="check-circle" size={13} />
        <span>{ui.exportProfile}</span>
        <Icon name="chevron" size={11} />
      </button>
      {#if profileOpen}
        <div class="menu" role="menu">
          {#each data.exportProfiles as p (p.name)}
            <button
              class="mi"
              class:sel={ui.exportProfile === p.name}
              onclick={() => {
                ui.exportProfile = p.name;
                profileOpen = false;
              }}
            >
              <span class="fmt fmt-{p.format}">{p.format}</span>
              <span class="mname">{p.name}</span>
              <span class="mout mono">{p.output}</span>
            </button>
          {/each}
          <div class="mdiv"></div>
          <button class="mi edit"><Icon name="settings" size={13} /> Edit profiles…</button>
        </div>
      {/if}
      <button class="run" class:busy={ui.compiling} onclick={runCompile} title="Run export">
        {#if ui.compiling}<span class="spin"></span>{:else}<Icon name="play" size={14} fill="currentColor" stroke={0} />{/if}
      </button>
    </div>
  </div>

  <div class="right">
    <button class="tool" onclick={() => (ui.commandPalette = true)} title="Command Palette (Ctrl+Shift+P)">
      <Icon name="command" size={16} />
    </button>
    <button class="tool" onclick={() => (ui.searchEverywhere = true)} title="Search Everywhere (Shift Shift)">
      <Icon name="search" size={16} />
    </button>
    <div class="net-wrap">
      <button class="tool net {ui.networkMode}" onclick={() => (netOpen = !netOpen)} title="Network mode">
        <Icon name={ui.networkMode === "offline" ? "wifi-off" : "wifi"} size={16} />
      </button>
      {#if netOpen}
        <div class="menu net-menu">
          {#if networkLocked()}
            <div class="locked-note"><Icon name="settings" size={12} /> Locked by your organization</div>
          {/if}
          {#each netModes as m (m)}
            <button
              class="mi"
              class:sel={ui.networkMode === m}
              disabled={networkLocked()}
              onclick={() => {
                if (networkLocked()) return;
                ui.networkMode = m;
                netOpen = false;
              }}
            >
              <Icon name={m === "offline" ? "wifi-off" : "wifi"} size={14} />
              <span class="mname">{m === "offline" ? "Offline" : m === "ask" ? "Ask each time" : "Online"}</span>
              {#if ui.networkMode === m}<Icon name="check" size={13} />{/if}
            </button>
          {/each}
        </div>
      {/if}
    </div>
    <button class="tool" onclick={toggleTheme} title="Toggle theme">
      <Icon name={ui.theme === "dark" ? "sun" : "moon"} size={16} />
    </button>
  </div>
</header>

<svelte:window
  onclick={(e) => {
    const t = e.target as HTMLElement;
    if (!t.closest(".runbar")) profileOpen = false;
    if (!t.closest(".net-wrap")) netOpen = false;
    if (!t.closest(".proj-wrap")) projOpen = false;
  }}
/>

<style>
  .titlebar {
    height: 46px;
    flex: none;
    background: var(--bg-0);
    border-bottom: 1px solid var(--border);
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    padding: 0 10px;
    gap: 12px;
  }
  .left {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .logo {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .brand {
    font-weight: 700;
    font-size: 14px;
    letter-spacing: -0.01em;
  }
  .divider {
    width: 1px;
    height: 20px;
    background: var(--border);
  }
  .proj-wrap {
    position: relative;
  }
  .proj-menu {
    left: 0;
    right: auto;
    min-width: 220px;
  }
  .project {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 5px 9px;
    border-radius: 6px;
    color: var(--fg-1);
  }
  .project:hover {
    background: var(--bg-3);
  }
  .pkind {
    font-size: 9.5px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--accent);
    background: var(--accent-soft);
    padding: 1px 6px;
    border-radius: 4px;
    font-weight: 700;
  }
  .pname {
    font-size: 13px;
    font-weight: 600;
  }
  .center {
    display: flex;
    justify-content: center;
  }
  .runbar {
    display: flex;
    align-items: center;
    gap: 2px;
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 3px;
    position: relative;
  }
  .runcfg {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 4px 10px;
    border-radius: 6px;
    color: var(--fg-1);
    font-size: 12.5px;
    font-weight: 500;
    min-width: 150px;
  }
  .runcfg :global(svg:first-child) {
    color: var(--ok);
  }
  .runcfg:hover {
    background: var(--bg-3);
  }
  .run {
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--ok);
    color: #fff;
    border-radius: 6px;
  }
  .run:hover {
    filter: brightness(1.08);
  }
  .run.busy {
    background: var(--accent);
  }
  .menu {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    min-width: 280px;
    background: var(--bg-3);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius);
    box-shadow: var(--shadow);
    padding: 5px;
    z-index: 50;
  }
  .net-menu {
    left: auto;
    right: 0;
    min-width: 180px;
  }
  .mi {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    padding: 7px 9px;
    border-radius: 6px;
    color: var(--fg-1);
    font-size: 12.5px;
    text-align: left;
  }
  .mi:hover:not(:disabled) {
    background: var(--bg-4);
  }
  .mi.sel {
    background: var(--accent-soft);
  }
  .mi:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
  .locked-note {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 9px 7px;
    font-size: 11px;
    color: var(--warn);
    border-bottom: 1px solid var(--border);
    margin-bottom: 4px;
  }
  .fmt {
    font-size: 9px;
    text-transform: uppercase;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 4px;
    color: #fff;
    flex: none;
  }
  .fmt-pdf {
    background: var(--error);
  }
  .fmt-html {
    background: var(--accent);
  }
  .fmt-svg,
  .fmt-png {
    background: var(--accent-2);
  }
  .mname {
    font-weight: 500;
  }
  .mout {
    margin-left: auto;
    font-size: 10.5px;
    color: var(--fg-3);
  }
  .mdiv {
    height: 1px;
    background: var(--border);
    margin: 5px 3px;
  }
  .mi.edit {
    color: var(--fg-2);
    font-size: 12px;
  }
  .right {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 2px;
  }
  .tool {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--fg-2);
    border-radius: 6px;
  }
  .tool:hover {
    background: var(--bg-3);
    color: var(--fg-0);
  }
  .net-wrap {
    position: relative;
  }
  .net.offline {
    color: var(--fg-2);
  }
  .net.ask {
    color: var(--warn);
  }
  .net.online {
    color: var(--ok);
  }
  .spin {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.4);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
