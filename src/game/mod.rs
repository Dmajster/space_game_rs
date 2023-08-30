use std::mem;

use glam::{Mat4, Vec3, Vec4};
use wgpu::util::DeviceExt;

pub mod opaque_pass;
pub mod shadow_pass;
pub mod z_pre_pass;

use crate::{
    app::{Res, ResMut},
    asset_server::AssetServer,
    rendering::{light::RenderLight, Renderer, MAX_LIGHTS_COUNT},
    scene::{scene_object::SceneObject, Scene},
};

use self::{
    opaque_pass::OpaqueRenderPass, shadow_pass::ShadowRenderPass, z_pre_pass::ZPreRenderPass,
};

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraGpu {
    pub view_projection: Mat4,
    pub world_position: Vec4,
}

impl CameraGpu {
    fn update(&mut self, camera_scene_object: &SceneObject) {
        self.view_projection = camera_scene_object
            .camera_component
            .as_ref()
            .unwrap()
            .build_view_projection_matrix(&camera_scene_object.transform_component);

        let position = camera_scene_object.transform_component.position;

        self.world_position = Vec4::new(position.x, position.y, position.z, 1.0);
    }
}

pub struct Game {
    // Move this to high level renderer
    pub lights_storage_buffer: wgpu::Buffer,

    camera_uniform: CameraGpu,
    pub camera_uniform_buffer: wgpu::Buffer,

    pub global_bind_group_layout: wgpu::BindGroupLayout,
    pub global_bind_group: wgpu::BindGroup,

    // pub shadow_pass: ShadowRenderPass,
    pub z_pre_pass: ZPreRenderPass,
    pub opaque_pass: OpaqueRenderPass,
}

impl Game {
    pub fn new(renderer: &mut Renderer) -> Self {
        let lights_storage_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lights storage buffer"),
            size: MAX_LIGHTS_COUNT * mem::size_of::<RenderLight>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_uniform = CameraGpu::default();

        let camera_uniform_buffer =
            renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("renderer bind group camera buffer"),
                    contents: bytemuck::cast_slice(&[camera_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let global_bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("renderer bind group layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::all(),
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let global_bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("renderer bind group"),
                layout: &global_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_uniform_buffer.as_entire_binding(),
                }],
            });

        let z_pre_pass = ZPreRenderPass::new(renderer, &global_bind_group_layout);
        // let shadow_pass = ShadowRenderPass::new(renderer, &lights_storage_buffer);
        let opaque_pass =
            OpaqueRenderPass::new(renderer, &global_bind_group_layout, &lights_storage_buffer);

        Self {
            lights_storage_buffer,
            camera_uniform,
            camera_uniform_buffer,
            global_bind_group_layout,
            global_bind_group,
            // shadow_pass,
            z_pre_pass,
            opaque_pass,
        }
    }
}

pub fn update(
    game: ResMut<Game>,
    scene: Res<Scene>,
    renderer: ResMut<Renderer>,
    asset_server: Res<AssetServer>,
) {
    let asset_server = asset_server.get();
    let mut app = game.get_mut();
    let mut renderer = renderer.get_mut();
    let scene = scene.get();

    renderer.create_render_meshes(&asset_server);
    renderer.create_render_materials(&asset_server);

    if let Some(camera_scene_object) = scene.get(scene.camera_scene_object_id) {
        app.camera_uniform.update(&camera_scene_object);

        renderer.queue.write_buffer(
            &app.camera_uniform_buffer,
            0,
            bytemuck::cast_slice(&[app.camera_uniform]),
        );
    }

    let lights = scene
        .scene_objects
        .iter()
        .filter_map(|scene_object| {
            if let Some(light_component) = &scene_object.light_component {
                let direction = match light_component.ty {
                    crate::components::light::LightType::DirectionalLight => {
                        -scene_object.transform_component.position
                    }
                    crate::components::light::LightType::PointLight => Vec3::NEG_Y,
                    crate::components::light::LightType::SpotLight => {
                        let transform = scene_object.transform_component.build_transform_matrix();
                        let (_, rotation, _) = transform.to_scale_rotation_translation();

                        rotation * Vec3::Z
                    }
                };

                Some(RenderLight {
                    ty: unsafe { mem::transmute(light_component.ty) },
                    position: scene_object.transform_component.position,
                    luminous_intensity: light_component.luminous_intensity,
                    direction,
                    inner_angle: light_component.inner_angle.to_radians(),
                    color: light_component.color,
                    outer_angle: light_component.outer_angle.to_radians(),
                    falloff_radius: light_component.falloff_radius,
                    unused0: 0.0,
                    unused1: 0.0,
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    renderer
        .queue
        .write_buffer(&app.lights_storage_buffer, 0, bytemuck::cast_slice(&lights));
}
