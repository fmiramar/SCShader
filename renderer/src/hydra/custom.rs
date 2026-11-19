//! Session-local, generation-bound, resource-free WGSL operations.
use super::{
    ir::{Argument, Node, Program},
    registry::ValueType,
};
pub const MAX_FUNCTIONS: usize = 16;
#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    pub name: String,
    pub kind: ValueType,
    pub defaults: Vec<f32>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Definition {
    pub name: String,
    pub generation: i32,
    pub category: u8,
    pub inputs: Vec<Input>,
    pub body: String,
}
pub fn identifier(name: &str, max: usize) -> bool {
    !name.is_empty()
        && name.len() <= max
        && name.as_bytes()[0].is_ascii_alphabetic()
        && name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
        && !name.starts_with("u_")
        && !name.starts_with("h_")
}
impl Definition {
    pub fn wire_name(&self) -> String {
        format!("u_{}_{}", self.name, self.generation)
    }
    pub fn types(&self) -> Vec<ValueType> {
        let mut types = if self.category >= 3 {
            vec![ValueType::Expression]
        } else {
            vec![]
        };
        types.extend(self.inputs.iter().map(|i| i.kind));
        types
    }
    pub fn decode(bytes: &[u8], generation: i32) -> Result<Self, String> {
        struct Reader<'a> {
            bytes: &'a [u8],
            offset: usize,
        }
        impl<'a> Reader<'a> {
            fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
                let end = self.offset.checked_add(n).ok_or("custom length overflow")?;
                let b = self
                    .bytes
                    .get(self.offset..end)
                    .ok_or("truncated custom definition")?;
                self.offset = end;
                Ok(b)
            }
            fn byte(&mut self) -> Result<u8, String> {
                Ok(self.take(1)?[0])
            }
            fn text(&mut self, n: usize) -> Result<String, String> {
                String::from_utf8(self.take(n)?.to_vec())
                    .map_err(|_| "custom text must be UTF-8".into())
            }
        }
        if bytes.len() > 3072 {
            return Err("custom definition exceeds 3 KiB".into());
        }
        let mut r = Reader { bytes, offset: 0 };
        if r.take(6)? != b"HCFN\0\x01" {
            return Err("unsupported custom definition version".into());
        }
        let length = r.byte()? as usize;
        let category = r.byte()?;
        let count = r.byte()? as usize;
        let body_length = u16::from_be_bytes(r.take(2)?.try_into().unwrap()) as usize;
        if length > 24 || count > 8 || body_length > 2048 {
            return Err("custom definition limits".into());
        }
        let name = r.text(length)?;
        let mut inputs = Vec::new();
        for _ in 0..count {
            let size = r.byte()? as usize;
            let kind = match r.byte()? {
                0 => ValueType::Float,
                1 => ValueType::Vector,
                _ => return Err("custom inputs support float/vec4 only".into()),
            };
            if size > 16 {
                return Err("custom input name limit".into());
            }
            let name = r.text(size)?;
            let mut defaults = Vec::new();
            for _ in 0..if kind == ValueType::Float { 1 } else { 4 } {
                defaults.push(f32::from_bits(u32::from_be_bytes(
                    r.take(4)?.try_into().unwrap(),
                )));
            }
            inputs.push(Input {
                name,
                kind,
                defaults,
            });
        }
        let body = r.text(body_length)?;
        if r.offset != bytes.len() {
            return Err("trailing custom definition bytes".into());
        }
        let definition = Self {
            name,
            generation,
            category,
            inputs,
            body,
        };
        definition.validate()?;
        Ok(definition)
    }
    pub fn declaration(&self, name: &str) -> String {
        let mut args = vec!["uv:vec2<f32>".to_string(), "time:f32".to_string()];
        if self.category == 2 || self.category == 3 {
            args.push("color:vec4<f32>".into());
        }
        if self.category >= 3 {
            args.push("other:vec4<f32>".into());
        }
        args.extend(self.inputs.iter().map(|i| {
            format!(
                "{}:{}",
                i.name,
                if i.kind == ValueType::Float {
                    "f32"
                } else {
                    "vec4<f32>"
                }
            )
        }));
        format!(
            "fn {name}({})->{}{{\n{}\n}}\n",
            args.join(","),
            if self.category == 1 || self.category == 4 {
                "vec2<f32>"
            } else {
                "vec4<f32>"
            },
            self.body
        )
    }
    pub fn validate(&self) -> Result<(), String> {
        if !identifier(&self.name, 24)
            || super::registry::spec(&self.name).is_some()
            || [
                "free",
                "out",
                "render",
                "call",
                "setFunction",
                "removeFunction",
            ]
            .contains(&self.name.as_str())
            || self.generation <= 0
            || self.category > 4
            || self.inputs.len() > 8
            || self.body.is_empty()
            || self.body.len() > 2048
            || !self.body.is_ascii()
            || self.body.contains('\0')
        {
            return Err("invalid/reserved custom name, signature or WGSL body limit".into());
        }
        let mut names = std::collections::HashSet::new();
        for i in &self.inputs {
            let count = match i.kind {
                ValueType::Float => 1,
                ValueType::Vector => 4,
                _ => return Err("custom inputs support float/vec4 only".into()),
            };
            if !identifier(&i.name, 16)
                || ["uv", "time", "color", "other"].contains(&i.name.as_str())
                || !names.insert(&i.name)
                || i.defaults.len() != count
                || i.defaults.iter().any(|v| !v.is_finite())
            {
                return Err("invalid custom input/default".into());
            }
        }
        let source = self.declaration("h_user");
        let module =
            wgpu::naga::front::wgsl::parse_str(&source).map_err(|e| e.emit_to_string(&source))?;
        if module.functions.len() != 1
            || !module.entry_points.is_empty()
            || !module.global_variables.is_empty()
            || !module.constants.is_empty()
            || !module.overrides.is_empty()
        {
            return Err(
                "custom bodies cannot declare modules, resources or extra functions".into(),
            );
        }
        wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .map_err(|e| e.to_string())?;
        for (_, ty) in module.types.iter() {
            if let wgpu::naga::TypeInner::Array { size, base, .. } = &ty.inner
                && (!matches!(size,wgpu::naga::ArraySize::Constant(n) if n.get()<=64)
                    || !matches!(
                        module.types[*base].inner,
                        wgpu::naga::TypeInner::Scalar(_) | wgpu::naga::TypeInner::Vector { .. }
                    ))
            {
                return Err("custom local arrays require at most 64 scalar/vector elements".into());
            }
        }
        let (_, function) = module.functions.iter().next().unwrap();
        if function.expressions.len() > 512 || function.local_variables.len() > 32 {
            return Err("custom expression/local limit".into());
        }
        let mut layout = wgpu::naga::proc::Layouter::default();
        layout.update(module.to_ctx()).map_err(|e| e.to_string())?;
        if function
            .local_variables
            .iter()
            .map(|(_, v)| u64::from(layout[v.ty].size))
            .sum::<u64>()
            > 4096
        {
            return Err("custom locals exceed 4 KiB".into());
        }
        fn bounded(block: &wgpu::naga::Block, count: &mut usize) -> Result<(), String> {
            use wgpu::naga::Statement;
            for s in block.iter() {
                *count += 1;
                if *count > 128 {
                    return Err("custom statement limit".into());
                }
                match s {
                    Statement::Block(b)=>bounded(b,count)?,
                    Statement::If{accept,reject,..}=>{bounded(accept,count)?;bounded(reject,count)?;},
                    Statement::Switch{cases,..}=>{for c in cases {bounded(&c.body,count)?;}},
                    Statement::Emit(_)|Statement::Store{..}|Statement::Return{..}=>{},
                    _=>return Err("custom bodies do not support loops, function dependencies, discard or synchronization".into()),
                }
            }
            Ok(())
        }
        bounded(&function.body, &mut 0)
    }
    pub fn check_node(&self, node: &Node, program: &Program) -> Result<(), String> {
        let types = self.types();
        if node.category != self.category
            || node.arguments.len() != types.len()
            || (node.receiver == 0) != (self.category == 0)
        {
            return Err("custom signature/category mismatch".into());
        }
        for (arg, kind) in node.arguments.iter().zip(types) {
            match (arg, kind) {
                (Argument::Float(v), ValueType::Float) if v.is_finite() => {}
                (Argument::Parameter(id), ValueType::Float)
                    if program.parameters.iter().any(|p| p.id == *id) => {}
                (Argument::Vector(v), ValueType::Vector) if v.iter().all(|x| x.is_finite()) => {}
                (Argument::Expression(_), ValueType::Expression) => {}
                _ => return Err("custom argument type mismatch".into()),
            }
        }
        Ok(())
    }
    pub fn fixture(&self) -> Program {
        let mut nodes = Vec::new();
        for id in 1..=if self.category >= 3 {
            2
        } else if self.category > 0 {
            1
        } else {
            0
        } {
            nodes.push(Node {
                id,
                name: "solid".into(),
                category: 0,
                receiver: 0,
                arguments: [0.2, 0.4, 0.6, 1.].map(Argument::Float).into(),
            });
        }
        let id = nodes.len() as u32 + 1;
        let mut arguments = if self.category >= 3 {
            vec![Argument::Expression(2)]
        } else {
            vec![]
        };
        arguments.extend(self.inputs.iter().map(|i| {
            if i.kind == ValueType::Float {
                Argument::Float(i.defaults[0])
            } else {
                Argument::Vector(i.defaults.clone().try_into().unwrap())
            }
        }));
        nodes.push(Node {
            id,
            name: self.wire_name(),
            category: self.category,
            receiver: if self.category == 0 { 0 } else { 1 },
            arguments,
        });
        Program {
            custom: vec![self.clone()],
            parameters: vec![],
            nodes,
            root: id,
        }
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    pub fn definition(category: u8, body: &str) -> Definition {
        Definition {
            name: "testFn".into(),
            generation: 7,
            category,
            inputs: vec![Input {
                name: "gain".into(),
                kind: ValueType::Float,
                defaults: vec![0.5],
            }],
            body: body.into(),
        }
    }
    pub fn payload(d: &Definition) -> Vec<u8> {
        let mut bytes = b"HCFN\0\x01".to_vec();
        bytes.extend([d.name.len() as u8, d.category, d.inputs.len() as u8]);
        bytes.extend((d.body.len() as u16).to_be_bytes());
        bytes.extend(d.name.as_bytes());
        for i in &d.inputs {
            bytes.extend([
                i.name.len() as u8,
                if i.kind == ValueType::Float { 0 } else { 1 },
            ]);
            bytes.extend(i.name.as_bytes());
            for v in &i.defaults {
                bytes.extend(v.to_bits().to_be_bytes());
            }
        }
        bytes.extend(d.body.as_bytes());
        bytes
    }
    #[test]
    fn schema_roundtrip_vector_and_dynamic_arguments() {
        let mut d = definition(2, "return color*tint*gain;");
        d.inputs.push(Input {
            name: "tint".into(),
            kind: ValueType::Vector,
            defaults: vec![1., 0.5, 0.25, 1.],
        });
        let bytes = payload(&d);
        assert_eq!(Definition::decode(&bytes, 7).unwrap(), d);
        for end in 0..bytes.len() {
            assert!(Definition::decode(&bytes[..end], 7).is_err());
        }
        let mut p = d.fixture();
        p.parameters.push(super::super::parameters::Parameter {
            id: 1,
            kind: 0,
            values: vec![0.5],
        });
        p.nodes.last_mut().unwrap().arguments[0] = Argument::Parameter(1);
        super::super::compiler::compile(&p).unwrap();
    }
    #[test]
    fn typed_categories_validate_and_compile() {
        for (kind, body) in [
            (0, "return vec4<f32>(uv,gain,1.0);"),
            (1, "return uv*gain;"),
            (2, "return color*gain;"),
            (3, "return mix(color,other,gain);"),
            (4, "return uv+other.xy*gain;"),
        ] {
            let d = definition(kind, body);
            d.validate().unwrap();
            super::super::compiler::compile(&d.fixture()).unwrap();
        }
    }
    #[test]
    fn invalid_and_unbounded_bodies_are_rejected() {
        for body in [
            "return vec2<f32>(1.0);",
            "loop {}",
            "return h_user(uv,time,gain);",
            "return vec4<f32>(missing);",
            "return vec4<f32>(1.0); } @group(0) @binding(0) var t:texture_2d<f32>; fn extra()->vec4<f32>{return vec4<f32>(0.0);",
            "var a:array<f32,100000>;return vec4<f32>(a[0]);",
        ] {
            assert!(definition(0, body).validate().is_err(), "{body}");
        }
        let mut d = definition(0, "return vec4<f32>(gain);");
        d.name = "osc".into();
        assert!(d.validate().is_err());
    }
}
