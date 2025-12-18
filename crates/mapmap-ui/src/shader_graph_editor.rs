//! Shader Graph Editor UI
//!
//! Phase 3: Effects Pipeline
//! Node-based visual shader editor using ImGui
//!

use crate::i18n::LocaleManager;
use imgui::*;
use mapmap_core::{NodeId, NodeType, ShaderGraph, ShaderNode};
use std::collections::HashMap;

/// Shader graph editor state
pub struct ShaderGraphEditor {
    /// Currently open graph
    pub graph: Option<ShaderGraph>,

    /// Selected nodes
    pub selected_nodes: Vec<NodeId>,

    /// Node positions in canvas space
    pub node_positions: HashMap<NodeId, (f32, f32)>,

    /// Canvas scroll offset
    pub canvas_offset: (f32, f32),

    /// Canvas zoom level
    pub zoom: f32,

    /// Currently dragging node
    pub dragging_node: Option<NodeId>,

    /// Connection being created (source_node, output_socket)
    pub connecting_from: Option<(NodeId, String)>,

    /// Show node palette
    pub show_palette: bool,

    /// Node palette category filter
    pub palette_category: String,

    /// Search filter for node palette
    pub palette_search: String,

    /// Show properties panel
    pub show_properties: bool,

    /// Show code preview
    pub show_code_preview: bool,

    /// Generated WGSL code cache
    pub generated_code: Option<String>,
}

impl Default for ShaderGraphEditor {
    fn default() -> Self {
        Self {
            graph: None,
            selected_nodes: Vec::new(),
            node_positions: HashMap::new(),
            canvas_offset: (0.0, 0.0),
            zoom: 1.0,
            dragging_node: None,
            connecting_from: None,
            show_palette: true,
            palette_category: "All".to_string(),
            palette_search: String::new(),
            show_properties: true,
            show_code_preview: false,
            generated_code: None,
        }
    }
}

impl ShaderGraphEditor {
    /// Create a new shader graph editor
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a shader graph
    pub fn load_graph(&mut self, graph: ShaderGraph) {
        // Initialize node positions from saved positions
        for (id, node) in &graph.nodes {
            self.node_positions.insert(*id, node.position);
        }

        self.graph = Some(graph);
        self.selected_nodes.clear();
    }

    /// Create a new empty graph
    pub fn new_graph(&mut self, name: String) {
        let graph = ShaderGraph::new(1, name);
        self.graph = Some(graph);
        self.node_positions.clear();
        self.selected_nodes.clear();
    }

    /// Draw the shader graph editor UI
    pub fn draw(&mut self, ui: &Ui, locale: &LocaleManager) -> Vec<ShaderGraphAction> {
        let mut actions = Vec::new();

        // Main menu bar
        if let Some(_menu_bar) = ui.begin_menu_bar() {
            if let Some(_menu) = ui.begin_menu(locale.t("sg-menu-file")) {
                if ui.menu_item(locale.t("sg-new-graph")) {
                    actions.push(ShaderGraphAction::NewGraph);
                }
                if ui.menu_item(locale.t("sg-load-graph")) {
                    actions.push(ShaderGraphAction::LoadGraph);
                }
                if ui.menu_item(locale.t("sg-save-graph")) {
                    actions.push(ShaderGraphAction::SaveGraph);
                }
                ui.separator();
                if ui.menu_item(locale.t("sg-generate-wgsl")) {
                    actions.push(ShaderGraphAction::GenerateCode);
                }
            }

            if let Some(_menu) = ui.begin_menu(locale.t("sg-menu-view")) {
                ui.checkbox(locale.t("sg-view-palette"), &mut self.show_palette);
                ui.checkbox(locale.t("sg-view-properties"), &mut self.show_properties);
                ui.checkbox(locale.t("sg-view-code"), &mut self.show_code_preview);
            }
        }

        // Draw panels
        if self.show_palette {
            self.draw_node_palette(ui, &mut actions, locale);
        }

        if self.show_properties && !self.selected_nodes.is_empty() {
            self.draw_properties_panel(ui, &mut actions, locale);
        }

        if self.show_code_preview {
            self.draw_code_preview(ui, locale);
        }

        // Draw main canvas
        self.draw_canvas(ui, &mut actions, locale);

        actions
    }

