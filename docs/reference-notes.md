# Reference evidence notes

Clean-room record of vMix behavior re-derived from the installed vMix
29.0.0.48 (binary SHA-256 `05a67e53c184d8547d2668cac3814dce34cd40e4cb0fa3e6df28acdd5a57fad5`,
recorded in the UI-039 gaps entry) and its locally cached official
documentation, used by the UI-041 verification sweep (2026-10-24) to check
the UI-040/`101c41f` alignment work. These notes pin behavior contracts for
RustMix's own tests only — no vendor code, assets or proprietary art enter
this repository (AGENTS §2, §15).

Strength taxonomy per answer:

- **direct** — stated by the install's own files, configuration, extracted
  identifier/enum strings, or the official documentation sampled during the
  pass.
- **inferred** — strong inference from install evidence; no inspected source
  states it outright. RustMix must not treat inferred items as contracts
  beyond what its own tests already pin.

## Overlays

| Question | Re-derived answer | Strength |
|---|---|---|
| What do `OverlayInput1–8`, `OverlayInput<n>Off` and `OverlayInput<n>Out` do? Is Off a clear while Out is not? | `OverlayInput1–8` toggles channel N on/off with the selected input using the channel's configured transition. **Off is the clear**: it immediately switches the channel off as a cut. **Out is the animated exit** (reverse of In), not an immediate clear. `Last` re-toggles using the channel's last-used input; `Zoom` toggles PiP/fullscreen; `OverlayInputAllOff` clears every channel; `PreviewOverlayInputN` previews without Program effect. | direct |
| Are overlay channels 9–16 the `Stinger1–8` slots? | `Stinger1–8` are transition functions (parameters Input, Mix) that switch Preview to Output under a pre-configured animation whose cut point swaps the picture. The state document exposes sixteen overlay channels while the wire vocabulary is only `OverlayInput1–8` plus `Stinger1–8`. That channels 9–16 are the stinger slots is a strong inference (stinger cut point, animation input GUID and display-underneath flag persist inside the same 16-slot, index 0–15 OverlaySettings array), but **no inspected doc states the numbering outright**. | inferred |
| What does the state XML look like for an OCCUPIED overlay channel? | Top-level `<overlays>` with sixteen `<overlay number="N">` elements. **An occupied channel carries the bound input's NUMBER as element text** — e.g. `<overlay number="2">1</overlay>` means channel 2 runs input 1 — not the GUID or title; empty channels are self-closed. Keys appear only in each input's own nested `<overlay index=… key=…/>` elements, which describe that input's internal layers (a separate concept). *Outcome this sweep: this shape moved from "claimed unevidenced" to direct evidence (official DeveloperAPI.html sample) and is now implemented and tested — see testing.md UI-041.* | direct |
| Can an overlay channel composite a bound input (PiP) over Program, and whose geometry applies? | Yes. Each channel's Overlay Settings offers Type Fullscreen or Picture In Picture; the input composites over Program using **that channel's own pan/zoom geometry** with optional border (colour/thickness/radius) and alpha. Positioning/scaling is the channel's, persisted per channel as `MatrixPositionN` — not the bound input's own Layers/Position settings. The input contributes its pixels (alpha channel and colour key honored). `OverlayInput<n>Zoom` animates that channel between fullscreen fill and its PiP box. | direct |

## Audio

