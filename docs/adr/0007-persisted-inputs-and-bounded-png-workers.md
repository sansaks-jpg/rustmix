# ADR 0007: persisted input configuration and bounded PNG workers

Status: experimental functional acceptance recorded for UI-010 and included in UI-012 run39111788801. Schema4 extends this design in ADR0011; new changes require UI-013 acceptance.

The UI previously kept decoded PNGs and effects only in an egui map. Saving a
project lost both, Cancel could leave effects active, Number did not affect
insertion, and Duplicate created a different procedural source.

Schema 2 stores typed `InputConfig` alongside each engine input. Schema 1 files
migrate to identity processing on load. Future versions remain rejected. PNGs
are external file references, not embedded media; older binaries cannot open
schema 2. Atomic saves, backup generations and size limits remain enforced.
Backup rotation preserves the exact validated bytes of the previous file, so
the schema 1 rollback copy remains usable by an older application after migration.

`input.add`, `input.duplicate` and `input.apply` validate complete candidate
states. Insertion preserves Program/Preview IDs; duplicates copy source/effects
and start muted. Layer references must exist and form an acyclic graph; removal
clears dependent references. Applying source edits preserves concurrent mixer
changes unless audio was explicitly edited in that dialog.

The editor owns a draft and decoded-image cache. OK commits the validated model
and matching cache together; Cancel discards them. Invalid Apply leaves the
editor open. The renderer uses committed state, while the editor shows its own
preview. Imported images offer Contain/Cover/Stretch and alpha over black.

Source pickers run on a separate worker and cancellation preserves selection. PNG decode runs on one worker with a capacity-32 request queue and one result
slot. File reads stop at 32 MiB + 1, dimensions are restricted to 4096 per axis,
and the decoder allocation budget is 64 MiB (upstream documents allocation
limits as best effort). Cached rasters fit 640x360; shared buffers avoid copying
the entire cache on every settings read. Project generations and editor tokens
discard obsolete results. Loading or missing PNGs show black with a visible
status, retaining the descriptor for relinking; no generator substitution.

Acceptance requires engine migration/roundtrip/atomicity/cycle/removal/duplicate
tests, PNG limits/corruption tests, real widget OK/Cancel/import tests, missing
file/relink and late-result tests, plus hosted native import/order/effects/
duplicate/save and a second-process reopen with matching Program pixels.

This is the still-image portion of M2. It introduces no new dependency or codec
runtime. Video, capture, real audio, GPU composition and output remain separate
unimplemented backends. Physical devices and endurance are not qualified.