    /// Draw node palette
    fn draw_node_palette(
        &mut self,
        ui: &Ui,
        actions: &mut Vec<ShaderGraphAction>,
        locale: &LocaleManager,
    ) {
        ui.window(locale.t("sg-window-palette"))
            .size([250.0, 400.0], Condition::FirstUseEver)
            .position([10.0, 40.0], Condition::FirstUseEver)
            .build(|| {
                // Search box
                ui.input_text("##search", &mut self.palette_search)
                    .hint(locale.t("sg-search-hint"))
                    .build();

                ui.separator();

                // Category buttons
                let categories = vec![
                    ("All", locale.t("sg-category-all")),
                    ("Input", locale.t("sg-category-input")),
                    ("Math", locale.t("sg-category-math")),
                    ("Color", locale.t("sg-category-color")),
                    ("Texture", locale.t("sg-category-texture")),
                    ("Effects", locale.t("sg-category-effects")),
                    ("Utility", locale.t("sg-category-utility")),
                ];
                for (category_id, category_label) in categories {
                    if ui.small_button(category_label) {
                        self.palette_category = category_id.to_string();
                    }
                    ui.same_line();
                }
                ui.new_line();

                ui.separator();

                // Node list
                let node_types = self.filter_node_types();
                for node_type in node_types {
                    if ui.button(get_node_type_name(&node_type, locale)) {
                        actions.push(ShaderGraphAction::AddNode(node_type));
                    }
                }
            });
    }

    /// Draw properties panel for selected node
    fn draw_properties_panel(
        &mut self,
        ui: &Ui,
        actions: &mut Vec<ShaderGraphAction>,
        locale: &LocaleManager,
    ) {
        ui.window(locale.t("sg-window-properties"))
            .size([300.0, 400.0], Condition::FirstUseEver)
            .position([ui.window_size()[0] - 310.0, 40.0], Condition::FirstUseEver)
            .build(|| {
                if let Some(node_id) = self.selected_nodes.first() {
                    if let Some(graph) = &self.graph {
                        if let Some(node) = graph.nodes.get(node_id) {
                            ui.text(format!(
                                "{} {}",
                                locale.t("sg-prop-node"),
                                get_node_type_name(&node.node_type, locale)
                            ));
                            ui.text(format!("{} {}", locale.t("sg-prop-id"), node.id));
                            ui.separator();

                            // Display and edit parameters
                            ui.text(locale.t("sg-prop-params"));
                            for (name, value) in &node.parameters {
                                ui.text(format!("{}: {:?}", name, value));
                                // TODO: Add parameter editing widgets
                            }

                            ui.separator();

                            // Display inputs
                            ui.text(locale.t("sg-prop-inputs"));
                            for input in &node.inputs {
                                let status = if input.connected_output.is_some() {
                                    locale.t("sg-prop-connected")
                                } else {
                                    locale.t("sg-prop-not-connected")
                                };
                                ui.text(format!(
                                    "  {} [{}] - {}",
                                    input.name,
                                    input.data_type.wgsl_type(),
                                    status
                                ));
                            }

                            ui.separator();

                            // Display outputs
                            ui.text(locale.t("sg-prop-outputs"));
                            for output in &node.outputs {
                                ui.text(format!(
                                    "  {} [{}]",
                                    output.name,
                                    output.data_type.wgsl_type()
                                ));
                            }

                            ui.separator();

                            // Delete button
                            if ui.button(locale.t("sg-delete-node")) {
                                actions.push(ShaderGraphAction::DeleteNode(*node_id));
                            }
                        }
                    }
                }
            });
    }

    /// Draw code preview panel
    fn draw_code_preview(&self, ui: &Ui, locale: &LocaleManager) {
        ui.window(locale.t("sg-window-code"))
            .size([600.0, 500.0], Condition::FirstUseEver)
            .build(|| {
                if let Some(code) = &self.generated_code {
                    ui.text_wrapped(code);
                } else {
                    ui.text(locale.t("sg-code-none"));
                }
            });
    }

    /// Draw main canvas
    fn draw_canvas(
        &mut self,
        ui: &Ui,
        _actions: &mut Vec<ShaderGraphAction>,
        locale: &LocaleManager,
    ) {
        ui.window(locale.t("sg-window-canvas"))
            .size([800.0, 600.0], Condition::FirstUseEver)
            .position([260.0, 40.0], Condition::FirstUseEver)
            .build(|| {
                let draw_list = ui.get_window_draw_list();
                let canvas_pos = ui.cursor_screen_pos();
                let canvas_size = ui.content_region_avail();

                // Background
                draw_list
                    .add_rect(
                        canvas_pos,
                        [
                            canvas_pos[0] + canvas_size[0],
                            canvas_pos[1] + canvas_size[1],
                        ],
                        [0.1, 0.1, 0.1, 1.0],
                    )
                    .filled(true)
                    .build();

                // Grid
                self.draw_grid(&draw_list, canvas_pos, canvas_size);

                // Draw nodes and connections
                if let Some(graph) = &self.graph {
                    // Draw connections first (behind nodes)
                    self.draw_connections(&draw_list, canvas_pos, graph);

                    // Draw nodes
                    for (node_id, node) in &graph.nodes {
                        self.draw_node(&draw_list, canvas_pos, *node_id, node, ui, locale);
                    }
                }

                // Handle mouse input
                if ui.is_window_hovered() {
                    // Pan canvas with middle mouse or right mouse
                    if ui.is_mouse_dragging(MouseButton::Middle)
                        || ui.is_mouse_dragging(MouseButton::Right)
                    {
                        let delta = ui.mouse_drag_delta_with_button(MouseButton::Middle);
                        self.canvas_offset.0 += delta[0];
                        self.canvas_offset.1 += delta[1];
                        ui.reset_mouse_drag_delta(MouseButton::Middle);
                    }

                    // Zoom with mouse wheel
                    let scroll = ui.io().mouse_wheel;
                    if scroll != 0.0 {
                        self.zoom *= 1.0 + scroll * 0.1;
                        self.zoom = self.zoom.clamp(0.5, 2.0);
                    }
                }
            });
    }

