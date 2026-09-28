use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, event_loop::OwnedDisplayHandle, window::Window};

use crate::{
    controls::{CONTROL_BYTES, ControlValue, Controls, Interpolation},
    overlay::Overlay,
    platform::{GpuBackend, GpuSelection},
    protocol::RESOURCE_ID,
};

const DEFAULT_SHADER: &str = include_str!("../shaders/fullscreen.wgsl");
const FULLSCREEN_VERTEX_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    let position = positions[vertex_index];
    var output: VertexOutput;
    output.position = vec4<f32>(position, 0.0, 1.0);
    output.uv = position * 0.5 + vec2<f32>(0.5);
    return output;
}
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShaderLanguage {
    Wgsl,
    Glsl,
    ShaderToy,
}

impl ShaderLanguage {
    pub fn parse(source_type: &str) -> Result<Self, String> {
        match source_type {
            "wgsl" => Ok(Self::Wgsl),
            "glsl" => Ok(Self::Glsl),
            "shadertoy" => Ok(Self::ShaderToy),
            _ => Err("shader source type must be wgsl, glsl, or shadertoy".to_owned()),
        }
    }

    fn is_glsl(self) -> bool {
        !matches!(self, Self::Wgsl)
    }

    fn label(self) -> &'static str {
        match self {
            Self::Wgsl => "WGSL",
            Self::Glsl => "GLSL",
            Self::ShaderToy => "Shadertoy GLSL",
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Uniforms {
    resolution: [f32; 2],
    time: f32,
    amount: f32,
    source_mix: f32,
    feedback: f32,
    padding: [f32; 2],
    mouse: [f32; 4],
    time_delta: f32,
    frame: f32,
    compat_padding: [f32; 2],
    date: [f32; 4],
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderStats {
    pub fps: f64,
    pub frame_index: u64,
    pub gpu_frame_ms: f64,
    pub cpu_frame_ms: f64,
}

struct ShaderResource {
    source: String,
    path: Option<PathBuf>,
    modified_at: Option<SystemTime>,
    language: ShaderLanguage,
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniforms: Uniforms,
    controls: Controls,
    control_buffer: wgpu::Buffer,
}

struct TextureResource {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    pixels: Option<Vec<u8>>,
}

/// Only CPU state crosses a device restart. Last-valid source is retained even
/// when the source file has since been removed or contains a failed edit.
struct ShaderSnapshot {
    source: String,
    path: Option<PathBuf>,
    modified_at: Option<SystemTime>,
    language: ShaderLanguage,
    uniforms: Uniforms,
    controls: Controls,
}

#[derive(Clone, Debug)]
pub struct GpuIssue {
    pub recoverable: bool,
    pub message: String,
}

struct BufferResource {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    values: Vec<f32>,
    storage_length: u32,
}

struct FeedbackTargets {
    textures: [TextureResource; 2],
    read_index: usize,
}

impl FeedbackTargets {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let textures = [
            create_texture_resource(device, "SCShader feedback A", width, height, format),
            create_texture_resource(device, "SCShader feedback B", width, height, format),
        ];
        for texture in &textures {
            // Keep initialization bounded under repeated resize too: the Metal
            // multipass retention workaround below also applies to clear passes.
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("SCShader feedback clear encoder"),
            });
            let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SCShader feedback clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            drop(pass);
            queue.submit(Some(encoder.finish()));
        }
        Self {
            textures,
            read_index: 0,
        }
    }

    fn read_view(&self) -> &wgpu::TextureView {
        &self.textures[self.read_index].view
    }

    fn write_view(&self) -> &wgpu::TextureView {
        &self.textures[1 - self.read_index].view
    }

    fn swap(&mut self) {
        self.read_index = 1 - self.read_index;
    }
}

#[derive(Debug)]
pub enum HotReload {
    Reloaded { resource_id: i32 },
    Failed { resource_id: i32, message: String },
}

pub struct Renderer {
    instance: wgpu::Instance,
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    gpu_issue: Arc<Mutex<Option<GpuIssue>>>,
    surface_recreated: bool,
    #[cfg(feature = "gpu-test-hooks")]
    simulate_surface_loss: bool,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline_layout: wgpu::PipelineLayout,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    fallback_texture: TextureResource,
    fallback_buffer: BufferResource,
    textures: HashMap<i32, TextureResource>,
    buffers: HashMap<i32, BufferResource>,
    source_texture_id: Option<i32>,
    spectrum_buffer_id: Option<i32>,
    feedback_targets: FeedbackTargets,
    graph_targets: FeedbackTargets,
    feedback_enabled: bool,
    started_at: Instant,
    backend_name: String,
    backend_choice: GpuBackend,
    gpu_selection: GpuSelection,
    display: OwnedDisplayHandle,
    separate_render_passes: bool,
    device_name: String,
    startup_diagnostic: Option<String>,
    stats: RenderStats,
    last_presented_at: Option<Instant>,
    shaders: HashMap<i32, ShaderResource>,
    active_shader_id: i32,
    graph_nodes: Vec<i32>,
    last_reload_check: Instant,
    last_uniform_time: f32,
    mouse_position: [f32; 2],
    mouse_click_position: [f32; 2],
    mouse_pressed: bool,
    overlay: Option<Overlay>,
    show_stats: bool,
    last_compile_ok: bool,
    scheduled_events: usize,
    late_events: u64,
}

impl Renderer {
    pub fn new(
        window: Arc<Window>,
        custom_shader: Option<&str>,
        vsync: bool,
        backend: GpuBackend,
        gpu_selection: GpuSelection,
        display: OwnedDisplayHandle,
    ) -> Result<Self, String> {
        pollster::block_on(Self::new_async(
            window,
            custom_shader,
            vsync,
            backend,
            gpu_selection,
            display,
        ))
    }

