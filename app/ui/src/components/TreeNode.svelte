<script lang="ts">
  import Icon from "./Icon.svelte";
  import type { FileNode } from "../lib/types";
  import { ui, openFile, isDirty } from "../lib/store.svelte";
  import TreeNode from "./TreeNode.svelte";

  import { untrack } from "svelte";
  let { node, depth = 0 }: { node: FileNode; depth?: number } = $props();
  // Expand the top couple of levels by default; depth is fixed per node.
  let open = $state(untrack(() => depth) < 2);

  const iconFor = (n: FileNode) => {
    if (n.kind === "dir") return open ? "folder-open" : "folder";
    if (n.kind === "typ") return "file-code";
    if (n.kind === "bib") return "quote";
    return "file";
  };

  function click() {
    if (node.kind === "dir") open = !open;
    else openFile(node.path);
  }

  const isActive = $derived(ui.activeTab === node.path);
</script>

<div
  class="row"
  class:active={isActive}
  style="padding-left: {8 + depth * 14}px"
  onclick={click}
  role="treeitem"
  aria-selected={isActive}
  tabindex="0"
  onkeydown={(e) => e.key === "Enter" && click()}
>
  {#if node.kind === "dir"}
    <span class="twist" class:open><Icon name="chevron" size={12} /></span>
  {:else}
    <span class="twist"></span>
  {/if}
  <span class="ic ic-{node.kind}"><Icon name={iconFor(node)} size={15} /></span>
  <span class="label">{node.name}</span>
  {#if node.kind !== "dir" && isDirty(node.path)}
    <span class="dirty" title="Unsaved"></span>
  {/if}
  {#if node.badge}<span class="badge">{node.badge}</span>{/if}
</div>

{#if node.kind === "dir" && open && node.children}
  {#each node.children as child (child.path)}
    <TreeNode node={child} depth={depth + 1} />
  {/each}
{/if}

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 24px;
    padding-right: 8px;
    cursor: pointer;
    color: var(--fg-1);
    user-select: none;
    white-space: nowrap;
  }
  .row:hover {
    background: var(--bg-4);
  }
  .row.active {
    background: var(--accent-soft);
    color: var(--fg-0);
    box-shadow: inset 2px 0 0 var(--accent);
  }
  .twist {
    width: 12px;
    display: inline-flex;
    color: var(--fg-3);
    transition: transform 0.12s ease;
  }
  .twist.open {
    transform: rotate(90deg);
  }
  .ic {
    display: inline-flex;
    color: var(--fg-2);
  }
  .ic-typ {
    color: var(--accent);
  }
  .ic-bib {
    color: var(--accent-2);
  }
  .ic-dir {
    color: var(--syn-heading);
  }
  .label {
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 12.5px;
  }
  .dirty {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--fg-2);
    margin-left: auto;
    flex: none;
  }
  .badge {
    margin-left: auto;
    font-size: 9.5px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 1px 5px;
    border-radius: 4px;
    background: var(--bg-4);
    color: var(--fg-2);
    border: 1px solid var(--border);
    flex: none;
  }
</style>
