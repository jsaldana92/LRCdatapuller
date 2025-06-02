#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


use eframe::egui;
use std::fs;
use std::path::Path;
use std::collections::HashMap;
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;
use rodio::Source;

const NO_TASKS_SOUND: &[u8] = include_bytes!("assets/sounds/no_tasks_folder.mp3");
const SAD_TRUMPET_SOUND: &[u8] = include_bytes!("assets/sounds/sad_trumpet.mp3");
const SUCCESS_SOUND: &[u8] = include_bytes!("assets/sounds/success.mp3");

struct CsvTransferApp {
    tasks_root: String,
    selected_folder: Option<String>,
    subfolder_checkboxes: HashMap<String, bool>,
    include_csv: bool,
    include_txt: bool,
    exclude_monkey: bool,
    exclude_block: bool,
    custom_exclude: String,
    move_after_copy: bool,
    result_message: Option<String>,
    result_details: Vec<String>,
    subfolder_columns: usize,
    subfolder_rows: usize,
    show_help: bool,
    show_about: bool,
    show_missing_tasks_popup: bool,
    has_played_missing_tasks_sound: bool,
    _audio_stream: rodio::OutputStream,
    audio_handle: rodio::OutputStreamHandle,
    sink: Option<Sink>,



}

impl Default for CsvTransferApp {
    fn default() -> Self {
        let (stream, handle) = OutputStream::try_default().unwrap();
        let tasks_root = "C:\\Tasks".to_string();
        let show_missing_tasks_popup = !Path::new(&tasks_root).exists();

        Self {
            tasks_root,
            selected_folder: None,
            subfolder_checkboxes: HashMap::new(),
            include_csv: true,
            include_txt: false,
            exclude_monkey: false,
            exclude_block: false,
            custom_exclude: String::new(),
            move_after_copy: false,
            result_message: None,
            result_details: vec![],
            subfolder_columns: 0,
            subfolder_rows: 0,
            show_help: false,
            show_about: false,
            show_missing_tasks_popup,
            has_played_missing_tasks_sound: false,
            _audio_stream: stream,
            audio_handle: handle,
            sink: None,

        }
    }
}

impl CsvTransferApp {
    fn play_embedded_sound(&mut self, data: &'static [u8]) {
        if let Ok(source) = Decoder::new(Cursor::new(data)) {
            if let Ok(sink) = Sink::try_new(&self.audio_handle) {
                sink.append(Box::new(source) as Box<dyn Source<Item = i16> + Send + Sync>);
                self.sink.replace(sink);
            }
        }
    }
}