    async fn new_async(
        window: Arc<Window>,
        custom_shader: Option<&str>,
        vsync: bool,
        backend: GpuBackend,
        gpu_selection: GpuSelection,
        display: OwnedDisplayHandle,
    ) -> Result<Self, String> {
        let size = window.inner_size();
        let backends = backend.mask(std::env::consts::OS)?;
        if !wgpu::Instance::enabled_backend_features().intersects(backends) {
            return Err(format!(
                "backend {backend:?} is not compiled into this renderer"
            ));
        }
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            // SC/Qt can put an older dxcompiler.dll in the working directory.
            // Auto would load it even though wgpu requires DXC >= 1.8.2502.
            // Our current shader subset works with the system FXC compiler.
            backend_options: wgpu::BackendOptions {
                dx12: wgpu::Dx12BackendOptions {
                    shader_compiler: wgpu::Dx12Compiler::Fxc,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..wgpu::InstanceDescriptor::new_with_display_handle(Box::new(display.clone()))
        });
        let surface = instance
            .create_surface(window.clone())
            .map_err(|error| format!("could not create GPU surface: {error}"))?;
        let adapter = gpu_selection
            .request_adapter(&instance, backends, &surface)
            .await?;
        let adapter_info = adapter.get_info();
        eprintln!(
            "SCShader GPU: {} ({:?}, {:?}), driver: {} {}",
            adapter_info.name,
            adapter_info.backend,
            adapter_info.device_type,
            adapter_info.driver,
            adapter_info.driver_info
        );
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("SCShader device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits {
                    // The public Float32 ABI permits a one-row 16K data texture
                    // where supported; wgpu's default device limit is only 8K.
                    max_texture_dimension_2d: adapter
                        .limits()
                        .max_texture_dimension_2d
                        .min(crate::protocol::MAX_BUFFER_VALUES),
                    ..Default::default()
                },
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|error| format!("could not create GPU device: {error}"))?;

