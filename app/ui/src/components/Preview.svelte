<script lang="ts">
  import Icon from "./Icon.svelte";
  import { ui, data, getFile, jumpToClick } from "../lib/store.svelte";

  type Mode = "fit-width" | "fit-page" | "custom";
  let mode = $state<Mode>("fit-width");
  let zoom = $state(100);
  let invert = $state(false);
  let menuOpen = $state(false);
  let scrollW = $state(400);
  let scrollH = $state(600);
  let scrollEl = $state<HTMLElement | null>(null);

  // Editor→preview sync highlight: which page + vertical fraction to pulse.
  let hl = $state<{ page: number; frac: number } | null>(null);
  let hlTimer: ReturnType<typeof setTimeout> | null = null;

  const compiled = $derived(data.compiled);
  const svgPages = $derived(compiled?.pages ?? []);
  const hasError = $derived((compiled?.diagnostics ?? []).some((d) => d.severity === "error"));

  // Natural page size in points (from the first SVG, else A4).
  const dims = $derived.by(() => {
    const m = svgPages[0]?.match(/viewBox="0 0 ([\d.]+) ([\d.]+)"/);
    if (m) return { w: parseFloat(m[1]), h: parseFloat(m[2]) };
    return { w: 595.28, h: 841.89 };
  });
  const naturalPx = $derived((dims.w * 96) / 72);
  const aspect = $derived(dims.h / dims.w);

  const pageWpx = $derived.by(() => {
    if (mode === "fit-width") return Math.max(120, scrollW - 40);
    if (mode === "fit-page") return Math.max(120, (scrollH - 40) / aspect);
    return Math.max(80, (naturalPx * zoom) / 100);
  });
  const effectivePct = $derived(naturalPx ? Math.round((pageWpx / naturalPx) * 100) : zoom);
  const label = $derived(mode === "fit-width" ? "Fit width" : mode === "fit-page" ? "Fit page" : effectivePct + "%");

  function toCustom(v: number) {
    zoom = v;
    mode = "custom";
    menuOpen = false;
  }
  function step(delta: number) {
    if (mode !== "custom") zoom = effectivePct;
    zoom = Math.max(30, Math.min(400, zoom + delta));
    mode = "custom";
  }

  // Structural-fallback derived data (browser dev / before first compile).
  const active = $derived(getFile(ui.activeTab));
  const content = $derived(active?.content ?? "");
  const docTitle = $derived(
    content.match(/#set\s+document\([^)]*title:\s*"([^"]+)"/)?.[1] ??
      content.match(/^=+\s+(.+)$/m)?.[1]?.replace(/<[\w:.-]+>\s*$/, "").trim() ??
      active?.name ??
      data.project?.name ??
      "Untitled"
  );
  const outline = $derived(
    [...content.matchAll(/^(=+)\s+(.+)$/gm)].map((m) => ({
      level: m[1].length,
      text: m[2].replace(/<[\w:.-]+>\s*$/, "").trim(),
    }))
  );

  function onPageClick(e: MouseEvent, i: number) {
    const el = e.currentTarget as HTMLElement;
    const rect = el.getBoundingClientRect();
    const xpt = ((e.clientX - rect.left) / rect.width) * dims.w;
    const ypt = ((e.clientY - rect.top) / rect.height) * dims.h;
    jumpToClick(i + 1, xpt, ypt);
  }

  // Editor caret moved → scroll the matching spot into view and pulse a marker.
  // Keyed on `seq` so recompiles (which change svgPages) don't re-trigger it.
  let lastSyncSeq = -1;
  $effect(() => {
    const s = ui.previewSync;
    if (!s || s.seq === lastSyncSeq) return;
    lastSyncSeq = s.seq;
    if (!scrollEl || svgPages.length === 0) return;
    const pages = scrollEl.querySelectorAll<HTMLElement>(".svgpage");
    const pageEl = pages[s.page - 1];
    if (!pageEl) return;
    const frac = Math.min(1, Math.max(0, s.y / dims.h));
    const pr = pageEl.getBoundingClientRect();
    const sr = scrollEl.getBoundingClientRect();
    const pageTopInScroll = pr.top - sr.top + scrollEl.scrollTop;
    const target = pageTopInScroll + frac * pageEl.clientHeight - scrollEl.clientHeight * 0.33;
    scrollEl.scrollTo({ top: Math.max(0, target), behavior: "smooth" });
    hl = { page: s.page, frac };
    if (hlTimer) clearTimeout(hlTimer);
    hlTimer = setTimeout(() => (hl = null), 1400);
  });
</script>

<section class="preview" style="width: {ui.previewWidth}px">
  <header>
    <span class="ttl"><Icon name="eye" size={14} /> Preview</span>
    {#if ui.compiling}<span class="mini-spin"></span>{/if}
    <span class="spacer"></span>

    <div class="zoomctl">
      <button class="zb" onclick={() => step(-10)} title="Zoom out">−</button>
      <div class="modewrap">
        <button class="modebtn" onclick={() => (menuOpen = !menuOpen)}>{label} <Icon name="chevron" size={10} /></button>
        {#if menuOpen}
          <div class="menu">
            <button class="mi" class:sel={mode === "fit-width"} onclick={() => { mode = "fit-width"; menuOpen = false; }}>Fit width</button>
            <button class="mi" class:sel={mode === "fit-page"} onclick={() => { mode = "fit-page"; menuOpen = false; }}>Fit page</button>
            <button class="mi" onclick={() => toCustom(100)}>Actual size (100%)</button>
            <div class="mdiv"></div>
            {#each [50, 75, 125, 150, 200] as z}
              <button class="mi" class:sel={mode === "custom" && zoom === z} onclick={() => toCustom(z)}>{z}%</button>
            {/each}
          </div>
        {/if}
      </div>
      <button class="zb" onclick={() => step(10)} title="Zoom in">+</button>
    </div>

    <button
      class="tgl"
      class:on={ui.syncScroll}
      title={ui.syncScroll ? "Sync preview to cursor: on" : "Sync preview to cursor: off"}
      onclick={() => (ui.syncScroll = !ui.syncScroll)}
    >
      <Icon name="link" size={14} />
    </button>
    <button class="tgl" class:on={invert} title="Invert pages" onclick={() => (invert = !invert)}>
      <Icon name="moon" size={14} />
    </button>
  </header>

  <div class="scroll" class:invert bind:this={scrollEl} bind:clientWidth={scrollW} bind:clientHeight={scrollH}>
    {#if svgPages.length > 0}
      <div class="pages-wrap">
        {#each svgPages as svg, i (i)}
          <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
          <div
            class="svgpage"
            style="width: {pageWpx}px"
            onclick={(e) => onPageClick(e, i)}
            role="button"
            tabindex="-1"
            title="Click to jump to source"
          >
            <!-- eslint-disable-next-line svelte/no-at-html-tags -->
            {@html svg}
            {#if hl && hl.page === i + 1}
              <span class="synchl" style="top: {hl.frac * 100}%"></span>
            {/if}
            <span class="pnum">{i + 1}</span>
          </div>
        {/each}
      </div>
    {:else if hasError}
      <div class="state error">
        <Icon name="warn" size={30} />
        <p>Compilation failed</p>
        <span>See the Problems panel for details.</span>
      </div>
    {:else}
      <div class="pages-wrap">
        <div class="page" style="width: {pageWpx}px; min-height: {pageWpx * aspect}px">
          <div class="doc">
            <div class="dtitle">{docTitle}</div>
            {#each outline as h, i (i)}
              <div class="h" style="margin-left: {(h.level - 1) * 10}px">
                {#if h.level === 1}{i}&nbsp;&nbsp;{/if}{h.text}
              </div>
              <div class="lines">
                {#each Array(3) as _}<span class="ln"></span>{/each}
                <span class="ln s"></span>
              </div>
            {/each}
          </div>
        </div>
      </div>
    {/if}
  </div>

  <footer>
    <span><span class="live" class:busy={ui.compiling}></span> {ui.compiling ? "compiling…" : ui.syncScroll ? "live · click ⇄ cursor sync" : "live · click to jump"}</span>
    {#if compiled}
      <span class="mono">{compiled.page_count} pp · {compiled.compile_ms} ms</span>
    {:else}
      <span class="mono">structural · compile for PDF</span>
    {/if}
  </footer>
</section>

<svelte:window onclick={(e) => { if (!(e.target as HTMLElement).closest(".modewrap")) menuOpen = false; }} />

<style>
  .preview {
    width: 420px;
    flex: none;
    min-width: 240px;
    background: var(--bg-1);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  header {
    height: 34px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 8px 0 12px;
    border-bottom: 1px solid var(--border);
    flex: none;
  }
  .ttl {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--fg-2);
  }
  .spacer {
    flex: 1;
  }
  .zoomctl {
    display: flex;
    align-items: center;
    gap: 2px;
    background: var(--bg-inset);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 1px;
  }
  .zb {
    width: 22px;
    height: 22px;
    color: var(--fg-2);
    border-radius: 4px;
    font-size: 15px;
    line-height: 1;
  }
  .zb:hover {
    background: var(--bg-4);
    color: var(--fg-0);
  }
  .modewrap {
    position: relative;
  }
  .modebtn {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    height: 22px;
    padding: 0 7px;
    font-size: 11px;
    color: var(--fg-1);
    border-radius: 4px;
    font-family: "JetBrains Mono", monospace;
    min-width: 76px;
    justify-content: center;
  }
  .modebtn:hover {
    background: var(--bg-4);
  }
  .menu {
    position: absolute;
    top: calc(100% + 5px);
    right: 0;
    min-width: 160px;
    background: var(--bg-3);
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    box-shadow: var(--shadow);
    padding: 4px;
    z-index: 30;
  }
  .mi {
    display: block;
    width: 100%;
    text-align: left;
    padding: 6px 9px;
    border-radius: 5px;
    font-size: 12px;
    color: var(--fg-1);
  }
  .mi:hover {
    background: var(--bg-4);
  }
  .mi.sel {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .mdiv {
    height: 1px;
    background: var(--border);
    margin: 4px 2px;
  }
  .tgl {
    width: 26px;
    height: 26px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--fg-3);
    border-radius: 5px;
  }
  .tgl:hover {
    color: var(--fg-1);
    background: var(--bg-4);
  }
  .tgl.on {
    color: var(--accent);
    background: var(--accent-soft);
  }
  .scroll {
    flex: 1;
    overflow: auto;
    background: var(--bg-inset);
    padding: 20px;
  }
  .pages-wrap {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 18px;
  }
  .svgpage {
    background: #fff;
    box-shadow: 0 4px 18px rgba(0, 0, 0, 0.35);
    border-radius: 2px;
    line-height: 0;
    position: relative;
    cursor: text;
    flex: none;
  }
  .svgpage :global(svg) {
    width: 100%;
    height: auto;
    display: block;
    /* Keep Typst's vector output crisp at any zoom in the WebView. */
    shape-rendering: geometricPrecision;
    text-rendering: geometricPrecision;
    image-rendering: -webkit-optimize-contrast;
  }
  .synchl {
    position: absolute;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--accent);
    box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent) 50%, transparent);
    pointer-events: none;
    animation: syncpulse 1.4s ease-out forwards;
  }
  @keyframes syncpulse {
    0% {
      opacity: 0;
      transform: scaleX(0.4);
    }
    15% {
      opacity: 0.95;
      transform: scaleX(1);
    }
    100% {
      opacity: 0;
    }
  }
  .invert .svgpage {
    filter: invert(1) hue-rotate(180deg);
  }
  .pnum {
    position: absolute;
    bottom: 5px;
    right: 8px;
    font-size: 9px;
    color: rgba(0, 0, 0, 0.35);
    line-height: 1;
    background: rgba(255, 255, 255, 0.7);
    padding: 1px 4px;
    border-radius: 3px;
  }
  .invert .pnum {
    color: rgba(255, 255, 255, 0.5);
    background: rgba(0, 0, 0, 0.4);
  }
  .page {
    background: #ffffff;
    color: #16181d;
    box-shadow: 0 4px 18px rgba(0, 0, 0, 0.35);
    border-radius: 2px;
    padding: 30px 26px;
    flex: none;
  }
  .invert .page {
    background: #1d1f24;
    color: #d7dae2;
  }
  .doc {
    font-family: "New Computer Modern", Georgia, serif;
  }
  .dtitle {
    font-size: 15px;
    font-weight: 700;
    text-align: center;
    margin-bottom: 16px;
    line-height: 1.3;
  }
  .h {
    font-weight: 700;
    margin: 10px 0 5px;
    font-size: 12px;
  }
  .lines {
    display: flex;
    flex-direction: column;
    gap: 5px;
    margin: 6px 0;
  }
  .ln {
    height: 4px;
    background: currentColor;
    opacity: 0.2;
    border-radius: 2px;
  }
  .ln.s {
    width: 55%;
  }
  .state {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 8px;
    padding: 60px 30px;
    color: var(--error);
  }
  .state p {
    font-size: 15px;
    color: var(--fg-1);
  }
  .state span {
    font-size: 12px;
    color: var(--fg-3);
  }
  footer {
    height: 26px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    border-top: 1px solid var(--border);
    font-size: 10.5px;
    color: var(--fg-3);
    flex: none;
  }
  footer span {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .live {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--ok);
    box-shadow: 0 0 0 3px var(--ok-soft);
  }
  .live.busy {
    background: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  .mini-spin {
    width: 12px;
    height: 12px;
    border: 2px solid var(--border-strong);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
