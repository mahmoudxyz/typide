<script lang="ts">
  import Icon from "./Icon.svelte";
  import Editor from "./Editor.svelte";
  import { ui, getFile, closeTab, isDirty, relPath } from "../lib/store.svelte";

  const iconFor = (path: string) => {
    const f = getFile(path);
    if (!f) return "file";
    if (f.language === "typst") return "file-code";
    if (f.language === "bibtex") return "quote";
    return "file";
  };
  const nameOf = (path: string) => getFile(path)?.name ?? path;

  const active = $derived(getFile(ui.activeTab));
  // Breadcrumb from the active path, relative to the project root.
  const crumbs = $derived(ui.activeTab ? relPath(ui.activeTab).split("/") : []);
</script>

<section class="area">
  <div class="tabs">
    {#each ui.openTabs as path (path)}
      <div class="tab" class:active={ui.activeTab === path} onclick={() => (ui.activeTab = path)} role="tab" tabindex="0" onkeydown={(e) => e.key === "Enter" && (ui.activeTab = path)}>
        <span class="ic ic-{getFile(path)?.language}"><Icon name={iconFor(path)} size={14} /></span>
        <span class="name">{nameOf(path)}</span>
        <button
          class="close"
          class:dirty={isDirty(path)}
          onclick={(e) => {
            e.stopPropagation();
            closeTab(path);
          }}
          title="Close"
        >
          {#if isDirty(path)}<span class="dot"></span>{:else}<Icon name="x" size={12} />{/if}
        </button>
      </div>
    {/each}
  </div>

  {#if active}
    <div class="breadcrumb">
      {#each crumbs as c, i}
        {#if i > 0}<span class="sep"><Icon name="chevron" size={11} /></span>{/if}
        <span class="crumb" class:last={i === crumbs.length - 1}>{c}</span>
      {/each}
      <span class="spacer"></span>
      <span class="lang mono">{active.language}</span>
    </div>
    {#key ui.activeTab === "" ? "empty" : "editor"}
      <Editor />
    {/key}
  {:else}
    <div class="empty">
      <Icon name="file-code" size={40} />
      <p>No file open</p>
      <span>Pick a file in the Project panel, or press <kbd>Shift</kbd> <kbd>Shift</kbd></span>
    </div>
  {/if}
</section>

<style>
  .area {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    background: var(--bg-2);
    overflow: hidden;
  }
  .tabs {
    display: flex;
    height: 36px;
    background: var(--bg-1);
    border-bottom: 1px solid var(--border);
    overflow-x: auto;
    flex: none;
  }
  .tabs::-webkit-scrollbar {
    height: 0;
  }
  .tab {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 0 10px 0 12px;
    height: 100%;
    border-right: 1px solid var(--border);
    color: var(--fg-2);
    cursor: pointer;
    white-space: nowrap;
    position: relative;
    max-width: 220px;
  }
  .tab:hover {
    background: var(--bg-3);
  }
  .tab.active {
    background: var(--bg-2);
    color: var(--fg-0);
  }
  .tab.active::after {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
    height: 2px;
    background: var(--accent);
  }
  .ic {
    display: inline-flex;
    color: var(--fg-3);
  }
  .ic-typst {
    color: var(--accent);
  }
  .ic-bibtex {
    color: var(--accent-2);
  }
  .name {
    font-size: 12.5px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .close {
    width: 18px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    color: var(--fg-3);
  }
  .close:hover {
    background: var(--bg-4);
    color: var(--fg-0);
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--fg-1);
  }
  .breadcrumb {
    height: 28px;
    display: flex;
    align-items: center;
    gap: 3px;
    padding: 0 14px;
    font-size: 11.5px;
    color: var(--fg-3);
    border-bottom: 1px solid var(--border);
    background: var(--bg-2);
    flex: none;
  }
  .crumb.last {
    color: var(--fg-1);
  }
  .sep {
    display: inline-flex;
    color: var(--fg-3);
    opacity: 0.6;
  }
  .spacer {
    flex: 1;
  }
  .lang {
    font-size: 10.5px;
    color: var(--fg-3);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: var(--fg-3);
  }
  .empty p {
    font-size: 15px;
    color: var(--fg-2);
  }
  .empty span {
    font-size: 12px;
  }
</style>
