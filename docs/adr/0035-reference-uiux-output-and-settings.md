# ADR 0035: inspectable output setup and page-specific operator UI

Date: 2026-10-21. Status: implemented UI candidate; verification recorded in UI-035.

The supplied reference separates profile selection, destination/format navigation,
read-only configuration, status and live actions. RustMix previously disabled the
entire Stream/MultiCorder forms and reused unrelated Settings/processing controls.

Keep presentation state in ui-egui: `output_dialogs` owns output navigation and
`settings_pages` owns specific summaries. Navigation and Close remain available;
absent backend configuration, save and live actions remain unavailable. Do not
store streaming secrets or claim metrics from counters without output workers.
Keep the existing typed engine boundary and reset only the selected input draft
group. No dependency, schema, unsafe or runtime licensing change.

Owner explicitly permits local Rust tests for this request while prohibiting an
application .exe build. Use test runners and metadata/Clippy only, with a test-only
software renderer of actual egui meshes for review. Label this evidence headless;
native GPU and physical-workstation acceptance remain separate. No alternate
production web UI or application executable is introduced.

Consequence: an operator can inspect the workflow without implying production
output support. Summaries on unavailable settings pages are intentionally smaller
than vendor configuration forms. Full configuration parity waits for real
backends, rather than silently staging or pretending to save unsupported values.
