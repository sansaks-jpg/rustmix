use super::*;

impl RustMixApp {
    pub(crate) fn sync_input_cache(&mut self) {
        self.input_fx
            .retain(|id, _| self.engine.project().input(*id).is_some());
        let ready = self
            .input_fx
            .values()
            .filter(|fx| fx.custom_rgba.is_some())
            .cloned()
            .collect::<Vec<_>>();
        for input in &self.engine.project().inputs {
            let fx = self.input_fx.entry(input.id).or_default();
            if fx.media_path != input.config.media_path {
                *fx = ready
                    .iter()
                    .find(|f| f.media_path == input.config.media_path && f.media_path.is_some())
                    .cloned()
                    .unwrap_or_default();
            }
            fx.config = input.config.clone();
        }
    }

    pub(crate) fn close_properties(&mut self) {
        self.properties = None;
        self.properties_fx = None;
        self.properties_audio_original = None;
        self.properties_texture = None;
        self.properties_loading = false;
        self.properties_token = self.properties_token.wrapping_add(1);
    }

    pub(crate) fn apply_properties(&mut self, mut draft: InputState) -> bool {
        if self.properties_loading {
            self.say(
                "Wait for the PNG to finish loading before applying".into(),
                true,
            );
            return false;
        }
        self.sync_fx_from_prop(draft.id);
        let mut fx = self.properties_fx.clone().unwrap_or_default();
        if fx.custom_rgba.is_none()
            && let Some(ready) = self.input_fx.get(&draft.id)
            && ready.media_path == fx.media_path
            && ready.custom_rgba.is_some()
        {
            fx.custom_rgba = ready.custom_rgba.clone();
            fx.image_size = ready.image_size;
            fx.image_pending = false;
            fx.image_error = None;
        }
        draft.config = fx.config.clone();
        let audio = (self.properties_audio_original.as_ref() != Some(&draft.audio))
            .then(|| draft.audio.clone());
        if self
            .try_execute(Command::ApplyInput {
                input: draft.clone(),
                audio,
            })
            .is_none()
        {
            self.properties = Some(draft);
            return false;
        }
        self.input_fx.insert(draft.id, fx);
        self.close_properties();
        true
    }

