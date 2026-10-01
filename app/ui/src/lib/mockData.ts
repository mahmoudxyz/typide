// Mock project used when the UI runs in a plain browser (no Tauri backend).
// Represents tests/fixtures/thesis-basic (ARCHITECTURE.md §21).
import type {
  Citation,
  Diagnostic,
  EditorFile,
  ExportProfile,
  FileNode,
  Job,
  Note,
  PackageInfo,
  ProjectInfo,
  Snapshot,
  StructureNode,
} from "./types";

export const project: ProjectInfo = {
  name: "phd-thesis",
  kind: "thesis",
  root: "~/research/phd-thesis",
  entrypoint: "main.typ",
  typst: "0.15.1",
  tinymist: "0.13.10",
  branch: "main",
};

export const tree: FileNode = {
  name: "phd-thesis",
  path: "",
  kind: "dir",
  children: [
    {
      name: "chapters",
      path: "chapters",
      kind: "dir",
      children: [
        { name: "01-introduction.typ", path: "chapters/01-introduction.typ", kind: "typ" },
        { name: "02-related-work.typ", path: "chapters/02-related-work.typ", kind: "typ" },
        { name: "03-method.typ", path: "chapters/03-method.typ", kind: "typ" },
        { name: "04-results.typ", path: "chapters/04-results.typ", kind: "typ" },
      ],
    },
    {
      name: "notes",
      path: "notes",
      kind: "dir",
      children: [
        { name: "index.typ", path: "notes/index.typ", kind: "typ" },
        { name: "attention-is-all.md", path: "notes/attention-is-all.md", kind: "md" },
        { name: "diffusion-survey.md", path: "notes/diffusion-survey.md", kind: "md" },
      ],
    },
    {
      name: "vendor",
      path: "vendor",
      kind: "dir",
      badge: "vendored",
      children: [
        {
          name: "packages",
          path: "vendor/packages",
          kind: "dir",
          children: [
            { name: "@preview", path: "vendor/packages/preview", kind: "dir" },
          ],
        },
      ],
    },
    { name: "fonts", path: "fonts", kind: "dir", children: [
      { name: "NewCM10-Regular.otf", path: "fonts/NewCM10-Regular.otf", kind: "font" },
    ] },
    { name: "main.typ", path: "main.typ", kind: "typ", badge: "entry" },
    { name: "refs.bib", path: "refs.bib", kind: "bib" },
    { name: "typide.toml", path: "typide.toml", kind: "toml" },
    { name: "typide.lock", path: "typide.lock", kind: "toml" },
  ],
};