| Question | Re-derived answer | Strength |
|---|---|---|
| Which audio buses exist and how are per-input sends configured? | Eight stereo mixes: Master (M) plus Buses A–G, and a separate Headphones monitoring output. Master is always enabled; A–G must be enabled by assigning a device (or "Enabled") in Settings → Audio Outputs. Per input, the mixer strip has M/A–G buttons, each a leveled send (`SetVolumeBusMixerM`/`A`–`G`), plus a per-input channel matrix to any bus channel with savable presets. New inputs default to a configurable bus (default Master). Headphones has its own volume and channel pair. | direct |
| What meter values surface per input/bus, and where? | UI: per-input meters beside input buttons, a Master meter in the main window, a dockable Audio Mixer pane, and per-input Audio Settings showing "Pre" and "Post" VU pairs with peak readouts in dBFS (default `-Inf`). Control surfaces: `OnAudioMeterDB6/12/18` activators. State XML: the master element carries volume (0–100), muted, `meterF1`/`meterF2` (observed as 0), `headphonesVolume`; inputs carry volume/muted/`meterF1`/`meterF2`. **Meter ballistics (attack/decay) are not evidenced anywhere inspected; the `meterF` scale/unit is not officially documented.** | direct |
| Are input meters taken pre or post gain/fader? | Post-fader by default, switchable to pre-fader via a global "Meters Pre-Fader" settings checkbox. The per-input Audio Settings dialog always shows both Pre and Post VU pairs with separate peak readouts and its own per-input pre-fader option. Solo feeds Headphones monitoring and defaults to PFL vs AFL, set globally and per input. | direct |
| What mix-minus and audio-follow-video semantics are evidenced? | Audio-follow-video is "Audio Auto": per-input `AudioAuto`/`On`/`Off` functions under a global "Automatically Mix Audio" setting, default True. Mix-minus is not a named feature; it is achieved with buses: a Video Call input's return feed ("Return Feed (To Guest)") has an Audio Source selectable from Master, Headphones, or BusA–G (`VideoCallAudioSource`), so the guest hears a bus excluding their own input; `BusXSendToMaster` folds bus mixes back to Master. vMix Call requires the 48 kHz master sample rate. | direct |

## Transitions

| Question | Re-derived answer | Strength |
|---|---|---|
| What is the complete transition enum? | `vMix.TransitionEffect` with exactly 20 named members (21 field rows counting `value__`), in metadata declaration order: Fade, Zoom, Wipe, Slide, Fly, CrossZoom, FlyRotate, Cube, CubeZoom, VerticalWipe, VerticalSlide, Merge, WipeReverse, SlideReverse, VerticalWipeReverse, VerticalSlideReverse, BarnDoor, RollerDoor, AlphaFade, Cut. `Stinger1–8` are transition buttons/functions configured in Overlay settings, **not** members of this enum. | direct |
| Any static evidence of transition curves, easing, durations or defaults? | Durations are configurable and **500 ms is the recurring default**: `QuickPlayDuration=500` and `FTBDuration=500` in vMix64.exe.config; per-slot `Duration=500` across all 16 OverlaySettings slots (`Effect=0` or `100`, `CutPoint=0`) in the 29.0.0.48 user.config and last.vmix. **No easing/curve parameter exists anywhere** — no easing/smoothstep/interpolation identifiers for transitions in vMix64.exe, vMixNative.dll, D2DEngine.dll (only Direct2D image-scaling constants) or the Phoenix DLLs; docs document only durations. The lone "Smooth" string is a Transition Display mode (Smooth/LowLatency/Legacy), not a curve. | direct |
| Do overlay/PiP inputs composite at live time during and after transitions? | Docs establish that overlays composite live on screen while a transition runs: the stinger page's "Display underneath overlays" option exists precisely because overlays (1–8) "currently on screen" remain visible during the transition animation; a previewed overlay "will be sent to the Output automatically when the next transition or cut occurs"; Merge transitions animate shared layers individually (others fade) — the live layer stack is transitioned, not flattened stills. Install side: each of 8 overlay channels has its own transition effect+duration, and render-target surface names (Output/MultiView/PictureInPicture) indicate per-input live surfaces. **That animated PiP content visibly keeps moving during an ordinary (non-stinger) transition is implied but stated nowhere verbatim.** | direct |
| Ordinal and default-value semantics? | `QuickPlayTransition` default = 0 (index of Fade **if** ordinals follow declaration order — ordinals were NOT read from the Constant table, so Fade=0..Cut=19 stays inferred); stinger-group slots default `Effect=100` (docs say the to/from stinger effect is "in most cases… Cut", so 100 likely encodes the stinger-effect default; slot mapping inferred); playlist entries carry per-item `m_Transition` + `m_TransitionDuration` + `m_Stinger`; T-Bar tooltip: "Manual fade using first transition effect." | inferred |

