<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { createEditor, type EditorHandle } from "../lib/editor";
  import { ui, getFile, updateContent, setEditorView, completeAt, hoverAt, spellCheck } from "../lib/store.svelte";

  let host: HTMLDivElement;
  let handle: EditorHandle | undefined;
  let current = "";

  onMount(() => {
    const f = getFile(ui.activeTab);
    handle = createEditor({
      parent: host,
      doc: f?.content ?? "",
      lang: f?.language ?? "typst",
      onChange: (doc) => updateContent(ui.activeTab, doc),
      onCursor: (line, col) => {
        ui.cursor = { line, col };
      },
      complete: (cursor, explicit) => completeAt(cursor, explicit),
      hover: (cursor) => hoverAt(cursor),
      spell: (text) => spellCheck(text),
    });
    current = ui.activeTab;
    setEditorView(handle.view);
  });

  onDestroy(() => {
    setEditorView(null);
    handle?.destroy();
  });

  // Swap document when the active tab changes.
  $effect(() => {
    const tab = ui.activeTab;
    if (handle && tab && tab !== current) {
      const f = getFile(tab);
      if (f) {
        handle.setDoc(f.content, f.language);
        current = tab;
      }
    }
  });
</script>

<div class="editor" bind:this={host}></div>

<style>
  .editor {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    background: var(--bg-2);
  }
  .editor :global(.cm-editor) {
    height: 100%;
  }
</style>
