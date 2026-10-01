<script lang="ts">
  import Icon from "./Icon.svelte";
  import { ui, type ToolId } from "../lib/store.svelte";

  const items: { id: ToolId; icon: string; label: string }[] = [
    { id: "project", icon: "folder", label: "Project" },
    { id: "structure", icon: "structure", label: "Structure" },
    { id: "notes", icon: "book", label: "Notes" },
    { id: "git", icon: "git", label: "Version Control" },
    { id: "packages", icon: "package", label: "Packages" },
    { id: "fonts", icon: "text", label: "Fonts" },
  ];

  function pick(id: ToolId) {
    if (ui.activeTool === id && ui.leftVisible) ui.leftVisible = false;
    else {
      ui.activeTool = id;
      ui.leftVisible = true;
    }
  }
</script>

<nav class="bar">
  <div class="group">
    {#each items as it (it.id)}
      <button
        class="item"
        class:active={ui.activeTool === it.id && ui.leftVisible}
        title={it.label}
        onclick={() => pick(it.id)}
      >
        <Icon name={it.icon} size={20} />
      </button>
    {/each}
  </div>
  <div class="group">
    <button class="item" title="Search Everywhere (Shift Shift)" onclick={() => (ui.searchEverywhere = true)}>
      <Icon name="compass" size={20} />
    </button>
    <button class="item" title="Settings"><Icon name="settings" size={20} /></button>
  </div>
</nav>

<style>
  .bar {
    width: 48px;
    background: var(--bg-0);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 8px 0;
    flex: none;
  }
  .group {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }
  .item {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--fg-3);
    border-radius: var(--radius-sm);
    position: relative;
    transition: color 0.12s, background 0.12s;
  }
  .item:hover {
    color: var(--fg-1);
    background: var(--bg-3);
  }
  .item.active {
    color: var(--accent);
  }
  .item.active::before {
    content: "";
    position: absolute;
    left: -8px;
    top: 9px;
    bottom: 9px;
    width: 3px;
    border-radius: 0 3px 3px 0;
    background: var(--accent);
  }
</style>