export const files: Record<string, EditorFile> = {
  "main.typ": {
    path: "main.typ",
    name: "main.typ",
    language: "typst",
    content: `#import "@preview/cetz:0.4.2": canvas, draw

#set document(title: "A Thesis on Neural Fields", author: "M. Ahmed")
#set page(paper: "a4", margin: 2.5cm, numbering: "1")
#set text(font: "New Computer Modern", size: 11pt, lang: "en")
#set heading(numbering: "1.1")

// ── Title page ──────────────────────────────────────────────
#align(center)[
  #v(4cm)
  #text(20pt, weight: "bold")[Learning Neural Fields for \\
  Reproducible Scientific Documents]
  #v(1cm)
  #text(14pt)[Mahmoud Ahmed]
  #v(0.4cm)
  Doctoral Thesis · #datetime.today().display()
]

#pagebreak()
#outline(title: "Contents", indent: auto)
#pagebreak()

= Introduction <intro>
#include "chapters/01-introduction.typ"

= Related Work <related>
#include "chapters/02-related-work.typ"

= Method <method>
#include "chapters/03-method.typ"

= Results <results>
#include "chapters/04-results.typ"

#bibliography("refs.bib", style: "ieee")
`,
  },
  "chapters/01-introduction.typ": {
    path: "chapters/01-introduction.typ",
    name: "01-introduction.typ",
    language: "typst",
    content: `Neural fields have reshaped how we represent continuous signals
@mildenhall2020nerf. In this thesis we study their use for
*reproducible* scientific documents.

As shown in @fig-overview, the pipeline has three stages. See also the
detailed treatment in @sec-appendix — a reference that currently has no
target, which Typide flags below.

#figure(
  rect(width: 80%, height: 3cm, fill: luma(240)),
  caption: [Overview of the reproducibility pipeline.],
) <fig-overview>

The transformer architecture @vaswani2017attention underpins our encoder.
`,
  },
  "refs.bib": {
    path: "refs.bib",
    name: "refs.bib",
    language: "bibtex",
    content: `@article{mildenhall2020nerf,
  title   = {NeRF: Representing Scenes as Neural Radiance Fields},
  author  = {Mildenhall, Ben and Srinivasan, Pratul P.},
  journal = {ECCV},
  year    = {2020}
}

@inproceedings{vaswani2017attention,
  title     = {Attention Is All You Need},
  author    = {Vaswani, Ashish and Shazeer, Noam},
  booktitle = {NeurIPS},
  year      = {2017}
}

@article{ho2020ddpm,
  title   = {Denoising Diffusion Probabilistic Models},
  author  = {Ho, Jonathan and Jain, Ajay and Abbeel, Pieter},
  journal = {NeurIPS},
  year    = {2020}
}`,
  },
  "typide.toml": {
    path: "typide.toml",
    name: "typide.toml",
    language: "toml",
    content: `[project]
name = "phd-thesis"
kind = "thesis"
entrypoint = "main.typ"

[toolchain]
typst = "0.15.1"
tinymist = "auto"

[packages]
mode = "vendor"
vendor-dir = "vendor/packages"

[fonts]
paths = ["fonts"]
use-system-fonts = true

[references]
files = ["refs.bib"]
zotero = { enabled = true, collection = "PhD", mode = "local-api" }

[[export]]
name = "Submission PDF"
format = "pdf"
output = "out/thesis.pdf"
pdf-standards = ["a-2a", "ua-1"]

[check]
profile = "thesis-submission"`,
  },
  "notes/attention-is-all.md": {
    path: "notes/attention-is-all.md",
    name: "attention-is-all.md",
    language: "markdown",
    content: `---
cite-key: vaswani2017attention
---

# Attention Is All You Need

The transformer replaces recurrence with **self-attention**. Key idea:
scaled dot-product attention over [[queries-keys-values]].

Relevant to [[03-method]] — our encoder is a thin transformer.

## Takeaways
- Positional encodings inject order.
- Multi-head attention = several projections in parallel.
`,
  },
};

export const diagnostics: Diagnostic[] = [
  {
    id: "broken-ref",
    file: "chapters/01-introduction.typ",
    line: 9,
    col: 47,
    severity: "error",
    message: "Reference @sec-appendix has no matching label in the project.",
    source: "typide",
    quickFix: "Create label / pick similar label",
  },
  {
    id: "uncited-entry",
    file: "refs.bib",
    line: 15,
    col: 1,
    severity: "warning",
    message: "Bibliography entry `ho2020ddpm` is never cited (thesis profile).",
    source: "typide",
    quickFix: "Remove entry / ignore",
  },
  {
    id: "image-no-alt",
    file: "chapters/01-introduction.typ",
    line: 12,
    col: 3,
    severity: "warning",
    message: "Figure has no `alt` text — required for PDF/UA-1 conformance.",
    source: "typide",
    quickFix: "Insert alt:",
  },
  {
    id: "system-font",
    file: "main.typ",
    line: 5,
    col: 17,
    severity: "hint",
    message: "Font \"New Computer Modern\" resolves from a bundled font, not the project — OK for reproducibility.",
    source: "typide",
  },
];

