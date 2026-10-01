<script lang="ts">
  import Icon from "./Icon.svelte";
  import {
    ui,
    data,
    newFile,
    newFolder,
    renameNode,
    deleteNode,
    duplicateNode,
    clipCopy,
    clipCut,
    pasteClip,
    copyPath,
    revealNode,
  } from "../lib/store.svelte";

  const m = $derived(ui.treeMenu);
  const isRoot = $derived(!!m && (!m.path || m.path === data.project?.root));
  const hasClip = $derived(!!data.fileClip);

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
    {#if hasClip}
      <button role="menuitem" onclick={() => run(() => pasteClip(m.path))}>
        <Icon name="download" size={14} /> Paste
      </button>
    {/if}

    {#if m.path && !isRoot}
      <div class="div"></div>
      <button role="menuitem" onclick={() => run(() => clipCut(m.path))}>
        <Icon name="scissors" size={14} /> Cut
      </button>
      <button role="menuitem" onclick={() => run(() => clipCopy(m.path))}>
        <Icon name="copy" size={14} /> Copy
      </button>
      <button role="menuitem" onclick={() => run(() => duplicateNode(m.path))}>
        <Icon name="layers" size={14} /> Duplicate
      </button>
      <button role="menuitem" onclick={() => run(() => renameNode(m.path))}>
        <Icon name="text" size={14} /> Rename…
      </button>

      <div class="div"></div>
      <button role="menuitem" onclick={() => run(() => copyPath(m.path, false))}>
        <Icon name="link" size={14} /> Copy Path
      </button>
      <button role="menuitem" onclick={() => run(() => copyPath(m.path, true))}>
        <Icon name="link" size={14} /> Copy Relative Path
      </button>
    {/if}

    <button role="menuitem" onclick={() => run(() => revealNode(m.path || data.project?.root || ""))}>
      <Icon name="folder-open" size={14} /> Reveal in File Manager
    </button>

    {#if m.path && !isRoot}
      <div class="div"></div>
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
