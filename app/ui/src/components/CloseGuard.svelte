<script lang="ts">
  import Icon from "./Icon.svelte";
  import { dirtyCount, saveAndQuit, discardAndQuit, cancelQuit } from "../lib/store.svelte";

  const n = $derived(dirtyCount());
</script>

<div class="overlay" role="presentation">
  <div class="dlg" role="dialog" aria-label="Unsaved changes" aria-modal="true">
    <div class="top">
      <span class="ic"><Icon name="warn" size={20} /></span>
      <div>
        <div class="title">Save changes before quitting?</div>
        <div class="sub">
          You have <b>{n}</b> unsaved file{n === 1 ? "" : "s"}. Unsaved work will be lost if you quit
          without saving.
        </div>
      </div>
    </div>
    <div class="btns">
      <button class="discard" onclick={discardAndQuit}>Discard &amp; quit</button>
      <button class="cancel" onclick={cancelQuit}>Cancel</button>
      <button class="save" onclick={saveAndQuit}>Save &amp; quit</button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(2px);
    z-index: 700;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .dlg {
    width: 440px;
    max-width: 92vw;
    background: var(--bg-3);
    border: 1px solid var(--border-strong);
    border-radius: 13px;
    box-shadow: var(--shadow);
    padding: 20px;
    animation: pop 0.14s ease;
  }
  @keyframes pop {
    from {
      transform: scale(0.97);
      opacity: 0;
    }
  }
  .top {
    display: flex;
    gap: 13px;
    align-items: flex-start;
  }
  .ic {
    width: 40px;
    height: 40px;
    flex: none;
    border-radius: 11px;
    background: var(--warn-soft, var(--accent-soft));
    color: var(--warn, var(--accent));
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .title {
    font-size: 16px;
    font-weight: 700;
  }
  .sub {
    font-size: 13px;
    color: var(--fg-1);
    margin-top: 3px;
    line-height: 1.45;
  }
  .btns {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    margin-top: 20px;
  }
  button {
    height: 36px;
    padding: 0 15px;
    border-radius: 8px;
    font-size: 12.5px;
    font-weight: 600;
  }
  .discard {
    margin-right: auto;
    background: transparent;
    color: var(--error);
    border: 1px solid color-mix(in srgb, var(--error) 40%, transparent);
  }
  .discard:hover {
    background: color-mix(in srgb, var(--error) 14%, transparent);
  }
  .cancel {
    background: var(--bg-2);
    color: var(--fg-1);
    border: 1px solid var(--border-strong);
  }
  .cancel:hover {
    background: var(--bg-4);
  }
  .save {
    background: var(--accent);
    color: var(--accent-fg);
    border: none;
  }
  .save:hover {
    filter: brightness(1.08);
  }
</style>