    pub(crate) fn request_add_input(&mut self) -> bool {
        if self.add_loading {
            return false;
        }
        let name = self.add_name.trim().to_owned();
        let path = PathBuf::from(&name);
        let is_file = path.extension().is_some() || name.contains(['/', '\\']);
        if self.add_tab == 1 && !is_file {
            self.say("Choose a PNG file for this image input".into(), true);
            return false;
        }
        if name.is_empty() {
            self.say("Enter a source name before adding an input".into(), true);
            return false;
        }
        let index = self.add_number.saturating_sub(1);
        let command = Command::AddInputAt {
            name: name.clone(),
            kind: self.add_kind,
            config: InputConfig::default(),
            index,
        };
        if let Err(error) = self.engine.enabled(&command) {
            self.say(error.to_string(), true);
            return false;
        }
        if self.engine.project().inputs.len() >= 32 || index > self.engine.project().inputs.len() {
            self.say(
                "Choose an available input position (maximum 32 inputs)".into(),
                true,
            );
            return false;
        }
        if is_file {
            if !path
                .extension()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.eq_ignore_ascii_case("png"))
            {
                self.say(
                    "Only PNG images are supported; video decoding is unavailable".into(),
                    true,
                );
                return false;
            }
            self.add_loading = self.media_loader.enqueue(
                path,
                media::MediaTarget::Add {
                    position: index,
                    token: self.add_token,
                },
            );
            if self.add_loading {
                self.say("Loading PNG…".into(), false);
            } else {
                self.say(
                    "PNG queue is full; try again after loading finishes".into(),
                    true,
                );
            }
            false
        } else {
            self.try_execute(command).is_some()
        }
    }

    pub(crate) fn request_property_png(&mut self, id: InputId, path: PathBuf) {
        if self.properties_loading {
            return;
        }
        self.properties_loading = self.media_loader.enqueue(
            path,
            media::MediaTarget::Properties {
                id,
                token: self.properties_token,
            },
        );
        if !self.properties_loading {
            self.say(
                "PNG queue is full; try again after loading finishes".into(),
                true,
            );
        }
    }

    pub(crate) fn poll_media(&mut self) {
        self.sync_input_cache();
        for (id, fx) in &mut self.input_fx {
            if fx.custom_rgba.is_none()
                && !fx.image_pending
                && fx.image_error.is_none()
                && let Some(path) = fx.media_path.clone()
            {
                fx.image_pending = self.media_loader.enqueue(
                    path.clone().into(),
                    media::MediaTarget::Restore { id: *id, path },
                );
            }
        }
        let Some((target, result)) = self.media_loader.poll() else {
            return;
        };
        match target {
            media::MediaTarget::Add { position, token }
                if self.add_open && token == self.add_token =>
            {
                self.add_loading = false;
                match result {
                    Ok(image) => {
                        let mut name = String::new();
                        for ch in Path::new(&image.path)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("PNG image")
                            .chars()
                        {
                            if name.len() + ch.len_utf8() > 64 {
                                break;
                            }
                            name.push(ch);
                        }
                        let config = InputConfig {
                            media_path: Some(image.path.clone()),
                            ..Default::default()
                        };
                        if let Some(result) = self.try_execute(Command::AddInputAt {
                            name,
                            kind: InputKind::Color,
                            config,
                            index: position,
                        }) && let Some(id) = result.affected.first()
                        {
                            self.input_fx.entry(*id).or_default().set_image(image);
                            self.add_open = false;
                        }
                    }
                    Err(error) => self.say(format!("{error}; no input added"), true),
                }
            }
            media::MediaTarget::Properties { id, token }
                if self.properties.as_ref().is_some_and(|p| p.id == id)
                    && token == self.properties_token =>
            {
                self.properties_loading = false;
                match result {
                    Ok(image) => {
                        self.properties_fx.get_or_insert_default().set_image(image);
                        self.say("PNG ready · press OK to apply".into(), false);
                    }
                    Err(error) => self.say(format!("{error}; current source retained"), true),
                }
            }
            media::MediaTarget::Restore { id, path }
                if self
                    .engine
                    .project()
                    .input(id)
                    .is_some_and(|i| i.config.media_path.as_ref() == Some(&path)) =>
            {
                let fx = self.input_fx.entry(id).or_default();
                fx.image_pending = false;
                match result {
                    Ok(image) => {
                        fx.set_image(image);
                        fx.media_path = Some(path);
                    }
                    Err(error) => {
                        fx.image_error = Some(error.clone());
                        self.say(
                            format!(
                                "Input {}: {error} · use Change in Input Settings to relink",
                                id.0
                            ),
                            true,
                        );
                    }
                }
            }
            _ => {} // A closed editor or replaced project cannot receive this image.
        }
        self.last_demo_frame = None;
    }

    pub(crate) fn draft_texture(
        &mut self,
        ctx: &egui::Context,
        draft: &InputState,
    ) -> Option<egui::TextureId> {
        let fx = self.properties_fx.as_ref()?;
        let ms = self
            .engine
            .playback(draft.id)
            .map_or(0, |p| p.position_ms());
        let base = render::source(draft, 384, 216, ms).ok()?;
        let pixels = if draft.online {
            panels::process_source_pixels(&base, 384, 216, fx, None)
        } else {
            base
        };
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [384, 216],
            &pixels.into_iter().flatten().collect::<Vec<_>>(),
        );
        if let Some(texture) = &mut self.properties_texture {
            texture.set(image, egui::TextureOptions::LINEAR);
        } else {
            self.properties_texture =
                Some(ctx.load_texture("input.draft", image, egui::TextureOptions::LINEAR));
        }
        self.properties_texture
            .as_ref()
            .map(egui::TextureHandle::id)
    }
}
