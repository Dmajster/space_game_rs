use crate::{rendering::RenderPass, App};
use egui::{ClippedPrimitive, FullOutput, Ui};
use egui_wgpu::renderer::ScreenDescriptor;
use egui_winit::State;
use wgpu::{CommandEncoder, TextureView};

pub struct EguiRenderPass {
    pub context: egui::Context,
    pub state: State,

    renderer: egui_wgpu::Renderer,
    clipped_primitives: Vec<ClippedPrimitive>,
    screen_descriptor: ScreenDescriptor,
    full_output: FullOutput,

    window_master: WindowMaster,
}

impl EguiRenderPass {
    pub fn new(app: &App) -> Self {
        Self {
            context: egui::Context::default(),
            renderer: egui_wgpu::Renderer::new(
                &app.renderer.device,
                app.renderer.surface_format,
                None,
                1,
            ),
            clipped_primitives: vec![],
            screen_descriptor: ScreenDescriptor {
                size_in_pixels: [
                    app.renderer.window.inner_size().width,
                    app.renderer.window.inner_size().height,
                ],
                pixels_per_point: 1.0,
            },
            full_output: FullOutput::default(),
            state: egui_winit::State::new(&app.renderer.window),
            window_master: WindowMaster::new(),
        }
    }
}

impl RenderPass for EguiRenderPass {
    fn prepare(&mut self, app: &App) {
        let raw_input = self.state.take_egui_input(&app.renderer.window);

        self.full_output = self.context.run(raw_input, |context| {
            egui::SidePanel::new(egui::panel::Side::Left, "Profiler")
                .show(context, |ui| self.window_master.on_update(ui));
        });

        self.state.handle_platform_output(
            &app.renderer.window,
            &self.context,
            self.full_output.platform_output.clone(),
        );

        self.clipped_primitives = self.context.tessellate(self.full_output.shapes.clone());

        for (id, image_delta) in &self.full_output.textures_delta.set {
            self.renderer.update_texture(
                &app.renderer.device,
                &app.renderer.queue,
                *id,
                image_delta,
            );
        }
    }

    fn render(&mut self, app: &App, mut encoder: &mut CommandEncoder, view: &TextureView) {
        self.renderer.update_buffers(
            &app.renderer.device,
            &app.renderer.queue,
            &mut encoder,
            self.clipped_primitives.as_slice(),
            &self.screen_descriptor,
        );

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        self.renderer.render(
            &mut render_pass,
            self.clipped_primitives.as_slice(),
            &self.screen_descriptor,
        );
    }

    fn cleanup(&mut self, _app: &App) {
        for id in &self.full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }
}

struct WindowMaster {
    tabs: Vec<Box<dyn GuiWindow>>,
    selected_tab_index: usize,
}

impl WindowMaster {
    pub fn new() -> Self {
        Self {
            tabs: vec![
                Box::new(WindowImporter::default()),
                Box::new(WindowProfiler::default()),
            ],
            selected_tab_index: 0,
        }
    }
}

impl GuiWindow for WindowMaster {
    fn window_name(&self) -> &'static str {
        "editor"
    }

    fn on_show(&mut self) {
        self.selected_tab_index = 0;
    }

    fn on_hide(&mut self) {}

    fn on_update(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for (tab_index, tab) in self.tabs.iter().enumerate() {
                if ui.button(tab.window_name()).clicked() {
                    self.selected_tab_index = tab_index;
                }
            }
        });

        self.tabs[self.selected_tab_index].on_update(ui);
    }
}

trait GuiWindow {
    fn window_name(&self) -> &'static str;

    fn on_show(&mut self);

    fn on_hide(&mut self);

    fn on_update(&mut self, ui: &mut Ui);
}

#[derive(Default)]
struct WindowImporter {}

impl GuiWindow for WindowImporter {
    fn window_name(&self) -> &'static str {
        "importer"
    }

    fn on_show(&mut self) {}

    fn on_hide(&mut self) {}

    fn on_update(&mut self, _ui: &mut Ui) {}
}

#[derive(Default)]
struct WindowProfiler {}

impl GuiWindow for WindowProfiler {
    fn window_name(&self) -> &'static str {
        "profiler"
    }

    fn on_show(&mut self) {}

    fn on_hide(&mut self) {}

    fn on_update(&mut self, ui: &mut Ui) {
        puffin_egui::profiler_ui(ui)
    }
}
