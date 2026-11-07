#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self, Color32, FontId, RichText, Stroke, TextStyle, Vec2};
use gooblelab::{
    google,
    notebook::{self, NotebookConfig, Template},
};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};

const REPO: &str = "https://github.com/narcisoJavier/GoobleLab";
const NOTEBOOK_URL: &str = "https://colab.research.google.com/github/narcisoJavier/GoobleLab/blob/main/MiniMax_H3_Colab.ipynb";
const INK: Color32 = Color32::from_rgb(23, 32, 31);
const MUTED: Color32 = Color32::from_rgb(88, 96, 82);
const RAIL: Color32 = Color32::from_rgb(244, 198, 42);
const GROUND: Color32 = Color32::from_rgb(246, 241, 227);
const ACCENT: Color32 = Color32::from_rgb(37, 61, 54);
const SURFACE: Color32 = Color32::from_rgb(255, 253, 247);
const BORDER: Color32 = Color32::from_rgb(204, 201, 185);

fn save_notebook(config: &NotebookConfig, path: &std::path::Path) -> Result<String, String> {
    let bytes = notebook::build_notebook(config, None)?;
    std::fs::write(path, bytes).map_err(|e| format!("Could not save notebook: {e}"))?;
    Ok(format!(
        "Saved {}. Open it in Colab; all settings remain editable.",
        path.display()
    ))
}

enum Message {
    SignedIn(Result<google::Session, String>),
    Uploaded(google::Session, Result<google::Published, String>),
}

struct Builder {
    config: NotebookConfig,
    step: usize,
    catalog: serde_json::Value,
    client_path: Option<PathBuf>,
    session: Option<google::Session>,
    published: Option<google::Published>,
    receiver: Option<mpsc::Receiver<Message>>,
    cancel: Arc<AtomicBool>,
    signing_in: bool,
    notice: Option<(bool, String)>,
    capture: Option<PathBuf>,
    frames: usize,
}

