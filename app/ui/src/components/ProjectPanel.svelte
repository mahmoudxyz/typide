<script lang="ts">
  import TreeNode from "./TreeNode.svelte";
  import { data } from "../lib/store.svelte";
</script>

<div class="tree" role="tree">
  {#each data.tree?.children ?? [] as node (node.path)}
    <TreeNode {node} depth={0} />
  {/each}
</div>

<style>
  .tree {
    padding: 4px 0;
  }
</style>
