# ADR 0026: Input category navigation and unavailable actions

Date: 2026-10-21
Status: candidate; exact hosted UI and native workflow evidence required.

Opening Add Input reconstructs the active page from InputKind. That loses the
operator's choice because file video, PNG and colour share the Color kind, while
audio and procedural bars share Bars. Reopening a Video or Audio selection thus
shows an unrelated generator page. The production quick menu also omits actual
video/image/audio/microphone input routes and calls a synthetic studio pattern a
virtual set. Retain the selected category, and route each quick choice through
its existing real configuration page with an accurate label. Leave the initial
Generators page and existing validated media/source commits intact.

Unavailable Streaming/MultiCorder Start controls retain a bright green active
fill, and Streaming Save and Close is enabled despite not saving anything.
Use neutral disabled Start controls and disable unsupported Save and Close with
its reason. Keep Close interactive. Do not start jobs, change engine state,
enable unsupported backends or claim that presentation equals functionality.

Exercise actual quick-menu clicks, retained category after Cancel/reopen, page
fields, and unchanged project/output state. Regress disabled painted controls
and click behavior. The native MP4 workflow must enter via the quick Video file
menu, validate/decode the generated fixture, switch, save and reopen as before.
Inspect the exact new native Streaming/MultiCorder images. Light theme remains
outside this follow-up's focus. No dependency, schema or proprietary asset added.
