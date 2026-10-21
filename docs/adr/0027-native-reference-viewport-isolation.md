# ADR 0027: Native reference viewport isolation

Date: 2026-10-21
Status: observed viewport isolation verified in ba3b209/run 39540613350; GPU reliability open.

UI-024 added a final 1917x1198 workspace capture to the main native smoke. All
native processes share a disposable eframe window store. The new final size is
therefore inherited by the later source/input process, previously1600x900.
acb00d7/run 39538665307 confirms its input samples at 1917x1198: main/reopen drain
successfully, but input capture stage 24 times out at 150 s with a pending readback.
Shutdown drain then times out and wgpu Queue::drop panics, exit-1073741819.
This demonstrates a leaked workload change; it does not establish that viewport
size causes every historical GPU failure or that reducing it fixes the driver.

After both large reference captures have actually completed, request1600x900,
wait for the native logical viewport to match, validate monitor geometry, record
the restoration JSON and only then close the main process. Require that evidence
in the hosted script. This preserves the large visual checks while isolating them
from the existing input/audio/video workflows. Keep all original time limits,
readback/drain requirements, error handling, picture/state assertions and cadence
targets. Do not bypass a failed GPU drain, suppress the destructor panic, or claim
a production GPU recovery/performance fix from a passing short hosted run.
