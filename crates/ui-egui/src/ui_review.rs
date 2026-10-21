//! Headless UI acceptance and optional exact-egui software captures.
//! These captures check layout/paint, not native Windows, GPU or physical DPI delivery.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]
use super::*;

fn review_context() -> egui::Context {
    let ctx = egui::Context::default();
    // Match the native host's optional Windows typography, without shipping it.
    if let Some(windows) = std::env::var_os("WINDIR") {
        let path = PathBuf::from(windows).join("Fonts").join("segoeui.ttf");
        if std::fs::metadata(&path).is_ok_and(|m| m.len() < 5_000_000)
            && let Ok(bytes) = std::fs::read(path)
        {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "windows-ui".into(),
                Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "windows-ui".into());
            ctx.set_fonts(fonts);
        }
    }
    ctx
}

fn review_app() -> RustMixApp {
    let mut app = RustMixApp::headless();
    // Keep source raster/transport stationary while reviewing configuration UI.
    app.engine
        .execute(Command::SetInputsPaused(true), 0)
        .unwrap();
    app
}

fn frame(
    app: &mut RustMixApp,
    ctx: &egui::Context,
    size: Vec2,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            time: Some(ctx.cumulative_pass_nr() as f64 / 30.0),
            events,
            ..Default::default()
        },
        |ui| app.frame_ui(ui),
    )
}

fn click(app: &mut RustMixApp, ctx: &egui::Context, id: &str) {
    let size = ctx.viewport_rect().size();
    for _ in 0..4 {
        let _ = frame(app, ctx, size, vec![]);
    }
    let pos = app.automation.get(id).expect(id).center();
    assert!(ctx.viewport_rect().contains(pos), "offscreen action {id}");
    for pressed in [true, false] {
        let _ = frame(
            app,
            ctx,
            size,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
    }
}

#[test]
fn output_navigation_is_inspectable_and_cannot_start_or_mutate_program() {
    for size in [
        Vec2::new(734.0, 466.0),
        Vec2::new(1100.0, 700.0),
        Vec2::new(1600.0, 900.0),
        Vec2::new(1920.0, 1200.0),
    ] {
        let mut app = review_app();
        let ctx = review_context();
        app.theme(&ctx);
        let before = app.engine.project().clone();
        app.streaming_setup = true;
        for _ in 0..5 {
            let _ = frame(&mut app, &ctx, size, vec![]);
        }
        for slot in 1..=5 {
            click(&mut app, &ctx, &format!("dialog.streaming.slot.{slot}"));
            assert_eq!(app.stream_channel, slot - 1);
        }
        click(&mut app, &ctx, "dialog.streaming.start");
        assert!(!app.stream_active);
        click(&mut app, &ctx, "dialog.streaming.advanced");
        assert!(app.stream_page == output_dialogs::StreamPage::Advanced);
        click(&mut app, &ctx, "dialog.streaming.back");
        click(&mut app, &ctx, "dialog.streaming.status");
        assert!(app.stream_page == output_dialogs::StreamPage::Status);
        click(&mut app, &ctx, "dialog.streaming.close");
        assert!(!app.streaming_setup);
        assert!(app.stream_page == output_dialogs::StreamPage::Main);
        app.multicorder_open = true;
        for _ in 0..5 {
            let _ = frame(&mut app, &ctx, size, vec![]);
        }
        for format in 0..3 {
            click(
                &mut app,
                &ctx,
                &format!("dialog.multicorder.format.{format}"),
            );
            assert_eq!(app.mc_format, format);
        }
        click(&mut app, &ctx, "dialog.multicorder.toggle");
        assert!(!app.mc_running);
        click(&mut app, &ctx, "dialog.multicorder.close");
        assert!(!app.multicorder_open);
        assert_eq!(app.engine.project(), &before);
    }
}

#[test]
fn settings_pages_show_specific_content_without_editing_unrelated_preferences() {
    let ctx = review_context();
    let mut app = review_app();
    app.theme(&ctx);
    app.dispatch(Action::Settings, &ctx);
    let before = app.engine.project().clone();
    for (page, heading) in [
        (SettingsPage::Outputs, "Outputs / NDI / OMT / SRT"),
        (SettingsPage::Decoders, "Decoders"),
        (SettingsPage::Recording, "Recording"),
        (SettingsPage::Audio, "Audio"),
        (SettingsPage::AudioOutputs, "Audio Outputs"),
        (SettingsPage::TallyLights, "Tally Lights"),
        (SettingsPage::Activators, "Activators"),
        (SettingsPage::Scripting, "Scripting"),
        (SettingsPage::Alerts, "Alerts"),
        (SettingsPage::About, "RustMix"),
    ] {
        app.settings_page = page;
        for _ in 0..4 {
            let _ = frame(&mut app, &ctx, Vec2::new(1600.0, 900.0), vec![]);
        }
        let output = frame(&mut app, &ctx, Vec2::new(1600.0, 900.0), vec![]);
        let painted: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
                _ => None,
            })
            .collect();
        assert!(painted.contains(&heading), "missing heading {heading}");
        assert!(!painted.contains(&"Enable single-key production shortcuts"));
        assert_eq!(app.settings_draft.as_ref(), Some(&before.preferences));
    }
    click(&mut app, &ctx, "dialog.settings.cancel");
    assert_eq!(app.engine.project(), &before);
}

