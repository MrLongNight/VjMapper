//! Inspector Panel - Context-sensitive property inspector
//!
//! Shows different content based on current selection:
//! - Layer selected: Transform, Effects, Blend Mode
//! - Output selected: Edge Blend, Calibration, Resolution
//! - Nothing selected: Project Settings summary

use egui::Ui;

use crate::i18n::LocaleManager;
use crate::icons::IconManager;
use crate::transform_panel::TransformPanel;
use mapmap_core::{Layer, OutputConfig, Transform};

/// The Inspector Panel provides context-sensitive property editing
pub struct InspectorPanel {
    /// Whether the inspector is visible
    pub visible: bool,
    /// Internal transform panel for layer properties
    #[allow(dead_code)]
    transform_panel: TransformPanel,
}

impl Default for InspectorPanel {
    fn default() -> Self {
        Self {
            visible: true,
            transform_panel: TransformPanel::default(),
        }
    }
}

/// Represents the current selection context for the inspector
pub enum InspectorContext<'a> {
    /// No selection
    None,
    /// A layer is selected
    Layer {
        layer: &'a Layer,
        transform: &'a Transform,
    },
    /// An output is selected
    Output(&'a OutputConfig),
}

impl InspectorPanel {
    /// Show the inspector panel as a right side panel
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        context: InspectorContext<'_>,
        i18n: &LocaleManager,
        icon_manager: Option<&IconManager>,
    ) -> Option<InspectorAction> {
        if !self.visible {
            return None;
        }

        let mut action = None;

        egui::SidePanel::right("inspector_panel")
            .resizable(true)
            .default_width(300.0)
            .min_width(250.0)
            .max_width(450.0)
            .show(ctx, |ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.heading(i18n.t("panel-inspector"));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✕").clicked() {
                            self.visible = false;
                        }
                    });
                });
                ui.separator();

                // Context-sensitive content
                match context {
                    InspectorContext::None => {
                        self.show_no_selection(ui, i18n);
                    }
                    InspectorContext::Layer { layer, transform } => {
                        action =
                            self.show_layer_inspector(ui, layer, transform, i18n, icon_manager);
                    }
                    InspectorContext::Output(output) => {
                        self.show_output_inspector(ui, output, i18n);
                    }
                }
            });

        action
    }

    /// Show placeholder when nothing is selected
    fn show_no_selection(&self, ui: &mut Ui, i18n: &LocaleManager) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(
                egui::RichText::new(i18n.t("inspector-no-selection"))
                    .size(16.0)
                    .color(egui::Color32::from_rgb(120, 120, 140)),
            );
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new(i18n.t("inspector-no-selection-tip"))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(100, 100, 120)),
            );
        });
    }

    /// Show layer properties inspector
    fn show_layer_inspector(
        &mut self,
        ui: &mut Ui,
        layer: &Layer,
        transform: &Transform,
        i18n: &LocaleManager,
        _icon_manager: Option<&IconManager>,
    ) -> Option<InspectorAction> {
        let action = None;

        // Layer header with icon
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("📦").size(18.0));
            ui.label(egui::RichText::new(&layer.name).size(14.0).strong());
        });
        ui.separator();

        // Transform section
        egui::CollapsingHeader::new(i18n.t("inspector-transform"))
            .default_open(true)
            .show(ui, |ui| {
                // Position
                ui.horizontal(|ui| {
                    ui.label(i18n.t("inspector-position"));
                    ui.label(format!(
                        "({:.1}, {:.1})",
                        transform.position.x, transform.position.y
                    ));
                });

                // Scale
                ui.horizontal(|ui| {
                    ui.label(i18n.t("inspector-scale"));
                    ui.label(format!(
                        "({:.2}, {:.2})",
                        transform.scale.x, transform.scale.y
                    ));
                });

                // Rotation
                ui.horizontal(|ui| {
                    ui.label(i18n.t("inspector-rotation"));
                    ui.label(format!("{:.1}°", transform.rotation.z.to_degrees()));
                });
            });

        // Blending section
        egui::CollapsingHeader::new(i18n.t("inspector-blending"))
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(i18n.t("inspector-opacity"));
                    ui.label(format!("{:.0}%", layer.opacity * 100.0));
                });

                ui.horizontal(|ui| {
                    ui.label(i18n.t("inspector-blend-mode"));
                    ui.label(format!("{:?}", layer.blend_mode));
                });
            });

        // Layer state
        egui::CollapsingHeader::new(i18n.t("inspector-state"))
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(i18n.t("inspector-visible"));
                    ui.label(if layer.visible { "✅" } else { "❌" });
                });

                ui.horizontal(|ui| {
                    ui.label(i18n.t("inspector-solo"));
                    ui.label(if layer.solo { "🔊" } else { "—" });
                });

                ui.horizontal(|ui| {
                    ui.label(i18n.t("inspector-bypass"));
                    ui.label(if layer.bypass { "⏸" } else { "—" });
                });
            });

        action
    }

    /// Show output properties inspector
    fn show_output_inspector(&self, ui: &mut Ui, output: &OutputConfig, i18n: &LocaleManager) {
        // Output header
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🖥").size(18.0));
            ui.label(egui::RichText::new(&output.name).size(14.0).strong());
        });
        ui.separator();

        // Resolution section
        egui::CollapsingHeader::new(i18n.t("inspector-resolution"))
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(i18n.t("inspector-size"));
                    ui.label(format!("{}x{}", output.resolution.0, output.resolution.1));
                });
            });

        // Canvas Region section
        egui::CollapsingHeader::new(i18n.t("inspector-canvas-region"))
            .default_open(true)
            .show(ui, |ui| {
                let region = &output.canvas_region;
                ui.horizontal(|ui| {
                    ui.label(i18n.t("inspector-position"));
                    ui.label(format!("({:.0}, {:.0})", region.x, region.y));
                });
                ui.horizontal(|ui| {
                    ui.label(i18n.t("inspector-size"));
                    ui.label(format!("{:.0}x{:.0}", region.width, region.height));
                });
            });

        // Edge Blend indicator
        egui::CollapsingHeader::new(i18n.t("inspector-edge-blend"))
            .default_open(false)
            .show(ui, |ui| {
                let eb = &output.edge_blend;
                ui.horizontal(|ui| {
                    ui.label(i18n.t("inspector-left"));
                    ui.label(format!("{:.0}px", eb.left.width * 100.0));
                });
                ui.horizontal(|ui| {
                    ui.label(i18n.t("inspector-right"));
                    ui.label(format!("{:.0}px", eb.right.width * 100.0));
                });
                ui.horizontal(|ui| {
                    ui.label(i18n.t("inspector-top"));
                    ui.label(format!("{:.0}px", eb.top.width * 100.0));
                });
                ui.horizontal(|ui| {
                    ui.label(i18n.t("inspector-bottom"));
                    ui.label(format!("{:.0}px", eb.bottom.width * 100.0));
                });
            });
    }
}

/// Actions that can be triggered from the Inspector
#[derive(Debug, Clone)]
pub enum InspectorAction {
    /// Update layer transform
    UpdateTransform(u64, Transform),
    /// Update layer opacity
    UpdateOpacity(u64, f32),
}
