<script lang="ts">
  import Icon from "./Icon.svelte";
  import { ui, restoreRecovery, discardRecovery, relPath } from "../lib/store.svelte";

  const drafts = $derived(ui.recovery ?? []);
</script>

<div class="banner" role="status">
  <span class="ic"><Icon name="history" size={16} /></span>
  <div class="msg">
    <div class="ttl">Unsaved changes recovered</div>
    <div class="sub">
      {drafts.length} file{drafts.length === 1 ? "" : "s"} from a previous session:
      <span class="files mono">{drafts.map((d) => relPath(d.path)).join(", ")}</span>
    </div>
  </div>
  <div class="btns">
    <button class="discard" onclick={discardRecovery}>Discard</button>
    <button class="restore" onclick={restoreRecovery}>Restore</button>
  </div>
</div>

<style>
  .banner {
    position: fixed;
    top: 44px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 650;
    display: flex;
    align-items: center;
    gap: 12px;
    max-width: 92vw;
    padding: 11px 14px;
    background: var(--bg-3);
    border: 1px solid var(--border-strong);
    border-radius: 11px;
    box-shadow: var(--shadow);
    animation: drop 0.16s ease;
  }
  @keyframes drop {
    from {
      transform: translate(-50%, -8px);
      opacity: 0;
    }
  }
  .ic {
    width: 32px;
    height: 32px;
    flex: none;
    border-radius: 9px;
    background: var(--accent-soft);
    color: var(--accent);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .msg {
    min-width: 0;
  }
  .ttl {
    font-size: 13px;
    font-weight: 700;
  }
  .sub {
    font-size: 11.5px;
    color: var(--fg-2);
    margin-top: 2px;
    max-width: 440px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .files {
    color: var(--fg-1);
  }
  .btns {
    display: flex;
    gap: 7px;
    flex: none;
  }
  button {
    height: 32px;
    padding: 0 13px;
    border-radius: 8px;
    font-size: 12px;
    font-weight: 600;
  }
  .discard {
    background: var(--bg-2);
    color: var(--fg-2);
    border: 1px solid var(--border-strong);
  }
  .discard:hover {
    background: var(--bg-4);
    color: var(--fg-0);
  }
  .restore {
    background: var(--accent);
    color: var(--accent-fg);
    border: none;
  }
  .restore:hover {
    filter: brightness(1.08);
  }
</style>