// A test-only triangle renderer consumes egui's actual tessellated meshes and
// font/image textures. No replacement HTML/mockup or application executable.
#[derive(Default)]
struct SoftwarePaint {
    textures: HashMap<egui::TextureId, egui::ColorImage>,
}

impl SoftwarePaint {
    fn update(&mut self, delta: &egui::TexturesDelta) {
        for (id, delta) in &delta.set {
            let egui::ImageData::Color(image) = &delta.image;
            if let Some([x, y]) = delta.pos {
                let target = self.textures.get_mut(id).expect("existing partial texture");
                for row in 0..image.height() {
                    let start = (row + y) * target.width() + x;
                    target.pixels[start..start + image.width()].copy_from_slice(
                        &image.pixels[row * image.width()..(row + 1) * image.width()],
                    );
                }
            } else {
                self.textures.insert(*id, (**image).clone());
            }
        }
    }
    fn save(&self, ctx: &egui::Context, output: egui::FullOutput, size: Vec2, path: &Path) {
        let mut image = image::RgbaImage::from_pixel(
            size.x as u32,
            size.y as u32,
            image::Rgba([30, 35, 40, 255]),
        );
        for clipped in ctx.tessellate(output.shapes, output.pixels_per_point) {
            let egui::epaint::Primitive::Mesh(mesh) = clipped.primitive else {
                panic!("UI review does not support GPU callbacks");
            };
            let texture = self.textures.get(&mesh.texture_id).expect("paint texture");
            for indices in mesh.indices.chunks_exact(3) {
                let vertices = indices
                    .iter()
                    .map(|i| mesh.vertices[*i as usize])
                    .collect::<Vec<_>>();
                let a = vertices[0];
                let b = vertices[1];
                let c = vertices[2];
                let edge = |a: Pos2, b: Pos2, p: Pos2| {
                    (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)
                };
                let area = edge(a.pos, b.pos, c.pos);
                if area.abs() < 0.0001 {
                    continue;
                }
                let bounds = Rect::from_points(&[a.pos, b.pos, c.pos])
                    .intersect(clipped.clip_rect)
                    .intersect(Rect::from_min_size(Pos2::ZERO, size));
                for y in
                    bounds.top().floor().max(0.0) as u32..bounds.bottom().ceil().min(size.y) as u32
                {
                    for x in bounds.left().floor().max(0.0) as u32
                        ..bounds.right().ceil().min(size.x) as u32
                    {
                        let p = Pos2::new(x as f32 + 0.5, y as f32 + 0.5);
                        if !clipped.clip_rect.contains(p) {
                            continue;
                        }
                        let weights = [
                            edge(b.pos, c.pos, p) / area,
                            edge(c.pos, a.pos, p) / area,
                            edge(a.pos, b.pos, p) / area,
                        ];
                        if weights.iter().any(|w| *w < 0.0) {
                            continue;
                        }
                        let uv = vertices
                            .iter()
                            .zip(weights)
                            .fold(Vec2::ZERO, |sum, (v, w)| sum + v.uv.to_vec2() * w);
                        let tx = (uv.x * texture.width() as f32 - 0.5)
                            .clamp(0.0, (texture.width() - 1) as f32);
                        let ty = (uv.y * texture.height() as f32 - 0.5)
                            .clamp(0.0, (texture.height() - 1) as f32);
                        let (ix, iy) = (tx.floor() as usize, ty.floor() as usize);
                        let (fx, fy) = (tx.fract(), ty.fract());
                        let mut tex = [0.0; 4];
                        for (dx, dy, w) in [
                            (0, 0, (1.0 - fx) * (1.0 - fy)),
                            (1, 0, fx * (1.0 - fy)),
                            (0, 1, (1.0 - fx) * fy),
                            (1, 1, fx * fy),
                        ] {
                            let pixel = texture.pixels[(iy + dy).min(texture.height() - 1)
                                * texture.width()
                                + (ix + dx).min(texture.width() - 1)]
                            .to_array();
                            for i in 0..4 {
                                tex[i] += f32::from(pixel[i]) * w;
                            }
                        }
                        let mut source = [0.0; 4];
                        for (v, w) in vertices.iter().zip(weights) {
                            for (i, value) in v.color.to_array().into_iter().enumerate() {
                                source[i] += f32::from(value) * w * tex[i] / 255.0;
                            }
                        }
                        let dest = image.get_pixel_mut(x, y);
                        for i in 0..3 {
                            dest.0[i] = (source[i]
                                + f32::from(dest.0[i]) * (1.0 - source[3] / 255.0))
                                .clamp(0.0, 255.0)
                                .round() as u8;
                        }
                    }
                }
            }
        }
        image.save(path).unwrap();
    }
}

