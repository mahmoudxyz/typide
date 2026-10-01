<script lang="ts">
  import Icon from "./Icon.svelte";
  import { ui, data, newFile, newFolder, renameNode, deleteNode } from "../lib/store.svelte";

  const m = $derived(ui.treeMenu);
  const isRoot = $derived(!!m && (!m.path || m.path === data.project?.root));

  function close() {
    ui.treeMenu = null;
  }
  function run(fn: () => void) {
    close();
    fn();
  }
</script>

{#if m}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="scrim" role="presentation" onclick={close} oncontextmenu={(e) => { e.preventDefault(); close(); }}></div>
  <div class="menu" style="left: {m.x}px; top: {m.y}px" role="menu">
    <button role="menuitem" onclick={() => run(() => newFile(m.path))}>
      <Icon name="file" size={14} /> New File
    </button>
    <button role="menuitem" onclick={() => run(() => newFolder(m.path))}>
      <Icon name="folder" size={14} /> New Folder
    </button>
    {#if m.path && !isRoot}
      <div class="div"></div>
      <button role="menuitem" onclick={() => run(() => renameNode(m.path))}>
        <Icon name="text" size={14} /> Rename…
      </button>
      <button role="menuitem" class="danger" onclick={() => run(() => deleteNode(m.path))}>
        <Icon name="x" size={14} /> Delete…
      </button>
    {/if}
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 740;
  }
  .menu {
    position: fixed;
    z-index: 741;
    min-width: 170px;
    background: var(--bg-3);
    border: 1px solid var(--border-strong);
    border-radius: 9px;
    box-shadow: var(--shadow);
    padding: 4px;
    animation: pop 0.1s ease;
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(-3px);
    }
  }
  button {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    text-align: left;
    padding: 7px 10px;
    border-radius: 6px;
    font-size: 12.5px;
    color: var(--fg-1);
  }
  button :global(svg) {
    color: var(--fg-3);
  }
  button:hover {
    background: var(--bg-4);
    color: var(--fg-0);
  }
  button.danger {
    color: var(--error);
  }
  button.danger:hover {
    background: color-mix(in srgb, var(--error) 14%, transparent);
  }
  button.danger :global(svg) {
    color: var(--error);
  }
  .div {
    height: 1px;
    background: var(--border);
    margin: 4px 2px;
  }
</style>
