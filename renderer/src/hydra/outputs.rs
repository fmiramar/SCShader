//! Four bounded outputs; Step 12 permits only acyclic same-frame reads.
use super::{
    gpu::Candidate,
    ir::{Argument, Node, Program},
    output::Output,
};
pub fn order(programs: &[Option<Program>; 4]) -> Result<Vec<usize>, String> {
    fn visit(
        i: usize,
        programs: &[Option<Program>; 4],
        marks: &mut [u8; 4],
        result: &mut Vec<usize>,
    ) -> Result<(), String> {
        if marks[i] == 1 {
            return Err("output feedback cycles require temporal buffering (Step 13)".into());
        }
        if marks[i] == 2 {
            return Ok(());
        }
        marks[i] = 1;
        if let Some(program) = &programs[i] {
            for node in &program.nodes {
                for arg in &node.arguments {
                    if let Argument::Output(slot) = arg {
                        let slot = usize::from(*slot);
                        if slot > 3 {
                            return Err("invalid output slot".into());
                        }
                        visit(slot, programs, marks, result)?;
                    }
                }
            }
        }
        marks[i] = 2;
        result.push(i);
        Ok(())
    }
    let mut result = Vec::new();
    let mut marks = [0; 4];
    for i in 0..4 {
        visit(i, programs, &mut marks, &mut result)?;
    }
    Ok(result)
}
pub struct Outputs {
    pub programs: [Option<Program>; 4],
    pub slots: [Option<Output>; 4],
    blank: Output,
    order: Vec<usize>,
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
            order: vec![0, 1, 2, 3],
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
        let mut programs = self.programs.clone();
        programs[index] = Some(program.clone());
        let order = order(&programs)?;
        let mut candidate = None;
        Candidate::prepare(device, program, &mut candidate)?;
        let output = Output::new(
            device,
            candidate.unwrap(),
            self.width,
            self.height,
            self.format,
        )?;
        self.slots[index] = Some(output);
        self.programs = programs;
        self.order = order;
        self.bind(device);
        Ok(())
    }
    fn bind(&mut self, device: &wgpu::Device) {
        for i in 0..4 {
            let inputs = self.slots[i]
                .as_ref()
                .and_then(|out| out.candidate._inputs.as_ref())
                .map(|layout| {
                    let views: Vec<_> = (0..4)
                        .map(|slot| {
                            // Never bind the destination texture, even for unused shader declarations.
                            let output = if slot == i {
                                &self.blank
                            } else {
                                self.slots[slot].as_ref().unwrap_or(&self.blank)
                            };
                            output.texture.create_view(&Default::default())
                        })
                        .collect();
                    let mut entries: Vec<_> = views
                        .iter()
                        .enumerate()
                        .map(|(slot, view)| wgpu::BindGroupEntry {
                            binding: slot as u32,
                            resource: wgpu::BindingResource::TextureView(view),
                        })
                        .collect();
                    entries.push(wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    });
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Hydra output reads"),
                        layout,
                        entries: &entries,
                    })
                });
            if let Some(output) = &mut self.slots[i] {
                output.inputs = inputs;
            }
        }
    }
    pub fn resized(&self, device: &wgpu::Device, width: u32, height: u32) -> Result<Self, String> {
        let mut next = Self::new(device, width, height, self.format)?;
        // Validate the full graph once; installation order must not introduce temporary cycles.
        for &i in &self.order {
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
            .map(|o| u64::from(o.width) * u64::from(o.height) * 4)
            .sum::<u64>()
    }
    pub fn encode_outputs(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        time: f32,
        delta: f32,
        frame: u32,
    ) {
        for &i in &self.order {
            if let Some(output) = &self.slots[i] {
                let mut encoder = device.create_command_encoder(&Default::default());
                output.encode_output(queue, &mut encoder, time, delta, frame);
                // Retain the existing Metal split-submission boundary between passes.
                queue.submit(Some(encoder.finish()));
            }
        }
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
            self.slots[mode as usize]
                .as_ref()
                .unwrap_or(&self.blank)
                .encode_present(encoder, view);
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
                    self.slots[i]
                        .as_ref()
                        .unwrap_or(&self.blank)
                        .encode_present_region(
                            encoder,
                            view,
                            Some([x as f32, y as f32, w as f32, h as f32]),
                        );
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn sample(slot: u8) -> Program {
        Program {
            nodes: vec![Node {
                id: 1,
                name: "src".into(),
                category: 0,
                receiver: 0,
                arguments: vec![Argument::Output(slot)],
            }],
            root: 1,
        }
    }
    #[test]
    fn static_dependencies_and_cycles() {
        let mut programs = [None, None, None, None];
        programs[0] = Some(sample(3));
        programs[3] = Some(sample(2));
        let result = order(&programs).unwrap();
        assert!(result.iter().position(|i| *i == 2) < result.iter().position(|i| *i == 3));
        assert!(result.iter().position(|i| *i == 3) < result.iter().position(|i| *i == 0));
        programs[2] = Some(sample(0));
        assert!(order(&programs).is_err());
        programs[2] = None;
        programs[1] = Some(sample(1));
        assert!(order(&programs).is_err());
    }
}