        let gpu_issue = Arc::new(Mutex::new(None));
        let lost_issue = gpu_issue.clone();
        device.set_device_lost_callback(move |reason, message| {
            // A real device loss takes precedence over secondary validation
            // errors produced while the backend is reporting the loss.
            *lost_issue.lock().unwrap_or_else(|error| error.into_inner()) = Some(GpuIssue {
                recoverable: true,
                message: format!("GPU device lost ({reason:?}): {message}"),
            });
        });
        let error_issue = gpu_issue.clone();
        device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
            let mut issue = error_issue
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            if issue.is_none() {
                *issue = Some(GpuIssue {
                    recoverable: false,
                    message: error.to_string(),
                });
            }
        }));

        let width = size.width.max(1);
        let height = size.height.max(1);
        let mut config = surface
            .get_default_config(&adapter, width, height)
            .ok_or_else(|| "GPU surface has no compatible configuration".to_owned())?;
        config.present_mode = present_mode(vsync);
        config.desired_maximum_frame_latency = 2;
        surface.configure(&device, &config);

        let uniforms = Uniforms {
            resolution: [width as f32, height as f32],
            time: 0.0,
            amount: 0.5,
            source_mix: 0.0,
            feedback: 0.0,
            padding: [0.0; 2],
            mouse: [0.0; 4],
            time_delta: 0.0,
            frame: 0.0,
            compat_padding: [0.0; 2],
            date: shadertoy_date(),
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SCShader uniforms"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SCShader resource layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(
                            std::mem::size_of::<Uniforms>() as u64,
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(CONTROL_BYTES as u64),
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SCShader pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("SCShader texture sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let fallback_texture = create_rgba_texture(
            &device,
            &queue,
            "SCShader fallback texture",
            1,
            1,
            &[0, 0, 0, 255],
        );
        let fallback_buffer =
            create_float_buffer_resource(&device, &queue, "SCShader fallback spectrum buffer", 1);
        let feedback_targets = FeedbackTargets::new(&device, &queue, width, height, config.format);
        let graph_targets = FeedbackTargets::new(&device, &queue, width, height, config.format);

        let (pipeline, controls, startup_diagnostic) = if let Some(source) = custom_shader {
            match create_pipeline(
                &device,
                &pipeline_layout,
                config.format,
                source,
                ShaderLanguage::Wgsl,
            ) {
                Ok((pipeline, controls)) => (pipeline, controls, None),
                Err(error) => {
                    let (fallback, controls) = create_pipeline(
                        &device,
                        &pipeline_layout,
                        config.format,
                        DEFAULT_SHADER,
                        ShaderLanguage::Wgsl,
                    )
                    .map_err(|fallback_error| {
                        format!(
                            "custom shader failed ({error}); built-in fallback also failed ({fallback_error})"
                        )
                    })?;
                    (
                        fallback,
                        controls,
                        Some(format!(
                            "custom WGSL validation failed; continuing with built-in diagnostic-safe shader: {error}"
                        )),
                    )
                }
            }
        } else {
            let (pipeline, controls) = create_pipeline(
                &device,
                &pipeline_layout,
                config.format,
                DEFAULT_SHADER,
                ShaderLanguage::Wgsl,
            )?;
            (pipeline, controls, None)
        };

        let initial_shader = ShaderResource {
            source: if startup_diagnostic.is_none() {
                custom_shader.unwrap_or(DEFAULT_SHADER)
            } else {
                DEFAULT_SHADER
            }
            .to_owned(),
            path: None,
            modified_at: None,
            language: ShaderLanguage::Wgsl,
            pipeline,
            uniform_buffer,
            uniforms,
            controls,
            control_buffer: create_control_buffer(&device),
        };
        let mut shaders = HashMap::new();
        shaders.insert(RESOURCE_ID, initial_shader);

        Ok(Self {
            instance,
            backend_choice: backend,
            gpu_selection,
            display,
            window,
            surface,
            gpu_issue,
            surface_recreated: false,
            #[cfg(feature = "gpu-test-hooks")]
            simulate_surface_loss: false,
            device,
            queue,
            config,
            pipeline_layout,
            bind_group_layout,
            sampler,
            fallback_texture,
            fallback_buffer,
            textures: HashMap::new(),
            buffers: HashMap::new(),
            source_texture_id: None,
            spectrum_buffer_id: None,
            feedback_targets,
            graph_targets,
            feedback_enabled: false,
            started_at: Instant::now(),
            backend_name: format!("{:?}", adapter_info.backend),
            separate_render_passes: adapter_info.backend == wgpu::Backend::Metal,
            device_name: adapter_info.name,
            last_compile_ok: startup_diagnostic.is_none(),
            startup_diagnostic,
            stats: RenderStats {
                // Timestamp queries are not enabled in the feasibility renderer.
                gpu_frame_ms: -1.0,
                ..Default::default()
            },
            last_presented_at: None,
            shaders,
            active_shader_id: RESOURCE_ID,
            graph_nodes: Vec::new(),
            last_reload_check: Instant::now(),
            last_uniform_time: 0.0,
            mouse_position: [0.0; 2],
            mouse_click_position: [0.0; 2],
            mouse_pressed: false,
            overlay: None,
            show_stats: false,
            scheduled_events: 0,
            late_events: 0,
        })
    }

    pub fn gpu_issue(&self) -> Option<GpuIssue> {
        self.gpu_issue
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    pub fn poll_gpu(&self) {
        let _ = self.device.poll(wgpu::PollType::Poll);
    }

    #[cfg(feature = "gpu-test-hooks")]
    pub fn feedback_pixel(&self) -> Result<Vec<u8>, String> {
        let texture = &self.feedback_targets.textures[self.feedback_targets.read_index]._texture;
        let size = texture
            .format()
            .block_copy_size(None)
            .ok_or("unreadable texture format")?;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("SCShader test readback"),
            size: 256,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: texture.width() / 2,
                    y: texture.height() / 2,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(256),
                    rows_per_image: Some(1),
                },
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer.map_async(wgpu::MapMode::Read, .., move |result| {
            let _ = sender.send(result);
        });
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(std::time::Duration::from_secs(2)),
            })
            .map_err(|error| error.to_string())?;
        receiver
            .recv_timeout(std::time::Duration::from_secs(2))
            .map_err(|error| error.to_string())?
            .map_err(|error| error.to_string())?;
        let bytes = buffer
            .get_mapped_range(..)
            .map_err(|error| error.to_string())?[..size as usize]
            .to_vec();
        buffer.unmap();
        Ok(bytes)
    }

    #[cfg(feature = "gpu-test-hooks")]
    pub fn simulate_device_loss(&self) {
        // Destroys only this process's logical device; never resets the GPU.
        self.device.destroy();
        // Some backends defer callbacks until an attempted operation. Ensure
        // the test follows the loss path before submitting more work.
        *self
            .gpu_issue
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(GpuIssue {
            recoverable: true,
            message: "test-requested logical device destruction".to_owned(),
        });
    }

    pub fn recover(mut self, vsync: bool) -> Result<Self, String> {
        let shaders: Vec<_> = self
            .shaders
            .drain()
            .map(|(id, shader)| {
                (
                    id,
                    ShaderSnapshot {
                        source: shader.source,
                        path: shader.path,
                        modified_at: shader.modified_at,
                        language: shader.language,
                        uniforms: shader.uniforms,
                        controls: shader.controls,
                    },
                )
            })
            .collect();
        let images: Vec<_> = self
            .textures
            .drain()
            .map(|(id, texture)| {
                (
                    id,
                    texture._texture.width(),
                    texture._texture.height(),
                    texture.pixels.unwrap_or_default(),
                )
            })
            .collect();
        let buffers: Vec<_> = self
            .buffers
            .drain()
            .map(|(id, buffer)| (id, buffer.values))
            .collect();
        let window = self.window.clone();
        let backend = self.backend_choice;
        let gpu_selection = self.gpu_selection.clone();
        let display = self.display.clone();
        let state = (
            self.started_at,
            self.stats,
            self.last_uniform_time,
            self.active_shader_id,
            std::mem::take(&mut self.graph_nodes),
            self.source_texture_id,
            self.spectrum_buffer_id,
            self.feedback_enabled,
            self.mouse_position,
            self.mouse_click_position,
            self.mouse_pressed,
            self.show_stats,
            self.last_compile_ok,
            self.scheduled_events,
            self.late_events,
        );
        // Drop all old GPU objects and the old presentation surface before
        // creating another surface for this same window.
        drop(self);
        let mut restored = Self::new(window, None, vsync, backend, gpu_selection, display)?;
        restored.shaders.clear();
        for (id, snapshot) in shaders {
            let (pipeline, _) = create_pipeline(
                &restored.device,
                &restored.pipeline_layout,
                restored.config.format,
                &snapshot.source,
                snapshot.language,
            )?;
            let mut uniforms = snapshot.uniforms;
            uniforms.resolution = [restored.config.width as f32, restored.config.height as f32];
            let uniform_buffer =
                restored
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("SCShader recovered uniforms"),
                        contents: bytemuck::bytes_of(&uniforms),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });
            restored.shaders.insert(
                id,
                ShaderResource {
                    source: snapshot.source,
                    path: snapshot.path,
                    modified_at: snapshot.modified_at,
                    language: snapshot.language,
                    pipeline,
                    uniforms,
                    uniform_buffer,
                    controls: snapshot.controls,
                    control_buffer: create_control_buffer(&restored.device),
                },
            );
        }
        for (id, width, height, pixels) in images {
            restored.textures.insert(
                id,
                create_rgba_texture(
                    &restored.device,
                    &restored.queue,
                    "SCShader recovered image",
                    width,
                    height,
                    &pixels,
                ),
            );
        }
        for (id, values) in buffers {
            let mut buffer = create_float_buffer_resource(
                &restored.device,
                &restored.queue,
                "SCShader recovered data",
                values.len() as u32,
            );
            buffer.values = values;
            upload_float_buffer(&restored.queue, &mut buffer);
            restored.buffers.insert(id, buffer);
        }
        (
            restored.started_at,
            restored.stats,
            restored.last_uniform_time,
            restored.active_shader_id,
            restored.graph_nodes,
            restored.source_texture_id,
            restored.spectrum_buffer_id,
            restored.feedback_enabled,
            restored.mouse_position,
            restored.mouse_click_position,
            restored.mouse_pressed,
            restored.show_stats,
            restored.last_compile_ok,
            restored.scheduled_events,
            restored.late_events,
        ) = state;
        restored.queue.submit([]);
        restored.poll_gpu();
        if let Some(issue) = restored.gpu_issue() {
            return Err(issue.message);
        }
        Ok(restored)
    }

    pub fn set_show_stats(&mut self, enabled: bool) {
        self.show_stats = enabled;
        if !enabled {
            self.overlay = None;
        }
    }

    pub fn show_stats(&self) -> bool {
        self.show_stats
    }

    pub fn set_compile_result(&mut self, ok: bool) {
        self.last_compile_ok = ok;
    }

    pub fn last_compile_ok(&self) -> bool {
        self.last_compile_ok
    }

    pub fn set_schedule_stats(&mut self, scheduled: usize, late: u64) {
        self.scheduled_events = scheduled;
        self.late_events = late;
    }

    /// Owned texture allocation estimate; excludes swapchain/driver allocation.
    pub fn texture_bytes(&self) -> u64 {
        let bytes = |texture: &wgpu::Texture| {
            u64::from(texture.width())
                * u64::from(texture.height())
                * u64::from(texture.format().block_copy_size(None).unwrap_or(4))
        };
        bytes(&self.fallback_texture._texture)
            + bytes(&self.fallback_buffer.texture)
            + self
                .textures
                .values()
                .map(|value| bytes(&value._texture))
                .sum::<u64>()
            + self
                .buffers
                .values()
                .map(|value| bytes(&value.texture))
                .sum::<u64>()
            + self
                .feedback_targets
                .textures
                .iter()
                .chain(self.graph_targets.textures.iter())
                .map(|value| bytes(&value._texture))
                .sum::<u64>()
            + self.overlay.as_ref().map_or(0, Overlay::bytes)
    }

    pub fn backend_name(&self) -> &str {
        &self.backend_name
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    pub fn startup_diagnostic(&self) -> Option<&str> {
        self.startup_diagnostic.as_deref()
    }

    pub fn status(&self) -> RenderStats {
        self.stats
    }

    pub fn set_vsync(&mut self, enabled: bool) {
        self.config.present_mode = present_mode(enabled);
        self.surface.configure(&self.device, &self.config);
    }

    pub fn set_mouse_position(&mut self, x: f64, y: f64) {
        self.mouse_position = [
            (x as f32).clamp(0.0, self.config.width as f32),
            (self.config.height as f32 - y as f32).clamp(0.0, self.config.height as f32),
        ];
        self.update_compat_mouse();
    }

    pub fn set_mouse_pressed(&mut self, pressed: bool) {
        if pressed && !self.mouse_pressed {
            self.mouse_click_position = self.mouse_position;
        }
        self.mouse_pressed = pressed;
        self.update_compat_mouse();
    }

    pub fn set_float(&mut self, resource_id: i32, name: &str, value: f32) -> Result<(), String> {
        if !value.is_finite() {
            return Err("uniform value must be finite".to_owned());
        }
        self.set_control(
            resource_id,
            name,
            ControlValue::float(value),
            0.0,
            Interpolation::Step,
        )
    }

    pub fn set_control(
        &mut self,
        resource_id: i32,
        name: &str,
        value: ControlValue,
        duration: f64,
        mode: Interpolation,
    ) -> Result<(), String> {
        let now = self.started_at.elapsed().as_secs_f64();
        let shader = self
            .shaders
            .get_mut(&resource_id)
            .ok_or_else(|| format!("shader resource {resource_id} does not exist"))?;
        shader.controls.set(name, value, duration, mode, now)
    }

    pub fn get_control(&mut self, resource_id: i32, name: &str) -> Result<ControlValue, String> {
        let now = self.started_at.elapsed().as_secs_f64();
        self.shaders
            .get_mut(&resource_id)
            .ok_or_else(|| format!("shader resource {resource_id} does not exist"))?
            .controls
            .get(name, now)
    }

    pub fn controls(&self, resource_id: i32) -> Option<&Controls> {
        self.shaders
            .get(&resource_id)
            .map(|shader| &shader.controls)
    }

    pub fn create_texture(&mut self, texture_id: i32, path: PathBuf) -> Result<(), String> {
        if texture_id <= 0 {
            return Err("texture ID must be positive".to_owned());
        }
        if self.textures.contains_key(&texture_id) {
            return Err(format!("texture resource {texture_id} already exists"));
        }
        let decoded = (|| {
            let mut reader = image::ImageReader::open(&path).map_err(|error| error.to_string())?;
            let mut limits = image::Limits::default();
            // Bound both cached RGBA storage and device dimensions. The decoder's
            // allocation hint is additional protection, not a hard process cap.
            let dimension = 4096.min(self.device.limits().max_texture_dimension_2d);
            limits.max_image_width = Some(dimension);
            limits.max_image_height = Some(dimension);
            limits.max_alloc = Some(128 * 1024 * 1024);
            reader.limits(limits);
            reader
                .decode()
                .map(|image| image.to_rgba8())
                .map_err(|error| error.to_string())
        })();
        let image = match decoded {
            Ok(image) => image,
            Err(error) => {
                self.textures.insert(
                    texture_id,
                    create_rgba_texture(
                        &self.device,
                        &self.queue,
                        "SCShader diagnostic image placeholder",
                        2,
                        2,
                        &[
                            255, 0, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 255, 255,
                        ],
                    ),
                );
                return Err(format!(
                    "could not decode image {}: {error}; using a magenta/black placeholder (maximum image extent 4096 per axis)",
                    path.display()
                ));
            }
        };
        let (width, height) = image.dimensions();
        if width == 0 || height == 0 {
            return Err(format!("image {} has an empty extent", path.display()));
        }
        let texture = create_rgba_texture(
            &self.device,
            &self.queue,
            &format!("SCShader image texture {texture_id}"),
            width,
            height,
            image.as_raw(),
        );
        self.textures.insert(texture_id, texture);
        Ok(())
    }

    pub fn free_texture(&mut self, texture_id: i32) -> Result<(), String> {
        if self.textures.remove(&texture_id).is_none() {
            return Err(format!("texture resource {texture_id} does not exist"));
        }
        if self.source_texture_id == Some(texture_id) {
            self.source_texture_id = None;
            self.shaders
                .values_mut()
                .for_each(|shader| shader.uniforms.source_mix = 0.0);
        }
        Ok(())
    }

    pub fn create_buffer(&mut self, buffer_id: i32, length: u32) -> Result<(), String> {
        if buffer_id <= 0 {
            return Err("buffer ID must be positive".to_owned());
        }
        if self.buffers.contains_key(&buffer_id) {
            return Err(format!("buffer resource {buffer_id} already exists"));
        }
        if length > self.device.limits().max_texture_dimension_2d {
            return Err(format!(
                "buffer length exceeds this GPU's one-row texture limit of {}",
                self.device.limits().max_texture_dimension_2d
            ));
        }
        if length == 0 || length > crate::protocol::MAX_BUFFER_VALUES {
            return Err(format!(
                "buffer length must be between 1 and {}",
                crate::protocol::MAX_BUFFER_VALUES
            ));
        }
        let buffer = create_float_buffer_resource(
            &self.device,
            &self.queue,
            &format!("SCShader float buffer {buffer_id}"),
            length,
        );
        self.buffers.insert(buffer_id, buffer);
        Ok(())
    }

    pub fn write_buffer(
        &mut self,
        buffer_id: i32,
        start: u32,
        values: &[f32],
    ) -> Result<(), String> {
        let buffer = self
            .buffers
            .get_mut(&buffer_id)
            .ok_or_else(|| format!("buffer resource {buffer_id} does not exist"))?;
        let start = usize::try_from(start).map_err(|_| "buffer start is too large".to_owned())?;
        let end = start
            .checked_add(values.len())
            .ok_or_else(|| "buffer write range overflows".to_owned())?;
        if end > buffer.values.len() {
            return Err(format!(
                "buffer write {}..{} exceeds buffer length {}",
                start,
                end,
                buffer.values.len()
            ));
        }
        if values.iter().any(|value| !value.is_finite()) {
            return Err("buffer values must be finite".to_owned());
        }
        buffer.values[start..end].copy_from_slice(values);
        upload_float_buffer(&self.queue, buffer);
        Ok(())
    }

    pub fn free_buffer(&mut self, buffer_id: i32) -> Result<(), String> {
        if self.buffers.remove(&buffer_id).is_none() {
            return Err(format!("buffer resource {buffer_id} does not exist"));
        }
        if self.spectrum_buffer_id == Some(buffer_id) {
            self.spectrum_buffer_id = None;
        }
        Ok(())
    }

    pub fn set_buffer(
        &mut self,
        resource_id: i32,
        name: &str,
        buffer_id: i32,
    ) -> Result<(), String> {
        self.require_active_shader(resource_id)?;
        if name != "spectrum" {
            return Err("the current buffer ABI exposes only the spectrum binding".to_owned());
        }
        if !self.buffers.contains_key(&buffer_id) {
            return Err(format!("buffer resource {buffer_id} does not exist"));
        }
        self.spectrum_buffer_id = Some(buffer_id);
        Ok(())
    }

    pub fn set_texture(
        &mut self,
        resource_id: i32,
        name: &str,
        texture_id: i32,
    ) -> Result<(), String> {
        self.require_active_shader(resource_id)?;
        if name != "source" {
            return Err("the current texture ABI exposes only the source texture".to_owned());
        }
        if !self.textures.contains_key(&texture_id) {
            return Err(format!("texture resource {texture_id} does not exist"));
        }
        self.source_texture_id = Some(texture_id);
        self.feedback_enabled = false;
        self.shaders.values_mut().for_each(|shader| {
            shader.uniforms.source_mix = 1.0;
            shader.uniforms.feedback = 0.0;
        });
        Ok(())
    }

    pub fn set_feedback(
        &mut self,
        resource_id: i32,
        source: &str,
        amount: f32,
    ) -> Result<(), String> {
        self.require_active_shader(resource_id)?;
        if !matches!(source, "previous" | "framebuffer") {
            return Err("feedback source must be previous or framebuffer".to_owned());
        }
        if !amount.is_finite() || !(0.0..=1.0).contains(&amount) {
            return Err("feedback amount must be finite and between 0 and 1".to_owned());
        }
        self.feedback_enabled = true;
        self.shaders.values_mut().for_each(|shader| {
            shader.uniforms.source_mix = amount;
            shader.uniforms.feedback = amount;
        });
        Ok(())
    }

    pub fn create_shader(
        &mut self,
        resource_id: i32,
        path: PathBuf,
        source_type: &str,
    ) -> Result<(), String> {
        let language = ShaderLanguage::parse(source_type)?;
        if self.shaders.contains_key(&resource_id) {
            return Err(format!(
                "shader resource {resource_id} already exists; use reload"
            ));
        }
        let source = read_shader_source(&path)?;
        let (pipeline, controls) = create_pipeline(
            &self.device,
            &self.pipeline_layout,
            self.config.format,
            &source,
            language,
        )
        .map_err(|message| {
            format!(
                "{} {} fragment: {message}",
                path.display(),
                language.label()
            )
        })?;
        let uniforms = self
            .shaders
            .get(&self.active_shader_id)
            .map(|shader| shader.uniforms)
            .ok_or_else(|| "renderer has no active shader".to_owned())?;
        let uniform_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("SCShader shader uniforms"),
                contents: bytemuck::bytes_of(&uniforms),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        self.shaders.insert(
            resource_id,
            ShaderResource {
                source,
                modified_at: shader_modified_at(&path),
                path: Some(path),
                language,
                pipeline,
                uniform_buffer,
                uniforms,
                controls,
                control_buffer: create_control_buffer(&self.device),
            },
        );
        self.active_shader_id = resource_id;
        Ok(())
    }

    pub fn reload_shader(&mut self, resource_id: i32) -> Result<(), String> {
        let path = self
            .shaders
            .get(&resource_id)
            .ok_or_else(|| format!("shader resource {resource_id} does not exist"))?
            .path
            .clone()
            .ok_or_else(|| format!("shader resource {resource_id} has no source path to reload"))?;
        let language = self
            .shaders
            .get(&resource_id)
            .map(|shader| shader.language)
            .ok_or_else(|| format!("shader resource {resource_id} does not exist"))?;
        self.replace_shader_from_path(resource_id, &path, language)
    }

    pub fn free_shader(&mut self, resource_id: i32) -> Result<(), String> {
        if resource_id == RESOURCE_ID
            || self
                .shaders
                .get(&resource_id)
                .is_none_or(|shader| shader.path.is_none())
        {
            return Err(format!("shader resource {resource_id} does not exist"));
        }
        self.shaders.remove(&resource_id);
        self.graph_nodes.retain(|id| *id != resource_id);
        if self.active_shader_id == resource_id {
            self.active_shader_id = RESOURCE_ID;
        }
        Ok(())
    }

    pub fn shader_count(&self) -> i32 {
        i32::try_from(self.shaders.len().saturating_sub(1)).unwrap_or(i32::MAX)
    }

    pub fn texture_count(&self) -> i32 {
        i32::try_from(self.textures.len()).unwrap_or(i32::MAX)
    }

    pub fn buffer_count(&self) -> i32 {
        i32::try_from(self.buffers.len()).unwrap_or(i32::MAX)
    }

    pub fn set_graph(&mut self, resource_ids: Vec<i32>) -> Result<(), String> {
        if resource_ids.is_empty() {
            self.graph_nodes.clear();
            return Ok(());
        }
        if resource_ids.iter().any(|id| !self.shaders.contains_key(id)) {
            return Err("render graph references a shader resource that does not exist".to_owned());
        }
        self.graph_nodes = resource_ids;
        Ok(())
    }

    pub fn poll_hot_reload(&mut self) -> Vec<HotReload> {
        if self.last_reload_check.elapsed().as_millis() < 250 {
            return Vec::new();
        }
        self.last_reload_check = Instant::now();
        let changed: Vec<_> = self
            .shaders
            .iter_mut()
            .filter_map(|(&resource_id, shader)| {
                let path = shader.path.as_ref()?;
                let modified_at = shader_modified_at(path);
                if shader.modified_at == modified_at {
                    return None;
                }
                // One diagnostic per edit, including invalid edits of an inactive pass.
                shader.modified_at = modified_at;
                Some((resource_id, path.clone(), shader.language))
            })
            .collect();
        changed
            .into_iter()
            .map(|(resource_id, path, language)| {
                match self.replace_shader_from_path(resource_id, &path, language) {
                    Ok(()) => {
                        self.last_compile_ok = true;
                        HotReload::Reloaded { resource_id }
                    }
                    Err(message) => {
                        self.last_compile_ok = false;
                        HotReload::Failed {
                            resource_id,
                            message,
                        }
                    }
                }
            })
            .collect()
    }

    fn replace_shader_from_path(
        &mut self,
        resource_id: i32,
        path: &Path,
        language: ShaderLanguage,
    ) -> Result<(), String> {
        let source = read_shader_source(path)?;
        let (pipeline, mut controls) = create_pipeline(
            &self.device,
            &self.pipeline_layout,
            self.config.format,
            &source,
            language,
        )
        .map_err(|message| {
            format!(
                "{} {} fragment: {message}",
                path.display(),
                language.label()
            )
        })?;
        let shader = self
            .shaders
            .get_mut(&resource_id)
            .ok_or_else(|| format!("shader resource {resource_id} does not exist"))?;
        controls.preserve(
            &mut shader.controls,
            self.started_at.elapsed().as_secs_f64(),
        );
        shader.controls = controls;
        shader.pipeline = pipeline;
        shader.path = Some(path.to_path_buf());
        shader.modified_at = shader_modified_at(path);
        shader.language = language;
        shader.source = source;
        Ok(())
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.shaders.values_mut().for_each(|shader| {
            shader.uniforms.resolution = [size.width as f32, size.height as f32];
        });
        self.surface.configure(&self.device, &self.config);
        self.feedback_targets = FeedbackTargets::new(
            &self.device,
            &self.queue,
            size.width,
            size.height,
            self.config.format,
        );
        self.graph_targets = FeedbackTargets::new(
            &self.device,
            &self.queue,
            size.width,
            size.height,
            self.config.format,
        );
        self.set_mouse_position(
            self.mouse_position[0] as f64,
            size.height as f64 - self.mouse_position[1] as f64,
        );
    }

    fn require_active_shader(&self, resource_id: i32) -> Result<(), String> {
        self.shaders
            .contains_key(&resource_id)
            .then_some(())
            .ok_or_else(|| format!("shader resource {resource_id} does not exist"))
    }

    fn update_compat_mouse(&mut self) {
        let signed_click = if self.mouse_pressed {
            self.mouse_click_position
        } else {
            [-self.mouse_click_position[0], -self.mouse_click_position[1]]
        };
        self.shaders.values_mut().for_each(|shader| {
            shader.uniforms.mouse = [
                self.mouse_position[0],
                self.mouse_position[1],
                signed_click[0],
                signed_click[1],
            ];
        });
    }

    fn source_view(&self) -> &wgpu::TextureView {
        if self.feedback_enabled {
            self.feedback_targets.read_view()
        } else if let Some(texture_id) = self.source_texture_id {
            self.textures
                .get(&texture_id)
                .map(|texture| &texture.view)
                .unwrap_or(&self.fallback_texture.view)
        } else {
            &self.fallback_texture.view
        }
    }

    fn spectrum_view(&self) -> &wgpu::TextureView {
        self.spectrum_buffer_id
            .and_then(|buffer_id| self.buffers.get(&buffer_id))
            .map(|buffer| &buffer.view)
            .unwrap_or(&self.fallback_buffer.view)
    }

    fn create_bind_group(
        &self,
        shader: &ShaderResource,
        source_view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SCShader frame resources"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: shader.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(source_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(self.spectrum_view()),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: shader.control_buffer.as_entire_binding(),
                },
            ],
        })
    }

    #[cfg(feature = "gpu-test-hooks")]
    pub fn simulate_surface_loss(&mut self) {
        self.simulate_surface_loss = true;
    }

    fn acquire_frame(&mut self) -> wgpu::CurrentSurfaceTexture {
        #[cfg(feature = "gpu-test-hooks")]
        if std::mem::take(&mut self.simulate_surface_loss) {
            return wgpu::CurrentSurfaceTexture::Lost;
        }
        self.surface.get_current_texture()
    }

    pub fn render(&mut self) -> Result<(), String> {
        let frame_started_at = Instant::now();
        // Acquire before staging per-frame uniforms. A skipped/occluded frame
        // must not accumulate uploads that never reach a queue submission.
        let (frame, reconfigure_after_present) = match self.acquire_frame() {
            wgpu::CurrentSurfaceTexture::Success(frame) => (frame, false),
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => (frame, true),
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                // Flush uploads from OSC data-buffer commands even without a
                // presentable frame, and service completed resource cleanup.
                self.queue.submit([]);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.queue.submit([]);
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                if self.surface_recreated {
                    return Err(
                        "GPU presentation surface was lost again before presenting a frame"
                            .to_owned(),
                    );
                }
                self.queue.submit([]);
                self.surface = self
                    .instance
                    .create_surface(self.window.clone())
                    .map_err(|error| format!("could not recreate GPU surface: {error}"))?;
                self.surface.configure(&self.device, &self.config);
                self.surface_recreated = true;
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err("GPU presentation surface validation failed".to_owned());
            }
        };
        self.surface_recreated = false;
        let now = self.started_at.elapsed().as_secs_f64();
        let time = now as f32;
        let time_delta = (time - self.last_uniform_time).max(0.0);
        let frame_index = self.stats.frame_index as f32;
        let date = shadertoy_date();
        self.shaders.values_mut().for_each(|shader| {
            shader.controls.tick(now);
            shader.uniforms.amount = shader.controls.amount();
            let bytes = shader.controls.bytes();
            if !bytes.is_empty() {
                self.queue.write_buffer(&shader.control_buffer, 0, bytes);
            }
            shader.uniforms.time = time;
            shader.uniforms.time_delta = time_delta;
            shader.uniforms.frame = frame_index;
            shader.uniforms.date = date;
        });
        self.last_uniform_time = time;
        self.shaders.values().for_each(|shader| {
            self.queue.write_buffer(
                &shader.uniform_buffer,
                0,
                bytemuck::bytes_of(&shader.uniforms),
            );
        });

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("SCShader frame encoder"),
            });

        let graph_nodes = if self.graph_nodes.is_empty() {
            vec![self.active_shader_id]
        } else {
            self.graph_nodes.clone()
        };
        let mut previous_is_graph_output = false;
        for (index, resource_id) in graph_nodes.iter().enumerate() {
            let is_last = index + 1 == graph_nodes.len();
            let shader = self
                .shaders
                .get(resource_id)
                .ok_or_else(|| format!("graph shader resource {resource_id} does not exist"))?;
            let source_view = if previous_is_graph_output {
                self.graph_targets.read_view()
            } else {
                self.source_view()
            };
            let bind_group = self.create_bind_group(shader, source_view);
            if is_last {
                encode_fullscreen_pass(
                    &mut encoder,
                    "SCShader feedback render pass",
                    self.feedback_targets.write_view(),
                    &shader.pipeline,
                    &bind_group,
                );
                self.submit_intermediate_pass(&mut encoder);
                encode_fullscreen_pass(
                    &mut encoder,
                    "SCShader presentation render pass",
                    &view,
                    &shader.pipeline,
                    &bind_group,
                );
            } else {
                encode_fullscreen_pass(
                    &mut encoder,
                    "SCShader graph render pass",
                    self.graph_targets.write_view(),
                    &shader.pipeline,
                    &bind_group,
                );
                self.submit_intermediate_pass(&mut encoder);
                self.graph_targets.swap();
                previous_is_graph_output = true;
            }
        }

        if self.show_stats {
            let lines = vec![
                format!("SCSHADER {}  LIVE DIAGNOSTICS", env!("CARGO_PKG_VERSION")),
                format!(
                    "FPS {:.1}  CPU {:.2} MS  GPU N/A",
                    self.stats.fps, self.stats.cpu_frame_ms
                ),
                format!("BACKEND {}", self.backend_name),
                format!("DEVICE {}", self.device_name),
                format!(
                    "SURFACE {} X {}  FRAME {}",
                    self.config.width, self.config.height, self.stats.frame_index
                ),
                format!(
                    "LAST COMPILE {}",
                    if self.last_compile_ok { "OK" } else { "FAILED" }
                ),
                format!(
                    "SCHEDULED {}  LATE {}",
                    self.scheduled_events, self.late_events
                ),
                format!(
                    "OWNED TEXTURES {:.2} MIB (EST.)",
                    self.texture_bytes() as f64 / 1_048_576.
                ),
                "OVERLAY EXCLUDED FROM FEEDBACK".to_owned(),
            ];
            let overlay = self
                .overlay
                .get_or_insert_with(|| Overlay::new(&self.device, self.config.format));
            overlay.update(&self.queue, self.config.width, self.config.height, &lines);
            self.submit_intermediate_pass(&mut encoder);
            self.overlay.as_ref().unwrap().encode(&mut encoder, &view);
        }
        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
        #[cfg(feature = "gpu-counters")]
        if std::env::var_os("SCSHADER_DIAGNOSTIC_WAIT").is_some() {
            self.device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(std::time::Duration::from_secs(2)),
                })
                .map_err(|error| format!("diagnostic GPU wait failed: {error}"))?;
        }
        self.feedback_targets.swap();
        if reconfigure_after_present {
            self.surface.configure(&self.device, &self.config);
        }
        let presented_at = Instant::now();
        self.stats.fps = self
            .last_presented_at
            .map(|previous| 1.0 / presented_at.duration_since(previous).as_secs_f64())
            .filter(|fps| fps.is_finite())
            .unwrap_or(0.0);
        self.stats.frame_index = self.stats.frame_index.saturating_add(1);
        self.stats.cpu_frame_ms =
            presented_at.duration_since(frame_started_at).as_secs_f64() * 1_000.0;
        self.last_presented_at = Some(presented_at);
        #[cfg(feature = "gpu-counters")]
        if self.stats.frame_index.is_multiple_of(600) {
            eprintln!(
                "SCShader GPU counters frame {}: {:?}",
                self.stats.frame_index,
                self.device.get_internal_counters()
            );
        }
        Ok(())
    }

    fn submit_intermediate_pass(&self, encoder: &mut wgpu::CommandEncoder) {
        if self.separate_render_passes {
            // wgpu 30 / Metal retained ~5.6 KiB per frame with two passes in one
            // submission on the tested AMD GPU. Ordered per-pass submissions
            // avoid that path without CPU waits, shader changes, or restarts.
            // Other backends retain the ordinary single-submission frame.
            // Reproduction and removal criteria: docs/MEMORY_INVESTIGATION.md.
            let next = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("SCShader next pass encoder"),
                });
            let finished = std::mem::replace(encoder, next).finish();
            self.queue.submit(Some(finished));
        }
    }
}

