<script lang="ts">
  import Icon from "./Icon.svelte";
  import { data } from "../lib/store.svelte";
  import type { StructureNode } from "../lib/types";

  const kindIcon: Record<string, string> = {
    heading: "structure",
    figure: "layers",
    table: "panel",
    equation: "bolt",
    label: "link",
    rule: "settings",
  };
  const kindColor: Record<string, string> = {
    heading: "var(--syn-heading)",
    figure: "var(--accent)",
    table: "var(--accent-2)",
    equation: "var(--syn-math)",
    label: "var(--syn-label)",
    rule: "var(--fg-2)",
  };
</script>

<div class="list">
  {#if data.structure.length === 0}
    <div class="snone">No headings in this file yet.</div>
  {/if}
  {#each data.structure as node, i (node.label + ":" + node.line + ":" + i)}
    {@render item(node, 0)}
  {/each}
</div>

{#snippet item(n: StructureNode, depth: number)}
  <div class="row" style="padding-left: {10 + depth * 14}px" title="line {n.line}">
    <span class="ic" style="color: {kindColor[n.kind]}">
      <Icon name={kindIcon[n.kind]} size={14} />
    </span>
    <span class="label" class:head={n.kind === "heading"}>{n.label}</span>
    <span class="ln">{n.line}</span>
  </div>
  {#if n.children}
    {#each n.children as c, i (c.label + ":" + c.line + ":" + i)}
      {@render item(c, depth + 1)}
    {/each}
  {/if}
{/snippet}

<style>
  .list {
    padding: 4px 0;
  }
  .snone {
    padding: 16px 14px;
    font-size: 12px;
    color: var(--fg-3);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 7px;
    height: 25px;
    padding-right: 10px;
    cursor: pointer;
    color: var(--fg-1);
  }
  .row:hover {
    background: var(--bg-4);
  }
  .ic {
    display: inline-flex;
    flex: none;
  }
  .label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12.5px;
  }
  .label.head {
    font-weight: 600;
    color: var(--fg-0);
  }
  .ln {
    margin-left: auto;
    font-size: 10.5px;
    color: var(--fg-3);
    font-family: "JetBrains Mono", monospace;
  }
</style>
