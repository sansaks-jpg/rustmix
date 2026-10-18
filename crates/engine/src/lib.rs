#![forbid(unsafe_code)]
pub mod audio;
pub mod input;
pub use input::{ImageFit, InputConfig};
pub mod persistence;
pub mod playback;
pub mod production;
pub use production::{
    Playlist, PlaylistItem, PlaylistRun, TransitionSlot, default_playlists,
    default_transition_slots,
};
pub mod render;
pub mod shortcuts;
pub use shortcuts::{ShortcutAction, ShortcutBinding, default_key_bindings};
pub mod video_clock;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Invalid(String),
    #[error("Project file operation failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid project JSON: {0}")]
    Json(#[from] serde_json::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
fn invalid(message: &str) -> Error {
    Error::Invalid(message.into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InputId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rate {
    pub numerator: u32,
    pub denominator: u32,
}
impl Rate {
    pub fn new(numerator: u32, denominator: u32) -> Result<Self> {
        if numerator == 0
            || denominator == 0
            || numerator > 120_000
            || denominator > 10_000
            || !(1.0..=120.0).contains(&(numerator as f64 / denominator as f64))
        {
            return Err(invalid("Frame rate must be between 1 and 120 fps"));
        }
        let (mut a, mut b) = (numerator, denominator);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        Ok(Self {
            numerator: numerator / a,
            denominator: denominator / a,
        })
    }
    pub fn label(self) -> String {
        format!(
            "{:.2}",
            self.numerator as f64 / self.denominator.max(1) as f64
        )
    }
    pub fn frame_at(self, millis: u64) -> u64 {
        millis.saturating_mul(u64::from(self.numerator))
            / (1000 * u64::from(self.denominator.max(1)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputKind {
    Grid,
    Geometry,
    Bars,
    Graphics,
    Studio,
    Color,
    Countdown,
}
impl InputKind {
    pub const ALL: [Self; 7] = [
        Self::Grid,
        Self::Geometry,
        Self::Bars,
        Self::Graphics,
        Self::Studio,
        Self::Color,
        Self::Countdown,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Grid => "Motion grid",
            Self::Geometry => "Geometry",
            Self::Bars => "Color bars",
            Self::Graphics => "Graphics",
            Self::Studio => "Virtual studio",
            Self::Color => "Color generator",
            Self::Countdown => "Countdown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioChannel {
    pub gain_db: f32,
    pub muted: bool,
    pub solo: bool,
    pub sends: [bool; 3],
}
impl Default for AudioChannel {
    fn default() -> Self {
        Self {
            gain_db: 0.0,
            muted: false,
            solo: false,
            sends: [true, false, false],
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputState {
    pub id: InputId,
    pub name: String,
    pub kind: InputKind,
    pub color: [u8; 3],
    pub online: bool,
    pub audio: AudioChannel,
    #[serde(default)]
    pub config: InputConfig,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Position {
    LowerLeft,
    LowerRight,
    TopLeft,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OverlayState {
    pub enabled: bool,
    pub title: String,
    pub subtitle: String,
    pub position: Position,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preferences {
    pub width: u32,
    pub height: u32,
    pub rate: Rate,
    pub high_contrast: bool,
    pub ui_scale: f32,
    pub shortcuts: bool,
    #[serde(default = "default_key_bindings")]
    pub key_bindings: Vec<ShortcutBinding>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceState {
    pub name: String,
    pub dock_json: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectState {
    pub schema: u32,
    pub name: String,
    pub revision: u64,
    pub inputs: Vec<InputState>,
    pub program: InputId,
    pub preview: InputId,
    pub duration_ms: u32,
    #[serde(default = "default_transition_slots")]
    pub transition_slots: [TransitionSlot; 4],
    #[serde(default = "default_playlists")]
    pub playlists: Vec<Playlist>,
    pub buses: [AudioChannel; 3],
    pub overlays: [OverlayState; 4],
    pub preferences: Preferences,
    pub workspace: WorkspaceState,
}
impl ProjectState {
    pub fn demo() -> Self {
        let names = [
            "Camera A",
            "Camera B",
            "Camera C",
            "Graphics",
            "News Background",
            "Color Source",
            "Countdown",
        ];
        let colors = [
            [38, 156, 192],
            [227, 147, 65],
            [100, 166, 142],
            [47, 139, 209],
            [40, 112, 167],
            [114, 79, 183],
            [35, 174, 157],
        ];
        let inputs = names
            .into_iter()
            .zip(InputKind::ALL)
            .zip(colors)
            .enumerate()
            .map(|(i, ((name, kind), color))| InputState {
                id: InputId(i as u32 + 1),
                name: name.into(),
                kind,
                color,
                online: true,
                audio: AudioChannel {
                    muted: i > 1,
                    ..Default::default()
                },
                config: InputConfig::default(),
            })
            .collect();
        Self {
            schema: 3,
            name: "Studio / Demo production".into(),
            revision: 0,
            inputs,
            program: InputId(1),
            preview: InputId(2),
            duration_ms: 1000,
            transition_slots: default_transition_slots(),
            playlists: default_playlists(),
            buses: std::array::from_fn(|_| AudioChannel::default()),
            overlays: std::array::from_fn(|i| OverlayState {
                enabled: false,
                title: if i == 0 {
                    "RUSTMIX / STUDIO".into()
                } else {
                    format!("Overlay {}", i + 1)
                },
                subtitle: "Independent live production • Synthetic demo".into(),
                position: Position::LowerLeft,
            }),
            preferences: Preferences {
                width: 1920,
                height: 1080,
                rate: Rate {
                    numerator: 30,
                    denominator: 1,
                },
                high_contrast: false,
                ui_scale: 1.0,
                shortcuts: true,
                key_bindings: default_key_bindings(),
            },
            workspace: WorkspaceState {
                name: "Production".into(),
                dock_json: String::new(),
            },
        }
    }
    pub fn input(&self, id: InputId) -> Option<&InputState> {
        self.inputs.iter().find(|s| s.id == id)
    }
    pub fn input_number(&self, id: InputId) -> usize {
        self.inputs
            .iter()
            .position(|s| s.id == id)
            .map_or(0, |index| index + 1)
    }
    pub fn validate(&self) -> Result<()> {
        if self.schema != 3 {
            return Err(invalid(
                "Unsupported project version; this application supports schema 3",
            ));
        }
        valid_text(&self.name, 120)?;
        if !(2..=32).contains(&self.inputs.len()) {
            return Err(invalid("A project needs 2–32 inputs"));
        }
        let mut ids = HashSet::new();
        for input in &self.inputs {
            if input.id.0 == 0 || !ids.insert(input.id) {
                return Err(invalid("Invalid or duplicate input ID"));
            }
            valid_text(&input.name, 64)?;
            valid_audio(&input.audio)?;
            input.config.validate()?;
        }
        for input in &self.inputs {
            let mut seen = HashSet::from([input.id]);
            let mut dependency = input.config.pip_source;
            while let Some(id) = dependency {
                if !seen.insert(id) {
                    return Err(invalid("Input layers cannot contain a cycle"));
                }
                let source = self
                    .input(id)
                    .ok_or_else(|| invalid("Layer source is missing"))?;
                dependency = source.config.pip_source;
            }
        }
        if self.input(self.program).is_none() || self.input(self.preview).is_none() {
            return Err(invalid("Program or Preview references a missing input"));
        }
        if !(100..=10000).contains(&self.duration_ms) {
            return Err(invalid("Transition duration must be 100–10000 ms"));
        }
        for a in &self.buses {
            valid_audio(a)?;
        }
        for overlay in &self.overlays {
            valid_text(&overlay.title, 96)?;
            if overlay.subtitle.len() > 160 || overlay.subtitle.chars().any(char::is_control) {
                return Err(invalid(
                    "Subtitle is too long or contains control characters",
                ));
            }
        }
        let p = &self.preferences;
        if !matches!(
            (p.width, p.height),
            (1280, 720) | (1920, 1080) | (2560, 1440) | (3840, 2160)
        ) {
            return Err(invalid("Choose a supported 16:9 canvas size"));
        }
        if Rate::new(p.rate.numerator, p.rate.denominator)? != p.rate {
            return Err(invalid("Frame rate must be reduced"));
        }
        if !p.ui_scale.is_finite() || !(0.8..=1.5).contains(&p.ui_scale) {
            return Err(invalid("UI scale must be 0.8–1.5"));
        }
        valid_text(&self.workspace.name, 64)?;
        if self.workspace.dock_json.len() > 65536 {
            return Err(invalid("Workspace layout exceeds 64 KiB"));
        }
        production::validate(self)?;
        shortcuts::validate(self)?;
        Ok(())
    }
}
fn valid_text(s: &str, max: usize) -> Result<()> {
    if s.trim().is_empty() || s.len() > max || s.chars().any(char::is_control) {
        Err(invalid(
            "Text is empty, too long, or contains control characters",
        ))
    } else {
        Ok(())
    }
}
fn valid_audio(a: &AudioChannel) -> Result<()> {
    if !a.gain_db.is_finite() || !(-60.0..=12.0).contains(&a.gain_db) {
        Err(invalid("Gain must be finite and between -60 and +12 dB"))
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionKind {
    Cut,
    Fade,
    Wipe,
    WipeReverse,
    VerticalWipe,
    VerticalWipeReverse,
    Slide,
    SlideReverse,
    VerticalSlide,
    VerticalSlideReverse,
    BarnDoor,
    RollerDoor,
}
impl TransitionKind {
    pub const ALL: [Self; 12] = [
        Self::Cut,
        Self::Fade,
        Self::Wipe,
        Self::WipeReverse,
        Self::VerticalWipe,
        Self::VerticalWipeReverse,
        Self::Slide,
        Self::SlideReverse,
        Self::VerticalSlide,
        Self::VerticalSlideReverse,
        Self::BarnDoor,
        Self::RollerDoor,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Cut => "Cut",
            Self::Fade => "Fade",
            Self::Wipe => "Wipe",
            Self::WipeReverse => "Wipe reverse",
            Self::VerticalWipe => "Vertical wipe",
            Self::VerticalWipeReverse => "Vertical wipe reverse",
            Self::Slide => "Slide",
            Self::SlideReverse => "Slide reverse",
            Self::VerticalSlide => "Vertical slide",
            Self::VerticalSlideReverse => "Vertical slide reverse",
            Self::BarnDoor => "Barn door",
            Self::RollerDoor => "Roller door",
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionState {
    pub from: InputId,
    pub to: InputId,
    pub kind: TransitionKind,
    pub start_ms: u64,
    pub duration_ms: u32,
    pub progress: f32,
    pub manual: bool,
}
#[derive(Debug, Clone)]
pub enum Command {
    SelectPreview(InputId),
    Cut,
    TakeInput(InputId),
    Transition(TransitionKind),
    SetManualProgress(f32),
    CancelTransition,
    SetDuration(u32),
    SetTransitionSlot(usize, TransitionSlot),
    RunTransitionSlot(usize),
    AddPlaylist(String),
    UpdatePlaylist(Playlist),
    RemovePlaylist(u32),
    StartPlaylist {
        id: u32,
        selected: usize,
    },
    StopPlaylist,
    PlaylistNext,
    PlaylistPrevious,
    FadeBlack,
    AddInput {
        name: String,
        kind: InputKind,
    },
    AddInputAt {
        name: String,
        kind: InputKind,
        config: InputConfig,
        index: usize,
    },
    DuplicateInput(InputId),
    RemoveInput(InputId),
    UpdateInput(InputState),
    ApplyInput {
        input: InputState,
        audio: Option<AudioChannel>,
    },
    SetInputAudio(InputId, AudioChannel),
    SetBus(usize, AudioChannel),
    SetOverlay(usize, OverlayState),
    SetPreferences(Preferences),
    SetWorkspace(WorkspaceState),
    RenameProject(String),
    SetInputPaused(InputId, bool),
    RestartInput(InputId),
    SetInputLoop(InputId, bool),
    SetInputsPaused(bool),
}
#[derive(Debug, Clone, Serialize)]
pub struct CommandDefinition {
    pub id: &'static str,
    pub version: u32,
    pub label: &'static str,
    pub category: &'static str,
    pub shortcut: &'static str,
    pub scope: &'static str,
    pub idempotent: bool,
    pub live: bool,
}
impl Command {
    pub fn definition(&self) -> CommandDefinition {
        let (id, label, category, shortcut, idempotent, live) = match self {
            Self::SelectPreview(_) => (
                "preview.select",
                "Select Preview",
                "Switcher",
                "1–7",
                true,
                false,
            ),
            Self::Cut => ("program.cut", "Cut", "Switcher", "Space", false, true),
            Self::TakeInput(_) => (
                "program.takeInput",
                "Take input directly",
                "Switcher",
                "",
                false,
                true,
            ),
            Self::Transition(_) => (
                "program.transition",
                "Transition",
                "Switcher",
                "Enter",
                false,
                true,
            ),
            Self::SetManualProgress(_) => ("program.manual", "T-Bar", "Switcher", "", true, true),
            Self::CancelTransition => (
                "program.cancel",
                "Cancel transition",
                "Switcher",
                "Esc",
                true,
                true,
            ),
            Self::SetDuration(_) => ("program.duration", "Duration", "Switcher", "", true, false),
            Self::SetTransitionSlot(..) => (
                "transition.slot.set",
                "Configure transition slot",
                "Switcher",
                "",
                true,
                false,
            ),
            Self::RunTransitionSlot(_) => (
                "transition.slot.run",
                "Run transition slot",
                "Switcher",
                "",
                false,
                true,
            ),
            Self::AddPlaylist(_) => ("playlist.add", "Add playlist", "Playlist", "", false, false),
            Self::UpdatePlaylist(_) => (
                "playlist.update",
                "Update playlist",
                "Playlist",
                "",
                true,
                false,
            ),
            Self::RemovePlaylist(_) => (
                "playlist.remove",
                "Remove playlist",
                "Playlist",
                "",
                false,
                false,
            ),
            Self::StartPlaylist { .. } => (
                "playlist.start",
                "Start playlist",
                "Playlist",
                "",
                false,
                true,
            ),
            Self::StopPlaylist => (
                "playlist.stop",
                "Stop playlist automation",
                "Playlist",
                "",
                true,
                true,
            ),
            Self::PlaylistNext => (
                "playlist.next",
                "Next playlist item",
                "Playlist",
                "",
                false,
                true,
            ),
            Self::PlaylistPrevious => (
                "playlist.previous",
                "Previous playlist item",
                "Playlist",
                "",
                false,
                true,
            ),
            Self::FadeBlack => (
                "program.fadeBlack",
                "Fade to black / restore",
                "Switcher",
                "B",
                false,
                true,
            ),
            Self::AddInput { .. } | Self::AddInputAt { .. } => {
                ("input.add", "Add input", "Inputs", "Ctrl+I", false, false)
            }
            Self::RemoveInput(_) => ("input.remove", "Remove input", "Inputs", "", false, false),
            Self::UpdateInput(_) => ("input.update", "Edit input", "Inputs", "", true, false),
            Self::ApplyInput { .. } => (
                "input.apply",
                "Apply input settings",
                "Inputs",
                "",
                true,
                true,
            ),
            Self::DuplicateInput(_) => (
                "input.duplicate",
                "Duplicate input",
                "Inputs",
                "",
                false,
                false,
            ),
            Self::SetInputAudio(..) => (
                "audio.input.set",
                "Input audio controls",
                "Audio",
                "",
                true,
                false,
            ),
            Self::SetBus(..) => (
                "audio.bus.set",
                "Set bus controls",
                "Audio",
                "",
                true,
                false,
            ),
            Self::SetOverlay(..) => (
                "overlay.set",
                "Set overlay",
                "Graphics",
                "F1–F4",
                true,
                true,
            ),
            Self::SetPreferences(_) => (
                "project.preferences",
                "Preferences",
                "Project",
                "",
                true,
                false,
            ),
            Self::SetWorkspace(_) => ("workspace.set", "Workspace", "View", "", true, false),
            Self::RenameProject(_) => (
                "project.rename",
                "Rename project",
                "Project",
                "",
                true,
                false,
            ),
            Self::SetInputPaused(..) => {
                ("input.pause", "Set source pause", "Inputs", "", true, true)
            }
            Self::RestartInput(_) => ("input.restart", "Restart source", "Inputs", "", true, true),
            Self::SetInputLoop(..) => ("input.loop", "Set source loop", "Inputs", "", true, false),
            Self::SetInputsPaused(_) => (
                "inputs.pause",
                "Pause all sources",
                "Inputs",
                "",
                true,
                true,
            ),
        };
        CommandDefinition {
            id,
            version: 1,
            label,
            category,
            shortcut,
            scope: "local-demo",
            idempotent,
            live,
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct CommandResult {
    pub command: &'static str,
    pub revision: u64,
    pub affected: Vec<InputId>,
}
#[derive(Clone)]
pub struct Engine {
    project: ProjectState,
    transition: Option<TransitionState>,
    black: f32,
    black_ramp: Option<(u64, f32, f32)>,
    last_ms: u64,
    playback: HashMap<InputId, playback::PlaybackClock>,
    inputs_paused: bool,
    playlist_run: Option<PlaylistRun>,
    playlist_alarm: Option<String>,
}
impl Default for Engine {
    fn default() -> Self {
        let project = ProjectState::demo();
        let playback = project
            .inputs
            .iter()
            .map(|i| (i.id, playback::PlaybackClock::new(0)))
            .collect();
        Self {
            project,
            transition: None,
            black: 0.0,
            black_ramp: None,
            last_ms: 0,
            playback,
            inputs_paused: false,
            playlist_run: None,
            playlist_alarm: None,
        }
    }
}
impl Engine {
    pub fn from_project(project: ProjectState) -> Result<Self> {
        Self::from_project_at(project, 0)
    }
    pub fn from_project_at(project: ProjectState, now: u64) -> Result<Self> {
        project.validate()?;
        let playback = project
            .inputs
            .iter()
            .map(|i| (i.id, playback::PlaybackClock::new(now)))
            .collect();
        Ok(Self {
            project,
            transition: None,
            black: 0.0,
            black_ramp: None,
            last_ms: now,
            playback,
            inputs_paused: false,
            playlist_run: None,
            playlist_alarm: None,
        })
    }
    fn insert_input(
        project: &mut ProjectState,
        name: String,
        kind: InputKind,
        config: InputConfig,
        index: usize,
    ) -> Result<InputId> {
        if index > project.inputs.len() || project.inputs.len() >= 32 {
            return Err(invalid(
                "Choose an available input position (maximum 32 inputs)",
            ));
        }
        let id = project
            .inputs
            .iter()
            .map(|i| i.id.0)
            .max()
            .and_then(|i| i.checked_add(1))
            .map(InputId)
            .ok_or_else(|| invalid("Input ID limit reached"))?;
        project.inputs.insert(
            index,
            InputState {
                id,
                name,
                kind,
                config,
                color: [37, 160, 183],
                online: true,
                audio: AudioChannel {
                    muted: true,
                    ..Default::default()
                },
            },
        );
        Ok(id)
    }
    pub fn project(&self) -> &ProjectState {
        &self.project
    }
    pub fn transition(&self) -> Option<&TransitionState> {
        self.transition.as_ref()
    }
    pub fn black_level(&self) -> f32 {
        self.black
    }
    pub fn playback(&self, id: InputId) -> Option<&playback::PlaybackClock> {
        self.playback.get(&id)
    }
    pub fn inputs_paused(&self) -> bool {
        self.inputs_paused
    }
    pub fn enabled(&self, command: &Command) -> Result<()> {
        match command {
            Command::SelectPreview(id) | Command::TakeInput(id)
                if self.project.input(*id).is_none() =>
            {
                Err(invalid("Input not found"))
            }
            Command::TakeInput(id) if *id == self.project.program => {
                Err(invalid("This input is already on Program"))
            }
            Command::Cut
            | Command::TakeInput(_)
            | Command::Transition(_)
            | Command::RunTransitionSlot(_)
            | Command::StartPlaylist { .. }
            | Command::PlaylistNext
            | Command::PlaylistPrevious
            | Command::SelectPreview(_)
            | Command::AddInput { .. }
            | Command::AddInputAt { .. }
            | Command::DuplicateInput(_)
            | Command::RemoveInput(_)
            | Command::UpdateInput(_)
            | Command::ApplyInput { .. }
                if self.transition.is_some() =>
            {
                Err(invalid("Finish or cancel the active transition first"))
            }
            Command::Transition(_) | Command::RunTransitionSlot(_)
                if self.project.preview == self.project.program =>
            {
                Err(invalid("Select a different Preview input"))
            }
            Command::SetManualProgress(_)
                if self.transition.as_ref().is_some_and(|t| !t.manual) =>
            {
                Err(invalid("Automatic transition is running"))
            }
            Command::RemoveInput(id)
                if *id == self.project.program || *id == self.project.preview =>
            {
                Err(invalid(
                    "Take this input out of Program and Preview before removing it",
                ))
            }
            Command::RemoveInput(_) if self.project.inputs.len() <= 2 => {
                Err(invalid("Keep at least two inputs"))
            }
            Command::AddInput { .. } | Command::AddInputAt { .. } | Command::DuplicateInput(_)
                if self.project.inputs.len() >= 32 =>
            {
                Err(invalid("Maximum 32 inputs reached"))
            }
            Command::DuplicateInput(id) if self.project.input(*id).is_none() => {
                Err(invalid("Input not found"))
            }
            Command::RunTransitionSlot(slot) | Command::SetTransitionSlot(slot, _)
                if *slot >= 4 =>
            {
                Err(invalid("Transition slot not found"))
            }
            Command::UpdatePlaylist(list)
                if self
                    .playlist_run
                    .as_ref()
                    .is_some_and(|run| run.id == list.id) =>
            {
                Err(invalid("Stop the playlist before editing it"))
            }
            Command::RemovePlaylist(id)
                if self.project.playlists.len() <= 1
                    || self.playlist_run.as_ref().is_some_and(|run| run.id == *id) =>
            {
                Err(invalid("Keep one playlist and stop it before removing it"))
            }
            Command::AddPlaylist(_) if self.project.playlists.len() >= 16 => {
                Err(invalid("Maximum 16 playlists"))
            }
            Command::StartPlaylist { .. } if self.playlist_run.is_some() => {
                Err(invalid("Stop the active playlist first"))
            }
            Command::StartPlaylist { id, selected } => {
                let list = self
                    .project
                    .playlists
                    .iter()
                    .find(|p| p.id == *id)
                    .ok_or_else(|| invalid("Playlist not found"))?;
                let index = if list.begin_selected { *selected } else { 0 };
                let item = list
                    .items
                    .get(index)
                    .ok_or_else(|| invalid("Add and select a playlist item first"))?;
                if self
                    .project
                    .input(item.input)
                    .is_none_or(|input| !input.online)
                {
                    return Err(invalid("Playlist target is offline"));
                }
                Ok(())
            }
            Command::PlaylistNext | Command::PlaylistPrevious => self
                .playlist_step_index(matches!(command, Command::PlaylistNext))
                .map(|_| ()),
            Command::RemoveInput(id)
                if self.playlist_run.as_ref().is_some_and(|run| {
                    self.project
                        .playlists
                        .iter()
                        .find(|p| p.id == run.id)
                        .is_some_and(|p| p.items.iter().any(|item| item.input == *id))
                }) =>
            {
                Err(invalid("Stop the playlist before removing its inputs"))
            }
            _ => Ok(()),
        }
    }
    pub fn execute(&mut self, command: Command, now: u64) -> Result<CommandResult> {
        self.enabled(&command)?;
        let id = command.definition().id;
        let mut next = self.project.clone();
        let mut transition = self.transition.clone();
        let mut black_ramp = self.black_ramp;
        let mut playlist_run = self.playlist_run.clone();
        let mut playlist_alarm = self.playlist_alarm.clone();
        if matches!(
            command,
            Command::Cut
                | Command::TakeInput(_)
                | Command::Transition(_)
                | Command::RunTransitionSlot(_)
                | Command::SetManualProgress(_)
                | Command::CancelTransition
        ) {
            playlist_run = None;
        }
        let mut affected = Vec::new();
        let transport = command.clone();
        match command {
            Command::SelectPreview(id) => {
                if next.input(id).is_none() {
                    return Err(invalid("Input not found"));
                }
                next.preview = id;
                affected.push(id);
            }
            Command::Cut => {
                std::mem::swap(&mut next.program, &mut next.preview);
                affected.extend([next.program, next.preview]);
            }
            Command::TakeInput(target) => {
                if next.input(target).is_none() {
                    return Err(invalid("Input not found"));
                }
                next.preview = next.program;
                next.program = target;
                affected.extend([next.program, next.preview]);
            }
            Command::Transition(kind) => {
                let slot = TransitionSlot {
                    kind,
                    duration_ms: next.duration_ms,
                };
                production::start_transition(&mut next, &mut transition, slot, now);
            }
            Command::RunTransitionSlot(index) => {
                let slot = next
                    .transition_slots
                    .get(index)
                    .copied()
                    .ok_or_else(|| invalid("Transition slot not found"))?;
                production::start_transition(&mut next, &mut transition, slot, now);
            }
            Command::SetTransitionSlot(index, slot) => {
                *next
                    .transition_slots
                    .get_mut(index)
                    .ok_or_else(|| invalid("Transition slot not found"))? = slot;
            }
            Command::AddPlaylist(name) => {
                let id = next
                    .playlists
                    .iter()
                    .map(|p| p.id)
                    .max()
                    .and_then(|id| id.checked_add(1))
                    .ok_or_else(|| invalid("Playlist ID limit reached"))?;
                next.playlists.push(Playlist::empty(id, name));
            }
            Command::UpdatePlaylist(list) => {
                let playlist_id = list.id;
                *next
                    .playlists
                    .iter_mut()
                    .find(|p| p.id == playlist_id)
                    .ok_or_else(|| invalid("Playlist not found"))? = list;
            }
            Command::RemovePlaylist(id) => {
                if !next.playlists.iter().any(|p| p.id == id) {
                    return Err(invalid("Playlist not found"));
                }
                next.playlists.retain(|p| p.id != id);
            }
            Command::StartPlaylist { id, selected } => {
                let list = next
                    .playlists
                    .iter()
                    .find(|p| p.id == id)
                    .ok_or_else(|| invalid("Playlist not found"))?;
                let index = if list.begin_selected { selected } else { 0 };
                if list.clear_overlays {
                    for overlay in &mut next.overlays {
                        overlay.enabled = false;
                    }
                }
                playlist_run = Some(production::cue(&mut next, &mut transition, id, index, now)?);
                playlist_alarm = None;
            }
            Command::StopPlaylist => {
                playlist_run = None;
            }
            Command::PlaylistNext | Command::PlaylistPrevious => {
                let index = self.playlist_step_index(matches!(command, Command::PlaylistNext))?;
                if let Some(index) = index {
                    let id = playlist_run
                        .as_ref()
                        .ok_or_else(|| invalid("Start a playlist first"))?
                        .id;
                    playlist_run =
                        Some(production::cue(&mut next, &mut transition, id, index, now)?);
                } else {
                    playlist_run = None;
                }
            }
            Command::SetManualProgress(p) => {
                if !p.is_finite() || !(0.0..=1.0).contains(&p) {
                    return Err(invalid("T-Bar position must be between 0 and 1"));
                }
                if next.program == next.preview {
                    return Err(invalid("Select a different Preview input"));
                }
                if p == 0.0 {
                    transition = None;
                } else if p == 1.0 {
                    std::mem::swap(&mut next.program, &mut next.preview);
                    transition = None;
                } else {
                    transition = Some(TransitionState {
                        from: next.program,
                        to: next.preview,
                        kind: TransitionKind::Fade,
                        start_ms: now,
                        duration_ms: next.duration_ms,
                        progress: p,
                        manual: true,
                    });
                }
            }
            Command::CancelTransition => {
                transition = None;
            }
            Command::SetDuration(d) => {
                next.duration_ms = d;
            }
            Command::FadeBlack => {
                let target = if black_ramp.is_some_and(|(_, _, t)| t > 0.0) || self.black > 0.5 {
                    0.0
                } else {
                    1.0
                };
                black_ramp = Some((now, self.black, target));
            }
            Command::AddInput { name, kind } => {
                let index = next.inputs.len();
                let id = Self::insert_input(&mut next, name, kind, InputConfig::default(), index)?;
                affected.push(id);
            }
            Command::AddInputAt {
                name,
                kind,
                config,
                index,
            } => {
                let id = Self::insert_input(&mut next, name, kind, config, index)?;
                affected.push(id);
            }
            Command::DuplicateInput(source) => {
                let index = next
                    .inputs
                    .iter()
                    .position(|i| i.id == source)
                    .ok_or_else(|| invalid("Input not found"))?;
                let original = next
                    .input(source)
                    .cloned()
                    .ok_or_else(|| invalid("Input not found"))?;
                let mut name = String::new();
                for ch in original.name.chars() {
                    if name.len() + ch.len_utf8() > 57 {
                        break;
                    }
                    name.push(ch);
                }
                name.push_str(" (Copy)");
                let id =
                    Self::insert_input(&mut next, name, original.kind, original.config, index + 1)?;
                if let Some(copy) = next.inputs.iter_mut().find(|i| i.id == id) {
                    copy.color = original.color;
                    copy.online = original.online;
                    copy.audio = AudioChannel {
                        muted: true,
                        solo: false,
                        ..original.audio
                    };
                }
                affected.push(id);
            }
            Command::RemoveInput(id) => {
                if next.input(id).is_none() {
                    return Err(invalid("Input not found"));
                }
                next.inputs.retain(|i| i.id != id);
                for list in &mut next.playlists {
                    list.items.retain(|item| item.input != id);
                }
                next.preferences
                    .key_bindings
                    .retain(|binding| binding.action != ShortcutAction::Preview(id));
                for input in &mut next.inputs {
                    if input.config.pip_source == Some(id) {
                        input.config.pip_source = None;
                        affected.push(input.id);
                    }
                }
                affected.push(id);
            }
            Command::ApplyInput { mut input, audio } => {
                let old = next
                    .inputs
                    .iter_mut()
                    .find(|i| i.id == input.id)
                    .ok_or_else(|| invalid("Input not found"))?;
                input.audio = audio.unwrap_or_else(|| old.audio.clone());
                affected.push(input.id);
                *old = input;
            }
            Command::UpdateInput(mut input) => {
                let old = next
                    .inputs
                    .iter_mut()
                    .find(|i| i.id == input.id)
                    .ok_or_else(|| invalid("Input not found"))?;
                input.audio = old.audio.clone();
                affected.push(input.id);
                *old = input;
            }

            Command::SetInputAudio(id, a) => {
                let input = next
                    .inputs
                    .iter_mut()
                    .find(|i| i.id == id)
                    .ok_or_else(|| invalid("Input not found"))?;
                input.audio = a;
                affected.push(id);
            }
            Command::SetBus(i, a) => {
                *next
                    .buses
                    .get_mut(i)
                    .ok_or_else(|| invalid("Bus not found"))? = a;
            }
            Command::SetOverlay(i, o) => {
                *next
                    .overlays
                    .get_mut(i)
                    .ok_or_else(|| invalid("Overlay slot not found"))? = o;
            }
            Command::SetPreferences(p) => {
                next.preferences = p;
            }
            Command::SetWorkspace(w) => {
                next.workspace = w;
            }
            Command::RenameProject(name) => {
                next.name = name;
            }
            Command::SetInputPaused(id, _)
            | Command::RestartInput(id)
            | Command::SetInputLoop(id, _) => {
                if next.input(id).is_none() {
                    return Err(invalid("Input not found"));
                }
                affected.push(id);
            }
            Command::SetInputsPaused(_) => {}
        }
        next.revision = next
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("State revision limit reached"))?;
        next.validate()?;
        self.tick_playback(now);
        self.project = next;
        self.playback
            .retain(|id, _| self.project.input(*id).is_some());
        for input in &self.project.inputs {
            self.playback
                .entry(input.id)
                .or_insert_with(|| playback::PlaybackClock::new(now.max(self.last_ms)));
        }
        match transport {
            Command::SetInputPaused(id, paused) => {
                if let Some(clock) = self.playback.get_mut(&id) {
                    clock.set_paused(paused);
                }
            }
            Command::RestartInput(id) => {
                if let Some(clock) = self.playback.get_mut(&id) {
                    clock.restart();
                }
            }
            Command::SetInputLoop(id, looping) => {
                if let Some(clock) = self.playback.get_mut(&id) {
                    clock.set_looping(looping);
                }
            }
            Command::SetInputsPaused(paused) => self.inputs_paused = paused,
            _ => {}
        }
        self.transition = transition;
        self.black_ramp = black_ramp;
        self.playlist_run = playlist_run;
        self.playlist_alarm = playlist_alarm;
        Ok(CommandResult {
            command: id,
            revision: self.project.revision,
            affected,
        })
    }
    pub fn tick(&mut self, now: u64) {
        let now = now.max(self.last_ms);
        self.tick_playback(now);
        self.last_ms = now;
        let mut completed = false;
        if let Some(t) = &mut self.transition
            && !t.manual
        {
            t.progress =
                (now.saturating_sub(t.start_ms) as f32 / t.duration_ms.max(1) as f32).min(1.0);
            completed = t.progress >= 1.0;
        }
        if completed && let Some(t) = self.transition.take() {
            self.project.program = t.to;
            self.project.preview = t.from;
            self.project.revision = self.project.revision.saturating_add(1);
            if let Some(run) = &mut self.playlist_run {
                run.hold_started_ms = Some(t.start_ms.saturating_add(u64::from(t.duration_ms)));
            }
        }
        self.tick_playlist(now);
        if let Some((start, from, to)) = self.black_ramp {
            let p = (now.saturating_sub(start) as f32 / self.project.duration_ms.max(1) as f32)
                .min(1.0);
            self.black = from + (to - from) * p;
            if p >= 1.0 {
                self.black_ramp = None;
            }
        }
    }
    fn tick_playback(&mut self, now: u64) {
        for clock in self.playback.values_mut() {
            clock.advance(now, self.inputs_paused);
        }
    }
}