impl eframe::App for CsvTransferApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::from_rgb(120, 120, 110);  // Panel background
        visuals.override_text_color = Some(egui::Color32::from_rgb(255, 255, 255)); //text color


        ctx.set_visuals(visuals);


        if self.show_missing_tasks_popup {
            if !self.has_played_missing_tasks_sound {
                self.play_embedded_sound(NO_TASKS_SOUND);
                self.has_played_missing_tasks_sound = true;
            }

            egui::Window::new("Missing Folder")
                .collapsible(false)
                .resizable(false)
                .default_width(400.0)
                .default_height(300.0)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label("Missing C:/Tasks/ directory\n\nPlease make sure the Tasks folder is in the correct location and not in /Documents/ or /Desktop/");
                        ui.add_space(20.0);
                        if ui.button("Refresh").clicked() {
                            if Path::new(&self.tasks_root).exists() {
                                self.show_missing_tasks_popup = false;
                            } 
                        }
                    }
                    )
                });
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("File Copier");

            if let Ok(entries) = fs::read_dir(&self.tasks_root) {
                ui.label("Select a folder in C:\\Tasks\\:");
                let mut folders: Vec<String> = entries
                    .flatten()
                    .filter_map(|entry| {
                        let path = entry.path();
                        if path.is_dir() {
                            Some(path.file_name()?.to_string_lossy().to_string())
                        } else {
                            None
                        }
                    })
                    .collect();

                folders.sort();
                let max_per_column = 4;
                let columns = (folders.len() + max_per_column - 1) / max_per_column;

                egui::ScrollArea::horizontal()
                    .id_salt("folder_scroll")
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for col in 0..columns {
                                ui.vertical(|ui| {
                                    for row in 0..max_per_column {
                                        let idx = col * max_per_column + row;
                                        if let Some(folder_name) = folders.get(idx) {
                                            ui.horizontal(|ui| {
                                                let label = egui::RichText::new(folder_name).monospace();
                                                if ui.button(label).clicked() {
                                                    self.selected_folder = Some(folder_name.clone());
                                                    self.populate_subfolders();
                                                }
                                            });
                                        }
                                    }
                                });
                            }
                        });
                        ui.add_space(10.0);
                    });
            }

            if !self.subfolder_checkboxes.is_empty() {
                ui.separator();
                ui.label("Select subfolder(s):");
                let max_per_column = 6;
                let mut subfolder_names: Vec<_> = self.subfolder_checkboxes.iter_mut().collect();
                subfolder_names.sort_by_key(|(name, _)| *name);
                let columns = (subfolder_names.len() + max_per_column - 1) / max_per_column;

                self.subfolder_columns = columns;
                self.subfolder_rows = std::cmp::min(max_per_column, subfolder_names.len());

                egui::ScrollArea::horizontal()
                    .id_salt("subfolder_scroll")
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for col in 0..columns {
                                ui.vertical(|ui| {
                                    for row in 0..max_per_column {
                                        let idx = col * max_per_column + row;
                                        if let Some((name, checked)) = subfolder_names.get_mut(idx) {
                                            ui.horizontal(|ui| {
                                                ui.checkbox(*checked, "");
                                                ui.label(egui::RichText::new(name.as_str()).monospace());
                                            });
                                        }
                                    }
                                });
                            }
                        });
                        ui.add_space(10.0);
                    });
            }

            ui.separator();
            ui.label("File Type Options:");
            ui.checkbox(&mut self.include_csv, ".csv");
            ui.checkbox(&mut self.include_txt, ".txt (Excludes 'para-' files)");
            if self.include_txt {
                ui.indent("txt_options_indent", |ui| {
                    ui.checkbox(&mut self.exclude_monkey, "Exclude 'monkey-' files");
                    ui.checkbox(&mut self.exclude_block, "Exclude 'block-' files");
                    ui.horizontal(|ui| {
                        ui.label("Custom Exclude Prefix:");
                        ui.text_edit_singleline(&mut self.custom_exclude);
                    });
                });
            }

            ui.separator();
            ui.checkbox(&mut self.move_after_copy, "Move copied files into 'copied' folder");

            ui.vertical_centered(|ui| {
                if ui.button("Copy Selected Files to D:\\data_from_puller").clicked() {
                    if !Path::new("D:/").exists() {
                        self.result_message = Some("No D:/ detected.\nPlease insert a usb drive".to_string());
                        self.play_embedded_sound(SAD_TRUMPET_SOUND);
                        self.result_details.clear();
                    } else {
                        self.copy_selected_files();
                    }
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Help").clicked() {
                        self.show_help = true;
                    }
                    if ui.button("About").clicked() {
                        self.show_about = true;
                    }
                });
            });

            if self.show_help {
                egui::Window::new("Help")
                    .collapsible(false)
                    .resizable(false)
                    .default_width(420.0)
                    .show(ctx, |ui| {
                        ui.label("File Locations:");
                        ui.label("- Your files must be located in: C:/Tasks/[last name]/[program name]/");
                        ui.label("- Your USB drive must be plugged into D:/");
                        ui.label("- Transferred files go to: D:/data_from_puller/ (auto-created if missing)");
                        ui.separator();
                        ui.label("File Rules:");
                        ui.label("- .csv files: Always transferred");
                        ui.label("- .txt files: Transferred unless name starts with 'para' (e.g., 'parameters.txt')");
                        ui.separator();
                        ui.label("Optional Filters for .txt files:");
                        ui.label("- Exclude files starting with 'monkey' or 'block'");
                        ui.label("- Add a custom prefix to exclude other files (e.g., 'test')");
                        ui.separator();
                        ui.label("Move Option:");
                        ui.label("- If enabled, transferred files will be moved to:");
                        ui.label("  C:/Tasks/[last name]/[program name]/copied/");
                        ui.label("- Folder is created automatically if it doesn’t exist");

                        if ui.button("Close").clicked() {
                            self.show_help = false;
                        }
                    });
            }

            if self.show_about {
                egui::Window::new("About")
                    .collapsible(false)
                    .resizable(false)
                    .default_width(420.0)
                    .show(ctx, |ui| {
                        ui.label("This tool was designed as an attempt to simplify and streamline end-of-day data transfers.");
                        ui.add_space(10.0);
                        ui.label("Questions or feedback?");
                        ui.label("Please contact: jsaldana92@gmail.com");
                        ui.label("Repo: https://github.com/jsaldana92/LRCdatapuller");

                        if ui.button("Close").clicked() {
                            self.show_about = false;
                        }
                    });
            }

            if let Some(message) = self.result_message.clone() {
                egui::Window::new("Done")
                    .collapsible(false)
                    .resizable(true)
                    .min_width(475.0)
                    .min_height(4000.0)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(&message);
                            if !self.result_details.is_empty() {
                                ui.separator();
                                egui::ScrollArea::vertical()
                                    .max_height(120.0)
                                    .show(ui, |ui| {
                                        for file in &self.result_details {
                                            ui.label(file);
                                        }
                                    });
                            }
                            ui.add_space(10.0);
                            if ui.button("OK").clicked() {
                                self.result_message = None;
                                self.result_details.clear();
                            }
                        });
                    });
            }
        });
    }
}

