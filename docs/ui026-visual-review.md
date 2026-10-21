# UI-026 input navigation and unavailable actions review

2026-10-21. Source 546578b, hosted run 39540152396 passes locked check,
warnings-as-errors Clippy, 261 tests across 17 suites (137 UI, none ignored/filtered),
release build, all 13 native processes and packaging. Final ba3b209 native
Streaming/MultiCorder/Video and workspace images were inspected; the UI source is
unchanged by its harness-only viewport restoration. No full visual-equivalence claim.
Local formatting/diff and repository/reference guards passed. One
formatting attempt hit Windows file-mapping error1224; retry and format check
passed. No Rust compile, test, executable build or native launch ran locally.

Add Input used InputKind to reconstruct its page, losing Video/PNG/Audio choices
whose rendering kinds are shared with generators. Preserve the actual selected
category after Cancel/reopen. The production quick menu now routes Video file,
PNG image, WAV audio and Microphone into their existing configuration pages, plus
accurately named procedural choices. The initial Generators page is unchanged.
No media validation, probing, confirmation or engine command is bypassed.

The disabled Streaming/MultiCorder Start surfaces are neutral. Streaming Save and
Close is disabled with its reason because no backend can save those settings;
Close remains interactive. Regress actual quick-menu clicks for all 11 categories,
their fields after Cancel/reopen, actual painted disabled surfaces, ignored clicks
and full project/output state preservation. The native MP4 workflow enters via
the new quick menu and retains its real decode/switch/save/reopen gates.

Native step 03:47:37..03:49:25 UTC, 108 s. Its short640x360 Program samples miss
50/59.94/60 fps:36.8510/49.3239/36.3976 fps, with 105/85/190 skipped frames.
This run passed before viewport-isolation follow-up ba3b209: it disproves a claim
that large windows always fail or that restoration alone fixes GPU reliability.
The final images show neutral Start surfaces, disabled Save and Close, reachable
Close and the correctly selected validated Video page. Preserve any native failure
and performance evidence. Complete visual
correspondence, real encoder outputs, hardware and endurance readiness remain open.
