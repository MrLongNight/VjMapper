//! MapMap - Professional Projection Mapping Suite
//!
//! Phase 2 Demo Application - Projection Mapping with Warping

use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use glam::{Mat4, Vec2};
mod window_manager;
use mapmap_core::{
    audio::{backend::AudioBackend, AudioAnalyzer, AudioConfig},
    LayerManager, Mapping, MappingManager, OutputId, Paint, PaintManager,
};
use mapmap_media::{
    FFmpegDecoder, PlaybackCommand, PlaybackState, TestPatternDecoder, VideoPlayer,
};
use mapmap_render::{
    ColorCalibrationRenderer, Compositor, EdgeBlendRenderer, MeshRenderer, QuadRenderer,
    RenderBackend, TextureDescriptor, WgpuBackend,
};
use mapmap_ui::{AppUI, Dashboard, ImGuiContext};
use std::collections::HashMap;
use std::time::Instant;
use tracing::{error, info};
use window_manager::WindowManager;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowId,
};

#[cfg(test)]
mod window_manager_test;

struct App {
    window_manager: WindowManager,
    backend: WgpuBackend,
    #[allow(dead_code)]
    quad_renderer: QuadRenderer,
    mesh_renderer: MeshRenderer,
    #[allow(dead_code)]
    compositor: Compositor,
    edge_blend_renderer: EdgeBlendRenderer,
    color_calibration_renderer: ColorCalibrationRenderer,
    imgui_context: ImGuiContext,
    ui_state: AppUI,
    dashboard: Dashboard,
    layer_manager: LayerManager,
    paint_manager: PaintManager,
    mapping_manager: MappingManager,
    output_manager: mapmap_core::OutputManager,
    player: Option<VideoPlayer>,
    #[allow(dead_code)]
    command_sender: Sender<PlaybackCommand>,
    status_receiver: Receiver<PlaybackState>,
    paint_textures: HashMap<u64, mapmap_render::TextureHandle>, // Paint ID -> Texture
    #[allow(dead_code)]
    layer_textures: HashMap<u64, mapmap_render::TextureHandle>, // Layer ID -> Texture
    intermediate_textures: HashMap<OutputId, mapmap_render::TextureHandle>, // Per-output intermediate textures
    audio_analyzer: AudioAnalyzer,
    audio_backend: Box<dyn AudioBackend + Send>,
    last_frame: Instant,
    frame_count: u32,
    fps: f32,
}

impl App {
    async fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        info!("Initializing MapMap Phase 2 Demo - Multi-Window Projection Mapping");

        // Create wgpu backend first (shared across all windows)
        let backend = WgpuBackend::new().await?;

        let mut window_manager = WindowManager::new();
        let main_output_id = window_manager.create_main_window(event_loop, &backend)?;
        let main_window_context = window_manager.get(main_output_id).unwrap();
        let surface_format = main_window_context.surface_config.format;

        // Create renderers
        let quad_renderer = QuadRenderer::new(backend.device(), surface_format)?;
        let mesh_renderer = MeshRenderer::new(backend.device.clone(), surface_format)?;
        let compositor = Compositor::new(backend.device.clone(), surface_format)?;
        let edge_blend_renderer = EdgeBlendRenderer::new(backend.device.clone(), surface_format)?;
        let color_calibration_renderer =
            ColorCalibrationRenderer::new(backend.device.clone(), surface_format)?;

        // Create ImGui context for main window
        let main_window_ref = &window_manager.get(main_output_id).unwrap().window;
        let imgui_context = ImGuiContext::new(
            main_window_ref,
            backend.device(),
            backend.queue(),
            surface_format,
        );

        // State management setup
        let layer_manager = LayerManager::new();
        let mut paint_manager = PaintManager::new();
        let mut mapping_manager = MappingManager::new();

        // Playback state machine setup
        let (command_sender, command_receiver) = crossbeam_channel::unbounded();
        let (status_sender, status_receiver) = crossbeam_channel::unbounded();
        let dashboard = Dashboard::new(command_sender.clone());

        // Create demo paints
        let paint1 = Paint::test_pattern(1, "Test Pattern 1");
        let paint_id_1 = paint_manager.add_paint(paint1);

        // Create a single player for the first paint
        let decoder = FFmpegDecoder::TestPattern(TestPatternDecoder::new(
            1920,
            1080,
            std::time::Duration::from_secs(5),
            30.0,
        ));
        let player = Some(VideoPlayer::new(decoder, command_receiver, status_sender));
        command_sender.send(PlaybackCommand::Play).ok();
        command_sender
            .send(PlaybackCommand::SetLoopMode(mapmap_media::LoopMode::Loop))
            .ok();

        // Create demo mapping
        let mapping1 = Mapping::quad(1, "Quad Mapping 1", paint_id_1);
        mapping_manager.add_mapping(mapping1);

        info!(
            "Initialization complete - {} paints, {} mappings created",
            paint_manager.paints().len(),
            mapping_manager.mappings().len()
        );