fn encode_fullscreen_pass(
    encoder: &mut wgpu::CommandEncoder,
    label: &str,
    view: &wgpu::TextureView,
    pipeline: &wgpu::RenderPipeline,
    bind_group: &wgpu::BindGroup,
) {
    let color_attachments = [Some(wgpu::RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            store: wgpu::StoreOp::Store,
        },
    })];
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &color_attachments,
        ..Default::default()
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.draw(0..3, 0..1);
}

fn present_mode(vsync: bool) -> wgpu::PresentMode {
    if vsync {
        wgpu::PresentMode::AutoVsync
    } else {
        wgpu::PresentMode::AutoNoVsync
    }
}

fn create_texture_resource(
    device: &wgpu::Device,
    label: &str,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
) -> TextureResource {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | if cfg!(feature = "gpu-test-hooks") {
                wgpu::TextureUsages::COPY_SRC
            } else {
                wgpu::TextureUsages::empty()
            },
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    TextureResource {
        _texture: texture,
        view,
        pixels: None,
    }
}

fn create_rgba_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    width: u32,
    height: u32,
    pixels: &[u8],
) -> TextureResource {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    TextureResource {
        _texture: texture,
        view,
        pixels: Some(pixels.to_vec()),
    }
}

fn create_float_buffer_resource(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    length: u32,
) -> BufferResource {
    let storage_length = length.next_multiple_of(64);
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: storage_length,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut resource = BufferResource {
        texture,
        view,
        values: vec![0.0; usize::try_from(storage_length).unwrap_or(0)],
        storage_length,
    };
    upload_float_buffer(queue, &mut resource);
    resource
        .values
        .truncate(usize::try_from(length).unwrap_or(0));
    resource
}