impl CsvTransferApp {
    fn populate_subfolders(&mut self) {
        self.subfolder_checkboxes.clear();
        if let Some(folder) = &self.selected_folder {
            let full_path = Path::new(&self.tasks_root).join(folder);
            if let Ok(entries) = fs::read_dir(full_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path.file_name().unwrap().to_string_lossy().to_string();
                        self.subfolder_checkboxes.insert(name, false);
                    }
                }
            }
        }
    }

    fn copy_selected_files(&mut self) {
        let mut copied_count = 0;
        let mut no_subfolders_selected = true;
        self.result_details.clear();

        if let Some(folder) = &self.selected_folder {
            let base_path = Path::new(&self.tasks_root).join(folder);
            for (subfolder, checked) in &self.subfolder_checkboxes {
                if *checked {
                    no_subfolders_selected = false;
                    let subfolder_path = base_path.join(subfolder);
                    if let Ok(files) = fs::read_dir(&subfolder_path) {
                        for entry in files.flatten() {
                            let path = entry.path();
                            let file_name = entry.file_name().to_string_lossy().to_lowercase();
                            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                            let mut should_copy = false;

                            if self.include_csv && ext == "csv" {
                                should_copy = true;
                            } else if self.include_txt && ext == "txt" {
                                if file_name.starts_with("para") {
                                    continue;
                                }
                                if self.exclude_monkey && file_name.starts_with("monkey") {
                                    continue;
                                }
                                if self.exclude_block && file_name.starts_with("block") {
                                    continue;
                                }
                                if !self.custom_exclude.is_empty() && file_name.starts_with(&self.custom_exclude.to_lowercase()) {
                                    continue;
                                }
                                should_copy = true;
                            }

                            if should_copy {
                                let dest_dir = Path::new("D:/data_from_puller");
                                let _ = fs::create_dir_all(dest_dir);
                                let dest_path = dest_dir.join(entry.file_name());
                                let _ = fs::copy(&path, &dest_path);
                                copied_count += 1;
                                self.result_details.push(format!(
                                    "{} -> {}\n",
                                    path.to_string_lossy().replace("/", "\\"),
                                    dest_path.to_string_lossy().replace("/", "\\")
                                ));

                                if self.move_after_copy {
                                    let copied_dir = subfolder_path.join("copied");
                                    let _ = fs::create_dir_all(&copied_dir);
                                    let _ = fs::rename(&path, copied_dir.join(entry.file_name()));
                                }
                            }
                        }
                    }
                }
            }
        }

        self.result_message = Some(if no_subfolders_selected {
            self.play_embedded_sound(SAD_TRUMPET_SOUND);
            "No subfolders selected.".to_string()
        } else if copied_count == 0 {
            self.play_embedded_sound(SAD_TRUMPET_SOUND);
            "No files matched your criteria.".to_string()
        } else {
            self.play_embedded_sound(SUCCESS_SOUND);
            format!("Copied {} file(s).", copied_count)
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size((500.0, 500.0)),
        ..Default::default()
    };
    eframe::run_native(
        "LRC DataPuller",
        options,
        Box::new(|_cc| Ok(Box::new(CsvTransferApp::default()))),
    )
}
