<script lang="ts">
  import Icon from "./Icon.svelte";
  import { ui, resolveConsent } from "../lib/store.svelte";
</script>

{#if ui.consent}
  <div class="overlay" role="presentation">
    <div class="dlg" role="dialog" aria-label="Network permission" aria-modal="true">
      <div class="top">
        <span class="ic"><Icon name="wifi" size={20} /></span>
        <div>
          <div class="title">Allow network access?</div>
          <div class="sub">Typide wants to <b>{ui.consent.reason}</b>.</div>
        </div>
      </div>
      <div class="host">
        <Icon name="compass" size={13} /> <span class="mono">{ui.consent.host}</span>
      </div>
      <p class="note">
        You chose <b>Ask each time</b>. Nothing is uploaded — only this download is requested.
      </p>
      <div class="btns">
        <button class="deny" onclick={() => resolveConsent("deny")}>Deny</button>
        <button class="once" onclick={() => resolveConsent("once")}>Allow once</button>
        <button class="always" onclick={() => resolveConsent("always")}>Allow this session</button>
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
    z-index: 600;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .dlg {
    width: 420px;
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
    background: var(--accent-soft);
    color: var(--accent);
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
    line-height: 1.4;
  }
  .host {
    display: flex;
    align-items: center;
    gap: 7px;
    margin: 14px 0 0;
    padding: 9px 12px;
    background: var(--bg-inset);
    border: 1px solid var(--border);
    border-radius: 8px;
    font-size: 12.5px;
    color: var(--fg-2);
  }
  .note {
    font-size: 11.5px;
    color: var(--fg-3);
    margin: 12px 0 16px;
    line-height: 1.5;
  }
  .btns {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
  button {
    height: 36px;
    padding: 0 15px;
    border-radius: 8px;
    font-size: 12.5px;
    font-weight: 600;
  }
  .deny {
    margin-right: auto;
    background: transparent;
    color: var(--fg-2);
    border: 1px solid var(--border-strong);
  }
  .deny:hover {
    background: var(--bg-4);
    color: var(--fg-0);
  }
  .once {
    background: var(--bg-2);
    color: var(--fg-1);
    border: 1px solid var(--border-strong);
  }
  .once:hover {
    background: var(--bg-4);
  }
  .always {
    background: var(--accent);
    color: var(--accent-fg);
    border: none;
  }
  .always:hover {
    filter: brightness(1.08);
  }
</style>
