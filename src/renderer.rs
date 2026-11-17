use std::f32::consts::TAU;
use std::sync::Arc;
use winit::window::Window;
use easy_gpu::assets::*;
use easy_gpu::assets_manager::*;
use easy_gpu::wgpu::{Extent3d, FilterMode, TextureFormat, TextureUsages, VertexFormat};
use easy_gpu::Renderer;
use rand::Rng;
use winit::dpi::PhysicalSize;
use crate::camera::Camera;

const N: u32 = 1048576;
const WIDTH: u32 = 2048;
const HEIGHT: u32 = 1024;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex{
    position: [f32;2],
}
impl Vertex {
    pub fn new(position: [f32;2]) -> Self {
        Vertex{position}
    }
}
impl GpuVertex for Vertex {
    fn buffer_layout() -> BufferLayout {
        BufferLayout::new()
            .stride(size_of::<Self>() as u64)
            .attribute(0,0,VertexFormat::Float32x2)
    }
}

pub struct Render{
    egpu: Renderer,
    buffer: Handle<Buffer>,
    camera: Camera,
    material: Handle<Material>,
    mesh: Handle<Mesh>,

    agent_bind_group_ab: Handle<ComputeBindGroup>,
    diffuse_bind_group_ab: Handle<ComputeBindGroup>,
    agent_bind_group_ba: Handle<ComputeBindGroup>,
    diffuse_bind_group_ba: Handle<ComputeBindGroup>,
    agent_pipeline: Handle<ComputePipeline>,
    diffuse_pipeline: Handle<ComputePipeline>,

    ab: bool,
}

impl Render{
    pub fn new(window: Arc<Window>)->Self{
        let mut egpu = pollster::block_on(Renderer::new(window.clone()));

        let scale = 4.0;
        let vertices = [
            Vertex::new([-scale, -scale]),
            Vertex::new([scale, -scale]),
            Vertex::new([scale, scale]),
            Vertex::new([-scale, scale])
        ];
        let indices = [0, 1, 2, 0, 2, 3];
        let mesh = egpu.create_mesh(&vertices, &indices);

        let camera = Camera::new();

        let buffer = egpu.create_buffer(
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            size_of::<Camera>() as u64
        );

        let trail_texture_builder = TextureBuilder::new()
            .format(TextureFormat::Rgba16Float)
            .size(Extent3d{
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            })
            .usage(
                TextureUsages::STORAGE_BINDING |
                    TextureUsages::TEXTURE_BINDING |
                    TextureUsages::COPY_SRC |
                    TextureUsages::COPY_DST |
                    TextureUsages::RENDER_ATTACHMENT
            );

        let trail_texture_a = trail_texture_builder.build(&mut egpu);
        let trail_texture_b = trail_texture_builder.build(&mut egpu);

        let tex_sampler = SamplerBuilder::new()
            .filter_mode(FilterMode::Nearest)
            .build(&mut egpu);

        let mut shader = egpu.load_shader(include_str!("render.wgsl"));

        let pipeline = RenderPipelineBuilder::new(shader)
            .vertex_layout(Vertex::buffer_layout())
            .material_layout(&[
                render_uniform(0),
                render_texture(1),
                sampler(2),
            ])
            .depth_format(TextureFormat::Depth24Plus)
            .depth_writes_enabled(false)
            .build(&mut egpu);

        let material = MaterialBuilder::new(pipeline)
            .buffer(0,buffer)
            .texture(1,trail_texture_a)
            .sampler(2,tex_sampler)
            .build(&mut egpu);

        let mut rng = rand::rng();
        let mut agents = Vec::with_capacity(N as usize);

        let center = [WIDTH as f32 /2.0, HEIGHT as f32 /2.0];
        let radius = 500.0;

        for _ in 0..N {
            let angle = rng.random_range(0.0..TAU);
            let distance = radius * rng.random::<f32>().sqrt();

            agents.push(Agent {
                pos: [
                    center[0] + distance * angle.cos(),
                    center[1] + distance * angle.sin(),
                ],
                dir: rng.random_range(0.0..TAU),
                pad: 0.0,
            });
        }

        let storage = egpu.create_buffer_with_contents(
            BufferUsages::STORAGE,
            bytemuck::cast_slice(&agents)
        );

        shader = egpu.load_shader(include_str!("update_agents.wgsl"));

        let agent_pipeline = ComputePipelineBuilder::new(shader)
            .bind_group_layout(&[
                compute_texture_float(0),
                storage_texture(1,TextureFormat::Rgba16Float),
                compute_storage(2,false)
            ])
            .build(&mut egpu);

        let agent_bind_group_ab = ComputeBindGroupBuilder::new(agent_pipeline)
            .texture(0,trail_texture_a)
            .texture(1,trail_texture_b)
            .buffer(2,storage)
            .build(&mut egpu);

        let agent_bind_group_ba = ComputeBindGroupBuilder::new(agent_pipeline)
            .texture(0,trail_texture_b)
            .texture(1,trail_texture_a)
            .buffer(2,storage)
            .build(&mut egpu);


        shader = egpu.load_shader(include_str!("diffuse_texture.wgsl"));

        let diffuse_pipeline = ComputePipelineBuilder::new(shader)
            .bind_group_layout(&[
                compute_texture_float(0),
                storage_texture(1,TextureFormat::Rgba16Float),
            ])
            .build(&mut egpu);

        let diffuse_bind_group_ab = ComputeBindGroupBuilder::new(diffuse_pipeline)
            .texture(0,trail_texture_a)
            .texture(1,trail_texture_b)
            .build(&mut egpu);

        let diffuse_bind_group_ba = ComputeBindGroupBuilder::new(diffuse_pipeline)
            .texture(0,trail_texture_b)
            .texture(1,trail_texture_a)
            .build(&mut egpu);


        Self{
            egpu,
            buffer,
            camera,
            material,
            mesh,
            agent_bind_group_ab,
            diffuse_bind_group_ab,
            agent_bind_group_ba,
            diffuse_bind_group_ba,
            agent_pipeline,
            diffuse_pipeline,
            ab: true,
        }
    }

    pub fn render(&mut self){
        let frame = self.egpu.begin_frame();

        if self.ab{
            frame.compute(
                self.diffuse_bind_group_ab,
                self.diffuse_pipeline,
                (WIDTH/16,HEIGHT/16,1),
            );
            frame.compute(
                self.agent_bind_group_ab,
                self.agent_pipeline,
                (N/64,1,1),
            );
        }
        else{
            frame.compute(
                self.diffuse_bind_group_ba,
                self.diffuse_pipeline,
                (WIDTH/16,HEIGHT/16,1),
            );
            frame.compute(
                self.agent_bind_group_ba,
                self.agent_pipeline,
                (N/64,1,1),
            );
        }
        self.ab = !self.ab;

        frame.draw(
            self.material,
            self.mesh,
        );

        self.egpu.render();
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>){
        self.egpu.resize_surface(size);
        self.camera.resize(size.width as f32, size.height as f32);
        self.egpu.write_buffer(
            self.buffer,
            Uniform{
                resolution: self.camera.res,
            }
        );
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniform{
    resolution: [f32;2],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Agent{
    pos: [f32;2],
    dir: f32,
    pad: f32,
}