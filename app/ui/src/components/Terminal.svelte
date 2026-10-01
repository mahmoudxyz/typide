<script lang="ts">
  import { onMount, onDestroy, tick } from "svelte";
  import { data } from "../lib/store.svelte";
  import { api, isTauri } from "../lib/api";

  type Line = { text: string; kind: "in" | "out" | "err" | "sys" };
  let lines = $state<Line[]>([]);
  let input = $state("");
  let running = $state(false);
  let host = $state<HTMLElement | null>(null);
  const history: string[] = [];
  let histIdx = -1;
  let unlisten: Array<() => void> = [];

  const cwd = $derived(data.project?.root ?? "~");

  onMount(async () => {
    if (!isTauri()) {
      lines = [{ text: "Terminal is available in the desktop app.", kind: "sys" }];
      return;
    }
    lines = [{ text: `Typide terminal — ${cwd}`, kind: "sys" }];
    unlisten.push(
      await api.onTerminalOutput((stream, line) => push(line, stream === "err" ? "err" : "out"))
    );
    unlisten.push(
      await api.onTerminalExit((code) => {
        if (code !== 0) push(`exited with code ${code}`, "sys");
        running = false;
      })
    );
  });
  onDestroy(() => unlisten.forEach((u) => u()));

  async function push(text: string, kind: Line["kind"]) {
    lines = [...lines, { text, kind }];
    await tick();
    if (host) host.scrollTop = host.scrollHeight;
  }

  async function submit() {
    const line = input.trim();
    if (!line || running) return;
    if (line === "clear" || line === "cls") {
      lines = [];
      input = "";
      return;
    }
    history.unshift(line);
    histIdx = -1;
    await push(`$ ${line}`, "in");
    input = "";
    running = true;
    try {
      await api.terminalExec(cwd, line);
    } catch (e) {
      push(String((e as any)?.message ?? e), "err");
      running = false;
    }
  }

  function key(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      submit();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (histIdx < history.length - 1) input = history[++histIdx] ?? input;
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      if (histIdx > 0) input = history[--histIdx] ?? "";
      else {
        histIdx = -1;
        input = "";
      }
    }
  }
</script>

<div class="term mono" bind:this={host}>
  {#each lines as l, i (i)}
    <div class="ln {l.kind}">{l.text || " "}</div>
  {/each}
  <div class="prompt">
    <span class="ps">$</span>
    <!-- svelte-ignore a11y_autofocus -->
    <input
      bind:value={input}
      onkeydown={key}
      placeholder={running ? "running…" : "run a command — git status, typst --version, ls…"}
      spellcheck="false"
      autocomplete="off"
    />
    {#if running}<span class="spin"></span>{/if}
  </div>
</div>

<style>
  .term {
    height: 100%;
    overflow-y: auto;
    padding: 8px 10px;
    font-size: 12px;
    line-height: 1.55;
    background: var(--bg-0);
    color: var(--fg-1);
  }
  .ln {
    white-space: pre-wrap;
    word-break: break-word;
  }
  .ln.in {
    color: var(--fg-0);
    font-weight: 600;
  }
  .ln.err {
    color: var(--error);
  }
  .ln.sys {
    color: var(--fg-3);
    font-style: italic;
  }
  .prompt {
    display: flex;
    align-items: center;
    gap: 7px;
    margin-top: 2px;
  }
  .ps {
    color: var(--accent);
    font-weight: 700;
  }
  .prompt input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    color: var(--fg-0);
    font-family: inherit;
    font-size: 12px;
  }
  .spin {
    width: 11px;
    height: 11px;
    border: 2px solid var(--border-strong);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    flex: none;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
