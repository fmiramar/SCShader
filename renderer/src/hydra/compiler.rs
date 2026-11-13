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
    Sample(u8),
    Combine {
        name: String,
        receiver: u32,
        other: u32,
        amount: f32,
    },
    Modulation(super::modulation::Modulation),
    Builtin(super::families::Builtin),
    Solid([f32; 4]),
    Osc([f32; 3]),
    Gradient(f32),
    Rotate {
        receiver: u32,
        args: [f32; 2],
    },
    Scale {
        receiver: u32,
        args: [f32; 5],
    },
    Color {
        receiver: u32,
        args: [f32; 4],
    },
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
        let mut depth = 1usize;
        let mut expanded = 1usize;
        for dependency in std::iter::once(node.receiver).filter(|id| *id != 0).chain(
            node.arguments.iter().filter_map(|arg| match arg {
                Argument::Expression(id) => Some(*id),
                _ => None,
            }),
        ) {
            let &(child_depth, cost) = known
                .get(&dependency)
                .ok_or("compiler missing dependency")?;
            depth = depth.max(child_depth + 1);
            expanded = expanded
                .checked_add(cost)
                .ok_or("compiler expansion overflow")?;
        }
        if depth > 64 || expanded > 4096 {
            return Err("compiler depth or expansion limit".into());
        }
        let (category, operation) = match node.name.as_str() {
            "src" => {
                let [Argument::Output(slot)] = node.arguments.as_slice() else {
                    return Err("src requires output slot".into());
                };
                if *slot > 3 {
                    return Err("invalid output slot".into());
                }
                (0, Operation::Sample(*slot))
            }
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
            "add" | "sub" | "blend" | "mult" | "layer" | "diff" | "mask" => {
                let spec = super::registry::spec(&node.name).unwrap();
                if node.arguments.len() != spec.args.len() {
                    return Err("combine arity mismatch".into());
                }
                let Argument::Expression(other) = node.arguments[0] else {
                    return Err("combine requires expression".into());
                };
                let amount = if node.arguments.len() == 2 {
                    scalars::<1>(&node.arguments[1..])?[0]
                } else {
                    1.0
                };
                (
                    3,
                    Operation::Combine {
                        name: node.name.clone(),
                        receiver: node.receiver,
                        other,
                        amount,
                    },
                )
            }
            name if super::registry::spec(name).is_some_and(|spec| spec.category == 4) => (
                4,
                Operation::Modulation(super::modulation::Modulation::lower(node)?),
            ),
            _ => {
                let builtin = super::families::Builtin::lower(node)?;
                (node.category, Operation::Builtin(builtin))
            }
        };
        if category != node.category || (category == 0) != (node.receiver == 0) {
            return Err("compiler category/receiver mismatch".into());
        }
        known.insert(node.id, (depth, expanded));
        result.push((node.id, operation));
    }
    if !known.contains_key(&program.root) {
        return Err("compiler missing root".into());
    }
    Ok(result)
}
// Exact IEEE bits avoid decimal formatting overflow/underflow at the f32 limits.
pub(super) fn literal(value: f32) -> String {
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
    source.push_str(include_str!("procedural.wgsl"));
    if program.nodes.iter().any(|node| node.name == "src") {
        source.push_str("@group(1) @binding(4) var output_sampler: sampler;\n");
        for slot in 0..4 {
            writeln!(
                source,
                "@group(1) @binding({slot}) var output{slot}: texture_2d<f32>;"
            )
            .unwrap();
        }
    }
    for (id, operation) in nodes {
        writeln!(source, "fn n{id}(uv: vec2<f32>) -> vec4<f32> {{").unwrap();
        match operation {
            Operation::Sample(slot) => {
                writeln!(
                    source,
                    "return textureSampleLevel(output{slot},output_sampler,vec2<f32>(fract(uv.x),1.0-fract(uv.y)),0.0);"
                )
                .unwrap();
            }
            Operation::Combine {
                name,
                receiver,
                other,
                amount,
            } => {
                writeln!(
                    source,
                    "let left=n{receiver}(uv); let right=n{other}(uv); let weight={};",
                    literal(amount)
                )
                .unwrap();
                let result = match name.as_str() {
                    "add" => "(left+right)*weight + left*(1.0-weight)",
                    "sub" => "(left-right)*weight + left*(1.0-weight)",
                    "blend" => "left*(1.0-weight)+right*weight",
                    "mult" => "left*(1.0-weight)+(left*right)*weight",
                    "diff" => "vec4<f32>(abs(left.rgb-right.rgb),max(left.a,right.a))",
                    "layer" => {
                        "vec4<f32>(mix(left.rgb,right.rgb,right.a),clamp(left.a+right.a,0.0,1.0))"
                    }
                    "mask" => {
                        "vec4<f32>(left.rgb*h_luminance(right.rgb),left.a*h_luminance(right.rgb))"
                    }
                    _ => unreachable!(),
                };
                writeln!(source, "return {result};").unwrap();
            }
            Operation::Modulation(operation) => operation.emit(&mut source),
            Operation::Builtin(builtin) => builtin.emit(&mut source),
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
    fn every_procedural_default_compiles() {
        use super::super::registry::{PROCEDURAL, ValueType};
        assert_eq!(PROCEDURAL.len(), 32);
        for spec in PROCEDURAL {
            let mut cursor = 0;
            let args = spec
                .args
                .iter()
                .map(|kind| match kind {
                    ValueType::Float => {
                        let value = spec._defaults[cursor];
                        cursor += 1;
                        Argument::Float(value)
                    }
                    ValueType::Vector => {
                        let values = spec._defaults[cursor..cursor + 4].try_into().unwrap();
                        cursor += 4;
                        Argument::Vector(values)
                    }
                    _ => panic!("unexpected procedural argument"),
                })
                .collect();
            let mut nodes = Vec::new();
            if spec.category != 0 {
                nodes.push(node(1, "solid", 0, 0, &[0.17, 0.38, 0.63, 0.72]));
            }
            let id = nodes.len() as u32 + 1;
            nodes.push(Node {
                id,
                name: spec.name.into(),
                category: spec.category,
                receiver: if spec.category == 0 { 0 } else { 1 },
                arguments: args,
            });
            compile(&Program { nodes, root: id }).unwrap_or_else(|e| panic!("{}: {e}", spec.name));
        }
    }
    #[test]
    fn nested_combines_validate_types_dependencies_and_bounds() {
        for spec in super::super::registry::COMBINE {
            let mut nodes = vec![
                node(1, "osc", 0, 0, &[10., 0.1, 0.]),
                node(2, "shape", 0, 0, &[4., 0.4, 0.01]),
                node(3, "rotate", 1, 2, &[0.2, 0.]),
            ];
            let mut args = vec![Argument::Expression(3)];
            if let Some(value) = spec._defaults.first() {
                args.push(Argument::Float(*value));
            }
            nodes.push(Node {
                id: 4,
                name: spec.name.into(),
                category: 3,
                receiver: 1,
                arguments: args,
            });
            let mut program = Program { nodes, root: 4 };
            let source = compile(&program).unwrap();
            assert_eq!(source.matches("@fragment").count(), 1);
            program.nodes[3].arguments[0] = Argument::Expression(9);
            assert!(compile(&program).is_err());
            program.nodes[3].arguments[0] = Argument::Float(0.5);
            assert!(compile(&program).is_err());
        }
        let mut nodes = vec![node(1, "solid", 0, 0, &[0.1, 0.2, 0.3, 0.4])];
        for id in 2..=13 {
            nodes.push(Node {
                id,
                name: "blend".into(),
                category: 3,
                receiver: id - 1,
                arguments: vec![Argument::Expression(id - 1), Argument::Float(0.5)],
            });
        }
        assert!(compile(&Program { nodes, root: 13 }).is_err());
    }
    #[test]
    fn modulation_signatures_and_undefined_constants() {
        assert_eq!(super::super::registry::MODULATION.len(), 11);
        for spec in super::super::registry::MODULATION {
            let mut args = vec![Argument::Expression(2)];
            args.extend(spec._defaults.iter().copied().map(Argument::Float));
            let mut program = Program {
                nodes: vec![
                    node(1, "osc", 0, 0, &[10., 0.1, 0.]),
                    node(2, "gradient", 0, 0, &[0.4]),
                    Node {
                        id: 3,
                        name: spec.name.into(),
                        category: 4,
                        receiver: 1,
                        arguments: args,
                    },
                ],
                root: 3,
            };
            let source = compile(&program).unwrap();
            assert_eq!(source.matches("@fragment").count(), 1);
            program.nodes[2].arguments[0] = Argument::Expression(3);
            assert!(compile(&program).is_err());
            program.nodes[2].arguments[0] = Argument::Float(0.5);
            assert!(compile(&program).is_err());
            program.nodes[2].arguments[0] = Argument::Expression(2);
            program.nodes[2].arguments[1] = Argument::Float(f32::NAN);
            assert!(compile(&program).is_err());
            if matches!(
                spec.name,
                "modulateKaleid" | "modulateScale" | "modulatePixelate"
            ) {
                for arg in &mut program.nodes[2].arguments[1..] {
                    *arg = Argument::Float(0.0);
                }
                assert!(compile(&program).is_err());
            }
        }
    }
    #[test]
    fn malformed_and_undefined_inputs_fail_before_gpu() {
        for n in [
            node(1, "osc", 0, 0, &[0., 0.1, 0.]),
            node(1, "solid", 0, 0, &[f32::NAN, 0., 0., 1.]),
            node(1, "unsupported", 0, 0, &[3., 0.1]),
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
