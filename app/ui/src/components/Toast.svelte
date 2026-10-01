<script lang="ts">
  import Icon from "./Icon.svelte";
  import { ui } from "../lib/store.svelte";
</script>

{#if ui.toast}
  <div class="toast {ui.toast.kind}" role="status">
    <Icon name={ui.toast.kind === "ok" ? "check-circle" : "warn"} size={16} />
    <span>{ui.toast.message}</span>
    <button class="x" onclick={() => (ui.toast = null)}><Icon name="x" size={13} /></button>
  </div>
{/if}

<style>
  .toast {
    position: fixed;
    bottom: 40px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 500;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 11px 14px;
    border-radius: 10px;
    background: var(--bg-3);
    border: 1px solid var(--border-strong);
    box-shadow: var(--shadow);
    font-size: 13px;
    color: var(--fg-0);
    max-width: 80vw;
    animation: rise 0.16s ease;
  }
  @keyframes rise {
    from {
      transform: translate(-50%, 8px);
      opacity: 0;
    }
  }
  .toast.ok :global(svg:first-child) {
    color: var(--ok);
  }
  .toast.error :global(svg:first-child) {
    color: var(--error);
  }
  .toast span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .x {
    display: inline-flex;
    color: var(--fg-3);
    border-radius: 4px;
    padding: 2px;
  }
  .x:hover {
    color: var(--fg-0);
    background: var(--bg-4);
  }
</style>
