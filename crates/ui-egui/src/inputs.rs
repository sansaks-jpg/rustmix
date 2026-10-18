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
            if let Some(fx) = self.add_image.clone()
                && fx.media_path.as_deref() == Some(name.as_str())
            {
                return self.commit_png_input(fx, index);
            }
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

    fn commit_png_input(&mut self, fx: InputFx, index: usize) -> bool {
        let mut name = String::new();
        let path = fx.media_path.as_deref().unwrap_or("PNG image.png");
        for ch in Path::new(path)
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
        if let Some(result) = self.try_execute(Command::AddInputAt {
            name,
            kind: InputKind::Color,
            config: fx.config.clone(),
            index,
        }) && let Some(id) = result.affected.first()
        {
            self.input_fx.insert(*id, fx);
            return true;
        }
        false
    }

    pub(crate) fn start_png_picker(&mut self, target: media::MediaTarget) {
        if self.image_picker.is_some() {
            self.say("A source file dialog is already open".into(), true);
            return;
        }
        match target {
            media::MediaTarget::AddPreview { .. } => self.add_loading = true,
            media::MediaTarget::Properties { .. } => self.properties_loading = true,
            _ => return,
        }
        let (tx, rx) = mpsc::sync_channel(1);
        self.image_picker = Some(rx);
        std::thread::spawn(move || {
            let path = rfd::FileDialog::new()
                .add_filter("PNG image", &["png"])
                .pick_file();
            let _ = tx.send(media::PickerDone { target, path });
        });
    }

    fn poll_image_picker(&mut self) {
        let Some(receiver) = &self.image_picker else {
            return;
        };
        let done = match receiver.try_recv() {
            Ok(done) => done,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.image_picker = None;
                self.add_loading = false;
                self.properties_loading = false;
                self.say(
                    "Source file dialog closed unexpectedly; current source retained".into(),
                    true,
                );
                return;
            }
        };
        self.image_picker = None;
        match done.target {
            media::MediaTarget::AddPreview { token }
                if self.add_open && token == self.add_token =>
            {
                self.add_loading = false;
                if let Some(path) = done.path {
                    self.add_loading = self
                        .media_loader
                        .enqueue(path, media::MediaTarget::AddPreview { token });
                }
            }
            media::MediaTarget::Properties { id, token }
                if self.properties.as_ref().is_some_and(|p| p.id == id)
                    && token == self.properties_token =>
            {
                self.properties_loading = false;
                if let Some(path) = done.path {
                    self.request_property_png(id, path);
                }
            }
            _ => {}
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
        self.poll_image_picker();
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
            media::MediaTarget::AddPreview { token }
                if self.add_open && token == self.add_token =>
            {
                self.add_loading = false;
                match result {
                    Ok(image) => {
                        self.add_name = image.path.clone();
                        self.add_kind = InputKind::Color;
                        self.add_tab = 1;
                        let mut fx = InputFx::default();
                        fx.set_image(image);
                        self.add_image = Some(fx);
                        self.add_preview = None;
                        self.say("PNG ready · press OK to add input".into(), false);
                    }
                    Err(error) => self.say(format!("{error}; previous selection retained"), true),
                }
            }
            media::MediaTarget::Add { position, token }
                if self.add_open && token == self.add_token =>
            {
                self.add_loading = false;
                match result {
                    Ok(image) => {
                        let mut fx = InputFx::default();
                        fx.set_image(image);
                        if self.commit_png_input(fx, position) {
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
        if let Some(draft) = &self.properties
            && let Some(fx) = &mut self.properties_fx
            && fx.custom_rgba.is_none()
            && fx.media_path.is_some()
            && let Some(current) = self.input_fx.get(&draft.id)
            && current.media_path == fx.media_path
        {
            fx.custom_rgba = current.custom_rgba.clone();
            fx.image_size = current.image_size;
            fx.image_pending = current.image_pending;
            fx.image_error = current.image_error.clone();
        }
        self.last_demo_frame = None;
    }

    pub(crate) fn draft_texture(
        &mut self,
        ctx: &egui::Context,
        draft: &InputState,
    ) -> Option<egui::TextureId> {
        let fx = self.properties_fx.clone()?;
        let mut project = self.engine.project().clone();
        if let Some(input) = project.inputs.iter_mut().find(|i| i.id == draft.id) {
            *input = draft.clone();
            input.config = fx.config.clone();
        }
        let mut raw = HashMap::new();
        let mut ids = vec![draft.id];
        let mut cursor = 0;
        while cursor < ids.len() && ids.len() <= 32 {
            if let Some(pip) = ids
                .get(cursor)
                .and_then(|id| project.input(*id))
                .and_then(|i| i.config.pip_source)
                && !ids.contains(&pip)
            {
                ids.push(pip);
            }
            cursor += 1;
        }
        for id in ids {
            let input = project.input(id)?;
            let ms = self.engine.playback(id).map_or(0, |p| p.position_ms());
            raw.insert(id, render::source(input, 384, 216, ms).ok()?);
        }
        let mut cache = self.input_fx.clone();
        cache.insert(draft.id, fx);
        let processed = panels::processed_sources(&project, &raw, &cache, 384, 216);
        let pixels = processed.get(&draft.id)?;
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [384, 216],
            &pixels.iter().flatten().copied().collect::<Vec<_>>(),
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