    /// Draw grid background
    fn draw_grid(&self, draw_list: &DrawListMut, canvas_pos: [f32; 2], canvas_size: [f32; 2]) {
        let grid_size = 50.0 * self.zoom;
        let grid_color = [0.2, 0.2, 0.2, 1.0];

        // Vertical lines
        let mut x = (canvas_pos[0] + self.canvas_offset.0 % grid_size) - grid_size;
        while x < canvas_pos[0] + canvas_size[0] {
            draw_list
                .add_line(
                    [x, canvas_pos[1]],
                    [x, canvas_pos[1] + canvas_size[1]],
                    grid_color,
                )
                .build();
            x += grid_size;
        }

        // Horizontal lines
        let mut y = (canvas_pos[1] + self.canvas_offset.1 % grid_size) - grid_size;
        while y < canvas_pos[1] + canvas_size[1] {
            draw_list
                .add_line(
                    [canvas_pos[0], y],
                    [canvas_pos[0] + canvas_size[0], y],
                    grid_color,
                )
                .build();
            y += grid_size;
        }
    }

    /// Draw a node
    fn draw_node(
        &self,
        draw_list: &DrawListMut,
        canvas_pos: [f32; 2],
        node_id: NodeId,
        node: &ShaderNode,
        _ui: &Ui,
        locale: &LocaleManager,
    ) {
        let pos = self
            .node_positions
            .get(&node_id)
            .copied()
            .unwrap_or((100.0, 100.0));
        let screen_pos = [
            canvas_pos[0] + pos.0 * self.zoom + self.canvas_offset.0,
            canvas_pos[1] + pos.1 * self.zoom + self.canvas_offset.1,
        ];

        let node_size = [150.0 * self.zoom, 80.0 * self.zoom];

        // Node background
        let is_selected = self.selected_nodes.contains(&node_id);
        let node_color = if is_selected {
            [0.3, 0.5, 0.7, 1.0]
        } else {
            [0.2, 0.3, 0.4, 1.0]
        };

        draw_list
            .add_rect(
                screen_pos,
                [screen_pos[0] + node_size[0], screen_pos[1] + node_size[1]],
                node_color,
            )
            .filled(true)
            .rounding(5.0)
            .build();

        // Node border
        draw_list
            .add_rect(
                screen_pos,
                [screen_pos[0] + node_size[0], screen_pos[1] + node_size[1]],
                [0.8, 0.8, 0.8, 1.0],
            )
            .rounding(5.0)
            .build();

        // Node title
        draw_list.add_text(
            [screen_pos[0] + 5.0, screen_pos[1] + 5.0],
            [1.0, 1.0, 1.0, 1.0],
            get_node_type_name(&node.node_type, locale),
        );
    }

    /// Draw connections between nodes
    fn draw_connections(&self, draw_list: &DrawListMut, canvas_pos: [f32; 2], graph: &ShaderGraph) {
        for node in graph.nodes.values() {
            for input in &node.inputs {
                if let Some((source_node, _output_name)) = &input.connected_output {
                    if let Some(source_pos) = self.node_positions.get(source_node) {
                        let dest_pos = self
                            .node_positions
                            .get(&node.id)
                            .copied()
                            .unwrap_or((0.0, 0.0));

                        let start = [
                            canvas_pos[0]
                                + source_pos.0 * self.zoom
                                + self.canvas_offset.0
                                + 150.0 * self.zoom,
                            canvas_pos[1]
                                + source_pos.1 * self.zoom
                                + self.canvas_offset.1
                                + 40.0 * self.zoom,
                        ];

                        let end = [
                            canvas_pos[0] + dest_pos.0 * self.zoom + self.canvas_offset.0,
                            canvas_pos[1]
                                + dest_pos.1 * self.zoom
                                + self.canvas_offset.1
                                + 40.0 * self.zoom,
                        ];

                        // Draw bezier curve
                        let ctrl1 = [start[0] + 50.0, start[1]];
                        let ctrl2 = [end[0] - 50.0, end[1]];

                        draw_list
                            .add_bezier_curve(start, ctrl1, ctrl2, end, [0.8, 0.8, 0.2, 1.0])
                            .thickness(2.0)
                            .build();
                    }
                }
            }
        }
    }

