//! Assignment Table UI Panel
//!
//! Editable table for assigning MapFlow, Mixxx, and Streamer.bot
//! functions to MIDI controls.

use egui::{Color32, RichText, Ui};

#[cfg(feature = "midi")]
use mapmap_control::midi::{
    AssignmentTable, AssignmentTarget, FunctionAssignment, MAPFLOW_FUNCTIONS, MIXXX_FUNCTIONS,
    STREAMERBOT_ACTIONS,
};

/// Assignment Table Panel
pub struct AssignmentTablePanel {
    /// Current assignment table
    #[cfg(feature = "midi")]
    table: AssignmentTable,

    /// Filter by target
    filter_target: Option<String>,

    /// Search filter
    search_filter: String,

    /// Selected element for editing
    selected_element: Option<String>,

    /// Show only assigned elements
    show_assigned_only: bool,

    /// Panel visibility
    pub visible: bool,
}

impl Default for AssignmentTablePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl AssignmentTablePanel {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "midi")]
            table: AssignmentTable::ecler_nuo4_defaults(),
            filter_target: None,
            search_filter: String::new(),
            selected_element: None,
            show_assigned_only: false,
            visible: false,
        }
    }

    #[cfg(feature = "midi")]
    pub fn load_table(&mut self, table: AssignmentTable) {
        self.table = table;
    }

    #[cfg(feature = "midi")]
    pub fn table(&self) -> &AssignmentTable {
        &self.table
    }

    #[cfg(feature = "midi")]
    pub fn table_mut(&mut self) -> &mut AssignmentTable {
        &mut self.table
    }

    /// Show the panel UI
    pub fn show(&mut self, ui: &mut Ui) {
        if !self.visible {
            return;
        }

        ui.horizontal(|ui| {
            ui.heading("📋 MIDI Zuweisungstabelle");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕").clicked() {
                    self.visible = false;
                }
            });
        });

        ui.separator();

        // Toolbar
        ui.horizontal(|ui| {
            // Filter dropdown
            ui.label("Filter:");
            egui::ComboBox::from_id_source("target_filter")
                .selected_text(self.filter_target.as_deref().unwrap_or("Alle"))
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(self.filter_target.is_none(), "Alle")
                        .clicked()
                    {
                        self.filter_target = None;
                    }
                    if ui
                        .selectable_label(
                            self.filter_target.as_deref() == Some("MapFlow"),
                            "MapFlow",
                        )
                        .clicked()
                    {
                        self.filter_target = Some("MapFlow".to_string());
                    }
                    if ui
                        .selectable_label(self.filter_target.as_deref() == Some("Mixxx"), "Mixxx")
                        .clicked()
                    {
                        self.filter_target = Some("Mixxx".to_string());
                    }
                    if ui
                        .selectable_label(
                            self.filter_target.as_deref() == Some("Streamer.bot"),
                            "Streamer.bot",
                        )
                        .clicked()
                    {
                        self.filter_target = Some("Streamer.bot".to_string());
                    }
                });

            ui.separator();

            // Search
            ui.label("🔍");
            ui.add(
                egui::TextEdit::singleline(&mut self.search_filter)
                    .desired_width(120.0)
                    .hint_text("Suchen..."),
            );

            if !self.search_filter.is_empty() && ui.button("✕").clicked() {
                self.search_filter.clear();
            }

            ui.separator();

            ui.checkbox(&mut self.show_assigned_only, "Nur zugewiesen");
        });

        ui.separator();

        #[cfg(feature = "midi")]
        {
            self.render_table(ui);
        }

        #[cfg(not(feature = "midi"))]
        {
            ui.label("MIDI-Feature nicht aktiviert");
        }
    }

    #[cfg(feature = "midi")]
    fn render_table(&mut self, ui: &mut Ui) {
        // Table header
        egui::Grid::new("assignment_table_header")
            .num_columns(5)
            .striped(false)
            .min_col_width(60.0)
            .show(ui, |ui| {
                ui.label(RichText::new("Element").strong());
                ui.label(RichText::new("Ziel").strong());
                ui.label(RichText::new("Funktion").strong());
                ui.label(RichText::new("Beschreibung").strong());
                ui.label(RichText::new("Aktionen").strong());
                ui.end_row();
            });

        ui.separator();

        // Table body with scroll
        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                egui::Grid::new("assignment_table_body")
                    .num_columns(5)
                    .striped(true)
                    .min_col_width(60.0)
                    .show(ui, |ui| {
                        // Clone assignments for iteration
                        let assignments: Vec<_> = self
                            .table
                            .assignments
                            .iter()
                            .filter(|a| {
                                // Apply search filter
                                if !self.search_filter.is_empty() {
                                    let search = self.search_filter.to_lowercase();
                                    if !a.element_id.to_lowercase().contains(&search)
                                        && !a.function.to_lowercase().contains(&search)
                                        && !a.description.to_lowercase().contains(&search)
                                    {
                                        return false;
                                    }
                                }

                                // Apply target filter
                                if let Some(ref target) = self.filter_target {
                                    match target.as_str() {
                                        "MapFlow" => a.target != AssignmentTarget::MapFlow,
                                        "Mixxx" => a.target != AssignmentTarget::Mixxx,
                                        "Streamer.bot" => a.target != AssignmentTarget::StreamerBot,
                                        _ => false,
                                    };
                                }

                                // Apply assigned only filter
                                if self.show_assigned_only && a.target == AssignmentTarget::None {
                                    return false;
                                }

                                true
                            })
                            .cloned()
                            .collect();

                        for assignment in &assignments {
                            // Element ID
                            let is_selected =
                                self.selected_element.as_ref() == Some(&assignment.element_id);
                            if ui
                                .selectable_label(is_selected, &assignment.element_id)
                                .clicked()
                            {
                                self.selected_element = Some(assignment.element_id.clone());
                            }

                            // Target with color
                            let (target_text, target_color) = match assignment.target {
                                AssignmentTarget::MapFlow => {
                                    ("MapFlow", Color32::from_rgb(100, 200, 100))
                                }
                                AssignmentTarget::Mixxx => {
                                    ("Mixxx", Color32::from_rgb(200, 150, 50))
                                }
                                AssignmentTarget::StreamerBot => {
                                    ("SB", Color32::from_rgb(150, 100, 200))
                                }
                                AssignmentTarget::None => ("-", Color32::GRAY),
                            };
                            ui.label(RichText::new(target_text).color(target_color));

                            // Function
                            ui.label(&assignment.function);

                            // Description
                            ui.label(&assignment.description);

                            // Actions
                            ui.horizontal(|ui| {
                                if ui.small_button("✏️").on_hover_text("Bearbeiten").clicked() {
                                    self.selected_element = Some(assignment.element_id.clone());
                                }
                                if ui.small_button("🗑").on_hover_text("Löschen").clicked() {
                                    // Mark for deletion (handled below)
                                }
                            });

                            ui.end_row();
                        }
                    });
            });

        // Edit panel at bottom if element selected
        if let Some(ref element_id) = self.selected_element.clone() {
            ui.separator();
            ui.heading(format!("Bearbeiten: {}", element_id));

            if let Some(assignment) = self.table.get_mut(&element_id) {
                ui.horizontal(|ui| {
                    ui.label("Ziel:");
                    egui::ComboBox::from_id_source("edit_target")
                        .selected_text(match assignment.target {
                            AssignmentTarget::MapFlow => "MapFlow",
                            AssignmentTarget::Mixxx => "Mixxx",
                            AssignmentTarget::StreamerBot => "Streamer.bot",
                            AssignmentTarget::None => "Keine",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut assignment.target,
                                AssignmentTarget::None,
                                "Keine",
                            );
                            ui.selectable_value(
                                &mut assignment.target,
                                AssignmentTarget::MapFlow,
                                "MapFlow",
                            );
                            ui.selectable_value(
                                &mut assignment.target,
                                AssignmentTarget::Mixxx,
                                "Mixxx",
                            );
                            ui.selectable_value(
                                &mut assignment.target,
                                AssignmentTarget::StreamerBot,
                                "Streamer.bot",
                            );
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Funktion:");

                    // Show available functions based on target
                    let functions: Vec<(&str, &str)> = match assignment.target {
                        AssignmentTarget::MapFlow => MAPFLOW_FUNCTIONS.to_vec(),
                        AssignmentTarget::Mixxx => MIXXX_FUNCTIONS.to_vec(),
                        AssignmentTarget::StreamerBot => STREAMERBOT_ACTIONS.to_vec(),
                        AssignmentTarget::None => vec![],
                    };

                    if !functions.is_empty() {
                        egui::ComboBox::from_id_source("edit_function")
                            .selected_text(&assignment.function)
                            .show_ui(ui, |ui| {
                                for (func, desc) in functions {
                                    if ui
                                        .selectable_label(
                                            assignment.function == func,
                                            format!("{} - {}", func, desc),
                                        )
                                        .clicked()
                                    {
                                        assignment.function = func.to_string();
                                        assignment.description = desc.to_string();
                                    }
                                }
                            });
                    } else {
                        ui.label("-");
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Beschreibung:");
                    ui.text_edit_singleline(&mut assignment.description);
                });

                ui.horizontal(|ui| {
                    ui.checkbox(&mut assignment.enabled, "Aktiviert");

                    if ui.button("Schließen").clicked() {
                        self.selected_element = None;
                    }
                });
            }
        }

        // Bottom toolbar
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("💾 Speichern").clicked() {
                // TODO: Save to file
            }
            if ui.button("📂 Laden").clicked() {
                // TODO: Load from file
            }
            if ui.button("🔄 Defaults").clicked() {
                self.table = AssignmentTable::ecler_nuo4_defaults();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{} Zuweisungen", self.table.assignments.len()));
            });
        });
    }
}