## Transport

| Question | Re-derived answer | Strength |
|---|---|---|
| What does `PlayPause` do exactly? | A single input-scoped trigger taking only an Input parameter and no Value — **it toggles**: pause if the input is playing, play if paused. The product's own registry carries no description for it (unlike Restart/Loop), so the toggle reading rests on the naming pair Play/Pause/PlayPause, third-party integration docs ("Switch Play & Pause for an input"), and the per-input `InputPlaying` feedback activator. `LivePlayPause` is the same but always targets the live/Program input. For transport-less inputs (stills, capture) nothing documents behavior; a forum report on the related `Pause` says it "has no effect" — best treated as a silent no-op, not an error. | direct |
| What are the preset/input category labels? | Exactly: All, Red, Green, Orange, Purple, Aqua, Blue, Custom1–Custom16, Search. **Aqua and Blue are real categories; Azure does not exist anywhere in vMix64.exe (zero case-insensitive hits); MidnightBlue is not a category** — it occurs only as a System.Drawing Color property getter used for UI colouring. The main window's per-category filter buttons are created dynamically; no `cmdInputs*` identifiers exist in the binary. | direct |
| Full shortcut/API function table for the five families? | Registry re-extracted from vMix64.exe identifier/enum strings; every name verified present as a standalone string. OVERLAY (62): `OverlayInput1–8` each with Off/In/Out/Zoom/Last (48), `PreviewOverlayInput1–8`, `OverlayInputAllOff`, `SetMultiViewOverlay`, `MoveMultiViewOverlay`, `MultiViewOverlay`/On/Off. TRANSITION (32): `FadeToBlack`, `SetFader` (T-bar, 255 cuts to Preview), `QuickPlay`, `Transition1–4`, `Stinger1–8`, `SetStingerGTInput1–8`, `CutDirect`, `SetTransitionEffect1–4`, `SetTransitionDuration1–4`; wire aliases Cut and Fade accepted per docs intro and used in shipped templates and the embedded web controller. AUDIO (107): `Audio`/On/Off, `AudioAuto`/On/Off, `SetVolume`, `SetVolumeFade`, `SetVolumeChannel1/2`, `SetVolumeChannelMixer1–16`, `SetVolumeBusMixer(M,A–G)`, `SetBalance`, `SetGain(+Ch1/2)`, `SetMasterVolume(+Fade)`, `SetHeadphonesVolume`, `SetBusA–GVolume(+Fade)`, `MasterAudioOn/Off`, `BusA|BusB|BusX` `AudioOn/Off`, `BusXSendToMaster(On/Off)`, `BusXSolo(On/Off)`, `Solo`/On/Off/AllOff, `SoloPFL`/On/Off, `AudioBus`/On/Off, `AudioMixerShowHide`, `AudioPlugin(Show/On/Off/OnOff)` for Input/Master/BusA/BusB/BusX, `AudioChannelMatrixApplyPreset`, `MasterAudio`, `BusAAudio`, `BusBAudio`, `BusXAudio`. TRANSPORT/PLAYBACK: `PlayPause`, `Play`, `Pause`, `Restart`, `Loop`/`LoopOn`/`LoopOff`, `LivePlayPause`, `ResetInput`, `SetPosition`, `SetRate`, `SetRateSlowMotion`, `SelectIndex`, `NextItem`/`PreviousItem`, `NextPicture`/`PreviousPicture`, `SaveVideoDelay`, `VideoDelay(Start/Stop/StartStop)Recording`, `MarkIn`/`MarkOut`/`MarkReset`/`MarkResetIn`/`MarkResetOut`, `SetCountdown`/`StartCountdown`/`PauseCountdown`/`StopCountdown`/`AdjustCountdown`/`ChangeCountdown`/`SuspendCountdown`, `Next`/`Previous`/`Start`/`Stop`/`SelectPlayList`, `AutoPlayNext(On/Off)`, `AutoPlayFirst(On/Off)`, `ListPlayOut`/`ShowHide`/`Add`/`Remove`/`RemoveAll`/`Export`/`Shuffle`, DataSource `PlayPause`/`Play`/`Pause`/`AutoNext(OnOff/On/Off)`/`PreviousRow`/`NextRow`/`SelectRow`, Replay `PlayPause`/`Play`/`Pause`/`FastForward`/`FastBackward`/`JumpFrames(+FastOn/Off)`/`ChangeSpeed`/`SetSpeed`. CATEGORY/PRESET: `SelectCategory` (values All,Red,Green,Orange,Purple,Aqua,Blue,Custom1-16,Search), `SetDynamicInput1–4`, `SetDynamicValue1–4`, `OpenPreset`, `SavePreset`, `LastPreset`; adjacent input management: `AddInput` (Value syntax `Video|path`, `Image|path`, `Photos|folder`, `Title|.gtzip`, `VideoList|.m3u`, `Colour|HTMLColor`, `AudioFile|`, `Flash|`, `PowerPoint|path`), `RemoveInput`. | direct |
| Transport setup|arm pairs on the main window? | **Five pairs, not six** (an earlier reference doc's "six transports" sentence was a miscount; its own table lists five). All ten control names exist in vMix64.exe, adjacent in designer order: (1) `cmdRecSetup` | `cmdRecStartStop` "Record" (the setup button doubles as the live recording timer); (2) `cmdExternal` | `cmdDeckLinkRender` "External" (id names a capture brand, label generic; wire functions StartExternal/StopExternal/StartStopExternal); (3) `cmdStreamingSetup` | `cmdStreaming` "Stream"; (4) `cmdMultiCorderSetup` | `cmdMultiCorderStartStop` "MultiCorder"; (5) `cmdPlayListSetup` | `cmdPlaylist` "PlayList". To the right the pattern stops (single icon buttons). SRT output has functions but no main-bar pair. | direct |

## Remains unevidenced

Behavior below has **no** direct reference evidence. Do not implement
against it as if it were a contract; record new evidence first if it must be
pinned.

- **Overlay channels 9–16 ≡ Stinger1–8 numbering** — inferred only (same
  16-slot persisted OverlaySettings array); no doc states the numbering
  outright. RustMix's "channels 9-16 are Stingers 1-8" control-API reason
  rests on this inference.
- **Transition ordinals** Fade=0..Cut=19 — inferred from declaration order;
  the Constant table was not read. **Stinger-group `Effect=100`** encoding
  the stinger-effect default — inferred.
- **Animated PiP visibly keeping motion during an ordinary (non-stinger)
  transition** — implied by the live-layer-stack evidence, stated nowhere
  verbatim. RustMix's clocked PiP renders motion continuously; no parity
  claim is made.
- **`meterF1`/`meterF2` scale/unit** — observed values only (0 in sampled
  state), not officially documented; **meter ballistics** (attack/decay)
  not evidenced anywhere inspected.
- **`PlayPause` on transport-less inputs** (stills, capture) — undocumented;
  one forum report says the related `Pause` "has no effect". RustMix rejects
  it with the engine's disabled reason rather than silently no-oping, which
  is a deliberate divergence until evidence exists.
- **Overlay state-XML occupied shape — resolved this sweep**: previously
  claimed unevidenced by a repository comment; the fresh pass produced the
  official DeveloperAPI.html sample (direct) and the shape is now
  implemented and regression-tested (testing.md UI-041,
  `occupied_overlay_channels_carry_the_bound_input_number_as_element_text`).
  Nothing about the standard-channel occupied form remains unevidenced.
