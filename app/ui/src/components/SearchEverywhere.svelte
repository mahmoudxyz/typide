<script lang="ts">
  import Icon from "./Icon.svelte";
  import { ui, data, openFile, relPath } from "../lib/store.svelte";

  interface Hit {
    kind: "file" | "symbol" | "label" | "citation";
    title: string;
    sub: string;
    icon: string;
    path?: string;
  }

  function collect(): Hit[] {
    const out: Hit[] = [];
    const walk = (n: any) => {
      if (!n) return;
      if (n.kind !== "dir") {
        out.push({ kind: "file", title: n.name, sub: relPath(n.path), icon: "file-code", path: n.path });
      }
      n.children?.forEach(walk);
    };
    if (data.tree) walk(data.tree);
    data.structure.forEach((s) => {
      if (s.kind === "heading")
        out.push({ kind: "symbol", title: s.label, sub: `heading · line ${s.line}`, icon: "structure" });
      s.children?.forEach((c) => {
        if (c.kind === "label")
          out.push({ kind: "label", title: c.label, sub: `label · line ${c.line}`, icon: "link" });
      });
    });
    data.citations.forEach((c) =>
      out.push({ kind: "citation", title: c.key, sub: `${c.authors} (${c.year})`, icon: "quote" })
    );
    return out;
  }
  const all = collect();

  let query = $state("");
  let sel = $state(0);
  const filtered = $derived(
    query
      ? all.filter(
          (h) =>
            h.title.toLowerCase().includes(query.toLowerCase()) ||
            h.sub.toLowerCase().includes(query.toLowerCase())
        )
      : all
  );

  const kindLabel: Record<string, string> = {
    file: "File",
    symbol: "Symbol",
    label: "Label",
    citation: "Citation",
  };
  const kindColor: Record<string, string> = {
    file: "var(--accent)",
    symbol: "var(--syn-heading)",
    label: "var(--syn-label)",
    citation: "var(--accent-2)",
  };

  function close() {
    ui.searchEverywhere = false;
    query = "";
    sel = 0;
  }
  function pick(h: Hit) {
    if (h.path) openFile(h.path);
    close();
  }
  function keydown(e: KeyboardEvent) {
    if (e.key === "Escape") close();
    else if (e.key === "ArrowDown") {
      sel = Math.min(filtered.length - 1, sel + 1);
      e.preventDefault();
    } else if (e.key === "ArrowUp") {
      sel = Math.max(0, sel - 1);
      e.preventDefault();
    } else if (e.key === "Enter" && filtered[sel]) pick(filtered[sel]);
  }
  $effect(() => {
    query;
    sel = 0;
  });
</script>

<div class="overlay" onclick={close} role="presentation">
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_interactive_supports_focus -->
  <div class="se" onclick={(e) => e.stopPropagation()} role="dialog" aria-label="Search everywhere" tabindex="-1">
    <div class="input">
      <Icon name="compass" size={18} />
      <!-- svelte-ignore a11y_autofocus -->
      <input autofocus bind:value={query} onkeydown={keydown} placeholder="Search files, symbols, labels, citations…" />
      <span class="scope">All</span>
    </div>
    <div class="list">
      {#each filtered as h, i (h.kind + ":" + h.title + ":" + i)}
        <button class="row" class:sel={i === sel} onclick={() => pick(h)} onmousemove={() => (sel = i)}>
          <span class="ic" style="color: {kindColor[h.kind]}"><Icon name={h.icon} size={15} /></span>
          <span class="t mono">{h.title}</span>
          <span class="sub">{h.sub}</span>
          <span class="kind" style="color: {kindColor[h.kind]}">{kindLabel[h.kind]}</span>
        </button>
      {/each}
      {#if filtered.length === 0}<div class="none">Nothing found</div>{/if}
    </div>
    <footer>
      <span><kbd>↑</kbd><kbd>↓</kbd> navigate</span>
      <span><kbd>↵</kbd> open</span>
      <span><kbd>Esc</kbd> close</span>
      <span class="idx">indexed with typst-syntax + FTS5</span>
    </footer>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(2px);
    z-index: 200;
    display: flex;
    justify-content: center;
    padding-top: 10vh;
  }
  .se {
    width: 700px;
    max-width: 94vw;
    max-height: 66vh;
    background: var(--bg-3);
    border: 1px solid var(--border-strong);
    border-radius: 12px;
    box-shadow: var(--shadow);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: pop 0.12s ease;
  }
  @keyframes pop {
    from {
      transform: translateY(-6px);
      opacity: 0;
    }
  }
  .input {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 0 16px;
    height: 54px;
    border-bottom: 1px solid var(--border);
    color: var(--fg-3);
  }
  .input input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    font-size: 16px;
    color: var(--fg-0);
  }
  .scope {
    font-size: 11px;
    color: var(--accent);
    background: var(--accent-soft);
    padding: 2px 9px;
    border-radius: 5px;
    font-weight: 600;
  }
  .list {
    overflow-y: auto;
    padding: 6px;
    flex: 1;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 11px;
    width: 100%;
    padding: 8px 10px;
    border-radius: 7px;
    text-align: left;
  }
  .row.sel {
    background: var(--accent-soft);
  }
  .ic {
    display: inline-flex;
    flex: none;
  }
  .t {
    font-size: 13px;
    color: var(--fg-0);
    flex: none;
    max-width: 40%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sub {
    font-size: 11.5px;
    color: var(--fg-3);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .kind {
    margin-left: auto;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 700;
    flex: none;
  }
  .none {
    padding: 24px;
    text-align: center;
    color: var(--fg-3);
  }
  footer {
    height: 34px;
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 0 16px;
    border-top: 1px solid var(--border);
    font-size: 11px;
    color: var(--fg-3);
  }
  footer span {
    display: flex;
    align-items: center;
    gap: 5px;
  }
  .idx {
    margin-left: auto;
    font-family: "JetBrains Mono", monospace;
    opacity: 0.7;
  }
</style>
