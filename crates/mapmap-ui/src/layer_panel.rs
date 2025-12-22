// crates/mapmap-ui/src/layer_panel.rs

use crate::i18n::LocaleManager;
use egui::{Context, Id, LayerId as EguiLayerId};
use mapmap_core::{BlendMode, LayerId, LayerManager};

#[derive(Debug, Clone)]
pub enum LayerPanelAction {
    AddLayer,
    RemoveLayer(LayerId),
    DuplicateLayer(LayerId),
    ToggleVisibility(LayerId),
    ToggleSolo(LayerId),
    ToggleLock(LayerId),
    SetOpacity(LayerId, f32),
    SetBlendMode(LayerId, BlendMode),
    ReorderLayer { old_index: usize, new_index: usize },
    SelectLayer(Option<LayerId>),
}

pub struct LayerPanel {
    pub visible: bool,
    action: Option<LayerPanelAction>,
    dragged_item: Option<(usize, LayerId)>,
}

impl Default for LayerPanel {
    fn default() -> Self {
        Self {
            visible: true,
            action: None,
            dragged_item: None,
        }
    }
}

impl LayerPanel {
    pub fn take_action(&mut self) -> Option<LayerPanelAction> {
        self.action.take()
    }

    pub fn render(
        &mut self,
        ctx: &Context,
        i18n: &LocaleManager,
        layer_manager: &mut LayerManager,
        selected_layer_id: Option<LayerId>,
    ) {
        if !self.visible {
            return;
        }

        egui::Window::new(i18n.t("panel-layers"))
            .open(&mut self.visible)
            .default_size([380.0, 400.0])
            .show(ctx, |ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.heading(i18n.t_args(
                        "label-total-layers",
                        &[("count", &layer_manager.layers().len().to_string())],
                    ));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("+").on_hover_text(i18n.t("btn-add-layer")).clicked() {
                            self.action = Some(LayerPanelAction::AddLayer);
                        }
                    });
                });
                ui.separator();

                // Layer List with manual Drag and Drop
                let dnd_id = Id::new("layer_dnd");
                let mut drop_index = None;

                for (i, layer) in layer_manager.layers().iter().enumerate() {
                    let _item_id = dnd_id.with(layer.id);
                    let is_selected = selected_layer_id == Some(layer.id);

                    let response = egui::Frame::group(ui.style())
                        .stroke(if is_selected {
                            ui.style().visuals.widgets.active.bg_stroke
                        } else {
                            ui.style().visuals.widgets.inactive.bg_stroke
                        })
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Drag Handle
                                let handle_response = ui.add(egui::Label::new("☰").sense(egui::Sense::drag()));
                                if handle_response.drag_started() {
                                    self.dragged_item = Some((i, layer.id));
                                }

                                // Visibility, Solo, Lock buttons
                                if ui.selectable_label(layer.visible, "👁").on_hover_text(i18n.t("check-visible")).clicked() {
                                    self.action = Some(LayerPanelAction::ToggleVisibility(layer.id));
                                }
                                if ui.selectable_label(layer.solo, "S").on_hover_text(i18n.t("check-solo")).clicked() {
                                    self.action = Some(LayerPanelAction::ToggleSolo(layer.id));
                                }
                                if ui.selectable_label(layer.locked, "🔒").on_hover_text(i18n.t("check-lock")).clicked() {
                                    self.action = Some(LayerPanelAction::ToggleLock(layer.id));
                                }

                                // Layer name
                                ui.label(&layer.name);

                                // Right-aligned buttons
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("🗑").on_hover_text(i18n.t("btn-remove")).clicked() {
                                        self.action = Some(LayerPanelAction::RemoveLayer(layer.id));
                                    }
                                    if ui.button("📋").on_hover_text(i18n.t("btn-duplicate")).clicked() {
                                        self.action = Some(LayerPanelAction::DuplicateLayer(layer.id));
                                    }
                                });
                            });

                            // Opacity Slider
                            let mut opacity = layer.opacity;
                            if ui.add(egui::Slider::new(&mut opacity, 0.0..=1.0)).changed() {
                                self.action = Some(LayerPanelAction::SetOpacity(layer.id, opacity));
                            }

                            // Blend Mode ComboBox
                            let mut selected_blend_mode = layer.blend_mode;
                            egui::ComboBox::from_id_source(layer.id)
                                .selected_text(format!("{:?}", selected_blend_mode))
                                .show_ui(ui, |ui| {
                                    for mode in BlendMode::all() {
                                        if ui.selectable_value(&mut selected_blend_mode, *mode, format!("{:?}", mode)).clicked() {
                                            self.action = Some(LayerPanelAction::SetBlendMode(layer.id, selected_blend_mode));
                                        }
                                    }
                                });
                        })
                        .response;

                    if response.clicked() {
                        self.action = Some(LayerPanelAction::SelectLayer(Some(layer.id)));
                    }

                    if ui.rect_contains_pointer(response.rect) && ui.input(|i| i.pointer.any_released()) {
                        if self.dragged_item.is_some() {
                           drop_index = Some(i);
                        }
                    }
                }

                if let Some((_, id)) = self.dragged_item {
                    if let Some(layer) = layer_manager.layers().iter().find(|l| l.id == id) {
                        let layer_id = EguiLayerId::new(egui::Order::Tooltip, Id::new("dnd_ghost"));
                        let _painter = ui.ctx().layer_painter(layer_id);
                        let ghost_rect = egui::Rect::from_min_size(
                            ui.input(|i| i.pointer.interact_pos().unwrap_or_default()),
                            egui::vec2(200.0, 50.0),
                        );
                        let mut ghost_ui = ui.child_ui(ghost_rect, *ui.layout());

                        egui::Frame::popup(ui.style()).show(&mut ghost_ui, |ui| {
                            ui.label(&layer.name);
                        });
                    }
                }

                if let Some(to) = drop_index {
                    if let Some((from, _id)) = self.dragged_item.take() {
                        if from != to {
                            self.action = Some(LayerPanelAction::ReorderLayer {
                                old_index: from,
                                new_index: to,
                            });
                        }
                    }
                }

                if ui.input(|i| i.pointer.any_released()) {
                    self.dragged_item = None;
                }
            });
    }
}
