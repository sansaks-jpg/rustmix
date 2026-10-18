//! Persisted production configuration and bounded, headless input sequencing.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionSlot {
    pub kind: TransitionKind,
    pub duration_ms: u32,
}
impl TransitionSlot {
    pub fn validate(self) -> Result<()> {
        if !(100..=10_000).contains(&self.duration_ms) {
            return Err(invalid("Transition duration must be 100–10000 ms"));
        }
        Ok(())
    }
}
pub fn default_transition_slots() -> [TransitionSlot; 4] {
    [
        TransitionKind::Fade,
        TransitionKind::Slide,
        TransitionKind::Wipe,
        TransitionKind::VerticalWipe,
    ]
    .map(|kind| TransitionSlot {
        kind,
        duration_ms: 1000,
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaylistItem {
    pub input: InputId,
    /// Hold after the incoming transition completes; independent of source duration.
    pub hold_ms: u32,
    pub transition: TransitionSlot,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Playlist {
    pub id: u32,
    pub name: String,
    pub items: Vec<PlaylistItem>,
    pub looped: bool,
    pub begin_selected: bool,
    pub clear_overlays: bool,
    pub manual: bool,
}
impl Playlist {
    pub fn empty(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            items: Vec::new(),
            looped: false,
            begin_selected: false,
            clear_overlays: false,
            manual: false,
        }
    }
}
pub fn default_playlists() -> Vec<Playlist> {
    vec![Playlist::empty(1, "Default".into())]
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistRun {
    pub id: u32,
    pub index: usize,
    pub hold_started_ms: Option<u64>,
}

pub(crate) fn validate(project: &ProjectState) -> Result<()> {
    for slot in project.transition_slots {
        slot.validate()?;
    }
    if project.playlists.is_empty() || project.playlists.len() > 16 {
        return Err(invalid("Keep 1–16 playlists"));
    }
    let mut ids = HashSet::new();
    let mut names = HashSet::new();
    for list in &project.playlists {
        valid_text(&list.name, 64)?;
        if list.id == 0 || !ids.insert(list.id) || !names.insert(list.name.trim().to_lowercase()) {
            return Err(invalid("Playlist IDs and names must be unique"));
        }
        if list.items.len() > 128 {
            return Err(invalid("Maximum 128 items per playlist"));
        }
        for item in &list.items {
            if project.input(item.input).is_none() {
                return Err(invalid("Playlist input is missing"));
            }
            if !(100..=86_400_000).contains(&item.hold_ms) {
                return Err(invalid("Playlist hold must be 100 ms to 24 hours"));
            }
            item.transition.validate()?;
        }
    }
    Ok(())
}

pub(crate) fn start_transition(
    project: &mut ProjectState,
    transition: &mut Option<TransitionState>,
    slot: TransitionSlot,
    now: u64,
) {
    if slot.kind == TransitionKind::Cut {
        std::mem::swap(&mut project.program, &mut project.preview);
        *transition = None;
    } else {
        *transition = Some(TransitionState {
            from: project.program,
            to: project.preview,
            kind: slot.kind,
            start_ms: now,
            duration_ms: slot.duration_ms,
            progress: 0.0,
            manual: false,
        });
    }
}

pub(crate) fn cue(
    project: &mut ProjectState,
    transition: &mut Option<TransitionState>,
    id: u32,
    index: usize,
    now: u64,
) -> Result<PlaylistRun> {
    let item = project
        .playlists
        .iter()
        .find(|p| p.id == id)
        .and_then(|p| p.items.get(index))
        .cloned()
        .ok_or_else(|| invalid("Select an existing playlist item"))?;
    let input = project
        .input(item.input)
        .ok_or_else(|| invalid("Playlist input is missing"))?;
    if !input.online {
        return Err(invalid(
            "Playlist stopped: target input is offline; Program retained",
        ));
    }
    if project.program != item.input {
        project.preview = item.input;
        start_transition(project, transition, item.transition, now);
    }
    Ok(PlaylistRun {
        id,
        index,
        hold_started_ms: transition.is_none().then_some(now),
    })
}

impl Engine {
    pub fn playlist_run(&self) -> Option<&PlaylistRun> {
        self.playlist_run.as_ref()
    }
    pub fn playlist_alarm(&self) -> Option<&str> {
        self.playlist_alarm.as_deref()
    }

    pub(crate) fn playlist_step_index(&self, forward: bool) -> Result<Option<usize>> {
        let run = self
            .playlist_run
            .as_ref()
            .ok_or_else(|| invalid("Start a playlist first"))?;
        let list = self
            .project
            .playlists
            .iter()
            .find(|p| p.id == run.id)
            .ok_or_else(|| invalid("Playlist is missing"))?;
        if forward {
            if run
                .index
                .checked_add(1)
                .is_some_and(|i| i < list.items.len())
            {
                Ok(run.index.checked_add(1))
            } else {
                Ok(list.looped.then_some(0))
            }
        } else if run.index > 0 {
            Ok(run.index.checked_sub(1))
        } else if list.looped {
            Ok(list.items.len().checked_sub(1))
        } else {
            Err(invalid("Already at the first playlist item"))
        }
    }

    pub(crate) fn tick_playlist(&mut self, now: u64) {
        let Some(run) = self.playlist_run.as_ref() else {
            return;
        };
        let item = self
            .project
            .playlists
            .iter()
            .find(|p| p.id == run.id)
            .and_then(|p| p.items.get(run.index));
        if item.is_none_or(|item| {
            self.project
                .input(item.input)
                .is_none_or(|input| !input.online)
        }) {
            self.playlist_run = None;
            self.playlist_alarm = Some("Playlist stopped: active input is unavailable".into());
            self.project.revision = self.project.revision.saturating_add(1);
            return;
        }
        let due =
            self.project
                .playlists
                .iter()
                .find(|p| p.id == run.id)
                .is_some_and(|p| !p.manual)
                && self.transition.is_none()
                && run.hold_started_ms.zip(item).is_some_and(|(start, item)| {
                    now.saturating_sub(start) >= u64::from(item.hold_ms)
                });
        // One advancement per tick. Late wakes never create a catch-up command loop.
        if due && let Err(error) = self.execute(Command::PlaylistNext, now) {
            self.playlist_run = None;
            self.playlist_alarm = Some(error.to_string());
            self.project.revision = self.project.revision.saturating_add(1);
        }
    }
}
