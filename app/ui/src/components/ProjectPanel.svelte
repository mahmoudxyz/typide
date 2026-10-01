<script lang="ts">
  import Icon from "./Icon.svelte";
  import TreeNode from "./TreeNode.svelte";
  import { data, ui, newFile, newFolder, pasteIntoProject } from "../lib/store.svelte";

  let dragOver = $state(false);

  function bgContextMenu(e: MouseEvent) {
    // Right-click on empty space → act on the project root.
    if (e.target !== e.currentTarget) return;
    e.preventDefault();
    data.selectedPath = "";
    ui.treeMenu = { x: e.clientX, y: e.clientY, path: data.project?.root ?? "", kind: "dir" };
  }
  function onPaste(e: ClipboardEvent) {
    if (!e.clipboardData) return;
    if ((e.clipboardData.files?.length ?? 0) > 0 || e.clipboardData.getData("text/uri-list")) {
      e.preventDefault();
      pasteIntoProject(data.selectedPath, e.clipboardData);
    }
  }
  function onDrop(e: DragEvent) {
    e.preventDefault();
    dragOver = false;
    pasteIntoProject(data.selectedPath, e.dataTransfer);
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions a11y_no_noninteractive_tabindex -->
<div
  class="wrap"
  class:dragOver
  tabindex="0"
  role="tree"
  oncontextmenu={bgContextMenu}
  onpaste={onPaste}
  ondragover={(e) => { e.preventDefault(); dragOver = true; }}
  ondragleave={() => (dragOver = false)}
  ondrop={onDrop}
>
  <div class="bar">
    <button title="New file" onclick={() => newFile(data.selectedPath)}><Icon name="file" size={13} /></button>
    <button title="New folder" onclick={() => newFolder(data.selectedPath)}><Icon name="folder" size={13} /></button>
    <span class="hint">right-click or paste ⇢ files/images</span>
  </div>
  <div class="tree">
    {#each data.tree?.children ?? [] as node (node.path)}
      <TreeNode {node} depth={0} />
    {/each}
    {#if (data.tree?.children?.length ?? 0) === 0}
      <div class="empty">Empty project — right-click to add a file.</div>
    {/if}
  </div>
  {#if dragOver}<div class="dropnote"><Icon name="download" size={16} /> Drop to add to the project</div>{/if}
</div>

<style>
  .wrap {
    position: relative;
    height: 100%;
    outline: none;
    display: flex;
    flex-direction: column;
  }
  .wrap.dragOver {
    box-shadow: inset 0 0 0 2px var(--accent);
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 4px 6px;
    border-bottom: 1px solid var(--border);
  }
  .bar button {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--fg-3);
    border-radius: 5px;
  }
  .bar button:hover {
    background: var(--bg-4);
    color: var(--fg-0);
  }
  .hint {
    margin-left: auto;
    font-size: 10px;
    color: var(--fg-3);
    padding-right: 4px;
  }
  .tree {
    padding: 4px 0;
    overflow-y: auto;
    flex: 1;
  }
  .empty {
    padding: 18px 14px;
    font-size: 12px;
    color: var(--fg-3);
    text-align: center;
  }
  .dropnote {
    position: absolute;
    inset: auto 10px 10px 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 10px;
    background: var(--accent-soft);
    color: var(--accent);
    border: 1px dashed var(--accent);
    border-radius: 8px;
    font-size: 12px;
    pointer-events: none;
  }
</style>
