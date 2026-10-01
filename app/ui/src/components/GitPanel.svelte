<script lang="ts">
  import Icon from "./Icon.svelte";
  import { ui, data, createSnapshot, restoreSnapshot, setRemote, syncRemote } from "../lib/store.svelte";

  let message = $state("");
  let busy = $state(false);
  let remoteUrl = $state("");

  async function snap() {
    busy = true;
    await createSnapshot(message);
    message = "";
    busy = false;
  }
</script>

<div class="wrap">
  <div class="snapbox">
    <input
      placeholder="Snapshot message (optional)…"
      bind:value={message}
      onkeydown={(e) => e.key === "Enter" && snap()}
    />
    <button class="btn primary" onclick={snap} disabled={busy}>
      <Icon name="history" size={14} /> Snapshot
    </button>
  </div>

  <div class="section-title">Remote</div>
  {#if data.remote}
    <div class="remote">
      <span class="url mono" title={data.remote}>{data.remote}</span>
      <button class="btn" onclick={syncRemote} disabled={ui.compiling}>
        <Icon name="refresh" size={14} /> Sync
      </button>
    </div>
  {:else}
    <div class="snapbox">
      <input
        placeholder="git remote URL (https or ssh)…"
        bind:value={remoteUrl}
        onkeydown={(e) => e.key === 'Enter' && setRemote(remoteUrl)}
      />
      <button class="btn" onclick={() => setRemote(remoteUrl)}>
        <Icon name="git" size={14} /> Add remote
      </button>
    </div>
  {/if}

  <div class="section-title">Timeline</div>
  {#if data.snapshots.length === 0}
    <div class="gnone"><Icon name="history" size={24} /><p>No snapshots yet</p><span>Create a snapshot to capture the current state.</span></div>
  {/if}
  <div class="timeline">
    {#each data.snapshots as s, i (s.id)}
      <div class="node">
        <div class="rail">
          <span class="dot" class:auto={s.auto}></span>
          {#if i < data.snapshots.length - 1}<span class="line"></span>{/if}
        </div>
        <div class="body">
          <div class="msg">{s.message}</div>
          <div class="meta">
            <span class="hash">{s.id}</span>
            <span>{s.when}</span>
            {#if s.auto}<span class="tag">auto</span>{/if}
            {#if i > 0}
              <button class="restore" onclick={() => restoreSnapshot(s.id)} title="Restore files to this snapshot">
                <Icon name="history" size={11} /> Restore
              </button>
            {/if}
          </div>
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .wrap {
    padding: 8px;
  }
  .snapbox {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 14px;
  }
  .snapbox input {
    height: 30px;
    padding: 0 10px;
    background: var(--bg-inset);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--fg-0);
    font-size: 12.5px;
    outline: none;
  }
  .snapbox input:focus {
    border-color: var(--accent);
  }
  .remote {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 14px;
  }
  .remote .url {
    flex: 1;
    font-size: 11px;
    color: var(--fg-2);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 6px 9px;
    background: var(--bg-inset);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .remote .btn {
    flex: none;
    width: auto;
    padding: 0 12px;
  }
  .restore {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 10px;
    color: var(--fg-2);
    padding: 2px 7px;
    border-radius: 4px;
    border: 1px solid var(--border-strong);
    background: var(--bg-2);
    margin-left: auto;
  }
  .restore:hover {
    background: var(--bg-4);
    color: var(--fg-0);
  }
  .btn {
    flex: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 30px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg-2);
    color: var(--fg-1);
    font-size: 12px;
    font-weight: 500;
  }
  .btn:hover {
    background: var(--bg-4);
  }
  .btn.primary {
    background: var(--accent);
    color: var(--accent-fg);
    border-color: transparent;
  }
  .btn.primary:hover {
    filter: brightness(1.08);
  }
  .section-title {
    font-size: 10.5px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--fg-3);
    margin: 4px 4px 8px;
    font-weight: 600;
  }
  .gnone {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 5px;
    padding: 24px 16px;
    color: var(--fg-3);
  }
  .gnone p {
    font-size: 13px;
    color: var(--fg-1);
  }
  .gnone span {
    font-size: 11.5px;
  }
  .node {
    display: flex;
    gap: 10px;
  }
  .rail {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 14px;
    padding-top: 3px;
  }
  .dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--accent);
    border: 2px solid var(--bg-1);
    box-shadow: 0 0 0 1px var(--accent);
    flex: none;
  }
  .dot.auto {
    background: var(--bg-1);
    box-shadow: 0 0 0 1px var(--fg-3);
  }
  .line {
    flex: 1;
    width: 2px;
    background: var(--border);
    margin: 2px 0;
    min-height: 18px;
  }
  .body {
    padding-bottom: 14px;
    min-width: 0;
  }
  .msg {
    font-size: 12.5px;
    color: var(--fg-0);
    line-height: 1.35;
  }
  .meta {
    display: flex;
    gap: 8px;
    align-items: center;
    margin-top: 3px;
    font-size: 11px;
    color: var(--fg-2);
  }
  .hash {
    font-family: "JetBrains Mono", monospace;
    color: var(--accent);
  }
  .tag {
    font-size: 9.5px;
    text-transform: uppercase;
    padding: 0 5px;
    border-radius: 4px;
    background: var(--bg-4);
    color: var(--fg-3);
  }
</style>