        let audio_config = AudioConfig::default();
        let audio_analyzer = AudioAnalyzer::new(audio_config);
        let mut audio_backend: Box<dyn AudioBackend + Send> = {
            #[cfg(test)]
            {
                Box::new(mapmap_core::audio::backend::mock_backend::MockBackend::new())
            }
            #[cfg(not(test))]
            {
                use mapmap_core::audio::backend::cpal_backend::CpalBackend;
                Box::new(CpalBackend::new(None)?)
            }
        };
        audio_backend.start()?;

        Ok(Self {
            window_manager,
            backend,
            quad_renderer,
            mesh_renderer,
            compositor,
            edge_blend_renderer,
            color_calibration_renderer,
            imgui_context,
            ui_state: AppUI::default(),
            dashboard,
            layer_manager,
            paint_manager,
            mapping_manager,
            output_manager: mapmap_core::OutputManager::new((1920, 1080)),
            player,
            command_sender,
            status_receiver,
            paint_textures: HashMap::new(),
            layer_textures: HashMap::new(),
            intermediate_textures: HashMap::new(),
            audio_analyzer,
            audio_backend,
            last_frame: Instant::now(),
            frame_count: 0,
            fps: 0.0,
        })
    }

    fn update(&mut self) {
        let now = Instant::now();
        let dt = now - self.last_frame;

        // Update audio analysis
        let samples = self.audio_backend.get_samples();
        if !samples.is_empty() {
            self.audio_analyzer
                .process_samples(&samples, dt.as_secs_f64());
        }

        // Update FPS counter
        self.frame_count += 1;
        if self.frame_count % 60 == 0 {
            self.fps = 1.0 / dt.as_secs_f32();
        }

        // Update player and dashboard
        if let Some(player) = self.player.as_mut() {
            if let Ok(new_state) = self.status_receiver.try_recv() {
                self.dashboard
                    .update_state(new_state, player.current_time(), player.duration());
            }

            if let Some(frame) = player.update(dt) {
                // For now, assume player is for paint_id 1
                let paint_id = 1;
                let tex_desc = TextureDescriptor {
                    width: frame.width,
                    height: frame.height,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                    mip_levels: 1,
                };

                match self.backend.create_texture(tex_desc) {
                    Ok(handle) => {
                        let rgba_data = frame.to_rgba();
                        if self
                            .backend
                            .upload_texture(handle.clone(), &rgba_data)
                            .is_ok()
                        {
                            self.paint_textures.insert(paint_id, handle);
                        }
                    }
                    Err(e) => error!("Failed to create texture for paint {}: {}", paint_id, e),
                }
            }
        }

        self.last_frame = now;
    }

    fn render(&mut self) -> Result<()> {
        let mut frames = Vec::new();
        let mut encoders = Vec::new();
        let window_ids: Vec<OutputId> = self.window_manager.window_ids().copied().collect();

        for &output_id in &window_ids {
            if let Some(window_context) = self.window_manager.get(output_id) {
                match window_context.surface.get_current_texture() {
                    Ok(frame) => frames.push((output_id, frame)),
                    Err(e) => {
                        error!("Surface error for output {}: {:?}", output_id, e);
                        continue;
                    }
                }
            }
        }

        for (output_id, frame) in &frames {
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder =
                self.backend
                    .device()
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some(&format!("Render Encoder Output {}", output_id)),
                    });

            let is_main_window = Some(*output_id) == self.window_manager.main_window_id();
            self.render_to_view(&mut encoder, &view, *output_id, is_main_window)?;
            encoders.push(encoder);
        }

        let command_buffers: Vec<_> = encoders.into_iter().map(|e| e.finish()).collect();
        self.backend.queue().submit(command_buffers);

        for (_, frame) in frames {
            frame.present();
        }

        Ok(())
    }

    fn render_to_view(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        output_id: OutputId,
        is_main_window: bool,
    ) -> Result<()> {
        let output_config = if !is_main_window {
            self.output_manager.get_output(output_id).cloned()
        } else {
            None
        };

        let needs_post_processing = output_config
            .as_ref()
            .map_or(false, |c| c.edge_blend.is_enabled() || c.color_calibration.is_enabled());

        if needs_post_processing && !self.intermediate_textures.contains_key(&output_id) {
            let window_context = self.window_manager.get(output_id).unwrap();
            let tex_desc = TextureDescriptor {
                width: window_context.surface_config.width,
                height: window_context.surface_config.height,
                format: wgpu::TextureFormat::Bgra8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                mip_levels: 1,
            };
            if let Ok(handle) = self.backend.create_texture(tex_desc) {
                self.intermediate_textures.insert(output_id, handle);
            }
        }

        let render_target_view = if needs_post_processing {
            self.intermediate_textures
                .get(&output_id)
                .map(|h| h.create_view())
        } else {
            None
        };
        let target_view = render_target_view.as_ref().unwrap_or(view);

        // Render mappings
        {
            let visible_mappings = self.mapping_manager.visible_mappings();
            let render_data: Vec<_> = visible_mappings
                .iter()
                .filter_map(|mapping| {
                    self.paint_textures.get(&mapping.paint_id).map(|texture| {
                        let (vbuf, ibuf) = self.mesh_renderer.create_mesh_buffers(&mapping.mesh);
                        let transform = Mat4::IDENTITY;
                        let ubuf = self
                            .mesh_renderer
                            .create_uniform_buffer(transform, mapping.opacity);
                        let ubg = self.mesh_renderer.create_uniform_bind_group(&ubuf);
                        let tview = texture.create_view();
                        let tbg = self.mesh_renderer.create_texture_bind_group(&tview);
                        (vbuf, ibuf, ubg, tbg, mapping.mesh.indices.len() as u32)
                    })
                })
                .collect();

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Mapping Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                    resolve_target: None,
                })],
                depth_stencil_attachment: None,
            });

            for (vbuf, ibuf, ubg, tbg, icount) in &render_data {
                self.mesh_renderer
                    .draw(&mut rpass, vbuf, ibuf, *icount, ubg, tbg, true);
            }
        }

        // Post-processing would go here...

        // Render UI on main window
        if is_main_window {
            let main_window = &self.window_manager.get(output_id).unwrap().window;
            let ui_state = &mut self.ui_state;
            let dashboard = &mut self.dashboard;
            let layer_manager = &mut self.layer_manager;
            let paint_manager = &mut self.paint_manager;
            let mapping_manager = &mut self.mapping_manager;
            let output_manager = &mut self.output_manager;
            let fps = self.fps;
            let frame_time = self.last_frame.elapsed().as_secs_f32() * 1000.0;
            let audio_analyzer = &self.audio_analyzer;

            self.imgui_context.render(
                main_window,
                self.backend.device(),
                self.backend.queue(),
                encoder,
                view,
                |ui| {
                    ui_state.render_menu_bar(ui);
                    dashboard.ui(ui); // New dashboard
                    ui_state.render_layer_panel(ui, layer_manager);
                    ui_state.render_paint_panel(ui, paint_manager);
                    ui_state.render_mapping_panel(ui, mapping_manager);
                    ui_state.render_transform_panel(ui, layer_manager);
                    ui_state.render_master_controls(ui, layer_manager);
                    ui_state.render_output_panel(ui, output_manager);
                    ui_state.render_edge_blend_panel(ui, output_manager);
                    ui_state.render_color_calibration_panel(ui, output_manager);
                    ui_state.render_stats(ui, fps, frame_time);
                    ui_state.render_audio_panel(ui, audio_analyzer);
                },
            );
        }

        Ok(())
    }

    fn handle_ui_actions(&mut self) -> bool {
        use mapmap_ui::UIAction;
        let actions = self.ui_state.take_actions();
        for action in actions {
            match action {
                UIAction::LoadVideo(path) => {
                    info!("Load video action: {}", path);
                    // Simplified: does not open file dialog for now
                    self.load_video_file(&path);
                }
                UIAction::Exit => return false,
                // Other actions...
                _ => {}
            }
        }
        true
    }

    fn load_video_file(&mut self, _path: &str) {
        // Dummy implementation for now to avoid breaking changes
        info!("load_video_file is not fully implemented in this refactor.");
    }

    fn handle_window_event(&mut self, window_id: WindowId, event: &WindowEvent) -> bool {
        let output_id = match self.window_manager.get_output_id_from_window_id(window_id) {
            Some(id) => id,
            None => return true,
        };

        match event {
            WindowEvent::CloseRequested => {
                if Some(output_id) == self.window_manager.main_window_id() {
                    info!("Main window closed, exiting application");
                    return false;
                } else {
                    info!("Closing output window {}", output_id);
                    self.window_manager.remove_window(output_id);
                    self.output_manager.remove_output(output_id);
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(ctx) = self.window_manager.get_mut(output_id) {
                    info!("Resized to {}x{}", size.width, size.height);
                    ctx.surface_config.width = size.width;
                    ctx.surface_config.height = size.height;
                    ctx.surface.configure(self.backend.device(), &ctx.surface_config);
                }
            }
            _ => {}
        }
        true
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    info!("Starting MapMap");
    let event_loop = EventLoop::new();
    let mut app = pollster::block_on(App::new(&event_loop))?;

    event_loop.run(move |event, event_loop_target, control_flow| {
        *control_flow = ControlFlow::Poll;

        if let Some(id) = app.window_manager.main_window_id() {
            if let Some(ctx) = app.window_manager.get(id) {
                app.imgui_context.handle_event(&ctx.window, &event);
            }
        }

        match event {
            Event::WindowEvent { event, window_id } => {
                if !app.handle_window_event(window_id, &event) {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::MainEventsCleared => {
                app.update();
                if !app.handle_ui_actions() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                if let Err(e) = app.window_manager.sync_windows(event_loop_target, &app.backend, &app.output_manager) {
                    error!("Failed to sync windows: {}", e);
                }
                for ctx in app.window_manager.iter() {
                    ctx.window.request_redraw();
                }
            }
            Event::RedrawRequested(_) => {
                if let Err(e) = app.render() {
                    error!("Render error: {}", e);
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}
