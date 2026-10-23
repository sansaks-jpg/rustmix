//! Reference-compatible control surface over the headless engine.
//!
//! Two layers, matching the contract observed on the analysed reference
//! switcher (see `rustmix-reference/vmix`, evidence `state-document-fresh.xml`
//! and `control-functions.txt`, vMix 29.0.0.48):
//!
//! * [`document`] renders engine state as the XML state document — same
//!   element/attribute vocabulary, capitalised `True`/`False` booleans and
//!   self-closing empty overlay channels. The `<version>` text identifies
//!   *this* implementation and never impersonates the reference product.
//! * [`control`] dispatches `GET /api/?Function=<name>&Input=<n>&Value=<v>`
//!   requests onto typed engine commands, and [`server`] exposes both over a
//!   loopback-only HTTP listener that is off by default.
//!
//! Honest divergences from the reference document, all deliberate:
//! * `type` attributes use rustmix's own source vocabulary (Video, Image,
//!   AudioFile, Capture, Browser, Photos, AudioInput, or the generator kind
//!   name) — the engine does not model the reference's forty input types yet.
//! * all sixteen overlay channels render empty: the engine's overlay bank is
//!   caption panels without input bindings, so claiming an occupied channel
//!   would be false.
//! * `key` attributes are deterministic GUID-shaped identifiers derived from
//!   the stable input ID (`00000000-0000-0000-0000-<id>`), unique per input
//!   within a session.

pub mod control;
pub mod document;
pub mod server;

pub use control::{DispatchError, Request, dispatch, parse_query};
pub use document::{Telemetry, render};
pub use server::{DEFAULT_PORT, Server};
