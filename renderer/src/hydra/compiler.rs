//! Independently implemented native compiler for the first Hydra operation family.
//! Pinned behavioral reference and native differences: docs/hydra/ARCHITECTURE.md.
use super::ir::{Argument, Program};
use std::{collections::HashMap, fmt::Write};

const MAX_WGSL: usize = 256 * 1024;
const PRELUDE: &str = r#"
struct HydraFrame {
    clock: vec4<f32>,
    resolution: vec4<f32>,
    pointer: vec4<f32>,
    counters: vec4<u32>,
};
@group(0) @binding(0) var<uniform> hydra: HydraFrame;
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};
@vertex fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    let positions = array<vec2<f32>, 3>(vec2<f32>(-1.0,-1.0), vec2<f32>(3.0,-1.0), vec2<f32>(-1.0,3.0));
    let position = positions[index];
    var result: VertexOutput;
    result.position = vec4<f32>(position, 0.0, 1.0);
    result.uv = position * 0.5 + vec2<f32>(0.5);
    return result;
}
"#;

#[derive(Debug)]
enum Operation {
    Solid([f32; 4]),
    Osc([f32; 3]),
    Gradient(f32),
    Rotate { receiver: u32, args: [f32; 2] },
    Scale { receiver: u32, args: [f32; 5] },
    Color { receiver: u32, args: [f32; 4] },
}
fn scalars<const N: usize>(args: &[Argument]) -> Result<[f32; N], String> {
    if args.len() != N {
        return Err("compiler argument count mismatch".into());
    }
    let mut result = [0.; N];
    for (slot, arg) in result.iter_mut().zip(args) {
        match arg {
            Argument::Float(value) if value.is_finite() => *slot = *value,
            _ => return Err("compiler requires finite scalar literals".into()),
        }
    }
    Ok(result)
}
fn lower(program: &Program) -> Result<Vec<(u32, Operation)>, String> {
    if program.nodes.is_empty() || program.nodes.len() > 256 {
        return Err("compiler node limit".into());
    }
    let mut known = HashMap::new();
    let mut result = Vec::with_capacity(program.nodes.len());
    for node in &program.nodes {
        if node.id == 0 || node.id > i32::MAX as u32 || known.contains_key(&node.id) {
            return Err("compiler invalid node ID".into());
        }
        let depth = if node.receiver == 0 {
            1
        } else {
            known
                .get(&node.receiver)
                .copied()
                .ok_or("compiler missing receiver")?
                + 1
        };
        if depth > 64 {
            return Err("compiler expression depth exceeds 64".into());
        }
        let (category, operation) = match node.name.as_str() {
            "solid" => (0, Operation::Solid(scalars(&node.arguments)?)),
            "osc" => {
                let args = scalars(&node.arguments)?;
                if args[0] == 0. {
                    return Err(
                        "osc frequency zero has undefined upstream division semantics".into(),
                    );
                }
                (0, Operation::Osc(args))
            }
            "gradient" => (0, Operation::Gradient(scalars::<1>(&node.arguments)?[0])),
            "rotate" => (
                1,
                Operation::Rotate {
                    receiver: node.receiver,
                    args: scalars(&node.arguments)?,
                },
            ),
            "scale" => {
                let args: [f32; 5] = scalars(&node.arguments)?;
                if args[0] * args[1] == 0.
                    || args[0] * args[2] == 0.
                    || !(args[0] * args[1]).is_finite()
                    || !(args[0] * args[2]).is_finite()
                {
                    return Err("scale requires finite nonzero axis extents".into());
                }
                (
                    1,
                    Operation::Scale {
                        receiver: node.receiver,
                        args,
                    },
                )
            }
            "color" => (
                2,
                Operation::Color {
                    receiver: node.receiver,
                    args: scalars(&node.arguments)?,
                },
            ),
            _ => {
                return Err(format!(
                    "operation {} is not supported by the initial WGSL compiler",
                    node.name
                ));
            }
        };
        if category != node.category || (category == 0) != (node.receiver == 0) {
            return Err("compiler category/receiver mismatch".into());
        }
        known.insert(node.id, depth);
        result.push((node.id, operation));
    }
    if !known.contains_key(&program.root) {
        return Err("compiler missing root".into());
    }
    Ok(result)
}
// Exact IEEE bits avoid decimal formatting overflow/underflow at the f32 limits.
fn literal(value: f32) -> String {
    format!("bitcast<f32>(0x{:08x}u)", value.to_bits())
}
fn vector(values: &[f32]) -> String {
    format!(
        "vec{}<f32>({})",
        values.len(),
        values
            .iter()
            .map(|v| literal(*v))
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub fn compile(program: &Program) -> Result<String, String> {
    let nodes = lower(program)?;
    let mut source = String::from(PRELUDE);
    for (id, operation) in nodes {
        writeln!(source, "fn n{id}(uv: vec2<f32>) -> vec4<f32> {{").unwrap();
        match operation {
            Operation::Solid(args) => {
                writeln!(source, "return {};", vector(&args)).unwrap();
            }
            Operation::Gradient(speed) => {
                writeln!(
                    source,
                    "return vec4<f32>(uv, sin(hydra.clock.x * {}), 1.0);",
                    literal(speed)
                )
                .unwrap();
            }
            Operation::Osc([frequency, sync, offset]) => {
                // Keep the upstream order of phase operations, including frequency division.
                writeln!(
                    source,
                    "let time_shift = hydra.clock.x * {};",
                    literal(sync)
                )
                .unwrap();
                writeln!(
                    source,
                    "let frequency = {}; let offset = {};",
                    literal(frequency),
                    literal(offset)
                )
                .unwrap();
                source.push_str("return vec4<f32>(0.5 + 0.5*sin((uv.x-offset/frequency+time_shift)*frequency), 0.5 + 0.5*sin((uv.x+time_shift)*frequency), 0.5 + 0.5*sin((uv.x+offset/frequency+time_shift)*frequency), 1.0);\n");
            }
            Operation::Rotate {
                receiver,
                args: [angle, speed],
            } => {
                writeln!(
                    source,
                    "let angle = {} + hydra.clock.x * {};",
                    literal(angle),
                    literal(speed)
                )
                .unwrap();
                source.push_str(
                    "let p = uv - vec2<f32>(0.5); let c = cos(angle); let s = sin(angle);\n",
                );
                writeln!(
                    source,
                    "return n{receiver}(vec2<f32>(c*p.x+s*p.y, -s*p.x+c*p.y)+vec2<f32>(0.5));"
                )
                .unwrap();
            }
            Operation::Scale {
                receiver,
                args: [amount, x, y, ox, oy],
            } => {
                writeln!(
                    source,
                    "let origin = {}; let extent = {} * {};",
                    vector(&[ox, oy]),
                    literal(amount),
                    vector(&[x, y])
                )
                .unwrap();
                writeln!(source, "return n{receiver}((uv-origin)/extent+origin);").unwrap();
            }
            Operation::Color { receiver, args } => {
                writeln!(
                    source,
                    "let c = n{receiver}(uv); let k = {};",
                    vector(&args)
                )
                .unwrap();
                source.push_str(
                    "return select((vec4<f32>(1.0)-c)*abs(k), c*k, k>=vec4<f32>(0.0));\n",
                );
            }
        }
        source.push_str("}\n");
        if source.len() > MAX_WGSL {
            return Err("generated WGSL exceeds 256 KiB".into());
        }
    }
    writeln!(source,"@fragment fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {{ return n{}(input.uv); }}", program.root).unwrap();
    if source.len() > MAX_WGSL {
        return Err("generated WGSL exceeds 256 KiB".into());
    }
    validate(&source)?;
    Ok(source)
}
fn validate(source: &str) -> Result<(), String> {
    let module =
        wgpu::naga::front::wgsl::parse_str(source).map_err(|error| error.emit_to_string(source))?;
    wgpu::naga::valid::Validator::new(
        wgpu::naga::valid::ValidationFlags::all(),
        wgpu::naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::ir::Node;
    use super::*;
    fn node(id: u32, name: &str, category: u8, receiver: u32, args: &[f32]) -> Node {
        Node {
            id,
            name: name.into(),
            category,
            receiver,
            arguments: args.iter().map(|v| Argument::Float(*v)).collect(),
        }
    }
    #[test]
    fn initial_family_validates_and_is_deterministic() {
        for source in [
            node(1, "solid", 0, 0, &[0.2, 0.4, 0.6, 1.]),
            node(1, "osc", 0, 0, &[10., 0.1, 0.]),
            node(1, "gradient", 0, 0, &[0.5]),
        ] {
            let program = Program {
                nodes: vec![
                    source,
                    node(2, "rotate", 1, 1, &[0.2, 0.]),
                    node(3, "scale", 1, 2, &[1.5, 1., 1., 0.5, 0.5]),
                    node(4, "color", 2, 3, &[-1., 0.5, 1., 1.]),
                ],
                root: 4,
            };
            let shader = compile(&program).unwrap();
            assert_eq!(shader, compile(&program).unwrap());
        }
    }
    #[test]
    fn malformed_and_undefined_inputs_fail_before_gpu() {
        for n in [
            node(1, "osc", 0, 0, &[0., 0.1, 0.]),
            node(1, "solid", 0, 0, &[f32::NAN, 0., 0., 1.]),
            node(1, "noise", 0, 0, &[3., 0.1]),
            node(1, "rotate", 1, 9, &[0., 0.]),
        ] {
            assert!(
                compile(&Program {
                    nodes: vec![n],
                    root: 1
                })
                .is_err()
            );
        }
    }
}
