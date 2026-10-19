# ADR 0017 — Direct extended-display Program output

Date: 2026-10-19. Status: implemented candidate; physical two-display qualification pending.

The top Fullscreen button sends the current Program texture directly to a separate,
borderless fullscreen window on an extended Windows display. It does not fullscreen
the operator application and requires no F11 or manual dragging. A second press
closes the output. The adjacent menu selects a display or opens the optional
windowed Program monitor. This supersedes the top-button/manual-placement behavior
described in ADR 0015.

Windows monitor inventory and the native operator HWND determine the target.
The default is the first non-overlapping display other than the operator display.
A stored display identity is revalidated before every Start. Missing, mirrored,
operator or unknown displays fail visibly without choosing an unrelated display.
The output is initially hidden until its native fullscreen geometry matches the
target. A bounded discovery worker periodically revalidates monitor changes.
Startup restores a selection only; it never opens an output automatically.

Four injected inventory/widget regressions cover routing, operator protection,
reordering/loss and hidden placement. They do not prove physical display output.
Two physical monitors, mixed DPI and hotplug remain unqualified; Windows may
temporarily relocate an already visible window before the watcher detects loss.
The current image remains the experimental 640x360 CPU Program composite, with
no embedded audio, SDI, virtual-camera or network-output claim.

All parallel agents were interrupted at the user's request before the build.
Unfinished MP4 player/UI work remains in isolated worktrees, outside this build.
