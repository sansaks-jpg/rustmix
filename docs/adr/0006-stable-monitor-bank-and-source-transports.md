# ADR 0006: Stable monitor geometry and independent source transport

2026-10-18 · UI-009 · Accepted implementation; hosted checks passed in [run 39089212707](https://github.com/sansaks-jpg/rustmix/actions/runs/39089212707), code `3a36cf2`.

The UI-008 monitor bank expanded without operator input because child layout
minimum sizes were propagated back into the resizable panel's persisted size.
The monitor bank now allocates its rectangle once and draws a clipped child
inside it. Its maximum also reserves 240 logical pixels for the input/mixer bank
after the toolbars. Old oversized panel state is clamped on restoration.

The operator requested a horizontal manual transition. The T-Bar now maps the
left/right track endpoints to 0/1 and retains its completion latch through mouse
release, so holding the endpoint cannot take the source twice.

Source transport belongs to the headless engine. Each procedural input has an
independent monotonic clock, pause, restart and loop state. The picture,
countdown, progress bar and timecode all read that clock. Global pause preserves
individual pause flags and does not stop the switcher's transition clock.
Sixty seconds is the procedural demonstration duration, not probed media length.
Transport state is ephemeral; it is reset when opening a project.

Each active bus source is rendered at its own transport time and processed
before the transition. Overlays and fade-to-black follow bus composition.
Custom PNG images are resized for both thumbnail and Program resolutions;
offline sources remain black even when an image/effect is configured.

This is CPU reference composition followed by wgpu presentation. A GPU display
does not establish a GPU media compositor, hardware decode/encode, independent
output clock or real sample-based audio. Status shows actual measured UI FPS
and CPU generation time. The prior minimum-FPS clamp was removed. Unsupported
record/stream/external/MultiCorder start paths cannot assert a live state.

Regression coverage includes many idle frames and repeated resizes, transport
commands and pixels, endpoint hold/release, effect surfaces, loop endpoints,
malformed compositor surfaces and unsupported output state. Tests and native
captures must run on GitHub-hosted Windows under the existing environment
policy; formatting/source checks alone are not runtime acceptance.
