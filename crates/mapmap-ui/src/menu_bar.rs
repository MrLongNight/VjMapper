//! Egui-based Main Menu Bar and Toolbar
//!
//! This module provides the main menu bar and toolbar for the application.

use crate::icons::AppIcon;
use crate::{AppUI, UIAction};

/// State-holding struct for the main menu bar.
#[derive(Default, Debug)]
pub struct MenuBar {}

/// Renders the main menu bar and returns any action triggered.
pub fn show(ctx: &egui::Context, ui_state: &mut AppUI) -> Vec<UIAction> {
    let mut actions = vec![];

    // Custom frame for modern look
    let frame = egui::Frame::none()
        .fill(ctx.style().visuals.window_fill())
        .inner_margin(egui::Margin::symmetric(16.0, 8.0));

    egui::TopBottomPanel::top("top_panel")
        .frame(frame)
        .show(ctx, |ui| {
            ui.style_mut().visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
            ui.style_mut().visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
            ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;

            // Helper for menu items with icons
            let menu_item = |ui: &mut egui::Ui, text: String, icon: Option<AppIcon>| -> bool {
                if let Some(mgr) = &ui_state.icon_manager {
                    if let Some(icon) = icon {
                        if let Some(texture) = mgr.get(icon) {
                            // Use image_and_text if available, or horizontal layout
                            // For simplicity and compatibility, we use horizontal layout for now to simulate an icon button
                            // But standard implementation is cleaner.
                            // Let's try egui::Button::image_and_text with texture and text (size inferred from handle or default?)
                            // Use image_and_text with Image widget
                            return ui
                                .add(egui::Button::image_and_text(
                                    egui::Image::new(texture)
                                        .fit_to_exact_size(egui::vec2(14.0, 14.0)),
                                    text,
                                ))
                                .clicked();
                        }
                    }
                }
                ui.button(text).clicked()
            };

            // --- Main Menu Bar ---
            egui::menu::bar(ui, |ui| {
                ui.style_mut().spacing.button_padding = egui::vec2(8.0, 4.0);

                // --- File Menu ---
                ui.menu_button(ui_state.i18n.t("menu-file"), |ui| {
                    if ui
                        .button(ui_state.i18n.t("menu-file-new-project"))
                        .clicked()
                    {
                        actions.push(UIAction::NewProject);
                        ui.close_menu();
                    }
                    if ui
                        .button(ui_state.i18n.t("menu-file-open-project"))
                        .clicked()
                    {
                        actions.push(UIAction::LoadProject(String::new()));
                        ui.close_menu();
                    }

                    // Recent files submenu
                    let recent_files = ui_state.recent_files.clone();
                    if !recent_files.is_empty() {
                        ui.menu_button(ui_state.i18n.t("menu-file-open-recent"), |ui| {
                            for path in &recent_files {
                                if ui.button(path).clicked() {
                                    actions.push(UIAction::LoadRecentProject(path.clone()));
                                    ui.close_menu();
                                }
                            }
                        });
                    }

                    ui.separator();

                    if menu_item(
                        ui,
                        ui_state.i18n.t("menu-file-save-project"),
                        Some(AppIcon::FloppyDisk),
                    ) {
                        actions.push(UIAction::SaveProject(String::new()));
                        ui.close_menu();
                    }
                    if ui.button(ui_state.i18n.t("menu-file-save-as")).clicked() {
                        actions.push(UIAction::SaveProjectAs);
                        ui.close_menu();
                    }
                    if ui.button(ui_state.i18n.t("menu-file-export")).clicked() {
                        actions.push(UIAction::Export);
                        ui.close_menu();
                    }

                    ui.separator();

                    if menu_item(
                        ui,
                        ui_state.i18n.t("menu-file-settings"),
                        Some(AppIcon::Cog),
                    ) {
                        actions.push(UIAction::OpenSettings);
                        ui.close_menu();
                    }

                    ui.separator();

                    if menu_item(
                        ui,
                        ui_state.i18n.t("menu-file-exit"),
                        Some(AppIcon::ButtonStop),
                    ) {
                        actions.push(UIAction::Exit);
                        ui.close_menu();
                    }
                });

                // --- Edit Menu ---
                ui.menu_button(ui_state.i18n.t("menu-edit"), |ui| {
                    if menu_item(
                        ui,
                        ui_state.i18n.t("menu-edit-undo"),
                        Some(AppIcon::ArrowLeft),
                    ) {
                        actions.push(UIAction::Undo);
                        ui.close_menu();
                    }
                    if menu_item(
                        ui,
                        ui_state.i18n.t("menu-edit-redo"),
                        Some(AppIcon::ArrowRight),
                    ) {
                        actions.push(UIAction::Redo);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button(ui_state.i18n.t("menu-edit-cut")).clicked() {
                        actions.push(UIAction::Cut);
                        ui.close_menu();
                    }
                    if ui.button(ui_state.i18n.t("menu-edit-copy")).clicked() {
                        actions.push(UIAction::Copy);
                        ui.close_menu();
                    }
                    if ui.button(ui_state.i18n.t("menu-edit-paste")).clicked() {
                        actions.push(UIAction::Paste);
                        ui.close_menu();
                    }
                    if ui.button(ui_state.i18n.t("menu-edit-delete")).clicked() {
                        actions.push(UIAction::Delete);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button(ui_state.i18n.t("menu-edit-select-all")).clicked() {
                        actions.push(UIAction::SelectAll);
                        ui.close_menu();
                    }
                });

                // --- View Menu ---
                ui.menu_button(ui_state.i18n.t("menu-view"), |ui| {
                    // Core Panels
                    ui.checkbox(&mut ui_state.show_timeline, "Timeline");
                    ui.checkbox(&mut ui_state.show_audio, "Audio");
                    ui.checkbox(&mut ui_state.show_layers, "Layers");
                    ui.checkbox(&mut ui_state.show_media_browser, "Media Browser");

                    ui.separator();

                    // Advanced Panels
                    ui.checkbox(&mut ui_state.show_shader_graph, "Shader Graph");
                    ui.checkbox(&mut ui_state.effect_chain_panel.visible, "Effect Chain");
                    ui.checkbox(&mut ui_state.show_outputs, "Outputs");

                    ui.separator();

                    // UI Options
                    ui.checkbox(&mut ui_state.show_toolbar, "Toolbar");
                    ui.checkbox(&mut ui_state.show_stats, "Performance");

                    ui.separator();

                    if menu_item(
                        ui,
                        ui_state.i18n.t("btn-fullscreen"),
                        Some(AppIcon::Monitor),
                    ) {
                        actions.push(UIAction::ToggleFullscreen);
                        ui.close_menu();
                    }
                    if menu_item(
                        ui,
                        ui_state.i18n.t("view-reset-layout"),
                        Some(AppIcon::AppWindow),
                    ) {
                        actions.push(UIAction::ResetLayout);
                        ui.close_menu();
                    }
                });

                // --- Help Menu ---
                ui.menu_button(ui_state.i18n.t("menu-help"), |ui| {
                    if ui.button(ui_state.i18n.t("menu-help-docs")).clicked() {
                        actions.push(UIAction::OpenDocs);
                        ui.close_menu();
                    }
                    if menu_item(
                        ui,
                        ui_state.i18n.t("menu-help-about"),
                        Some(AppIcon::InfoCircle),
                    ) {
                        actions.push(UIAction::OpenAbout);
                        ui.close_menu();
                    }
                    if ui.button(ui_state.i18n.t("menu-help-license")).clicked() {
                        actions.push(UIAction::OpenLicense);
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.menu_button("Language", |ui| {
                        if ui.button("English").clicked() {
                            actions.push(UIAction::SetLanguage("en".to_string()));
                            ui.close_menu();
                        }
                        if ui.button("Deutsch").clicked() {
                            actions.push(UIAction::SetLanguage("de".to_string()));
                            ui.close_menu();
                        }
                    });
                });
            });

            ui.add_space(4.0);

            // --- Toolbar ---
            if ui_state.show_toolbar {
                ui.horizontal(|ui| {
                    ui.style_mut().spacing.button_padding = egui::vec2(8.0, 4.0);

                    let icon_size = 32.0;

                    // Helper for icon buttons
                    let mut icon_btn = |icon: AppIcon, tooltip: &str| -> bool {
                        if let Some(mgr) = &ui_state.icon_manager {
                            if let Some(img) = mgr.image(icon, icon_size) {
                                return ui
                                    .add(egui::ImageButton::new(img).frame(false))
                                    .on_hover_text(tooltip)
                                    .clicked();
                            }
                        }
                        ui.button(tooltip).clicked()
                    };

                    if icon_btn(AppIcon::FloppyDisk, &ui_state.i18n.t("toolbar-save")) {
                        actions.push(UIAction::SaveProject(String::new()));
                    }
                    if icon_btn(AppIcon::ArrowLeft, &ui_state.i18n.t("toolbar-undo")) {
                        actions.push(UIAction::Undo);
                    }
                    if icon_btn(AppIcon::ArrowRight, &ui_state.i18n.t("toolbar-redo")) {
                        actions.push(UIAction::Redo);
                    }
                    if icon_btn(AppIcon::Cog, &ui_state.i18n.t("menu-file-settings")) {
                        actions.push(UIAction::OpenSettings);
                    }

                    ui.separator();

                    // === AUDIO LEVEL (from audio analysis) ===
                    let audio_level = ui_state.current_audio_level;
                    let audio_color = if audio_level > 0.8 {
                        egui::Color32::from_rgb(255, 80, 80) // Red - clipping
                    } else if audio_level > 0.5 {
                        egui::Color32::from_rgb(255, 200, 80) // Yellow
                    } else {
                        egui::Color32::from_rgb(80, 200, 80) // Green
                    };
                    ui.label("🔊");
                    ui.add(
                        egui::ProgressBar::new(audio_level)
                            .fill(audio_color)
                            .desired_width(60.0),
                    );

                    // === SPACER - push performance to right ===
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // === PERFORMANCE INDICATORS (Traffic Light) ===
                        let fps = ui_state.current_fps;
                        let frame_time = ui_state.current_frame_time_ms;

                        // Traffic light based on FPS
                        let perf_color = if fps >= 55.0 {
                            egui::Color32::from_rgb(80, 200, 80) // Green - good
                        } else if fps >= 30.0 {
                            egui::Color32::from_rgb(255, 200, 80) // Yellow - warning
                        } else {
                            egui::Color32::from_rgb(255, 80, 80) // Red - bad
                        };

                        // Traffic light indicator
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                        ui.painter().circle_filled(rect.center(), 6.0, perf_color);

                        // Frame time
                        ui.label(format!("{:.1}ms", frame_time));

                        // FPS
                        ui.colored_label(perf_color, format!("{:.0} FPS", fps));
                    });
                });
            }

            ui.add_space(4.0);
            ui.separator();
        });

    actions
}
