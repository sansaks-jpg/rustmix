# ADR 0025: Monitor picture and production panel proportions

Date: 2026-10-21
Status: selected corrections verified in ba3b209/run 39540613350; full equivalence open.

The user explicitly deprioritized Light theme and requested the remaining UI/UX.
The accepted UI-024 native 1917x1198 capture shows 809x455 pictures vertically
centered in 809x537 dark surfaces. The transport sits below those surfaces.
The supplied Interface reference instead places the 16:9 picture directly below
the source header, leaving unused workspace below the monitor. Its default mixer
occupies about 15% of the width; RustMix's Production default reserves 24%.

Allocate the monitor surface at the fitted picture height and keep it top-aligned.
Retain pillarboxing when height constrains the picture, original source colours,
the existing transport actions, the resizable monitor bank and its lower input
space reservation. Change only the initial/reset Production dock to 85% Inputs,
15% Audio. Preserve saved layouts and the wider Audio workspace. Existing audio
horizontal scrolling keeps every strip reachable. Engine/output state, texture
mapping, schemas, assets and dependencies do not change.

Record dock panel rectangles for verification. Regress real settled layouts at
1100x700 through 1920x1400, header/picture/transport adjacency, full visible Master
and both group rails, six large-workspace source pictures per row, Audio workspace
switching and saved dock restoration without exchanging Program and Preview.
Extend the existing native smoke with a 1917x1168 client capture: the supplied
1917x1198 screenshot includes approximately 30 px of OS caption, while GPU readback
does not. Keep the original raster capture, strict GPU drain and cadence gates.
Do not claim exact equivalence or a performance improvement before verification.
