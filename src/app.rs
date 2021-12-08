use std::io::Write;

use eframe::{egui, epi};
use nfd;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    editor_text: String,
    save_path: Option<String>,
    output: String,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            editor_text: String::new(),
            save_path: None,
            output: String::new(),
        }
    }
}

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "Intuitive GUI"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }
    
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, _: &mut epi::Frame<'_>) {
        let Self { editor_text, save_path , output} = self;
        
        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("Select Output Folder").clicked() {
                        *save_path = save_dialog(save_path.as_deref())
                    }
                    else if ui.button("Compile").clicked() {
                        while save_path.is_none() {
                            *save_path = save_dialog(save_path.as_deref())
                        }
                        
                        let save = save_path.as_ref().unwrap().trim_end_matches(".iv");
                        let inp = save.to_owned() + ".iv";
                        
                        std::fs::write(&inp, editor_text.clone()).unwrap();
                        let out = std::process::Command::new("intuitive")
                            .current_dir(std::env::current_dir().unwrap())
                            .args([inp, save.into()])
                            .output().unwrap();
                        
                        let stdout = std::str::from_utf8(&out.stdout).unwrap();
                        let stderr = std::str::from_utf8(&out.stderr).unwrap();
                        *output = if stdout.is_empty() {
                            std::io::stderr().write_all(&out.stderr).unwrap();
                            stderr.to_owned()
                        }
                        else {
                            std::io::stdout().write_all(&out.stdout).unwrap();
                            stdout.to_owned()
                        }
                    };
                });
                ui.label(format!("Output: {}", output));
            });
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                egui::ScrollArea::vertical().show(ui, |ui | { 
                    ui.add_sized(ui.available_size(), egui::TextEdit::multiline(editor_text).code_editor())
                });    
            });
        });
    }
}

fn save_dialog(default_path: Option<&str>) -> Option<String> {
    let file = nfd::open_save_dialog(None, default_path).unwrap();
        match file {
            nfd::Response::Okay(path) => {
                let p = std::path::Path::new(&path);
                std::env::set_current_dir(p.parent().unwrap()).unwrap();
                Some(p.file_name().unwrap().to_str().unwrap().into())
            },
            nfd::Response::OkayMultiple(_) | nfd::Response::Cancel => None,
        }
}
