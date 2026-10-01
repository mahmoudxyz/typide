<script lang="ts">
  import Icon from "./Icon.svelte";
  import { ui, createProject } from "../lib/store.svelte";
  import { generators, templates, availableTypst, type Generator } from "../lib/mockData";

  let gen = $state<Generator>(generators[0]);
  let name = $state("untitled");
  let location = $state("~/research");
  let typst = $state(availableTypst[0]);
  let mode = $state<"vendor" | "cache">(generators[0].defaultMode);
  let gitInit = $state(true);
  let vault = $state(generators[0].vault);
  let refs = $state(generators[0].refs);
  let zotero = $state(false);
  let tplQuery = $state("");
  let selectedTpl = $state(templates[0].spec);

  function pickGen(g: Generator) {
    gen = g;
    mode = g.defaultMode;
    vault = g.vault;
    refs = g.refs;
    if (name === "untitled" || !name) name = g.id === "template" ? "untitled" : g.id;
  }

  const fullPath = $derived(`${location.replace(/\/$/, "")}/${name || "untitled"}`);
  const filteredTpl = $derived(
    templates.filter(
      (t) =>
        t.title.toLowerCase().includes(tplQuery.toLowerCase()) ||
        t.spec.toLowerCase().includes(tplQuery.toLowerCase())
    )
  );
  const canCreate = $derived(name.trim().length > 0 && location.trim().length > 0);

  const srcColor: Record<string, string> = {
    universe: "var(--accent)",
    local: "var(--accent-2)",
    institutional: "var(--syn-heading)",
  };

  let creating = $state(false);
  let error = $state("");

  function close() {
    ui.wizardOpen = false;
  }
  async function browse() {
    const picked = await openFromPickerLocation();
    if (picked) location = picked;
  }
  // Folder picker just for the Location field (does not open a project).
  async function openFromPickerLocation(): Promise<string | null> {
    const mod = await import("../lib/api");
    return mod.api.pickFolder();
  }
  async function create() {
    if (!canCreate || creating) return;
    creating = true;
    error = "";
    try {
      await createProject({
        path: fullPath,
        templateId: gen.id,
        name: name.trim(),
        typst,
        packagesMode: mode,
        gitInit,
        refs,
        vault,
        templateSpec: gen.id === "template" ? selectedTpl : undefined,
      });
    } catch (e) {
      error = String((e as any)?.message ?? e);
      creating = false;
    }
  }
</script>