export const jobs: Job[] = [
  { id: 1, title: "Compile · main.typ", fraction: 1, message: "42 pages · 118 ms", state: "done" },
  { id: 2, title: "Resolve packages", fraction: 1, message: "3 packages · lockfile verified", state: "done" },
  { id: 3, title: "Index project", fraction: 0.72, message: "scanning chapters/04-results.typ", state: "running" },
];

export const structure: StructureNode[] = [
  {
    label: "Introduction",
    kind: "heading",
    line: 34,
    level: 1,
    children: [
      { label: "Fig: reproducibility pipeline", kind: "figure", line: 12 },
      { label: "<intro>", kind: "label", line: 34 },
    ],
  },
  { label: "Related Work", kind: "heading", line: 37, level: 1, children: [{ label: "<related>", kind: "label", line: 37 }] },
  {
    label: "Method",
    kind: "heading",
    line: 40,
    level: 1,
    children: [
      { label: "Eq: attention softmax", kind: "equation", line: 22 },
      { label: "<method>", kind: "label", line: 40 },
    ],
  },
  { label: "Results", kind: "heading", line: 43, level: 1, children: [{ label: "Table: benchmark", kind: "table", line: 8 }] },
];

export const packages: PackageInfo[] = [
  { spec: "@preview/cetz:0.4.2", name: "cetz", version: "0.4.2", latest: "0.4.2", status: "vendored", description: "Drawing with a TikZ-like API." },
  { spec: "@preview/oxifmt:0.2.1", name: "oxifmt", version: "0.2.1", latest: "0.3.0", status: "cached", description: "String formatting utilities." },
  { spec: "@preview/glossarium:0.5.4", name: "glossarium", version: "0.5.4", latest: "0.5.4", status: "vendored", description: "Glossaries and acronyms." },
  { spec: "@preview/lovelace:0.3.0", name: "lovelace", version: "0.3.0", latest: "0.3.0", status: "missing", description: "Pseudocode / algorithms." },
];

export const citations: Citation[] = [
  { key: "mildenhall2020nerf", title: "NeRF: Representing Scenes as Neural Radiance Fields", authors: "Mildenhall, Srinivasan", year: "2020", source: "bib", cited: true },
  { key: "vaswani2017attention", title: "Attention Is All You Need", authors: "Vaswani, Shazeer", year: "2017", source: "bib", cited: true },
  { key: "ho2020ddpm", title: "Denoising Diffusion Probabilistic Models", authors: "Ho, Jain, Abbeel", year: "2020", source: "bib", cited: false },
  { key: "kingma2014adam", title: "Adam: A Method for Stochastic Optimization", authors: "Kingma, Ba", year: "2014", source: "zotero", cited: false },
];

export const notes: Note[] = [
  { path: "notes/attention-is-all.md", title: "Attention Is All You Need", citeKey: "vaswani2017attention", backlinks: ["03-method"], outgoing: ["queries-keys-values", "03-method"] },
  { path: "notes/diffusion-survey.md", title: "Diffusion Models — Survey", citeKey: "ho2020ddpm", backlinks: [], outgoing: ["ddpm-training"] },
  { path: "notes/index.typ", title: "Vault index", backlinks: ["attention-is-all", "diffusion-survey"], outgoing: [] },
];

export const exportProfiles: ExportProfile[] = [
  { name: "Submission PDF", format: "pdf", output: "out/thesis.pdf", standards: ["a-2a", "ua-1"] },
  { name: "Draft PDF", format: "pdf", output: "out/draft.pdf" },
  { name: "Notes website", format: "html", output: "out/site" },
];

export const snapshots: Snapshot[] = [
  { id: "a1f92c", message: "Snapshot before migration to 0.15.1", when: "2 hours ago", auto: true },
  { id: "7d3e01", message: "Add results chapter draft", when: "yesterday", auto: false },
  { id: "40b5aa", message: "Auto-snapshot before rename 'fig-arch' → 'fig-overview'", when: "yesterday", auto: true },
  { id: "12cc78", message: "Initial thesis from template", when: "3 days ago", auto: false },
];

