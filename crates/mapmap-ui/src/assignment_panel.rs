use egui::{Context, Window};
use mapmap_core::{
    assignment::Assignment,
    AppState,
};

/// Panel for managing control assignments
pub struct AssignmentPanel {
    pub visible: bool,
    #[allow(dead_code)]
    selected_source_id: Option<u64>,
}

impl Default for AssignmentPanel {
    fn default() -> Self {
        Self {
            visible: false,
            selected_source_id: None,
        }
    }
}

impl AssignmentPanel {
    pub fn show(&mut self, ctx: &Context, state: &mut AppState) {
        let mut is_open = self.visible;
        Window::new("Control Assignments")
            .open(&mut is_open)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Assignments");
                ui.separator();

                // List existing assignments
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut to_remove = None;
                    for (i, assignment) in state.assignment_manager.assignments.iter_mut().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("Source ID: {}", assignment.source_id));
                                ui.label("->");
                                ui.label(format!("{:?}", assignment.target));
                            });

                            ui.horizontal(|ui| {
                                ui.checkbox(&mut assignment.enabled, "Enabled");
                                ui.checkbox(&mut assignment.invert, "Invert");
                            });

                            ui.horizontal(|ui| {
                                ui.label("Min:");
                                ui.add(egui::DragValue::new(&mut assignment.min).speed(0.01));
                                ui.label("Max:");
                                ui.add(egui::DragValue::new(&mut assignment.max).speed(0.01));
                            });

                            if ui.button("Remove").clicked() {
                                to_remove = Some(i);
                            }
                        });
                    }

                    if let Some(index) = to_remove {
                        state.assignment_manager.remove(index);
                    }
                });

                ui.separator();

                // Add new (Manual) - simplified for now
                if ui.button("Add Manual Assignment").clicked() {
                    state.assignment_manager.add(Assignment::default());
                }

                // TODO: "Learn" mode would go here, listening to last input
            });

        self.visible = is_open;
    }
}
