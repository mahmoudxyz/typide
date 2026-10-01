<script lang="ts">
  import { ui } from "../lib/store.svelte";

  let value = $state("");
  let danger = $derived(ui.prompt?.confirmText === "Delete");

  // Seed the field each time a new prompt opens, and select the name stem.
  let lastRef: unknown = null;
  $effect(() => {
    if (ui.prompt && ui.prompt !== lastRef) {
      lastRef = ui.prompt;
      value = ui.prompt.value;
    }
  });

  function close() {
    ui.prompt = null;
    value = "";
  }
  function confirm() {
    const p = ui.prompt;
    if (!p || !value.trim()) return;
    const cb = p.onConfirm;
    close();
    cb(value);
  }
  function onkey(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      confirm();
    } else if (e.key === "Escape") {
      close();
    }
  }
  function autofocus(el: HTMLInputElement) {
    el.focus();
    // Select the file-name stem (before the extension) for quick renaming.
    const dot = el.value.lastIndexOf(".");
    el.setSelectionRange(0, dot > 0 ? dot : el.value.length);
  }
</script>

{#if ui.prompt}
  <div class="overlay" role="presentation" onclick={close}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="dlg" role="dialog" aria-label={ui.prompt.title} aria-modal="true" tabindex="-1" onclick={(e) => e.stopPropagation()}>
      <div class="title">{ui.prompt.title}</div>
      <label class="lbl" for="prompt-input">{ui.prompt.label}</label>
      <input id="prompt-input" use:autofocus bind:value spellcheck="false" onkeydown={onkey} />
      <div class="btns">
        <button class="cancel" onclick={close}>Cancel</button>
        <button class="ok" class:danger disabled={!value.trim()} onclick={confirm}>
          {ui.prompt.confirmText}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(2px);
    z-index: 720;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .dlg {
    width: 400px;
    max-width: 92vw;
    background: var(--bg-3);
    border: 1px solid var(--border-strong);
    border-radius: 13px;
    box-shadow: var(--shadow);
    padding: 18px;
    animation: pop 0.14s ease;
  }
  @keyframes pop {
    from {
      transform: scale(0.97);
      opacity: 0;
    }
  }
  .title {
    font-size: 15px;
    font-weight: 700;
  }
  .lbl {
    display: block;
    font-size: 11.5px;
    color: var(--fg-2);
    margin: 14px 0 6px;
  }
  input {
    width: 100%;
    height: 36px;
    padding: 0 12px;
    background: var(--bg-2);
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    font-size: 13px;
    color: var(--fg-0);
    outline: none;
    font-family: "JetBrains Mono", monospace;
  }
  input:focus {
    border-color: var(--accent);
  }
  .btns {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    margin-top: 16px;
  }
  button {
    height: 34px;
    padding: 0 15px;
    border-radius: 8px;
    font-size: 12.5px;
    font-weight: 600;
  }
  .cancel {
    background: var(--bg-2);
    color: var(--fg-1);
    border: 1px solid var(--border-strong);
  }
  .cancel:hover {
    background: var(--bg-4);
  }
  .ok {
    background: var(--accent);
    color: var(--accent-fg);
    border: none;
  }
  .ok:hover:not(:disabled) {
    filter: brightness(1.08);
  }
  .ok.danger {
    background: var(--error);
    color: #fff;
  }
  .ok:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