export const previewPages = Array.from({ length: 6 }, (_, i) => ({ n: i + 1, changed: i === 1 }));

// ── Launcher / wizard data ─────────────────────────────────────────────────
export interface RecentProject {
  name: string;
  path: string;
  kind: "thesis" | "paper" | "notes" | "package" | "generic";
  lastOpened: string;
  pinned?: boolean;
}

export const recentProjects: RecentProject[] = [
  { name: "phd-thesis", path: "~/research/phd-thesis", kind: "thesis", lastOpened: "just now", pinned: true },
  { name: "iclr-2026-submission", path: "~/papers/iclr-2026", kind: "paper", lastOpened: "yesterday" },
  { name: "research-notebook", path: "~/notes/research-notebook", kind: "notes", lastOpened: "3 days ago" },
  { name: "typst-uni-template", path: "~/dev/typst-uni-template", kind: "package", lastOpened: "last week" },
  { name: "lecture-slides", path: "~/teaching/dsp-2025", kind: "generic", lastOpened: "2 weeks ago" },
];

export interface Generator {
  id: string;
  kind: "thesis" | "paper" | "notes" | "package" | "generic" | "template";
  title: string;
  icon: string;
  blurb: string;
  defaultMode: "vendor" | "cache";
  vault: boolean;
  refs: boolean;
}

export const generators: Generator[] = [
  { id: "thesis", kind: "thesis", title: "Thesis", icon: "book", blurb: "A long-form dissertation with chapters, bibliography, and strict submission checks. Packages are vendored for years-later reproducibility.", defaultMode: "vendor", vault: true, refs: true },
  { id: "paper", kind: "paper", title: "Paper / Article", icon: "file-code", blurb: "A conference or journal article — single file or a few, with references and a template-friendly layout.", defaultMode: "cache", vault: false, refs: true },
  { id: "notes", kind: "notes", title: "Research Notebook", icon: "structure", blurb: "A notes vault with wiki-links, backlinks, literature notes, and full-text search. Great for reading groups.", defaultMode: "cache", vault: true, refs: true },
  { id: "package", kind: "package", title: "Typst Package", icon: "package", blurb: "A reusable package or template with typst.toml, lib.typ, README, and LICENSE — publishable to a local namespace or Universe.", defaultMode: "cache", vault: false, refs: false },
  { id: "generic", kind: "generic", title: "Empty Project", icon: "file", blurb: "A blank project with just main.typ and typide.toml. Add structure as you go.", defaultMode: "cache", vault: false, refs: false },
  { id: "template", kind: "template", title: "From Template…", icon: "layers", blurb: "Start from a Typst Universe template, a local template directory, or an institutional (git / network-share) template source.", defaultMode: "cache", vault: false, refs: false },
];

export interface Template {
  spec: string;
  title: string;
  author: string;
  source: "universe" | "local" | "institutional";
  blurb: string;
}

export const templates: Template[] = [
  { spec: "@preview/modern-uni-thesis:0.2.0", title: "Modern University Thesis", author: "typst-community", source: "universe", blurb: "Clean thesis layout with title page, declaration, and ToC." },
  { spec: "@preview/charged-ieee:0.1.3", title: "Charged IEEE", author: "typst-community", source: "universe", blurb: "IEEE conference paper template." },
  { spec: "@preview/clean-math-paper:0.2.0", title: "Clean Math Paper", author: "typst-community", source: "universe", blurb: "Theorem environments and math-first layout." },
  { spec: "@local/lab-thesis:1.0.0", title: "Lab Thesis (local)", author: "@local", source: "local", blurb: "Your lab's scaffolding — installed in the @local namespace." },
  { spec: "@uni/faculty-thesis:2.4.0", title: "Faculty of Science Thesis", author: "policy: mirror.uni.example", source: "institutional", blurb: "Institutional template from the configured network share." },
];

export const availableTypst = ["0.15.1", "0.14.2", "0.13.1"];