#[test]
fn processing_pages_and_reset_only_touch_their_own_draft() {
    let ctx = review_context();
    let mut app = review_app();
    app.theme(&ctx);
    app.dispatch(Action::Properties(InputId(1)), &ctx);
    let before = app.engine.project().clone();
    for page in [
        InputPage::Position,
        InputPage::ColourAdjust,
        InputPage::ColourCorrection,
        InputPage::Effects,
    ] {
        app.input_page = page;
        app.prop_zoom = 2.0;
        app.prop_pan_x = 0.3;
        app.prop_pan_y = 0.4;
        app.prop_lift = 0.2;
        app.prop_gain = 1.7;
        app.prop_saturation = 0.5;
        app.prop_mirror = true;
        app.prop_sharpen = true;
        for _ in 0..4 {
            let _ = frame(&mut app, &ctx, Vec2::new(1600.0, 900.0), vec![]);
        }
        let out = frame(&mut app, &ctx, Vec2::new(1600.0, 900.0), vec![]);
        let labels: Vec<_> = out
            .shapes
            .iter()
            .filter_map(|s| match &s.shape {
                egui::Shape::Text(t) => Some(t.galley.job.text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(labels.contains(&"Pan Y"), page == InputPage::Position);
        assert_eq!(
            labels.contains(&"Saturation"),
            matches!(page, InputPage::ColourAdjust | InputPage::ColourCorrection)
        );
        click(&mut app, &ctx, "dialog.properties.processing.reset");
        if page == InputPage::Position {
            assert_eq!(
                (app.prop_zoom, app.prop_pan_x, app.prop_pan_y),
                (1.0, 0.0, 0.0)
            );
            assert_eq!(app.prop_gain, 1.7);
            assert!(app.prop_mirror);
        } else if page == InputPage::Effects {
            assert!(!app.prop_mirror && !app.prop_sharpen);
            assert_eq!(app.prop_gain, 1.7);
            assert_eq!(app.prop_zoom, 2.0);
        } else {
            assert_eq!(
                (app.prop_lift, app.prop_gain, app.prop_saturation),
                (0.0, 1.0, 1.0)
            );
            assert_eq!(app.prop_zoom, 2.0);
            assert!(app.prop_mirror);
        }
        assert_eq!(app.engine.project(), &before);
    }
    click(&mut app, &ctx, "dialog.properties.cancel");
    assert_eq!(app.engine.project(), &before);
}

#[test]
fn ui_review_capture_existing_pages() {
    let Ok(directory) = std::env::var("RUSTMIX_UI_REVIEW_DIR") else {
        return;
    };
    let directory = PathBuf::from(directory);
    std::fs::create_dir_all(&directory).unwrap();
    let selected: Option<Vec<usize>> = std::env::var("RUSTMIX_UI_REVIEW_PAGES").ok().map(|value| {
        value
            .split(',')
            .map(|page| page.parse().expect("numeric review page"))
            .collect()
    });
    for size in [Vec2::new(1100.0, 700.0), Vec2::new(1600.0, 900.0)] {
        for page in 0..70 {
            if selected
                .as_ref()
                .is_some_and(|pages| !pages.contains(&page))
            {
                continue;
            }
            if size.x < 1600.0 && page >= 24 {
                continue;
            }
            let mut app = review_app();
            let ctx = review_context();
            app.theme(&ctx);
            let name = match page {
                0 => "workspace",
                1 => {
                    app.add_open = true;
                    "add-input"
                }
                2 => {
                    app.dispatch(Action::Properties(InputId(1)), &ctx);
                    "input-settings"
                }
                3 => {
                    app.dispatch(Action::OverlaySettings(0), &ctx);
                    "overlay"
                }
                4 => {
                    app.external_settings = Some(ExternalSettingsDraft {
                        selected: 1,
                        outputs: app.engine.project().outputs.iter().cloned().collect(),
                        error: None,
                        applied: None,
                    });
                    "external"
                }
                5 => {
                    app.recording_setup = true;
                    "recording"
                }
                6 => {
                    app.streaming_setup = true;
                    "streaming"
                }
                7 => {
                    app.streaming_setup = true;
                    app.stream_page = output_dialogs::StreamPage::Advanced;
                    "stream-advanced"
                }
                8 => {
                    app.streaming_setup = true;
                    app.stream_page = output_dialogs::StreamPage::Status;
                    "stream-status"
                }
                9 => {
                    app.multicorder_open = true;
                    "multicorder"
                }
                10 => {
                    app.playlist_open = true;
                    "playlist"
                }
                11 => {
                    app.shortcuts_open = true;
                    "shortcuts"
                }
                12..=23 => {
                    app.dispatch(Action::Settings, &ctx);
                    let (selected, name) = [
                        (SettingsPage::Display, "settings-display"),
                        (SettingsPage::Options, "settings-options"),
                        (SettingsPage::Outputs, "settings-outputs"),
                        (SettingsPage::Decoders, "settings-decoders"),
                        (SettingsPage::Audio, "settings-audio"),
                        (SettingsPage::AudioOutputs, "settings-audio-outputs"),
                        (SettingsPage::Recording, "settings-recording"),
                        (SettingsPage::TallyLights, "settings-tally"),
                        (SettingsPage::Activators, "settings-activators"),
                        (SettingsPage::Scripting, "settings-scripting"),
                        (SettingsPage::Alerts, "settings-alerts"),
                        (SettingsPage::About, "settings-about"),
                    ][page - 12];
                    app.settings_page = selected;
                    name
                }
                24..=49 => {
                    app.add_open = true;
                    app.select_add_category(page - 24);
                    "add-category"
                }
                50..=58 => {
                    app.dispatch(Action::Properties(InputId(1)), &ctx);
                    app.input_page = [
                        InputPage::General,
                        InputPage::ColourAdjust,
                        InputPage::ColourCorrection,
                        InputPage::ColourKey,
                        InputPage::Effects,
                        InputPage::Position,
                        InputPage::Layers,
                        InputPage::Audio,
                        InputPage::Status,
                    ][page - 50];
                    "input-page"
                }
                59..=63 => {
                    app.dispatch(Action::Settings, &ctx);
                    app.settings_page = [
                        SettingsPage::Appearance,
                        SettingsPage::Categories,
                        SettingsPage::Workspace,
                        SettingsPage::Shortcuts,
                        SettingsPage::ExternalOutput,
                    ][page - 59];
                    "settings-page"
                }
                64 => {
                    app.about = true;
                    "about"
                }
                65 => {
                    app.audio_ui.open = true;
                    "audio-io"
                }
                66 => {
                    app.basic_mode = true;
                    "basic"
                }
                67 => {
                    app.multiview = true;
                    "multiview"
                }
                68 => {
                    app.search_open = true;
                    "input-search"
                }
                _ => {
                    app.confirm_new = true;
                    "new-preset"
                }
            };
            let mut painter = SoftwarePaint::default();
            for _ in 0..6 {
                let out = frame(&mut app, &ctx, size, vec![]);
                painter.update(&out.textures_delta);
            }
            let out = frame(&mut app, &ctx, size, vec![]);
            painter.update(&out.textures_delta);
            let prefix = if (24..64).contains(&page) {
                format!("{name}-{page}-{}", size.x as u32)
            } else {
                format!("{name}-{}", size.x as u32)
            };
            let bounds: HashMap<_, _> = app
                .automation
                .iter()
                .map(|(id, r)| (id, [r.left(), r.top(), r.width(), r.height()]))
                .collect();
            std::fs::write(
                directory.join(format!("{prefix}.json")),
                serde_json::to_vec_pretty(&bounds).unwrap(),
            )
            .unwrap();
            painter.save(&ctx, out, size, &directory.join(format!("{prefix}.png")));
        }
    }
}
