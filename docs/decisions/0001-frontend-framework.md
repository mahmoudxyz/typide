# ADR 0001 — Frontend framework: Svelte 5 + Vite

- **Status:** Accepted (M0)
- **Date:** 2026-09-30

## Context

ARCHITECTURE.md §3 requires choosing a frontend framework in M0 and recording
it here. The options considered were **Svelte 5**, **SolidJS**, and a minimal
custom setup. Constraints: small bundle, no heavy framework runtime, fast
keystroke-to-preview (§21 budget: < 300 ms), first-class TypeScript, and a
component model that scales to an IDE with many tool windows.

## Decision

Use **Svelte 5 (runes) + Vite + TypeScript**, with **CodeMirror 6** as the
editor component (§3).

Rationale:
- Svelte 5 runes (`$state`, `$derived`, `$effect`) give fine-grained
  reactivity with almost no runtime overhead — a good fit for a live editor +
  preview where re-render cost matters.
- Vite gives fast HMR in dev and small production bundles; it is also Tauri's
  documented default.
- CodeMirror 6 is modular, has a real LSP-friendly extension model, and lets us
  bind syntax colors to CSS variables so light/dark themes need no editor
  reconfiguration.

## Consequences

- One store module per domain (`store.svelte.ts`), backend is source of truth
  (§18.4). The frontend never writes files directly.
- A thin `api` layer will bridge Tauri commands (§18.2) with mock data so the UI
  runs in a plain browser during development.
- Revisit if SolidJS shows a decisive ergonomics/perf win once the editor and
  preview are under real load (M1 acceptance measures this).