fn upload_float_buffer(queue: &wgpu::Queue, buffer: &mut BufferResource) {
    let storage_length = usize::try_from(buffer.storage_length).unwrap_or(0);
    let mut padded = vec![0.0_f32; storage_length];
    padded[..buffer.values.len()].copy_from_slice(&buffer.values);
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &buffer.texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytemuck::cast_slice(&padded),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * buffer.storage_length),
            rows_per_image: Some(1),
        },
        wgpu::Extent3d {
            width: buffer.storage_length,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
}

fn read_shader_source(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|error| format!("could not read shader {}: {error}", path.display()))
}

fn shader_modified_at(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
}

fn create_control_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("SCShader typed controls"),
        size: CONTROL_BYTES as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn create_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
    source: &str,
    language: ShaderLanguage,
) -> Result<(wgpu::RenderPipeline, Controls), String> {
    // Reflection/compatibility errors must occur before opening the GPU error scope.
    let source = match language {
        ShaderLanguage::ShaderToy => Cow::Owned(shadertoy_glsl(source)?),
        _ => Cow::Borrowed(source),
    };
    let controls = Controls::reflect(&source, language.is_glsl())?;
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let vertex = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("SCShader fullscreen vertex"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(if language.is_glsl() {
            FULLSCREEN_VERTEX_SHADER
        } else {
            &source
        })),
    });
    let fragment = match language {
        ShaderLanguage::Wgsl => device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SCShader WGSL fragment"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&source)),
        }),
        ShaderLanguage::Glsl | ShaderLanguage::ShaderToy => {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("SCShader Naga GLSL fragment"),
                source: wgpu::ShaderSource::Glsl {
                    shader: source,
                    stage: wgpu::naga::ShaderStage::Fragment,
                    defines: &[],
                },
            })
        }
    };
    let fragment_entry = if language.is_glsl() {
        "main"
    } else {
        "fs_main"
    };
    let targets = [Some(wgpu::ColorTargetState {
        format,
        blend: Some(wgpu::BlendState::REPLACE),
        write_mask: wgpu::ColorWrites::ALL,
    })];
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("SCShader fullscreen pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &vertex,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &fragment,
            entry_point: Some(fragment_entry),
            compilation_options: Default::default(),
            targets: &targets,
        }),
        multiview_mask: None,
        cache: None,
    });
    match pollster::block_on(error_scope.pop()) {
        Some(error) => Err(error.to_string()),
        None => Ok((pipeline, controls)),
    }
}

