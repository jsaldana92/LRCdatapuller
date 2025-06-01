// src/main.rs

use eframe::egui;
use std::fs;
use std::path::Path;
use std::collections::HashMap;

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
    subfolder_columns: usize,
    subfolder_rows: usize,
}

impl Default for CsvTransferApp {
    fn default() -> Self {
        Self {
            tasks_root: "C:\\Tasks".to_string(),
            selected_folder: None,
            subfolder_checkboxes: HashMap::new(),
            include_csv: true,
            include_txt: false,
            exclude_monkey: false,
            exclude_block: false,
            custom_exclude: String::new(),
            move_after_copy: false,
            result_message: None,
            subfolder_columns: 0,
            subfolder_rows: 0,
        }
    }
}

impl eframe::App for CsvTransferApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("CSV File Copier");

            // Folder selection with horizontal scrollable area
            if let Ok(entries) = fs::read_dir(&self.tasks_root) {
                ui.label("Folders in C:\\Tasks:");
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
                        ui.add_space(10.0); // Padding below scroll bar
                    });
            }

            // Subfolder checkboxes in horizontal scrollable area
            if !self.subfolder_checkboxes.is_empty() {
                ui.separator();
                ui.label("Select subfolders:");
                let max_per_column = 6;
                let mut subfolder_names: Vec<_> = self.subfolder_checkboxes.iter_mut().collect();
                subfolder_names.sort_by_key(|(name, _)| *name);
                let columns = (subfolder_names.len() + max_per_column - 1) / max_per_column;
                let rows = std::cmp::min(max_per_column, subfolder_names.len());

                self.subfolder_columns = columns;
                self.subfolder_rows = rows;

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
                        ui.add_space(10.0); // Padding below scroll bar
                    });
            }

            // File filters
            ui.separator();
            ui.label("File Type Options:");
            ui.checkbox(&mut self.include_csv, ".csv");
            ui.checkbox(&mut self.include_txt, ".txt");
            if self.include_txt {
                ui.checkbox(&mut self.exclude_monkey, "Exclude 'monkey*'");
                ui.checkbox(&mut self.exclude_block, "Exclude 'block*'");
                ui.horizontal(|ui| {
                    ui.label("Custom Exclude Prefix:");
                    ui.text_edit_singleline(&mut self.custom_exclude);
                });
            }

            // Move option
            ui.separator();
            ui.checkbox(&mut self.move_after_copy, "Move copied files into 'copied' folder");

            // Copy button
            if ui.button("Copy Selected Files to D:\\data_from_puller").clicked() {
                self.copy_selected_files();
            }

            // Result popup
            if let Some(message) = self.result_message.clone() {
                egui::Window::new("Done").collapsible(false).show(ctx, |ui| {
                    ui.label(&message);
                    if ui.button("OK").clicked() {
                        self.result_message = None;
                    }
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
        let mut copied_files = vec![];
        let mut copied_count = 0;
        let mut no_subfolders_selected = true;

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
                                copied_files.push(entry.file_name().to_string_lossy().to_string());
                                copied_count += 1;

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
            "No subfolders selected.".to_string()
        } else if copied_count == 0 {
            "No files matched your criteria.".to_string()
        } else {
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
        "DataPuller",
        options,
        Box::new(|_cc| Ok(Box::new(CsvTransferApp::default()))),
    )
}
