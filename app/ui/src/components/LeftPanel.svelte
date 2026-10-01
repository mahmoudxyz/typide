<script lang="ts">
  import Icon from "./Icon.svelte";
  import { ui, data, newFile, newFolder } from "../lib/store.svelte";
  import ProjectPanel from "./ProjectPanel.svelte";
  import StructurePanel from "./StructurePanel.svelte";
  import NotesPanel from "./NotesPanel.svelte";
  import GitPanel from "./GitPanel.svelte";
  import PackagesPanel from "./PackagesPanel.svelte";
  import FontsPanel from "./FontsPanel.svelte";

  const titles: Record<string, string> = {
    project: "Project",
    structure: "Structure",
    notes: "Notes",
    git: "Version Control",
    packages: "Packages",
    fonts: "Fonts",
  };
</script>

<aside class="panel" style="width: {ui.leftWidth}px">
  <header>
    <span class="title">{titles[ui.activeTool]}</span>
    <div class="hactions">
      {#if ui.activeTool === "project"}
        <button title="New file" onclick={() => newFile(data.selectedPath)}><Icon name="file" size={14} /></button>
        <button title="New folder" onclick={() => newFolder(data.selectedPath)}><Icon name="folder" size={14} /></button>
      {/if}
      <button title="Collapse" onclick={() => (ui.leftVisible = false)}>
        <Icon name="sidebar" size={14} />
      </button>
    </div>
  </header>
  <div class="content">
    {#if ui.activeTool === "project"}<ProjectPanel />
    {:else if ui.activeTool === "structure"}<StructurePanel />
    {:else if ui.activeTool === "notes"}<NotesPanel />
    {:else if ui.activeTool === "git"}<GitPanel />
    {:else if ui.activeTool === "packages"}<PackagesPanel />
    {:else if ui.activeTool === "fonts"}<FontsPanel />
    {/if}
  </div>
</aside>

<style>
  .panel {
    width: 272px;
    flex: none;
    background: var(--bg-1);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  header {
    height: 34px;
    display: flex;
    align-items: center;
    padding: 0 8px 0 12px;
    border-bottom: 1px solid var(--border);
    flex: none;
  }
  .title {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--fg-2);
  }
  .hactions {
    margin-left: auto;
    display: flex;
    gap: 2px;
  }
  .hactions button {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--fg-3);
    border-radius: 4px;
  }
  .hactions button:hover {
    color: var(--fg-1);
    background: var(--bg-4);
  }
  .content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }
</style>
