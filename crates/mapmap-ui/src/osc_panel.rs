use crate::AppUI;
use imgui::{Condition, Ui};
use mapmap_control::ControlManager;

/// Renders the OSC server status panel.
pub fn render_osc_panel(ui: &Ui, app_ui: &mut AppUI, control_manager: &mut ControlManager) {
    if !app_ui.show_osc_panel {
        return;
    }

    ui.window(app_ui.i18n.t("panel-osc-title"))
        .size([400.0, 500.0], Condition::FirstUseEver)
        .build(|| {
            ui.text(app_ui.i18n.t("header-osc-server"));
            ui.separator();

            // Server status
            let server_status_key = if control_manager.osc_server.is_some() {
                "status-running"
            } else {
                "status-stopped"
            };
            ui.text(format!(
                "{}: {}",
                app_ui.i18n.t("label-status"),
                app_ui.i18n.t(server_status_key)
            ));

            // Port configuration
            ui.input_text(app_ui.i18n.t("label-port"), &mut app_ui.osc_port_input)
                .build();
            if ui.button(app_ui.i18n.t("btn-start-server")) {
                if let Ok(port) = app_ui.osc_port_input.parse() {
                    if let Err(e) = control_manager.init_osc_server(port) {
                        tracing::error!("Failed to start OSC server: {}", e);
                    }
                }
            }

            ui.separator();

            // OSC Clients (Feedback)
            ui.text(app_ui.i18n.t("header-feedback-clients"));
            let mut clients_to_remove = Vec::new();
            for client in &control_manager.osc_clients {
                ui.text(client.destination_str());
                ui.same_line();
                if ui.small_button(format!(
                    "{}##{}",
                    app_ui.i18n.t("btn-remove"),
                    client.destination_str()
                )) {
                    clients_to_remove.push(client.destination_str());
                }
            }

            for addr in clients_to_remove {
                control_manager.remove_osc_client(&addr);
            }

            ui.input_text(
                app_ui.i18n.t("label-add-client"),
                &mut app_ui.osc_client_input,
            )
            .build();
            if ui.button(app_ui.i18n.t("btn-add")) {
                if let Err(e) = control_manager.add_osc_client(&app_ui.osc_client_input) {
                    tracing::error!("Failed to add OSC client: {}", e);
                } else {
                    app_ui.osc_client_input.clear();
                }
            }

            ui.separator();

            // Mappings
            ui.text(app_ui.i18n.t("header-address-mappings"));
            ui.text(app_ui.i18n.t("text-osc-edit-tip"));

            let mut mappings_to_remove = Vec::new();
            for (addr, target) in &control_manager.osc_mapping.map {
                ui.text(format!("{} -> {:?}", addr, target));
                ui.same_line();
                if ui.small_button(format!("{}##{}", app_ui.i18n.t("btn-remove"), addr)) {
                    mappings_to_remove.push(addr.clone());
                }
            }

            for addr in &mappings_to_remove {
                control_manager.osc_mapping.remove_mapping(addr);
            }
            if !mappings_to_remove.is_empty() {
                if let Err(e) = control_manager.osc_mapping.save("osc_mappings.json") {
                    let err_msg = format!("Failed to save OSC mappings: {}", e);
                    tracing::error!("{}", err_msg);
                    eprintln!("{}", err_msg);
                    // Do not exit process, just log error.
                }
            }
        });
}