fn shadertoy_glsl(source: &str) -> Result<String, String> {
    if source.contains("void main(") {
        return Err(
            "Shadertoy compatibility expects mainImage(out vec4, in vec2), not a user main()"
                .to_owned(),
        );
    }
    if !source.contains("mainImage") {
        return Err("Shadertoy compatibility requires a mainImage function".to_owned());
    }
    let source = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("#version"))
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!(
        r#"#version 450
layout(set = 0, binding = 0) uniform SCShaderUniforms {{
    vec2 resolution;
    float time;
    float amount;
    float source_mix;
    float feedback;
    vec2 padding;
    vec4 mouse;
    float time_delta;
    float frame;
    vec2 compat_padding;
    vec4 date;
}} sc;
layout(set = 0, binding = 1) uniform texture2D sc_source_texture;
layout(set = 0, binding = 2) uniform sampler sc_source_sampler;

#define iResolution vec3(sc.resolution, 1.0)
#define iTime sc.time
#define iTimeDelta sc.time_delta
#define iFrame int(sc.frame)
#define iMouse sc.mouse
#define iDate sc.date
#define iChannel0 sampler2D(sc_source_texture, sc_source_sampler)
#define iChannel1 sampler2D(sc_source_texture, sc_source_sampler)
#define iChannel2 sampler2D(sc_source_texture, sc_source_sampler)
#define iChannel3 sampler2D(sc_source_texture, sc_source_sampler)
#define scBeat 0.0
#define scTempo 0.0
#define scPhase sc.time
#define scAmplitude sc.amount
#define scPitch 0.0
#define scPitchConfidence 0.0
#define scOnset 0.0

layout(location = 0) out vec4 scshader_color;

{source}

void main() {{
    vec4 color = vec4(0.0);
    mainImage(color, gl_FragCoord.xy);
    scshader_color = color;
}}
"#
    ))
}

fn shadertoy_date() -> [f32; 4] {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64())
        .unwrap_or(0.0);
    let whole_seconds = seconds.floor() as i64;
    let days = whole_seconds.div_euclid(86_400);
    let seconds_of_day = whole_seconds.rem_euclid(86_400) as f32 + (seconds.fract() as f32);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    [year as f32, month as f32, day as f32, seconds_of_day]
}
