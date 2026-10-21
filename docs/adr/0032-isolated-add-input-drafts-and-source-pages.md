# ADR 0032: Isolated Add Input drafts and truthful source pages

Date: 2026-10-21
Status: implemented source candidate; hosted compile/test/native acceptance pending.
Scope: ADD-UX-01 through ADD-UX-06 from the focused Add Input audit.

The existing Add Input dialog shares a filename/name field and a decoded PNG
draft across generator pages. Its commit handler infers file import from label
punctuation. Selecting Colour or Title after a validated PNG can therefore add
the old image. Image/Colour/Title share a generic generator form; WAV OK ignores
readiness; source icons cycle by ordinal and unavailable categories cannot be
inspected. The supplied screenshots are behavioral references, not assets.

Before implementation, acceptance was specified in `docs/add-input-uiux-audit.md`:
PNG → Colour/Title/generator → OK must preserve source identity and Program/Preview;
PNG path edits must invalidate pixels; generator names containing a dot must remain
labels; unvalidated media, capacity and transition blockers must disable OK;
unavailable pages must never create dummy inputs. Compact/large form geometry,
keyboard navigation, native screenshots and source-isolation regressions are required.

Keep these concerns in ui-egui. `add_input` owns presentation, explicit category
order/icon semantics, selection cleanup and queries of the engine's AddInputAt
preconditions. Engine commands remain the only project mutation path. No backend,
dependency, project schema, dock schema or platform crate changes are introduced.

Use a separate PNG path, decoded-image draft and inline error. Only Image can
commit its validated PNG. Remove the media worker's automatic Add target: file
checking publishes a draft, and only a later explicit OK commits. Selecting a
different category retires incompatible drafts and advances the token. A pending
video probe remains tracked across category changes until completion; its old
token cannot restore metadata or permit overlapping category-triggered probes.
Cancel retains the existing probe cancellation policy.

Image gets a path/Browse row, actual image preview, Check image, Clear selection
and status. Clear selection never deletes the media. Generators get a label and
their original sample library. Labels containing dots/slashes are not file imports.
Colour explicitly offers the existing gradient/bars generators; solid-colour and
transparent source creation remain unavailable. Title is an inspectable unavailable
page; the original lower-third sample belongs to Generators. Countdown is named
as a generator, not GT template compatibility.

Keep existing category ordinals/legacy automation aliases; order the reference
source categories deliberately and add unavailable DVD, Camera, NDI/OMT/Desktop,
Video Delay and Telestrator pages. Sidebar navigation stays interactive when a
backend is absent; OK is disabled with a reason. Quick routes share the same
category-selection helper. Each category has a stable automation ID, selected
accessibility metadata, focus feedback and an original semantic vector icon.
Opening a retained/quick category reveals it in the scrolling navigation.

The WAV page aligns path/Browse, validation details and name; microphone remains
an unarmed descriptor selector. OK queries current validation plus capacity,
position and transition preconditions. The engine still revalidates the actual
commit. No microphone, network guest, recorder, stream or SDK installation is
started by selecting/inspecting a category.

Prepared validation adds seven focused UI regressions and one pending-video
probe regression. Existing PNG, WAV and quick-route regressions are adapted to
the explicit source/check contract; their source identity, malformed-file,
save/reopen and Program/Preview assertions remain. The independent mandatory
`--ui-add-smoke` process generates a procedural PNG and uses actual sidebar and
Cancel clicks, requires six native PNG/element captures, checks draft retirement,
readiness and unchanged full project/output state, and retains stage/readback/GPU
drain/error gates. Its own limit is 180 seconds; existing limits are not relaxed.

Local formatting, repository inventory/layer/asset guard, diff inspection and
PowerShell syntax are permitted. No local Rust compile, Rust tests, executable
build or native launch. A new exact-revision hosted run and visual review are
required; the old green run does not validate this candidate. Physical hardware,
DPI, cadence, endurance, full form/reference equivalence and missing backends stay open.
