<script lang="ts">
  // A thin draggable splitter. `axis` is the pointer axis it tracks; `invert`
  // flips the direction (used for panels anchored to the right/bottom edge).
  let {
    axis,
    get,
    set,
    min = 180,
    max = 900,
    invert = false,
    oncommit,
  }: {
    axis: "x" | "y";
    get: () => number;
    set: (v: number) => void;
    min?: number;
    max?: number;
    invert?: boolean;
    oncommit?: (v: number) => void;
  } = $props();

  let dragging = $state(false);
  let startPos = 0;
  let startVal = 0;

  function move(e: PointerEvent) {
    const pos = axis === "x" ? e.clientX : e.clientY;
    let d = pos - startPos;
    if (invert) d = -d;
    set(Math.max(min, Math.min(max, startVal + d)));
  }
  function up() {
    dragging = false;
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    window.removeEventListener("pointermove", move);
    window.removeEventListener("pointerup", up);
    oncommit?.(get());
  }
  function down(e: PointerEvent) {
    dragging = true;
    startPos = axis === "x" ? e.clientX : e.clientY;
    startVal = get();
    document.body.style.cursor = axis === "x" ? "col-resize" : "row-resize";
    document.body.style.userSelect = "none";
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
    e.preventDefault();
  }
</script>

<div
  class="resizer {axis}"
  class:dragging
  onpointerdown={down}
  role="separator"
  aria-orientation={axis === "x" ? "vertical" : "horizontal"}
  tabindex="-1"
></div>

<style>
  .resizer {
    flex: none;
    position: relative;
    z-index: 5;
    background: transparent;
    transition: background 0.12s;
  }
  .resizer.x {
    width: 5px;
    margin: 0 -2.5px;
    cursor: col-resize;
  }
  .resizer.y {
    height: 5px;
    margin: -2.5px 0;
    cursor: row-resize;
  }
  .resizer:hover,
  .resizer.dragging {
    background: var(--accent);
  }
</style>
