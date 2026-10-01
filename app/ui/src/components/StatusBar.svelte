<script lang="ts">
  import Icon from "./Icon.svelte";
  import { ui, data } from "../lib/store.svelte";

  const errors = $derived(data.diagnostics.filter((d) => d.severity === "error").length);
  const warns = $derived(data.diagnostics.filter((d) => d.severity === "warning").length);
</script>

<footer class="status">
  <div class="left">
    <button class="seg"><Icon name="git" size={13} /> {data.project?.branch ?? "—"}</button>
    <button class="seg" onclick={() => (ui.bottomVisible = !ui.bottomVisible)}>
      <span class="err" class:zero={errors === 0}><Icon name="warn" size={12} /> {errors}</span>
      <span class="wrn" class:zero={warns === 0}><Icon name="warn" size={12} /> {warns}</span>
    </button>
    {#if data.project}
      <span class="seg dim">{data.project.kind}</span>
    {/if}
  </div>

  <div class="right">
    <span class="seg dim mono">Ln {ui.cursor.line}, Col {ui.cursor.col}</span>
    <span class="seg dim mono">UTF-8</span>
    <span class="seg dim mono">Typst {data.project?.typst ?? "—"}</span>
    <span class="seg dim">Tinymist {data.project?.tinymist ?? "—"}</span>
    <button class="seg net {ui.networkMode}">
      <Icon name={ui.networkMode === "offline" ? "wifi-off" : "wifi"} size={13} />
      {ui.networkMode}
    </button>
  </div>
</footer>

<style>
  .status {
    height: 24px;
    flex: none;
    background: var(--bg-0);
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 4px;
    font-size: 11px;
    color: var(--fg-2);
  }
  .left,
  .right {
    display: flex;
    align-items: center;
  }
  .seg {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 0 8px;
    height: 24px;
    color: var(--fg-2);
    border-radius: 3px;
  }
  button.seg:hover {
    background: var(--bg-3);
    color: var(--fg-0);
  }
  .dim {
    color: var(--fg-3);
  }
  .err {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    color: var(--error);
  }
  .wrn {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    color: var(--warn);
  }
  .err.zero,
  .wrn.zero {
    color: var(--fg-3);
  }
  .net {
    font-weight: 600;
    text-transform: capitalize;
  }
  .net.offline {
    color: var(--fg-2);
  }
  .net.ask {
    color: var(--warn);
  }
  .net.online {
    color: var(--ok);
  }
</style>