<div class="overlay" role="presentation">
  <div class="wizard" role="dialog" aria-label="New Project">
    <header>
      <span class="ttl">New Project</span>
      <button class="x" onclick={close} title="Cancel"><Icon name="x" size={16} /></button>
    </header>

    <div class="cols">
      <aside class="gens">
        {#each generators as g (g.id)}
          <button class="gen" class:on={gen.id === g.id} onclick={() => pickGen(g)}>
            <span class="gic"><Icon name={g.icon} size={17} /></span>
            {g.title}
          </button>
        {/each}
      </aside>

      <section class="form">
        <div class="ghead">
          <span class="ghic"><Icon name={gen.icon} size={20} /></span>
          <div>
            <div class="gtitle">{gen.title}</div>
            <div class="gblurb">{gen.blurb}</div>
          </div>
        </div>

        {#if gen.kind === "template"}
          <div class="tpl-search">
            <Icon name="search" size={15} />
            <input placeholder="Search templates (Universe · local · institutional)" bind:value={tplQuery} />
          </div>
          <div class="tpls">
            {#each filteredTpl as t (t.spec)}
              <button class="tpl" class:on={selectedTpl === t.spec} onclick={() => (selectedTpl = t.spec)}>
                <div class="trow">
                  <span class="ttitle">{t.title}</span>
                  <span class="tsrc" style="color: {srcColor[t.source]}">{t.source}</span>
                </div>
                <div class="tspec mono">{t.spec}</div>
                <div class="tblurb">{t.blurb}</div>
              </button>
            {/each}
          </div>
        {/if}

        <div class="fields">
          <label class="field">
            <span class="lbl">Name</span>
            <input class="in" bind:value={name} spellcheck="false" />
          </label>
          <label class="field">
            <span class="lbl">Location</span>
            <div class="loc">
              <input class="in" bind:value={location} spellcheck="false" />
              <button class="browse" onclick={browse}><Icon name="folder-open" size={15} /> Browse</button>
            </div>
          </label>
          <div class="pathpreview">
            <Icon name="folder" size={13} />
            <span class="mono">{fullPath}</span>
          </div>

          <div class="grid2">
            <label class="field">
              <span class="lbl">Typst toolchain</span>
              <div class="select">
                <select bind:value={typst}>
                  {#each availableTypst as v}
                    <option value={v}>{v}{v === availableTypst[0] ? "  (bundled)" : ""}</option>
                  {/each}
                </select>
                <Icon name="chevron" size={12} />
              </div>
            </label>
            <div class="field">
              <span class="lbl">Packages</span>
              <div class="seg">
                <button class:on={mode === "cache"} onclick={() => (mode = "cache")}>Cache</button>
                <button class:on={mode === "vendor"} onclick={() => (mode = "vendor")}>Vendor</button>
              </div>
            </div>
          </div>

          <div class="toggles">
            <label class="tog"><input type="checkbox" bind:checked={gitInit} /> <span>Initialize Git repo (snapshots)</span></label>
            {#if gen.kind !== "package"}
              <label class="tog"><input type="checkbox" bind:checked={refs} /> <span>Add bibliography (<code>refs.bib</code>)</span></label>
              <label class="tog"><input type="checkbox" bind:checked={vault} /> <span>Create notes vault (<code>notes/</code>)</span></label>
              <label class="tog"><input type="checkbox" bind:checked={zotero} /> <span>Enable Zotero integration (local API)</span></label>
            {/if}
          </div>

          {#if mode === "vendor"}
            <div class="hint"><Icon name="check-circle" size={13} /> Vendored: packages are copied in & committed — compiles years later with no network.</div>
          {/if}
        </div>
      </section>
    </div>

    <footer>
      {#if error}
        <span class="ferr"><Icon name="warn" size={13} /> {error}</span>
      {:else}
        <span class="fnote"><Icon name="info" size={13} /> Creates <code>typide.toml</code> + entrypoint from the {gen.title} generator</span>
      {/if}
      <div class="fbtns">
        <button class="btn" onclick={close}>Cancel</button>
        <button class="btn primary" disabled={!canCreate || creating} onclick={create}>
          {#if creating}<span class="spin"></span> Creating…{:else}Create{/if}
        </button>
      </div>
    </footer>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(3px);
    z-index: 300;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .wizard {
    width: 880px;
    max-width: 94vw;
    height: 640px;
    max-height: 92vh;
    background: var(--bg-1);
    border: 1px solid var(--border-strong);
    border-radius: 14px;
    box-shadow: var(--shadow);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: pop 0.14s ease;
  }
  @keyframes pop {
    from {
      transform: scale(0.98);
      opacity: 0;
    }
  }
  header {
    height: 50px;
    display: flex;
    align-items: center;
    padding: 0 16px;
    border-bottom: 1px solid var(--border);
    flex: none;
  }
  .ttl {
    font-size: 15px;
    font-weight: 700;
  }
  .x {
    margin-left: auto;
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
  .cols {
    flex: 1;
    display: flex;
    min-height: 0;
  }
  .gens {
    width: 220px;
    flex: none;
    background: var(--bg-0);
    border-right: 1px solid var(--border);
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .gen {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 10px 12px;
    border-radius: 8px;
    color: var(--fg-1);
    font-size: 13.5px;
    text-align: left;
    font-weight: 500;
  }
  .gen:hover {
    background: var(--bg-3);
  }
  .gen.on {
    background: var(--accent-soft);
    color: var(--fg-0);
  }
  .gic {
    color: var(--fg-3);
    display: inline-flex;
  }
  .gen.on .gic {
    color: var(--accent);
  }
  .form {
    flex: 1;
    padding: 22px 24px;
    overflow-y: auto;
    min-width: 0;
  }
  .ghead {
    display: flex;
    gap: 13px;
    margin-bottom: 20px;
  }
  .ghic {
    width: 42px;
    height: 42px;
    flex: none;
    border-radius: 11px;
    background: var(--accent-soft);
    color: var(--accent);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .gtitle {
    font-size: 16px;
    font-weight: 700;
  }
  .gblurb {
    font-size: 12.5px;
    color: var(--fg-2);
    margin-top: 4px;
    line-height: 1.5;
  }
  .tpl-search {
    display: flex;
    align-items: center;
    gap: 9px;
    height: 38px;
    padding: 0 13px;
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 9px;
    color: var(--fg-3);
    margin-bottom: 10px;
  }
  .tpl-search input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    font-size: 13px;
    color: var(--fg-0);
  }
  .tpls {
    display: flex;
    flex-direction: column;
    gap: 7px;
    margin-bottom: 20px;
  }
  .tpl {
    text-align: left;
    padding: 11px 13px;
    border: 1px solid var(--border);
    border-radius: 9px;
    background: var(--bg-2);
  }
  .tpl:hover {
    border-color: var(--border-strong);
  }
  .tpl.on {
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .trow {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .ttitle {
    font-size: 13.5px;
    font-weight: 600;
    color: var(--fg-0);
  }
  .tsrc {
    margin-left: auto;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 700;
  }
  .tspec {
    font-size: 11px;
    color: var(--accent);
    margin-top: 3px;
  }
  .tblurb {
    font-size: 11.5px;
    color: var(--fg-2);
    margin-top: 4px;
  }
  .fields {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .lbl {
    font-size: 11.5px;
    font-weight: 600;
    color: var(--fg-2);
  }
  .in {
    height: 36px;
    padding: 0 12px;
    background: var(--bg-2);
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    font-size: 13px;
    color: var(--fg-0);
    outline: none;
  }
  .in:focus {
    border-color: var(--accent);
  }
  .loc {
    display: flex;
    gap: 8px;
  }
  .loc .in {
    flex: 1;
  }
  .browse {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 0 14px;
    border-radius: 8px;
    border: 1px solid var(--border-strong);
    background: var(--bg-2);
    color: var(--fg-1);
    font-size: 12.5px;
  }
  .browse:hover {
    background: var(--bg-4);
  }
  .pathpreview {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 9px 12px;
    background: var(--bg-inset);
    border: 1px solid var(--border);
    border-radius: 8px;
    font-size: 12px;
    color: var(--fg-2);
  }
  .grid2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
  }
  .select {
    position: relative;
    display: flex;
    align-items: center;
  }
  .select select {
    width: 100%;
    height: 36px;
    padding: 0 30px 0 12px;
    background: var(--bg-2);
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    font-size: 13px;
    color: var(--fg-0);
    appearance: none;
    outline: none;
    cursor: pointer;
  }
  .select :global(svg) {
    position: absolute;
    right: 10px;
    color: var(--fg-3);
    transform: rotate(90deg);
    pointer-events: none;
  }
  .seg {
    display: flex;
    background: var(--bg-2);
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    padding: 2px;
    gap: 2px;
    height: 36px;
  }
  .seg button {
    flex: 1;
    border-radius: 6px;
    color: var(--fg-2);
    font-size: 12.5px;
  }
  .seg button.on {
    background: var(--accent);
    color: var(--accent-fg);
  }
  .toggles {
    display: flex;
    flex-direction: column;
    gap: 11px;
    margin-top: 2px;
  }
  .tog {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
    color: var(--fg-1);
    cursor: pointer;
  }
  .tog input {
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
  }
  .hint {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--ok);
    background: var(--ok-soft);
    padding: 9px 12px;
    border-radius: 8px;
  }
  code {
    font-family: "JetBrains Mono", monospace;
    background: var(--bg-3);
    padding: 0 5px;
    border-radius: 4px;
    font-size: 0.92em;
  }
  footer {
    height: 60px;
    flex: none;
    display: flex;
    align-items: center;
    padding: 0 20px;
    border-top: 1px solid var(--border);
    background: var(--bg-0);
  }
  .fnote {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--fg-3);
  }
  .ferr {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--error);
  }
  .spin {
    width: 13px;
    height: 13px;
    border: 2px solid rgba(255, 255, 255, 0.4);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    display: inline-block;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .fbtns {
    margin-left: auto;
    display: flex;
    gap: 10px;
  }
  .btn {
    height: 38px;
    padding: 0 20px;
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
  .btn.primary:hover:not(:disabled) {
    filter: brightness(1.08);
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
