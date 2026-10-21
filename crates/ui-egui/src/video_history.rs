//! Bounded local-video history for the operator's application preferences.

use rustmix_engine::VideoSource;

const MAX_RECENT_FILES: usize = 32;
const MAX_HISTORY_JSON_BYTES: usize = 32 * 4096 * 2 + 1024;

#[derive(Default)]
pub(crate) struct VideoHistory {
    paths: Vec<String>,
}

impl VideoHistory {
    pub(crate) fn from_json(json: Option<&str>) -> Self {
        let mut history = Self::default();
        let Some(json) = json.filter(|value| value.len() <= MAX_HISTORY_JSON_BYTES) else {
            return history;
        };
        if let Ok(paths) = serde_json::from_str::<Vec<String>>(json) {
            for path in paths.into_iter().take(MAX_RECENT_FILES).rev() {
                history.record(&path);
            }
        }
        history
    }

    pub(crate) fn record(&mut self, path: &str) -> bool {
        if path.len() > 4096
            || (VideoSource::File {
                path: path.to_owned(),
            })
            .validate()
            .is_err()
        {
            return false;
        }
        self.paths.retain(|entry| entry != path);
        self.paths.insert(0, path.to_owned());
        self.paths.truncate(MAX_RECENT_FILES);
        true
    }

    pub(crate) fn paths(&self) -> &[String] {
        &self.paths
    }

    pub(crate) fn clear(&mut self) {
        self.paths.clear();
    }

    pub(crate) fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self.paths)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn history_promotes_existing_files_caps_and_round_trips_without_touching_media() {
        let mut history = VideoHistory::default();
        for number in 0..40 {
            assert!(history.record(&format!("C:/media/{number}.mp4")));
        }
        assert_eq!(history.paths().len(), MAX_RECENT_FILES);
        assert_eq!(
            history.paths().first().map(String::as_str),
            Some("C:/media/39.mp4")
        );
        assert!(history.record("C:/media/15.mp4"));
        assert_eq!(
            history.paths().first().map(String::as_str),
            Some("C:/media/15.mp4")
        );
        assert_eq!(
            history
                .paths()
                .iter()
                .filter(|path| *path == "C:/media/15.mp4")
                .count(),
            1
        );
        let json = history.to_json().unwrap();
        assert_eq!(
            VideoHistory::from_json(Some(&json)).paths(),
            history.paths()
        );
        history.clear();
        assert!(history.paths().is_empty());
    }

    #[test]
    fn preference_history_rejects_malformed_huge_remote_control_and_non_mp4_values() {
        for json in [
            "not json",
            "{}",
            "[7]",
            "[\"https://example.com/file.mp4\"]",
            "[\"C:/media/file.avi\"]",
            "[\"C:/media/a\\n.mp4\"]",
        ] {
            assert!(
                VideoHistory::from_json(Some(json)).paths().is_empty(),
                "invalid history: {json}"
            );
        }
        let oversized = " ".repeat(MAX_HISTORY_JSON_BYTES + 1);
        assert!(VideoHistory::from_json(Some(&oversized)).paths().is_empty());
        let mut history = VideoHistory::default();
        assert!(!history.record(&format!("C:/{}.mp4", "a".repeat(4096))));
        assert!(!history.record("//server/share/file.mp4"));
        assert!(history.paths().is_empty());
    }
}
