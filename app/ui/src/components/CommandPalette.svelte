<script lang="ts">
  import Icon from "./Icon.svelte";
  import { ui, toggleTheme, makeOfflineReady, exportActiveProfile } from "../lib/store.svelte";

  interface Cmd {
    title: string;
    hint: string;
    icon: string;
    run: () => void;
  }

  const commands: Cmd[] = [
    { title: "Export: Run 'PDF'", hint: "compile + export", icon: "play", run: () => exportActiveProfile() },
    { title: "Browse Typst Universe…", hint: "search + add packages", icon: "package", run: () => (ui.packageBrowser = true) },
    { title: "Make project offline-ready", hint: "resolve · vendor · lock", icon: "download", run: () => makeOfflineReady() },
    { title: "Manage fonts", hint: "font database", icon: "text", run: () => (ui.activeTool = "fonts") },
    { title: "Refactor: Rename symbol…", hint: "project-wide, previewed", icon: "bolt", run: () => {} },
    { title: "Migrate project to Typst 0.15.1", hint: "snapshot + visual diff", icon: "refresh", run: () => {} },
    { title: "Run submission checker", hint: "thesis-submission profile", icon: "check-circle", run: () => (ui.bottomVisible = true) },
    { title: "New literature note from citation", hint: "notes vault", icon: "book", run: () => (ui.activeTool = "notes") },
    { title: "Create snapshot", hint: "version control", icon: "history", run: () => (ui.activeTool = "git") },
    { title: "Sync with remote", hint: "fetch · merge · push", icon: "refresh", run: () => (ui.activeTool = "git") },
    { title: "Toggle theme", hint: "light / dark", icon: "moon", run: toggleTheme },
    { title: "Toggle preview", hint: "", icon: "eye", run: () => (ui.previewVisible = !ui.previewVisible) },
    { title: "Insert citation…", hint: "fuzzy by author/title/year", icon: "quote", run: () => (ui.citationPicker = true) },
    { title: "Toolchain: Manage versions…", hint: "install typst versions", icon: "layers", run: () => (ui.toolchainManager = true) },
  ];

  let query = $state("");
  let sel = $state(0);

  const filtered = $derived(
    commands.filter((c) => c.title.toLowerCase().includes(query.toLowerCase()))
  );

  function close() {
    ui.commandPalette = false;
    query = "";
    sel = 0;
  }
  function exec(c: Cmd) {
    c.run();
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
    } else if (e.key === "Enter" && filtered[sel]) exec(filtered[sel]);
  }
  $effect(() => {
    query;
    sel = 0;
  });
</script>

<div class="overlay" onclick={close} role="presentation">
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_interactive_supports_focus -->
  <div class="palette" onclick={(e) => e.stopPropagation()} role="dialog" aria-label="Command palette" tabindex="-1">
    <div class="input">
      <Icon name="command" size={17} />
      <!-- svelte-ignore a11y_autofocus -->
      <input
        autofocus
        bind:value={query}
        onkeydown={keydown}
        placeholder="Type a command…"
      />
      <kbd>Esc</kbd>
    </div>
    <div class="list">
      {#each filtered as c, i (c.title)}
        <button class="row" class:sel={i === sel} onclick={() => exec(c)} onmousemove={() => (sel = i)}>
          <span class="ic"><Icon name={c.icon} size={15} /></span>
          <span class="t">{c.title}</span>
          {#if c.hint}<span class="h">{c.hint}</span>{/if}
        </button>
      {/each}
      {#if filtered.length === 0}
        <div class="none">No matching commands</div>
      {/if}
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(2px);
    z-index: 200;
    display: flex;
    justify-content: center;
    padding-top: 12vh;
  }
  .palette {
    width: 640px;
    max-width: 92vw;
    max-height: 60vh;
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
    border-radius: 7px;
    text-align: left;
    color: var(--fg-1);
  }
  .row.sel {
    background: var(--accent-soft);
    color: var(--fg-0);
  }
  .ic {
    color: var(--fg-3);
    display: inline-flex;
  }
  .row.sel .ic {
    color: var(--accent);
  }
  .t {
    font-size: 13px;
  }
  .h {
    margin-left: auto;
    font-size: 11px;
    color: var(--fg-3);
    font-family: "JetBrains Mono", monospace;
  }
  .none {
    padding: 20px;
    text-align: center;
    color: var(--fg-3);
    font-size: 13px;
  }
</style>
