<script lang="ts">
  import Icon from "./Icon.svelte";
  import { ui, loadProject, openFromPicker, toggleTheme, recents, relTime, networkLocked } from "../lib/store.svelte";

  let section = $state<"projects" | "customize" | "learn">("projects");
  let query = $state("");

  const kindColor: Record<string, string> = {
    thesis: "var(--accent)",
    paper: "var(--accent-2)",
    notes: "var(--syn-heading)",
    package: "var(--syn-keyword)",
    generic: "var(--fg-2)",
  };
  const kindIcon: Record<string, string> = {
    thesis: "book",
    paper: "file-code",
    notes: "structure",
    package: "package",
    generic: "file",
  };

  const filtered = $derived(
    recents.filter(
      (p) =>
        p.name.toLowerCase().includes(query.toLowerCase()) ||
        p.path.toLowerCase().includes(query.toLowerCase())
    )
  );

  const initials = (name: string) =>
    name
      .split(/[-_ ]/)
      .slice(0, 2)
      .map((w) => w[0]?.toUpperCase() ?? "")
      .join("");
</script>

<div class="launcher">
  <aside class="rail">
    <div class="brand">
      <svg width="26" height="26" viewBox="0 0 24 24" aria-hidden="true">
        <rect x="2" y="2" width="20" height="20" rx="6" fill="var(--accent)" />
        <path d="M7 8h10M12 8v9" stroke="#fff" stroke-width="2.4" stroke-linecap="round" />
      </svg>
      <div>
        <div class="bname">Typide</div>
        <div class="bver">0.1.0 · Typst 0.15.1</div>
      </div>
    </div>

    <nav>
      <button class="nav" class:on={section === "projects"} onclick={() => (section = "projects")}>
        <Icon name="folder" size={17} /> Projects
      </button>
      <button class="nav" class:on={section === "customize"} onclick={() => (section = "customize")}>
        <Icon name="settings" size={17} /> Customize
      </button>
      <button class="nav" class:on={section === "learn"} onclick={() => (section = "learn")}>
        <Icon name="book" size={17} /> Learn
      </button>
    </nav>

    <div class="rail-foot">
      <span class="net {ui.networkMode}">
        <Icon name={ui.networkMode === "offline" ? "wifi-off" : "wifi"} size={13} />
        {ui.networkMode === "offline" ? "Offline" : ui.networkMode === "ask" ? "Ask each time" : "Online"}
      </span>
      <button class="theme" onclick={toggleTheme} title="Toggle theme">
        <Icon name={ui.theme === "dark" ? "sun" : "moon"} size={15} />
      </button>
    </div>
  </aside>

  <main class="pane">
    {#if section === "projects"}
      <header class="phead">
        <div class="search">
          <Icon name="search" size={16} />
          <input placeholder="Search projects" bind:value={query} />
        </div>
        <div class="acts">
          <button class="btn primary" onclick={() => (ui.wizardOpen = true)}>
            <Icon name="file-code" size={15} /> New Project
          </button>
          <button class="btn" onclick={openFromPicker}>
            <Icon name="folder-open" size={15} /> Open
          </button>
          <button class="btn">
            <Icon name="git" size={15} /> Clone Repository
          </button>
        </div>
      </header>

      <div class="recents">
        {#if recents.length === 0}
          <div class="empty">
            <Icon name="folder" size={42} />
            <p>No recent projects</p>
            <span>Create a new project or open an existing folder to get started.</span>
            <div class="ecta">
              <button class="btn primary" onclick={() => (ui.wizardOpen = true)}>
                <Icon name="file-code" size={15} /> New Project
              </button>
              <button class="btn" onclick={openFromPicker}>
                <Icon name="folder-open" size={15} /> Open Folder
              </button>
            </div>
          </div>
        {:else if filtered.length === 0}
          <div class="empty"><Icon name="folder" size={40} /><p>No matching projects</p></div>
        {/if}
        {#each filtered as p (p.path)}
          <button class="card" onclick={() => loadProject(p.path)}>
            <span class="avatar" style="--c: {kindColor[p.kind] ?? 'var(--fg-2)'}">{initials(p.name)}</span>
            <div class="cbody">
              <div class="crow"><span class="cname">{p.name}</span></div>
              <div class="cpath mono">{p.path}</div>
            </div>
            <div class="cmeta">
              <span class="kind" style="color: {kindColor[p.kind] ?? 'var(--fg-2)'}">
                <Icon name={kindIcon[p.kind] ?? 'file'} size={12} /> {p.kind}
              </span>
              <span class="when">{relTime(p.openedAt)}</span>
            </div>
          </button>
        {/each}
      </div>
    {:else if section === "customize"}
      <div class="simple">
        <h2>Customize</h2>
        <div class="opt">
          <span>Color theme</span>
          <div class="seg">
            <button class:on={ui.theme === "dark"} onclick={() => ui.theme === "light" && toggleTheme()}>Dark</button>
            <button class:on={ui.theme === "light"} onclick={() => ui.theme === "dark" && toggleTheme()}>Light</button>
          </div>
        </div>
        <div class="opt">
          <span>Keymap</span>
          <div class="seg"><button class="on">IntelliJ</button><button>VS Code</button></div>
        </div>
        <div class="opt">
          <span>Network mode{#if networkLocked()} <span class="lk">· managed</span>{/if}</span>
          <div class="seg">
            {#each ["offline", "ask", "online"] as m}
              <button class:on={ui.networkMode === m} disabled={networkLocked()} onclick={() => !networkLocked() && (ui.networkMode = m as any)}>{m}</button>
            {/each}
          </div>
        </div>
        <p class="note"><Icon name="info" size={13} /> Some settings may be locked by an institutional <code>policy.toml</code>.</p>
      </div>
    {:else}
      <div class="simple">
        <h2>Learn Typide</h2>
        {#each [["Typst in 10 minutes", "language basics inside the editor"], ["Set up Zotero (offline)", "local API + Better BibTeX fallback"], ["Make a thesis reproducible", "vendor packages, embed fonts, lock toolchain"], ["Migrate 0.14 → 0.15", "run the migration assistant with visual diff"]] as [t, d]}
          <a class="learn" href="#learn">
            <Icon name="book" size={16} />
            <div><div class="lt">{t}</div><div class="ld">{d}</div></div>
            <Icon name="chevron" size={14} />
          </a>
        {/each}
      </div>
    {/if}
  </main>
</div>

<style>
  .launcher {
    height: 100%;
    display: flex;
    background: var(--bg-1);
  }
  .rail {
    width: 260px;
    flex: none;
    background: var(--bg-0);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    padding: 22px 16px 14px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 0 6px 22px;
  }
  .bname {
    font-size: 17px;
    font-weight: 700;
  }
  .bver {
    font-size: 11px;
    color: var(--fg-3);
    margin-top: 1px;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .nav {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 9px 12px;
    border-radius: 8px;
    color: var(--fg-1);
    font-size: 13.5px;
    font-weight: 500;
    text-align: left;
  }
  .nav:hover {
    background: var(--bg-3);
  }
  .nav.on {
    background: var(--accent-soft);
    color: var(--fg-0);
  }
  .rail-foot {
    margin-top: auto;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 6px 0;
    border-top: 1px solid var(--border);
  }
  .net {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 11.5px;
    color: var(--fg-2);
  }
  .net.ask {
    color: var(--warn);
  }
  .net.online {
    color: var(--ok);
  }
  .theme {
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--fg-2);
    border-radius: 7px;
  }
  .theme:hover {
    background: var(--bg-3);
    color: var(--fg-0);
  }
  .pane {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 24px 30px;
    overflow-y: auto;
  }
  .phead {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-bottom: 22px;
  }
  .search {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 9px;
    height: 40px;
    padding: 0 14px;
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 9px;
    color: var(--fg-3);
    max-width: 420px;
  }
  .search input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    font-size: 14px;
    color: var(--fg-0);
  }
  .acts {
    display: flex;
    gap: 8px;
    margin-left: auto;
  }
  .btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 40px;
    padding: 0 16px;
    border-radius: 9px;
    border: 1px solid var(--border-strong);
    background: var(--bg-2);
    color: var(--fg-1);
    font-size: 13px;
    font-weight: 500;
  }
  .btn:hover {
    background: var(--bg-4);
  }
  .btn.primary {
    background: var(--accent);
    color: var(--accent-fg);
    border-color: transparent;
  }
  .btn.primary:hover {
    filter: brightness(1.08);
  }
  .recents {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .card {
    display: flex;
    align-items: center;
    gap: 15px;
    padding: 14px 16px;
    border-radius: 11px;
    border: 1px solid var(--border);
    background: var(--bg-2);
    text-align: left;
    transition: border-color 0.12s, background 0.12s, transform 0.06s;
  }
  .card:hover {
    border-color: var(--accent);
    background: var(--bg-3);
  }
  .card:active {
    transform: translateY(1px);
  }
  .avatar {
    width: 44px;
    height: 44px;
    flex: none;
    border-radius: 11px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    font-size: 15px;
    color: #fff;
    background: color-mix(in srgb, var(--c) 82%, #000 10%);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--c) 60%, transparent);
  }
  .cbody {
    flex: 1;
    min-width: 0;
  }
  .crow {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .cname {
    font-size: 14.5px;
    font-weight: 600;
    color: var(--fg-0);
  }
  .pin {
    color: var(--accent);
    display: inline-flex;
  }
  .cpath {
    font-size: 12px;
    color: var(--fg-3);
    margin-top: 2px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .cmeta {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 5px;
    flex: none;
  }
  .kind {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    text-transform: capitalize;
    font-weight: 600;
  }
  .when {
    font-size: 11px;
    color: var(--fg-3);
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    color: var(--fg-3);
    padding: 70px 40px;
    text-align: center;
  }
  .empty p {
    font-size: 16px;
    color: var(--fg-1);
    margin-top: 4px;
  }
  .empty span {
    font-size: 13px;
    max-width: 340px;
  }
  .ecta {
    display: flex;
    gap: 10px;
    margin-top: 14px;
  }
  .simple {
    max-width: 560px;
  }
  .simple h2 {
    font-size: 20px;
    margin-bottom: 20px;
    font-weight: 700;
  }
  .opt {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 0;
    border-bottom: 1px solid var(--border);
    font-size: 14px;
  }
  .seg {
    display: flex;
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 2px;
    gap: 2px;
  }
  .seg button {
    padding: 6px 14px;
    border-radius: 6px;
    color: var(--fg-2);
    font-size: 12.5px;
    text-transform: capitalize;
  }
  .seg button.on {
    background: var(--accent);
    color: var(--accent-fg);
  }
  .seg button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .lk {
    color: var(--warn);
    font-size: 11px;
  }
  .note {
    margin-top: 20px;
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12.5px;
    color: var(--fg-3);
  }
  code {
    font-family: "JetBrains Mono", monospace;
    background: var(--bg-3);
    padding: 1px 5px;
    border-radius: 4px;
  }
  .learn {
    display: flex;
    align-items: center;
    gap: 13px;
    padding: 14px 16px;
    border-radius: 10px;
    border: 1px solid var(--border);
    background: var(--bg-2);
    margin-bottom: 8px;
    text-decoration: none;
    color: var(--fg-1);
  }
  .learn:hover {
    border-color: var(--accent);
    background: var(--bg-3);
  }
  .learn > div {
    flex: 1;
  }
  .lt {
    font-size: 13.5px;
    font-weight: 600;
    color: var(--fg-0);
  }
  .ld {
    font-size: 12px;
    color: var(--fg-3);
    margin-top: 2px;
  }
</style>