impl Builder {
    fn new(cc: &eframe::CreationContext<'_>, capture: Option<PathBuf>, step: usize) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        let windows = std::env::var_os("WINDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("C:/Windows"));
        for (name, file, family) in [
            ("windows-ui", "segoeui.ttf", egui::FontFamily::Proportional),
            (
                "guidance",
                "georgia.ttf",
                egui::FontFamily::Name("guidance".into()),
            ),
        ] {
            if let Ok(bytes) = std::fs::read(windows.join("Fonts").join(file)) {
                fonts
                    .font_data
                    .insert(name.into(), Arc::new(egui::FontData::from_owned(bytes)));
                fonts
                    .families
                    .entry(family)
                    .or_default()
                    .insert(0, name.into());
            }
        }
        let fallback_fonts = fonts.families[&egui::FontFamily::Proportional].clone();
        fonts
            .families
            .entry(egui::FontFamily::Name("guidance".into()))
            .or_insert(fallback_fonts);
        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.set_theme(egui::Theme::Light);
        let mut style = (*cc.egui_ctx.style_of(egui::Theme::Light)).clone();
        style.spacing.item_spacing = Vec2::new(10.0, 10.0);
        style.spacing.interact_size.y = 36.0;
        style.spacing.button_padding = Vec2::new(14.0, 9.0);
        style.spacing.scroll = egui::style::ScrollStyle::solid();
        style
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(16.0));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::proportional(16.0));
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::proportional(28.0));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::proportional(13.0));
        style.visuals = egui::Visuals::light();
        style.visuals.override_text_color = Some(INK);
        style.visuals.panel_fill = GROUND;
        style.visuals.hyperlink_color = ACCENT;
        style.visuals.selection.bg_fill = ACCENT;
        style.visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);
        style.visuals.widgets.inactive.bg_fill = SURFACE;
        style.visuals.widgets.inactive.weak_bg_fill = SURFACE;
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
        style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(239, 230, 198);
        style.visuals.widgets.active.weak_bg_fill = Color32::from_rgb(230, 219, 179);
        cc.egui_ctx.set_style_of(egui::Theme::Light, style);
        // Review the actual local-export result without requiring a file-picker interaction.
        let notice = if cfg!(debug_assertions)
            && std::env::args().any(|arg| arg == "--preview-export")
            && let Some(path) = &capture
        {
            Some(
                match save_notebook(&NotebookConfig::default(), &path.with_extension("ipynb")) {
                    Ok(message) => (false, message),
                    Err(error) => (true, error),
                },
            )
        } else {
            None
        };
        Self {
            config: NotebookConfig::default(),
            step,
            catalog: serde_json::from_str(include_str!("../../presets.json")).unwrap(),
            client_path: None,
            session: None,
            published: None,
            receiver: None,
            cancel: Arc::new(AtomicBool::new(false)),
            signing_in: false,
            notice,
            capture,
            frames: 0,
        }
    }

    fn busy(&self) -> bool {
        self.receiver.is_some()
    }

    fn primary(ui: &mut egui::Ui, text: &str) -> egui::Response {
        ui.add(egui::Button::new(RichText::new(text).color(Color32::WHITE)).fill(ACCENT))
    }

    fn field(ui: &mut egui::Ui, label: &str, value: &mut String) {
        let id = ui.label(RichText::new(label).strong()).id;
        ui.add(egui::TextEdit::singleline(value).desired_width(f32::INFINITY))
            .labelled_by(id);
    }

    fn title(ui: &mut egui::Ui, title: &str, subtitle: &str) {
        ui.heading(title);
        ui.label(
            RichText::new(subtitle)
                .font(FontId::new(16.0, egui::FontFamily::Name("guidance".into())))
                .color(MUTED),
        );
        ui.add_space(16.0);
    }

    fn nav(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("progress")
            .exact_size(205.0)
            .resizable(false)
            .frame(egui::Frame::new().fill(RAIL).inner_margin(24))
            .show(ui, |ui| {
                ui.visuals_mut().override_text_color = Some(INK);
                ui.label(RichText::new("GoobleLab").size(25.0).strong().color(INK));
                ui.add_space(10.0);
                ui.label(
                    RichText::new("Build your next\nColab notebook.")
                        .font(FontId::new(16.0, egui::FontFamily::Name("guidance".into())))
                        .color(INK),
                );
                ui.add_space(32.0);
                for (i, label) in ["1  Template", "2  Settings", "3  Export/upload"]
                    .iter()
                    .enumerate()
                {
                    let selected = self.step == i;
                    let fill = if selected { SURFACE } else { RAIL };
                    if selected {
                        let rect = egui::Rect::from_min_size(
                            ui.cursor().min,
                            Vec2::new(ui.available_width() + 24.0, 44.0),
                        );
                        ui.painter().rect_filled(rect, 0, SURFACE);
                    }
                    if ui
                        .add_enabled(
                            !self.busy(),
                            egui::Button::new(RichText::new(*label).color(INK))
                                .fill(fill)
                                .stroke(Stroke::NONE)
                                .min_size(Vec2::new(155.0, 44.0)),
                        )
                        .clicked()
                    {
                        if i == 2 {
                            match notebook::validate(&self.config) {
                                Ok(()) => {
                                    self.step = i;
                                    self.notice = None;
                                    self.published = None;
                                }
                                Err(error) => {
                                    self.step = 1;
                                    self.notice = Some((true, error));
                                }
                            }
                        } else {
                            self.step = i;
                            self.notice = None;
                        }
                    }
                }
                ui.add_space((ui.available_height() - 130.0).max(20.0));
                ui.label(
                    RichText::new("Optional desktop companion.\nThe notebook does the work.")
                        .size(13.0)
                        .color(INK),
                );
                ui.add_space(12.0);
                ui.hyperlink_to(RichText::new("Project & notebooks").color(INK), REPO);
            });
    }

    fn summary(&self, ui: &mut egui::Ui) {
        egui::Panel::right("summary")
            .exact_size(235.0)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(238, 233, 220))
                    .inner_margin(22),
            )
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("notebook-plan")
                    .show(ui, |ui| {
                        ui.label(RichText::new("Notebook plan").size(19.0).strong());
                        ui.add_space(20.0);
                        let preset = &self.catalog["presets"][&self.config.preset_id];
                        let mut frames =
                            (self.config.seconds * 24.0).round_ties_even().max(5.0) as u32;
                        frames += (5 + 17 - frames % 17) % 17;
                        let image = self
                            .config
                            .image_path
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .map(|s| s.to_string_lossy().into_owned());
                        for (label, value) in [
                            ("Workflow", self.config.template.label().to_owned()),
                            ("Model", "MiniMax H3".into()),
                            (
                                "Preset",
                                preset["label"].as_str().unwrap_or("").replace("H3 ", ""),
                            ),
                            (
                                "Output",
                                format!(
                                    "{} × {}\n{:.2} seconds · 24 fps",
                                    self.config.width,
                                    self.config.height,
                                    frames as f64 / 24.0
                                ),
                            ),
                            ("Colab runtime", "A100 · High RAM".into()),
                            ("Input image", image.unwrap_or_else(|| "None".into())),
                            (
                                "Custom LoRA",
                                if self.config.lora_source.trim().is_empty() {
                                    "None".into()
                                } else {
                                    format!("Strength {:.2}", self.config.lora_strength)
                                },
                            ),
                        ] {
                            ui.label(RichText::new(label).size(13.0).color(MUTED));
                            ui.label(RichText::new(value).strong());
                            ui.add_space(12.0);
                        }
                        ui.separator();
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Template > notebook > Colab")
                                .size(13.0)
                                .color(MUTED),
                        );
                        ui.label(
                    RichText::new(
                        "GPU execution happens in Colab. No local GPU needed for this builder.",
                    )
                    .size(13.0)
                    .color(MUTED),
                );
                    });
            });
    }

    fn template(&mut self, ui: &mut egui::Ui) {
        Self::title(
            ui,
            "Choose your workflow",
            "Start with a supported MiniMax H3 template.",
        );
        for (template, description) in [
            (
                Template::TextToVideo,
                "Describe a scene, motion and sound. No input image needed.",
            ),
            (
                Template::ImageToVideo,
                "Bring a first frame and describe what happens next.",
            ),
            (
                Template::ReferenceToVideo,
                "Guide the character or style with a reference image.",
            ),
        ] {
            let selected = self.config.template == template;
            let mut text = egui::text::LayoutJob::default();
            text.append(
                &format!("{}\n", template.label()),
                0.0,
                egui::TextFormat {
                    font_id: FontId::proportional(18.0),
                    color: INK,
                    ..Default::default()
                },
            );
            text.append(
                description,
                0.0,
                egui::TextFormat {
                    font_id: FontId::new(15.0, egui::FontFamily::Name("guidance".into())),
                    color: MUTED,
                    ..Default::default()
                },
            );
            if ui
                .add_sized(
                    [ui.available_width(), 105.0],
                    egui::Button::new(text)
                        .fill(if selected {
                            Color32::from_rgb(255, 249, 219)
                        } else {
                            SURFACE
                        })
                        .stroke(Stroke::new(
                            if selected { 2.0 } else { 1.0 },
                            if selected { ACCENT } else { BORDER },
                        )),
                )
                .clicked()
            {
                self.config.template = template;
                self.config.preset_id = if template == Template::ReferenceToVideo {
                    "ref2va-base"
                } else {
                    "fl2va-turbo-8"
                }
                .into();
                if template == Template::TextToVideo {
                    self.config.image_path = None;
                }
                self.published = None;
            }
            ui.add_space(4.0);
        }
        ui.add_space(12.0);
        ui.label(
            RichText::new("Curated templates use the same native ComfyUI nodes as the notebook.")
                .size(13.0)
                .color(MUTED),
        );
        ui.hyperlink_to(
            "Browse the official ComfyUI gallery",
            "https://comfy.org/workflows",
        );
        ui.label(RichText::new("Other gallery workflows are not imported automatically; their models and nodes must be supported first.").size(13.0).color(MUTED));
    }

    fn settings(&mut self, ui: &mut egui::Ui) {
        Self::title(
            ui,
            "Configure your video",
            "These choices are saved as editable notebook settings.",
        );
        Self::field(ui, "Project name", &mut self.config.name);
        ui.add_space(6.0);
        ui.label(RichText::new("Model preset").strong());
        let label = self.catalog["presets"][&self.config.preset_id]["label"]
            .as_str()
            .unwrap_or("Select a preset");
        let family = if self.config.template == Template::ReferenceToVideo {
            "ref2va"
        } else {
            "fl2va"
        };
        egui::ComboBox::from_id_salt("preset")
            .selected_text(label)
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for (id, preset) in self.catalog["presets"].as_object().unwrap() {
                    if preset["family"] == family {
                        ui.selectable_value(
                            &mut self.config.preset_id,
                            id.clone(),
                            preset["label"].as_str().unwrap(),
                        );
                    }
                }
            });
        if self.config.preset_id == "ref2va-turbo-4" {
            ui.colored_label(
                Color32::from_rgb(132, 79, 12),
                "Experimental reference Turbo: trained at 544p. Try 960 × 544.",
            );
        }
        ui.add_space(8.0);
        let prompt_label = ui.label(RichText::new("Prompt").strong()).id;
        if self.config.template == Template::ReferenceToVideo {
            ui.label(
                RichText::new("Refer to your image as <Picture 1> in the prompt.")
                    .size(13.0)
                    .color(MUTED),
            );
        }
        ui.add(
            egui::TextEdit::multiline(&mut self.config.prompt)
                .desired_rows(7)
                .desired_width(f32::INFINITY),
        )
        .labelled_by(prompt_label);
        ui.add_space(8.0);
        egui::Grid::new("output-settings")
            .num_columns(2)
            .spacing([22.0, 12.0])
            .show(ui, |ui| {
                ui.label("Resolution");
                let mut resolution = (self.config.width, self.config.height);
                egui::ComboBox::from_id_salt("resolution")
                    .selected_text(format!("{} × {}", resolution.0, resolution.1))
                    .show_ui(ui, |ui| {
                        for (size, label) in [
                            ((1344, 768), "Landscape · 1344 × 768"),
                            ((768, 1344), "Portrait · 768 × 1344"),
                            ((960, 544), "Reference draft · 960 × 544"),
                            ((864, 480), "Draft · 864 × 480"),
                            ((768, 768), "Square · 768 × 768"),
                        ] {
                            ui.selectable_value(&mut resolution, size, label);
                        }
                    });
                (self.config.width, self.config.height) = resolution;
                ui.end_row();
                ui.label("Duration (seconds)");
                ui.add(
                    egui::DragValue::new(&mut self.config.seconds)
                        .range(4.0..=15.0)
                        .speed(0.5),
                );
                ui.end_row();
                ui.label("Seed");
                ui.add(egui::DragValue::new(&mut self.config.seed));
                ui.end_row();
            });
        if self.config.template != Template::TextToVideo {
            ui.add_space(12.0);
            ui.label(RichText::new("Input image").strong());
            ui.horizontal_wrapped(|ui| {
                if ui.button("Choose image…").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                        .pick_file()
                {
                    self.config.image_path = Some(path);
                }
                if let Some(path) = &self.config.image_path {
                    ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                } else {
                    ui.label("No image selected");
                }
            });
            ui.label(RichText::new("Drive publishing uploads this image with the notebook. Local export asks for it inside Colab. PNG/JPEG/WebP, up to 25 MiB.").size(13.0).color(MUTED));
        }
        ui.add_space(10.0);
        egui::CollapsingHeader::new("Optional custom LoRA").show(ui, |ui| {
            Self::field(ui, "Hugging Face file URL or mounted Drive path", &mut self.config.lora_source);
            ui.add(egui::Slider::new(&mut self.config.lora_strength, 0.0..=2.0).text("Strength"));
            ui.label(RichText::new("Use a ComfyUI-compatible LoRA for the selected H3 family. Acceleration LoRAs are already included in Turbo presets. Put private access tokens in Colab Secrets, never in this field.").size(13.0).color(MUTED));
        });
    }

    fn export(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        Self::title(
            ui,
            "Your notebook is ready",
            "Export it locally or publish a private copy to your Drive.",
        );
        ui.label(RichText::new(&self.config.name).size(20.0).strong());
        ui.label(format!(
            "{} · {}",
            self.config.template.label(),
            self.config.preset_id
        ));
        ui.add_space(18.0);
        if Self::primary(ui, "Save notebook…").clicked()
            && let Some(path) = rfd::FileDialog::new()
                .set_file_name(format!("{}.ipynb", notebook::file_stem(&self.config.name)))
                .add_filter("Colab notebook", &["ipynb"])
                .save_file()
        {
            let result = save_notebook(&self.config, &path);
            self.notice = Some(match result {
                Ok(message) => (false, message),
                Err(e) => (true, e),
            });
        }
        ui.label(
            RichText::new("No Google sign-in needed for notebook export.")
                .size(13.0)
                .color(MUTED),
        );
        ui.add_space(22.0);
        ui.separator();
        ui.add_space(18.0);
        ui.label(RichText::new("Publish to Google Drive").size(20.0).strong());
        ui.label("Upload the notebook and selected image together, then open Colab.");
        ui.add_space(8.0);
        ui.hyperlink_to(
            "One-time Google OAuth setup instructions",
            format!("{REPO}/blob/main/docs/GOOGLE_OAUTH.md"),
        );
        ui.horizontal_wrapped(|ui| {
            if ui.button("Choose Desktop OAuth JSON…").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("Google Desktop client", &["json"])
                    .pick_file()
            {
                self.client_path = Some(path);
                self.session = None;
            }
            if let Some(path) = &self.client_path {
                ui.label(path.file_name().unwrap_or_default().to_string_lossy());
            }
        });
        if self.session.is_none() {
            if ui
                .add_enabled(
                    self.client_path.is_some(),
                    egui::Button::new("Sign in with Google"),
                )
                .clicked()
            {
                let path = self.client_path.clone().unwrap();
                self.cancel = Arc::new(AtomicBool::new(false));
                let cancel = self.cancel.clone();
                let (tx, rx) = mpsc::channel();
                self.receiver = Some(rx);
                self.signing_in = true;
                self.notice = Some((
                    false,
                    "Complete Google sign-in in your system browser. You can cancel below.".into(),
                ));
                let ctx = ctx.clone();
                std::thread::spawn(move || {
                    let _ = tx.send(Message::SignedIn(google::sign_in(&path, cancel)));
                    ctx.request_repaint();
                });
            }
        } else {
            ui.horizontal(|ui| {
                ui.label("Google Drive connected for this session.");
                if ui.small_button("Forget session").clicked() {
                    self.session = None;
                }
            });
            if Self::primary(ui, "Upload to Drive & open Colab").clicked() {
                let mut session = self.session.clone().unwrap();
                let config = self.config.clone();
                let (tx, rx) = mpsc::channel();
                self.receiver = Some(rx);
                self.signing_in = false;
                self.notice = Some((
                    false,
                    "Creating a private project folder and uploading your notebook…".into(),
                ));
                let ctx = ctx.clone();
                std::thread::spawn(move || {
                    let result = google::upload_project(&mut session, &config);
                    let _ = tx.send(Message::Uploaded(session, result));
                    ctx.request_repaint();
                });
            }
        }
        ui.add_space(12.0);
        ui.label(RichText::new("Tokens stay in app memory and are never written into the notebook. Each publish creates a new private folder; existing files are preserved.").size(13.0).color(MUTED));
        if let Some(published) = &self.published {
            ui.add_space(16.0);
            ui.hyperlink_to("Open uploaded notebook in Colab", &published.colab_url);
            ui.hyperlink_to("View the project folder in Drive", &published.drive_url);
        }
        ui.add_space(20.0);
        ui.label(RichText::new("In Colab").strong());
        ui.label("Choose A100 + high RAM, then run Steps 1–5. Step 6 leaves the backend running unless you enable its stop switch. Delete the runtime when finished to release the GPU.");
    }

    fn poll_job(&mut self) {
        let message = self.receiver.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(message) = message {
            self.receiver = None;
            self.signing_in = false;
            match message {
                Message::SignedIn(result) => match result {
                    Ok(session) => {
                        self.session = Some(session);
                        self.notice = Some((
                            false,
                            "Connected. Your notebook can now be uploaded to Drive.".into(),
                        ));
                    }
                    Err(error) => {
                        self.notice = Some((true, error));
                    }
                },
                Message::Uploaded(session, result) => {
                    self.session = Some(session);
                    match result {
                        Ok(published) => {
                            let opened = webbrowser::open(&published.colab_url).is_ok();
                            self.published = Some(published);
                            self.notice = Some((false, if opened { "Uploaded. Colab is open in your browser; choose a runtime and run the notebook." } else { "Uploaded. Use the Colab link below to open your notebook." }.into()));
                        }
                        Err(error) => {
                            self.notice = Some((true, error));
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for Builder {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_job();
        let ctx = ui.ctx().clone();
        self.nav(ui);
        self.summary(ui);
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(GROUND).inner_margin(28))
            .show(ui, |ui| {
                let busy = self.busy();
                let footer_height = if self.notice.is_some() { 190.0 } else { 82.0 };
                let scroll_height = (ui.available_height() - footer_height).max(200.0);
                egui::ScrollArea::vertical()
                    .id_salt("work-area")
                    .max_height(scroll_height)
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_enabled_ui(!busy, |ui| match self.step {
                            0 => self.template(ui),
                            1 => self.settings(ui),
                            _ => self.export(ui, &ctx),
                        });
                    });
                ui.add_space(16.0);
                if let Some((error, text)) = &self.notice {
                    egui::ScrollArea::vertical()
                        .id_salt("status-message")
                        .max_height(90.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new(text).color(if *error {
                                Color32::from_rgb(166, 38, 38)
                            } else {
                                ACCENT
                            }));
                        });
                    ui.add_space(8.0);
                }
                if busy {
                    ui.horizontal_wrapped(|ui| {
                        ui.spinner();
                        ui.label(if self.signing_in {
                            "Waiting for browser sign-in…"
                        } else {
                            "Uploading to your Drive…"
                        });
                        if self.signing_in && ui.button("Cancel sign-in").clicked() {
                            self.cancel.store(true, Ordering::Relaxed);
                        }
                    });
                    ctx.request_repaint_after(Duration::from_millis(100));
                } else {
                    ui.horizontal(|ui| {
                        if self.step > 0 && ui.button("Back").clicked() {
                            self.step -= 1;
                            self.notice = None;
                        }
                        if self.step == 0 && Self::primary(ui, "Continue to settings").clicked() {
                            self.step = 1;
                            self.notice = None;
                        }
                        if self.step == 1 && Self::primary(ui, "Review notebook").clicked() {
                            match notebook::validate(&self.config) {
                                Ok(()) => {
                                    self.step = 2;
                                    self.notice = None;
                                    self.published = None;
                                }
                                Err(e) => self.notice = Some((true, e)),
                            }
                        }
                    });
                    ui.add_space(6.0);
                    ui.hyperlink_to("Use the notebook directly", NOTEBOOK_URL);
                }
            });
        self.frames += 1;
        if self.frames >= 3 {
            if let Some(path) = self.capture.take() {
                let capture_ctx = ctx.clone();
                ctx.request_screenshot(move |image| {
                    let bytes: Vec<u8> = image
                        .pixels
                        .iter()
                        .flat_map(|pixel| pixel.to_array())
                        .collect();
                    let _ = image::save_buffer(
                        path,
                        &bytes,
                        image.size[0] as u32,
                        image.size[1] as u32,
                        image::ColorType::Rgba8,
                    );
                    capture_ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                });
            }
        } else if self.capture.is_some() {
            ctx.request_repaint();
        }
    }
}

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).is_some_and(|a| a == "--export-default") {
        let path = args.get(2).expect("--export-default needs a filename");
        save_notebook(&NotebookConfig::default(), std::path::Path::new(path))
            .expect("save notebook");
        return Ok(());
    }
    let capture = args
        .iter()
        .position(|a| a == "--screenshot")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);
    let step = args
        .iter()
        .position(|a| a == "--step")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0)
        .min(2);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(if args.iter().any(|a| a == "--compact") {
                [900.0, 660.0]
            } else {
                [1100.0, 760.0]
            })
            .with_min_inner_size([900.0, 660.0]),
        ..Default::default()
    };
    eframe::run_native(
        "GoobleLab — Notebook Builder",
        options,
        Box::new(move |cc| Ok(Box::new(Builder::new(cc, capture, step)))),
    )
}
