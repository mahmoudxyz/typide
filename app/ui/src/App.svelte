<script lang="ts">
  import { onMount } from "svelte";
  import { ui, applyTheme, persistSize, saveAndCompile, loadProject, completeFirstRun, initJobs, initPolicy, initCloseGuard, initRecoveryAutosave } from "./lib/store.svelte";
  import Resizer from "./components/Resizer.svelte";
  import TitleBar from "./components/TitleBar.svelte";
  import ActivityBar from "./components/ActivityBar.svelte";
  import LeftPanel from "./components/LeftPanel.svelte";
  import EditorArea from "./components/EditorArea.svelte";
  import Preview from "./components/Preview.svelte";
  import BottomPanel from "./components/BottomPanel.svelte";
  import StatusBar from "./components/StatusBar.svelte";
  import CommandPalette from "./components/CommandPalette.svelte";
  import SearchEverywhere from "./components/SearchEverywhere.svelte";
  import Launcher from "./components/Launcher.svelte";
  import NewProjectWizard from "./components/NewProjectWizard.svelte";
  import FirstRunSetup from "./components/FirstRunSetup.svelte";
  import PackageBrowser from "./components/PackageBrowser.svelte";
  import CitationPicker from "./components/CitationPicker.svelte";
  import ToolchainManager from "./components/ToolchainManager.svelte";
  import ConsentDialog from "./components/ConsentDialog.svelte";
  import CloseGuard from "./components/CloseGuard.svelte";
  import RecoveryBanner from "./components/RecoveryBanner.svelte";
  import Toast from "./components/Toast.svelte";

  onMount(() => {
    applyTheme();
    initPolicy();
    initJobs();
    initCloseGuard();
    initRecoveryAutosave();
    // Dev convenience: open the demo project directly with ?demo in the URL.
    if (typeof location !== "undefined" && new URLSearchParams(location.search).has("demo")) {
      const params = new URLSearchParams(location.search);
      completeFirstRun();
      loadProject("demo")
        .then(() => {
          const tool = params.get("tool");
          if (tool) ui.activeTool = tool as any;
          if (params.has("browser")) ui.packageBrowser = true;
        })
        .catch((e) => console.error("demo load failed", e));
    }
  });

  let lastShift = 0;

  function onKey(e: KeyboardEvent) {
    const mod = e.ctrlKey || e.metaKey;

    // On the launcher, only Ctrl/Cmd+N (new project) is wired.
    if (ui.view === "launcher") {
      if (mod && e.key.toLowerCase() === "n") {
        e.preventDefault();
        ui.wizardOpen = true;
      }
      return;
    }

    // Insert citation: Alt+C
    if (e.altKey && e.key.toLowerCase() === "c") {
      e.preventDefault();
      ui.citationPicker = true;
      return;
    }

    // Save: Ctrl/Cmd+S
    if (mod && !e.shiftKey && e.key.toLowerCase() === "s") {
      e.preventDefault();
      saveAndCompile();
      return;
    }

    // Command palette: Ctrl/Cmd+Shift+P
    if (mod && e.shiftKey && e.key.toLowerCase() === "p") {
      e.preventDefault();
      ui.commandPalette = true;
      return;
    }
    // Search Everywhere: Ctrl/Cmd+P (single), or double-Shift
    if (mod && !e.shiftKey && e.key.toLowerCase() === "p") {
      e.preventDefault();
      ui.searchEverywhere = true;
      return;
    }
    // Toggles
    if (mod && e.key.toLowerCase() === "b") {
      e.preventDefault();
      ui.leftVisible = !ui.leftVisible;
      return;
    }
    if (mod && e.key.toLowerCase() === "j") {
      e.preventDefault();
      ui.bottomVisible = !ui.bottomVisible;
      return;
    }
    // Double-tap Shift → Search Everywhere
    if (e.key === "Shift" && !mod) {
      const now = Date.now();
      if (now - lastShift < 350 && !ui.searchEverywhere && !ui.commandPalette) {
        ui.searchEverywhere = true;
        lastShift = 0;
      } else {
        lastShift = now;
      }
    } else if (e.key !== "Shift") {
      lastShift = 0;
    }
  }
</script>

<svelte:window onkeydown={onKey} />

{#if ui.view === "launcher"}
  <Launcher />
{:else}
  <div class="app">
    <TitleBar />
    <div class="body">
      <ActivityBar />
      <div class="workspace">
        <div class="main-row">
          {#if ui.leftVisible}
            <LeftPanel />
            <Resizer axis="x" min={200} max={520} get={() => ui.leftWidth} set={(v) => (ui.leftWidth = v)} oncommit={(v) => persistSize("left", v)} />
          {/if}
          <EditorArea />
          {#if ui.previewVisible}
            <Resizer axis="x" min={260} max={820} invert get={() => ui.previewWidth} set={(v) => (ui.previewWidth = v)} oncommit={(v) => persistSize("preview", v)} />
            <Preview />
          {/if}
        </div>
        {#if ui.bottomVisible}
          <Resizer axis="y" min={120} max={520} invert get={() => ui.bottomHeight} set={(v) => (ui.bottomHeight = v)} oncommit={(v) => persistSize("bottom", v)} />
          <BottomPanel />
        {/if}
      </div>
    </div>
    <StatusBar />
  </div>

  {#if ui.commandPalette}<CommandPalette />{/if}
  {#if ui.searchEverywhere}<SearchEverywhere />{/if}
  {#if ui.packageBrowser}<PackageBrowser />{/if}
  {#if ui.citationPicker}<CitationPicker />{/if}
{/if}

{#if ui.toolchainManager}<ToolchainManager />{/if}
<ConsentDialog />
{#if ui.closeGuard}<CloseGuard />{/if}
{#if ui.recovery}<RecoveryBanner />{/if}

{#if ui.wizardOpen}<NewProjectWizard />{/if}
{#if ui.showFirstRun}<FirstRunSetup />{/if}
<Toast />

<style>
  .app {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .body {
    flex: 1;
    display: flex;
    min-height: 0;
  }
  .workspace {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .main-row {
    flex: 1;
    display: flex;
    min-height: 0;
  }
</style>
