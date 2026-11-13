//! Four paired histories with fixed numeric ordering. See TEMPORAL_SEMANTICS.md.
use super::{
    gpu::Candidate,
    ir::{Argument, Node, Program},
    output::Output,
};
pub struct Outputs {
    pub programs: [Option<Program>; 4],
    slots: [Option<[Output; 2]>; 4],
    blank: Output,
    last: usize,
    sampler: wgpu::Sampler,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
}
impl Outputs {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Result<Self, String> {
        if width == 0
            || height == 0
            || width > 4096
            || height > 4096
            || width > device.limits().max_texture_dimension_2d
            || height > device.limits().max_texture_dimension_2d
        {
            return Err("four-output dimensions exceed 4096/device limit".into());
        }
        let program = Program {
            nodes: vec![Node {
                id: 1,
                name: "solid".into(),
                category: 0,
                receiver: 0,
                arguments: vec![Argument::Float(0.); 4],
            }],
            root: 1,
        };
        let mut candidate = None;
        Candidate::prepare(device, &program, &mut candidate)?;
        let blank = Output::new(device, candidate.unwrap(), 1, 1, format)?;
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Hydra output linear clamp sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Ok(Self {
            programs: std::array::from_fn(|_| None),
            slots: std::array::from_fn(|_| None),
            blank,
            last: 0,
            sampler,
            width,
            height,
            format,
        })
    }
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        index: usize,
        program: &Program,
    ) -> Result<(), String> {
        if index > 3 {
            return Err("invalid destination output".into());
        }
        // Reuse only currently live pipelines, keyed by normalized generated WGSL.
        // No inactive cache survives output replacement or session disposal.
        let source = super::compiler::compile(program)?;
        let mut candidate = self
            .slots
            .iter()
            .flatten()
            .map(|pair| &pair[0].candidate)
            .find(|candidate| candidate.source == source)
            .cloned();
        Candidate::prepare(device, program, &mut candidate)?;
        let candidate = candidate.unwrap();
        // Construct both images before replacing any accepted state. New WebGPU
        // textures are zero-initialized before first read (including after resize).
        let pair = [
            Output::new(
                device,
                candidate.clone(),
                self.width,
                self.height,
                self.format,
            )?,
            Output::new(device, candidate, self.width, self.height, self.format)?,
        ];
        self.slots[index] = Some(pair);
        self.programs[index] = Some(program.clone());
        self.bind(device);
        Ok(())
    }
    fn bind(&mut self, device: &wgpu::Device) {
        for i in 0..4 {
            for dst in 0..2 {
                let inputs = self.slots[i]
                    .as_ref()
                    .and_then(|pair| pair[dst].candidate._inputs.as_ref())
                    .map(|layout| {
                        // Before this pass, lower slots have already swapped;
                        // higher slots have not. Self reads its last completed
                        // image. Binding 5 is native prev(), always self history.
                        let views: Vec<_> = (0..5)
                            .map(|binding| {
                                let slot = if binding == 4 { i } else { binding };
                                let bank = if slot <= i { 1 - dst } else { dst };
                                self.slots[slot]
                                    .as_ref()
                                    .map_or(&self.blank, |p| &p[bank])
                                    .texture
                                    .create_view(&Default::default())
                            })
                            .collect();
                        let mut entries: Vec<_> = views
                            .iter()
                            .enumerate()
                            .map(|(slot, view)| wgpu::BindGroupEntry {
                                binding: if slot == 4 { 5 } else { slot as u32 },
                                resource: wgpu::BindingResource::TextureView(view),
                            })
                            .collect();
                        entries.push(wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        });
                        device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Hydra temporal reads"),
                            layout,
                            entries: &entries,
                        })
                    });
                if let Some(pair) = &mut self.slots[i] {
                    pair[dst].inputs = inputs;
                }
            }
        }
    }
    pub fn current(&self, index: usize) -> &Output {
        self.slots[index]
            .as_ref()
            .map_or(&self.blank, |pair| &pair[self.last])
    }
    pub fn resized(&self, device: &wgpu::Device, width: u32, height: u32) -> Result<Self, String> {
        let mut next = Self::new(device, width, height, self.format)?;
        for i in 0..4 {
            if let Some(program) = &self.programs[i] {
                next.prepare(device, i, program)?;
            }
        }
        Ok(next)
    }
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    pub fn texture_bytes(&self) -> u64 {
        4 + self
            .slots
            .iter()
            .flatten()
            .flatten()
            .map(|o| u64::from(o.width) * u64::from(o.height) * 4)
            .sum::<u64>()
    }
    pub fn encode_outputs(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        time: f32,
        delta: f32,
        frame: u32,
    ) {
        let dst = 1 - self.last;
        for pair in self.slots.iter().flatten() {
            let output = &pair[dst];
            let mut encoder = device.create_command_encoder(&Default::default());
            output.encode_output(queue, &mut encoder, time, delta, frame);
            // Retain the existing Metal split-submission boundary between passes.
            queue.submit(Some(encoder.finish()));
        }
        // All assigned outputs advance together, so two prebuilt bind groups
        // per output suffice. No per-frame texture/bind-group allocation.
        self.last = dst;
    }
    pub fn encode_present(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        mode: i32,
        width: u32,
        height: u32,
    ) {
        if (0..4).contains(&mode) {
            self.current(mode as usize).encode_present(encoder, view);
        } else {
            self.blank.encode_present(encoder, view);
            for i in 0..4 {
                let x = if i / 2 == 0 { 0 } else { width / 2 };
                let y = if i % 2 == 0 { 0 } else { height / 2 };
                let w = if i / 2 == 0 {
                    width / 2
                } else {
                    width - width / 2
                };
                let h = if i % 2 == 0 {
                    height / 2
                } else {
                    height - height / 2
                };
                if w > 0 && h > 0 {
                    self.current(i).encode_present_region(
                        encoder,
                        view,
                        Some([x as f32, y as f32, w as f32, h as f32]),
                    );
                }
            }
        }
    }
}