    /// Filter node types based on category and search
    fn filter_node_types(&self) -> Vec<NodeType> {
        let all_types = vec![
            NodeType::TextureInput,
            NodeType::TimeInput,
            NodeType::UVInput,
            NodeType::ParameterInput,
            NodeType::AudioInput,
            NodeType::Add,
            NodeType::Subtract,
            NodeType::Multiply,
            NodeType::Divide,
            NodeType::Sin,
            NodeType::Cos,
            NodeType::Power,
            NodeType::Mix,
            NodeType::Clamp,
            NodeType::Smoothstep,
            NodeType::ColorRamp,
            NodeType::HSVToRGB,
            NodeType::RGBToHSV,
            NodeType::Brightness,
            NodeType::Contrast,
            NodeType::Desaturate,
            NodeType::TextureSample,
            NodeType::UVTransform,
            NodeType::UVDistort,
            NodeType::Blur,
            NodeType::Glow,
            NodeType::ChromaticAberration,
            NodeType::Kaleidoscope,
            NodeType::EdgeDetect,
            NodeType::Split,
            NodeType::Combine,
            NodeType::Remap,
            NodeType::Output,
        ];

        all_types
            .into_iter()
            .filter(|node_type| {
                // Filter by category
                let category_match =
                    self.palette_category == "All" || node_type.category() == self.palette_category;

                // Filter by search
                let search_match = self.palette_search.is_empty()
                    || node_type
                        .display_name()
                        .to_lowercase()
                        .contains(&self.palette_search.to_lowercase());

                category_match && search_match
            })
            .collect()
    }
}

/// Actions that can be performed in the shader graph editor
#[derive(Debug, Clone)]
pub enum ShaderGraphAction {
    NewGraph,
    LoadGraph,
    SaveGraph,
    AddNode(NodeType),
    DeleteNode(NodeId),
    ConnectNodes(NodeId, String, NodeId, String),
    DisconnectInput(NodeId, String),
    SelectNode(NodeId),
    DeselectAll,
    GenerateCode,
}

fn get_node_type_name(node_type: &NodeType, locale: &LocaleManager) -> String {
    match node_type {
        NodeType::TextureInput => locale.t("node-texture-input"),
        NodeType::TimeInput => locale.t("node-time-input"),
        NodeType::UVInput => locale.t("node-uv-input"),
        NodeType::ParameterInput => locale.t("node-parameter-input"),
        NodeType::AudioInput => locale.t("node-audio-input"),
        NodeType::Add => locale.t("node-add"),
        NodeType::Subtract => locale.t("node-subtract"),
        NodeType::Multiply => locale.t("node-multiply"),
        NodeType::Divide => locale.t("node-divide"),
        NodeType::Sin => locale.t("node-sin"),
        NodeType::Cos => locale.t("node-cos"),
        NodeType::Power => locale.t("node-power"),
        NodeType::Mix => locale.t("node-mix"),
        NodeType::Clamp => locale.t("node-clamp"),
        NodeType::Smoothstep => locale.t("node-smoothstep"),
        NodeType::ColorRamp => locale.t("node-color-ramp"),
        NodeType::HSVToRGB => locale.t("node-hsv-to-rgb"),
        NodeType::RGBToHSV => locale.t("node-rgb-to-hsv"),
        NodeType::Brightness => locale.t("node-brightness"),
        NodeType::Contrast => locale.t("node-contrast"),
        NodeType::Desaturate => locale.t("node-desaturate"),
        NodeType::TextureSample => locale.t("node-texture-sample"),
        NodeType::TextureSampleLod => locale.t("node-texture-sample-lod"),
        NodeType::TextureCombine => locale.t("node-texture-combine"),
        NodeType::UVTransform => locale.t("node-uv-transform"),
        NodeType::UVDistort => locale.t("node-uv-distort"),
        NodeType::Blur => locale.t("node-blur"),
        NodeType::Glow => locale.t("node-glow"),
        NodeType::PixelSort => locale.t("node-pixel-sort"),
        NodeType::Displacement => locale.t("node-displacement"),
        NodeType::ChromaticAberration => locale.t("node-chromatic-aberration"),
        NodeType::Kaleidoscope => locale.t("node-kaleidoscope"),
        NodeType::EdgeDetect => locale.t("node-edge-detect"),
        NodeType::Split => locale.t("node-split"),
        NodeType::Combine => locale.t("node-combine"),
        NodeType::Remap => locale.t("node-remap"),
        NodeType::Output => locale.t("node-output"),
    }
}
