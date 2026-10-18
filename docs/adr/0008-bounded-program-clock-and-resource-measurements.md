# ADR 0008: bounded Program producer and measured resource use

Status: implemented, hosted verification pending (UI-011).

The UI requested a repaint every 33 ms and previously rendered Program inside
that callback. Choosing 60p could not make that path independently produce 60
frames per second. Transient RGBA flatten/copy buffers and full-rate thumbnail
updates also did unnecessary work.

An offline Program worker now receives one replaceable immutable engine/assets
snapshot and publishes one replaceable RGBA frame. It never queues obsolete
frames for catch-up. Per-source and composite caches retain only active
dependencies, with at most 32 source surfaces and one composite at 640x360.
Decoded PNG buffers and unchanged frames use shared immutable storage. Shutdown
signals and joins the worker. Operator commands publish promptly; source edit
transactions publish after their matching image cache is committed.

The rational clock computes deadlines from integer frame indices, including
60000/1001. A late wake skips obsolete frame indices and records the count.
On Windows the worker uses the high-resolution waitable timer behind [Rust 1.94 thread::sleep](https://github.com/rust-lang/rust/blob/1.94.1/library/std/src/sys/thread/windows.rs), with a 10 ms maximum sleep and no busy spin.
Nanosecond metadata does not claim an audio/device clock or hard real-time
Windows scheduling. Source procedural sampling is still millisecond based.
Thumbnails update at up to 15 fps; the Program worker follows the selected rate.
UI FPS, produced Program FPS and skipped frames are separate measured values.
The status shows the actual 640x360 raster instead of implying full HD from the
project preference. Record/stream/external remain disabled.

The native harness samples process working set, private bytes and CPU time for
every launched process. An opt-in measurement exercises moving Program at 50,
60000/1001 and 60 fps with seven procedural inputs, using actual production
timestamps/counts. It records missed deadlines and whether each short sample
met its target. A failed cadence target is reported as unqualified even when
functional tests/package checks pass. RAM samples exclude dedicated VRAM and
do not establish a memory leak/endurance guarantee. No physical AMD test occurs.

Progressive rate references: [ITU-R BT.709-6](https://www.itu.int/rec/R-REC-BT.709/en)
and [vMix Master Frame Rate / Output Size](https://www.vmix.com/help29/Display.html).
They inform timing and UI terminology; this SDR CPU prototype does not claim
BT.709 color compliance, 1080p60 broadcast output, decoding/capture, real audio,
encoder support, GPU compositor parity or production qualification.
