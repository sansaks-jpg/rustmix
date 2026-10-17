# ADR 0001 — Minimal native synthetic UI

2026-10-17 · accepted for UI-001.

The repository originally contained only AGENTS.md. Build one usable synthetic UI slice before introducing hardware/codec integrations.

Use three packages: rustmix-engine (headless domain, commands, CPU demo and persistence modules), rustmix-ui (L4 interface), rustmix (L5 executable). Foundation/project/render modules are temporarily private concerns of the engine package rather than empty crates. They have no dependency on UI or OS types. Extract them when a second consumer justifies the boundaries. Engine → external serde/thiserror/tempfile only; UI → engine; binary → UI + eframe host. No same-layer package dependencies.

Installed Rust is 1.94.1. Registry metadata verified that eframe 0.37.0 and egui_dock 0.21.1 require Rust 1.95, so select compatible eframe 0.35.0 / egui_dock 0.20.1 (egui 0.35). Check in Cargo.lock. rfd 0.17.2 supplies native dialogs. The dependency baseline is permissively licensed Rust code; font notices need separate preservation. No runtime codec libraries or vendor SDKs.

CPU RGBA demo composition is uploaded to egui textures and presented through wgpu. There is no custom GPU video compositor, zero-copy pipeline or GPU/CPU parity claim. This intentionally keeps the first visual milestone testable without hardware sources. Display-referred sRGB blending is a demo limitation.

Domain changes are validated in a cloned candidate project, then committed with one revision and command result. Transitions pin the from/to IDs, use monotonic integer milliseconds, and swap at progress 1. Manual zero cancels, one commits; auto rejects manual scrubbing. Preview/source structural changes during transitions are rejected. FTB is an independent ramp over the composed monitor. Active transitions and black level are transient and reset on project open.

Use a single worker at a time for native dialogs and disk IO with bounded result handoff. Atomic saves preserve two validated backup generations and an explicit recovery journal. Autosave is opt-in recovery through Settings, never silent source switching. No application data is transmitted.

Build policy changed by explicit owner request: create a public GitHub repo and build executable only on GitHub-hosted Windows. Local actions are source edits, formatting, dependency metadata/fetch, Git review and publishing authorized source. Do not run cargo check/test/build locally because these can produce executables. Native test harness runs solely on the disposable hosted runner. Do not use cua-driver.
