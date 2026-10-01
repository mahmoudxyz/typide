<script lang="ts">
  import Icon from "./Icon.svelte";
  import Terminal from "./Terminal.svelte";
  import { ui, data, openFile, relPath, cancelJob, type BottomTab } from "../lib/store.svelte";

  const errors = $derived(data.diagnostics.filter((d) => d.severity === "error").length);
  const warns = $derived(data.diagnostics.filter((d) => d.severity === "warning").length);

  const sevIcon: Record<string, string> = { error: "warn", warning: "warn", hint: "info" };
  const sevColor: Record<string, string> = {
    error: "var(--error)",
    warning: "var(--warn)",
    hint: "var(--hint)",
  };

  const tabs: { id: BottomTab; label: string; icon: string }[] = [
    { id: "problems", label: "Problems", icon: "alert" },
    { id: "jobs", label: "Jobs", icon: "layers" },
    { id: "terminal", label: "Terminal", icon: "terminal" },
  ];
</script>

<section class="bottom" style="height: {ui.bottomHeight}px">
  <header>
    <div class="tabs">
      {#each tabs as t (t.id)}
        <button class="tab" class:active={ui.bottomTab === t.id} onclick={() => (ui.bottomTab = t.id)}>
          <Icon name={t.icon} size={14} />
          {t.label}
          {#if t.id === "problems" && data.diagnostics.length}
            <span class="count">{data.diagnostics.length}</span>
          {/if}
        </button>
      {/each}
    </div>
    <button class="hide" onclick={() => (ui.bottomVisible = false)} title="Hide panel">
      <Icon name="x" size={14} />
    </button>
  </header>

  <div class="body">
    {#if ui.bottomTab === "problems"}
      <div class="summary">
        <span style="color: var(--error)"><Icon name="warn" size={13} /> {errors} errors</span>
        <span style="color: var(--warn)"><Icon name="warn" size={13} /> {warns} warnings</span>
        <span class="prof">profile: thesis-submission</span>
      </div>
      {#if data.diagnostics.length === 0}
        <div class="clean"><Icon name="check-circle" size={16} /> No problems detected in this project.</div>
      {/if}
      {#each data.diagnostics as d, i (d.id + d.file + d.line + ":" + d.col + ":" + i)}
        <div class="prob" onclick={() => openFile(d.file)} role="button" tabindex="0" onkeydown={() => {}}>
          <span class="sev" style="color: {sevColor[d.severity]}"><Icon name={sevIcon[d.severity]} size={15} /></span>
          <div class="pbody">
            <div class="msg">{d.message}</div>
            <div class="meta">
              <span class="loc mono">{relPath(d.file)}:{d.line}:{d.col}</span>
              <span class="rule mono">{d.id}</span>
              <span class="src">{d.source}</span>
              {#if d.quickFix}
                <button class="qf" onclick={(e) => e.stopPropagation()}>
                  <Icon name="bolt" size={11} /> {d.quickFix}
                </button>
              {/if}
            </div>
          </div>
        </div>
      {/each}
    {:else if ui.bottomTab === "jobs"}
      {#if data.jobs.length === 0}
        <div class="clean"><Icon name="check-circle" size={16} /> No background jobs running.</div>
      {/if}
      {#each data.jobs as j (j.id)}
        <div class="job">
          <span class="jstate {j.state}">
            {#if j.state === "running"}<span class="spin"></span>
            {:else if j.state === "done"}<Icon name="check-circle" size={15} />
            {:else}<Icon name="warn" size={15} />{/if}
          </span>
          <div class="jbody">
            <div class="jrow">
              <span class="jtitle">{j.title}</span>
              <span class="jmsg">{j.message}</span>
            </div>
            {#if j.state === "running"}
              <div class="bar"><div class="fill" style="width: {j.fraction * 100}%"></div></div>
            {/if}
          </div>
          {#if j.state === "running"}
            <button class="cancel" title="Cancel" onclick={() => cancelJob(j.id)}><Icon name="x" size={13} /></button>
          {/if}
        </div>
      {/each}
    {:else}
      <Terminal />
    {/if}
  </div>
</section>

<style>
  .bottom {
    height: 220px;
    flex: none;
    background: var(--bg-1);
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  header {
    height: 34px;
    display: flex;
    align-items: center;
    padding-right: 8px;
    border-bottom: 1px solid var(--border);
    flex: none;
  }
  .tabs {
    display: flex;
    height: 100%;
  }
  .tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 14px;
    font-size: 12px;
    color: var(--fg-2);
    position: relative;
  }
  .tab:hover {
    color: var(--fg-0);
  }
  .tab.active {
    color: var(--fg-0);
  }
  .tab.active::after {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 2px;
    background: var(--accent);
  }
  .count {
    font-size: 10px;
    background: var(--bg-4);
    color: var(--fg-1);
    padding: 0 5px;
    border-radius: 8px;
    min-width: 16px;
    text-align: center;
  }
  .hide {
    margin-left: auto;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--fg-3);
    border-radius: 4px;
  }
  .hide:hover {
    background: var(--bg-4);
    color: var(--fg-0);
  }
  .body {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }
  .summary {
    display: flex;
    gap: 16px;
    align-items: center;
    padding: 6px 14px;
    font-size: 11.5px;
    border-bottom: 1px solid var(--border);
  }
  .summary span {
    display: flex;
    align-items: center;
    gap: 5px;
  }
  .prof {
    margin-left: auto;
    color: var(--fg-3);
    font-family: "JetBrains Mono", monospace;
    font-size: 10.5px;
  }
  .clean {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 16px 14px;
    color: var(--ok);
    font-size: 12.5px;
  }
  .prob {
    display: flex;
    gap: 9px;
    padding: 8px 14px;
    cursor: pointer;
    border-bottom: 1px solid color-mix(in srgb, var(--border) 55%, transparent);
  }
  .prob:hover {
    background: var(--bg-3);
  }
  .sev {
    flex: none;
    margin-top: 1px;
  }
  .pbody {
    min-width: 0;
  }
  .msg {
    font-size: 12.5px;
    color: var(--fg-0);
    line-height: 1.35;
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 4px;
    font-size: 11px;
    color: var(--fg-3);
    flex-wrap: wrap;
  }
  .loc {
    color: var(--accent);
  }
  .rule {
    color: var(--fg-2);
    background: var(--bg-3);
    padding: 0 5px;
    border-radius: 3px;
  }
  .qf {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 10.5px;
    padding: 2px 8px;
    border-radius: 4px;
    border: 1px solid color-mix(in srgb, var(--accent) 35%, transparent);
    color: var(--accent);
    background: var(--accent-soft);
  }
  .qf:hover {
    background: color-mix(in srgb, var(--accent) 20%, transparent);
  }
  .job {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 14px;
  }
  .jstate {
    flex: none;
    display: flex;
  }
  .jstate.done {
    color: var(--ok);
  }
  .jstate.running {
    color: var(--accent);
  }
  .spin {
    width: 14px;
    height: 14px;
    border: 2px solid var(--border-strong);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    display: inline-block;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .jbody {
    flex: 1;
    min-width: 0;
  }
  .jrow {
    display: flex;
    justify-content: space-between;
    gap: 10px;
  }
  .jtitle {
    font-size: 12.5px;
    color: var(--fg-0);
  }
  .jmsg {
    font-size: 11px;
    color: var(--fg-3);
  }
  .bar {
    height: 4px;
    background: var(--bg-4);
    border-radius: 3px;
    margin-top: 6px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: linear-gradient(90deg, var(--accent), var(--accent-2));
    border-radius: 3px;
    transition: width 0.3s;
  }
  .cancel {
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--fg-3);
    border-radius: 4px;
  }
  .cancel:hover {
    background: var(--error-soft);
    color: var(--error);
  }
</style>
