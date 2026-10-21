//! Operator entry points supported by the current core build.
use super::SettingsPage;

pub(super) fn settings_page(page: SettingsPage) -> bool {
    matches!(
        page,
        SettingsPage::Display
            | SettingsPage::Options
            | SettingsPage::Appearance
            | SettingsPage::Categories
            | SettingsPage::Decoders
            | SettingsPage::Audio
            | SettingsPage::Workspace
            | SettingsPage::Shortcuts
            | SettingsPage::Alerts
            | SettingsPage::About
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::super::*;

    #[test]
    fn untouched_sessions_cannot_rotate_away_recovery_and_each_dirty_revision_saves_once() {
        let mut app = RustMixApp::headless();
        app.engine = Engine::new_preset(VideoStandard::default(), 0).unwrap();
        assert!(
            !app.checkpoint_needed(),
            "A new untouched session must preserve previous recovery"
        );
        app.engine
            .execute(Command::RenameProject("Unsaved show".into()), 0)
            .unwrap();
        assert!(app.checkpoint_needed());
        app.last_checkpoint_revision = Some(app.engine.project().revision);
        assert!(
            !app.checkpoint_needed(),
            "Idle repeats must not rotate identical backups"
        );
        app.engine.execute(Command::Cut, 100).unwrap();
        assert!(app.checkpoint_needed());
        app.saved_revision = app.engine.project().revision;
        assert!(!app.checkpoint_needed());
    }

    #[test]
    fn repeated_egui_layout_passes_do_not_inflate_ui_frame_counts() {
        let mut app = RustMixApp::headless();
        app.engine = Engine::new_preset(VideoStandard::default(), 0).unwrap();
        let ctx = egui::Context::default();
        let mut passes = 0;
        let _ = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 900.0))),
                ..Default::default()
            },
            |ui| {
                passes += 1;
                app.frame_ui(ui);
                if passes == 1 {
                    ui.ctx().request_discard("verify sizing pass accounting");
                }
            },
        );
        assert!(passes >= 2);
        assert_eq!(app.fps_frames, 1);
    }

    #[test]
    fn normal_operator_entry_points_include_core_controls_without_future_output_menus() {
        let mut app = RustMixApp::headless();
        app.engine = Engine::new_preset(VideoStandard::default(), 0).unwrap();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        for _ in 0..4 {
            let _ = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 900.0))),
                    ..Default::default()
                },
                |ui| app.frame_ui(ui),
            );
        }
        for id in [
            "production.add",
            "switcher.cut",
            "switcher.fade",
            "output.playlist",
            "output.overlay",
            "output.shortcuts",
            "output.audioPane",
        ] {
            assert!(
                app.automation.contains_key(id),
                "Missing connected core control {id}"
            );
        }
        for id in [
            "output.record",
            "output.record.settings",
            "output.stream",
            "output.stream.settings",
            "output.external",
            "output.multicorder",
        ] {
            assert!(
                !app.automation.contains_key(id),
                "Future feature exposed: {id}"
            );
        }
        app.dispatch(Action::Add, &ctx);
        for _ in 0..4 {
            let _ = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 900.0))),
                    ..Default::default()
                },
                |ui| app.frame_ui(ui),
            );
        }
        for tab in 0..26 {
            assert_eq!(
                app.automation
                    .contains_key(&format!("dialog.add.category.{tab}")),
                add_input::CATEGORY_ORDER.contains(&tab),
                "Source category {tab}"
            );
        }
    }
}
