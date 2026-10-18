# ADR 0003 — Bounded operator configuration

2026-10-17 · UI-004 · historical implementation accepted by subsequent hosted widget/native gates; see ui-progress.md for exact revisions. UI-013 changes require their own acceptance.

The supplied main-window, Add Input, Input Settings and Settings references were observed locally. Retain the generic switcher workflow and original RustMix design. Avoid putting configuration into the live transition area.

The three configuration windows share viewport bounds, minimum sizes and resizable defaults. Their navigation and content columns scroll separately; confirmation controls remain outside the scroll areas. A small generated sample is cached independently of project inputs, so removing or marking an existing source offline cannot make the source catalog show the wrong sample.

Expose direct monitor properties access, text alongside color filters, matching counts and explicit offline fallback. Route every UI removal request through engine eligibility and one modal confirmation. Rebuild automation bounds each frame so removed or hidden controls cannot appear available to a test harness. No engine command, source routing, dependency or project schema change is needed.

Acceptance: reachable footers and navigation at minimum/large logical sizes, no Program mutation from editing or filtering, cancellation preserves staged project/source values, and independent sample availability after source removal. Require hosted tests and real native dialog screenshot review at 1100×700, 1600×900 and 1920×1080. Local formatting/source guards cannot establish rendered correctness or broadcast readiness.
