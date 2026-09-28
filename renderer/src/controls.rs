//! User controls live at group 0, binding 4; the legacy built-ins stay at binding 0.
//! Layout comes from Naga, never from source-text guesses or SC dictionary order.
use std::collections::HashSet;

use wgpu::naga::{self, AddressSpace, ScalarKind, TypeInner};

pub const CONTROL_BYTES: usize = 16_384;
pub const MAX_CONTROLS: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlType {
    Float,
    Int,
    Uint,
    Bool,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
}

impl ControlType {
    pub fn parse(name: &str) -> Result<Self, String> {
        match name {
            "float" => Ok(Self::Float),
            "int" => Ok(Self::Int),
            "uint" => Ok(Self::Uint),
            "bool" => Ok(Self::Bool),
            "vec2" => Ok(Self::Vec2),
            "vec3" => Ok(Self::Vec3),
            "vec4" => Ok(Self::Vec4),
            "mat2" => Ok(Self::Mat2),
            "mat3" => Ok(Self::Mat3),
            "mat4" => Ok(Self::Mat4),
            _ => Err(format!("unsupported uniform type {name:?}")),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Float => "float",
            Self::Int => "int",
            Self::Uint => "uint",
            Self::Bool => "bool",
            Self::Vec2 => "vec2",
            Self::Vec3 => "vec3",
            Self::Vec4 => "vec4",
            Self::Mat2 => "mat2",
            Self::Mat3 => "mat3",
            Self::Mat4 => "mat4",
        }
    }

    pub fn len(self) -> usize {
        match self {
            Self::Vec2 => 2,
            Self::Vec3 => 3,
            Self::Vec4 | Self::Mat2 => 4,
            Self::Mat3 => 9,
            Self::Mat4 => 16,
            _ => 1,
        }
    }

    fn is_float(self) -> bool {
        !matches!(self, Self::Int | Self::Uint | Self::Bool)
    }

    fn byte_offset(self, component: usize) -> usize {
        // Matrices are column-major. vec3 columns occupy 16 bytes, not 12.
        match self {
            Self::Mat3 => (component / 3) * 16 + (component % 3) * 4,
            _ => component * 4,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ControlValue {
    pub kind: ControlType,
    words: [u32; 16],
}

impl ControlValue {
    fn zero(kind: ControlType) -> Self {
        Self {
            kind,
            words: [0; 16],
        }
    }

    pub fn float(value: f32) -> Self {
        let mut result = Self::zero(ControlType::Float);
        result.words[0] = value.to_bits();
        result
    }

    pub fn from_blob(kind: ControlType, bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != kind.len() * 4 {
            return Err(format!(
                "{} expects exactly {} bytes, received {}",
                kind.name(),
                kind.len() * 4,
                bytes.len()
            ));
        }
        let mut result = Self::zero(kind);
        for (index, bytes) in bytes.chunks_exact(4).enumerate() {
            let word = u32::from_be_bytes(bytes.try_into().expect("four bytes"));
            if (kind.is_float() && !f32::from_bits(word).is_finite())
                || (kind == ControlType::Bool && word > 1)
            {
                return Err("uniform values must be finite; bool must be 0 or 1".to_owned());
            }
            result.words[index] = word;
        }
        Ok(result)
    }

    pub fn blob(&self) -> Vec<u8> {
        self.words[..self.kind.len()]
            .iter()
            .flat_map(|word| word.to_be_bytes())
            .collect()
    }

    pub fn as_float(&self) -> f32 {
        f32::from_bits(self.words[0])
    }

    fn interpolate(&self, target: &Self, fraction: f64) -> Self {
        let mut result = self.clone();
        for index in 0..self.kind.len() {
            // f64 intermediates avoid overflowing on opposite-sign finite f32 endpoints.
            let from = f64::from(f32::from_bits(self.words[index]));
            let to = f64::from(f32::from_bits(target.words[index]));
            result.words[index] = ((from * (1.0 - fraction) + to * fraction) as f32).to_bits();
        }
        result
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interpolation {
    Step,
    Linear,
    Smooth,
}

impl Interpolation {
    pub fn parse(name: &str) -> Result<Self, String> {
        match name {
            "step" => Ok(Self::Step),
            "linear" => Ok(Self::Linear),
            "smooth" => Ok(Self::Smooth),
            _ => Err("interpolation must be step, linear, or smooth".to_owned()),
        }
    }
}

struct Ramp {
    from: ControlValue,
    to: ControlValue,
    start: f64,
    duration: f64,
    mode: Interpolation,
}

struct Control {
    name: String,
    offset: Option<usize>, // None is the backward-compatible binding-0 amount control.
    value: ControlValue,
    ramp: Option<Ramp>,
}

pub struct Controls {
    entries: Vec<Control>,
    bytes: Vec<u8>,
    used_bytes: usize,
}

impl Controls {
    pub fn legacy() -> Self {
        Self {
            entries: vec![Control {
                name: "amount".to_owned(),
                offset: None,
                value: ControlValue::float(0.5),
                ramp: None,
            }],
            bytes: vec![0; CONTROL_BYTES],
            used_bytes: 0,
        }
    }

    pub fn reflect(source: &str, glsl: bool) -> Result<Self, String> {
        let module = if glsl {
            naga::front::glsl::Frontend::default()
                .parse(
                    &naga::front::glsl::Options::from(naga::ShaderStage::Fragment),
                    source,
                )
                .map_err(|error| error.emit_to_string(source))?
        } else {
            naga::front::wgsl::parse_str(source).map_err(|error| error.emit_to_string(source))?
        };
        let mut result = Self::legacy();
        // GPU uniform address spaces cannot contain native bool. An explicit, checked
        // annotation gives a u32 member logical bool semantics on both sides of OSC.
        let mut bools = HashSet::new();
        for line in source.lines() {
            if let Some(annotation) = line.trim().strip_prefix("// @scshader ") {
                let words: Vec<_> = annotation.split_whitespace().collect();
                if words.len() != 2 || words[0] != "bool" || !bools.insert(words[1]) {
                    return Err(
                        "expected a unique '// @scshader bool memberName' annotation".to_owned(),
                    );
                }
            }
        }
        let mut found = false;
        for (_, variable) in module.global_variables.iter() {
            if !variable
                .binding
                .as_ref()
                .is_some_and(|binding| binding.group == 0 && binding.binding == 4)
            {
                continue;
            }
            if found || variable.space != AddressSpace::Uniform {
                return Err("binding 4 must be one uniform struct".to_owned());
            }
            found = true;
            let TypeInner::Struct { members, span } = &module.types[variable.ty].inner else {
                return Err("binding 4 must be a flat uniform struct".to_owned());
            };
            if *span as usize > CONTROL_BYTES || members.len() >= MAX_CONTROLS {
                return Err("user controls exceed 127 members or 16 KiB".to_owned());
            }
            result.used_bytes = *span as usize;
            for member in members {
                let name = member
                    .name
                    .as_ref()
                    .ok_or("control member must have a name")?;
                if name.len() > 64 {
                    return Err("control names must not exceed 64 UTF-8 bytes".to_owned());
                }
                if name == "amount" {
                    return Err("amount is reserved for the binding-0 control".to_owned());
                }
                let mut kind = match module.types[member.ty].inner {
                    TypeInner::Scalar(scalar) if scalar.width == 4 => match scalar.kind {
                        ScalarKind::Float => ControlType::Float,
                        ScalarKind::Sint => ControlType::Int,
                        ScalarKind::Uint => ControlType::Uint,
                        _ => return Err(format!("unsupported uniform type for {name}")),
                    },
                    TypeInner::Vector { size, scalar }
                        if scalar.kind == ScalarKind::Float && scalar.width == 4 =>
                    {
                        match size {
                            naga::VectorSize::Bi => ControlType::Vec2,
                            naga::VectorSize::Tri => ControlType::Vec3,
                            naga::VectorSize::Quad => ControlType::Vec4,
                        }
                    }
                    TypeInner::Matrix {
                        columns,
                        rows,
                        scalar,
                    } if columns == rows
                        && scalar.kind == ScalarKind::Float
                        && scalar.width == 4 =>
                    {
                        match rows {
                            naga::VectorSize::Bi => ControlType::Mat2,
                            naga::VectorSize::Tri => ControlType::Mat3,
                            naga::VectorSize::Quad => ControlType::Mat4,
                        }
                    }
                    _ => {
                        return Err(format!(
                            "{name}: supported controls are 32-bit scalars, float vectors and square matrices; no arrays or nested structs"
                        ));
                    }
                };
                if bools.remove(name.as_str()) {
                    if kind != ControlType::Uint {
                        return Err(format!("bool control {name} must use u32/uint GPU storage"));
                    }
                    kind = ControlType::Bool;
                }
                let offset = member.offset as usize;
                if offset + kind.byte_offset(kind.len() - 1) + 4 > CONTROL_BYTES {
                    return Err("control layout exceeds 16 KiB".to_owned());
                }
                result.entries.push(Control {
                    name: name.clone(),
                    offset: Some(offset),
                    value: ControlValue::zero(kind),
                    ramp: None,
                });
            }
        }
        if !bools.is_empty() {
            return Err("bool annotation names an absent binding-4 member".to_owned());
        }
        Ok(result)
    }

    pub fn reflection(&self) -> impl Iterator<Item = (&str, ControlType)> {
        self.entries
            .iter()
            .map(|entry| (entry.name.as_str(), entry.value.kind))
    }

    pub fn get(&mut self, name: &str, now: f64) -> Result<ControlValue, String> {
        self.tick(now);
        self.entries
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.value.clone())
            .ok_or_else(|| format!("uniform {name:?} does not exist"))
    }

    pub fn set(
        &mut self,
        name: &str,
        value: ControlValue,
        duration: f64,
        mode: Interpolation,
        now: f64,
    ) -> Result<(), String> {
        if !duration.is_finite() || duration < 0.0 {
            return Err("glide duration must be finite and non-negative".to_owned());
        }
        self.tick(now);
        let entry = self
            .entries
            .iter_mut()
            .find(|entry| entry.name == name)
            .ok_or_else(|| format!("uniform {name:?} does not exist"))?;
        if entry.value.kind != value.kind {
            return Err(format!(
                "uniform {name} expects {}, received {}",
                entry.value.kind.name(),
                value.kind.name()
            ));
        }
        if mode != Interpolation::Step && !value.kind.is_float() {
            return Err("integer and boolean uniforms support only step interpolation".to_owned());
        }
        if mode == Interpolation::Step || duration == 0.0 {
            entry.value = value;
            entry.ramp = None;
        } else {
            entry.ramp = Some(Ramp {
                from: entry.value.clone(),
                to: value,
                start: now,
                duration,
                mode,
            });
        }
        Ok(())
    }

    pub fn tick(&mut self, now: f64) {
        for entry in &mut self.entries {
            if let Some(ramp) = &entry.ramp {
                let fraction = ((now - ramp.start) / ramp.duration).clamp(0.0, 1.0);
                let weight = if ramp.mode == Interpolation::Smooth {
                    fraction * fraction * (3.0 - 2.0 * fraction)
                } else {
                    fraction
                };
                entry.value = ramp.from.interpolate(&ramp.to, weight);
                if fraction >= 1.0 {
                    entry.value = ramp.to.clone();
                    entry.ramp = None;
                }
            }
        }
    }

    pub fn preserve(&mut self, previous: &mut Self, now: f64) {
        previous.tick(now);
        for entry in &mut self.entries {
            if let Some(old) = previous
                .entries
                .iter_mut()
                .find(|old| old.name == entry.name && old.value.kind == entry.value.kind)
            {
                entry.value = old.value.clone();
                entry.ramp = old.ramp.take();
            }
        }
    }

    pub fn amount(&self) -> f32 {
        self.entries[0].value.as_float()
    }

    pub fn bytes(&mut self) -> &[u8] {
        for entry in &self.entries {
            if let Some(offset) = entry.offset {
                for index in 0..entry.value.kind.len() {
                    let at = offset + entry.value.kind.byte_offset(index);
                    self.bytes[at..at + 4].copy_from_slice(&entry.value.words[index].to_le_bytes());
                }
            }
        }
        &self.bytes[..self.used_bytes]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn floats(kind: ControlType, values: &[f32]) -> ControlValue {
        ControlValue::from_blob(
            kind,
            &values
                .iter()
                .flat_map(|v| v.to_be_bytes())
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }

    #[test]
    fn typed_blobs_are_exact_and_finite() {
        assert!(ControlValue::from_blob(ControlType::Vec3, &[0; 8]).is_err());
        assert!(ControlValue::from_blob(ControlType::Float, &f32::NAN.to_be_bytes()).is_err());
        assert!(ControlValue::from_blob(ControlType::Bool, &2_u32.to_be_bytes()).is_err());
        for kind in [ControlType::Int, ControlType::Uint] {
            assert_eq!(
                ControlValue::from_blob(kind, &u32::MAX.to_be_bytes())
                    .unwrap()
                    .blob(),
                u32::MAX.to_be_bytes()
            );
        }
    }

    #[test]
    fn reflects_offsets_and_matrix_padding() {
        let mut controls = Controls::reflect("struct C { tint: vec3<f32>, gain: f32, transform: mat3x3<f32>, enabled: u32 }; @group(0) @binding(4) var<uniform> c: C;\n// @scshader bool enabled", false).unwrap();
        controls
            .set(
                "transform",
                floats(ControlType::Mat3, &[1., 2., 3., 4., 5., 6., 7., 8., 9.]),
                0.,
                Interpolation::Step,
                0.,
            )
            .unwrap();
        let bytes = controls.bytes();
        assert_eq!(&bytes[16..20], &1_f32.to_le_bytes());
        assert_eq!(&bytes[28..32], &[0; 4]);
        assert_eq!(&bytes[32..36], &4_f32.to_le_bytes());
        assert_eq!(&bytes[56..60], &9_f32.to_le_bytes());
        assert_eq!(
            controls.reflection().last(),
            Some(("enabled", ControlType::Bool))
        );
    }

    #[test]
    fn rejects_unsupported_layouts_and_annotations() {
        for source in [
            "struct C { a: array<vec4<f32>, 2> }; @group(0) @binding(4) var<uniform> c: C;",
            "// @scshader bool missing",
            "struct C { a: f32 }; @group(0) @binding(4) var<uniform> c: C;\n// @scshader bool a",
        ] {
            assert!(Controls::reflect(source, false).is_err());
        }
    }

    #[test]
    fn glsl_reflects_flat_controls_using_naga() {
        let source = "#version 450\nlayout(set=0,binding=4,std140) uniform Controls { vec4 tint; mat4 transform; int count; uint enabled; } controls;\n// @scshader bool enabled\nlayout(location=0) out vec4 color; void main() { color = controls.tint; }";
        let c = Controls::reflect(source, true).unwrap();
        assert_eq!(
            c.reflection().collect::<Vec<_>>(),
            vec![
                ("amount", ControlType::Float),
                ("tint", ControlType::Vec4),
                ("transform", ControlType::Mat4),
                ("count", ControlType::Int),
                ("enabled", ControlType::Bool)
            ]
        );
    }

    #[test]
    fn mismatched_types_and_integer_glides_leave_values_unchanged() {
        let mut c = Controls::reflect(
            "struct C { count: i32 }; @group(0) @binding(4) var<uniform> c: C;",
            false,
        )
        .unwrap();
        assert!(
            c.set(
                "count",
                ControlValue::float(1.),
                0.,
                Interpolation::Step,
                0.
            )
            .is_err()
        );
        assert!(
            c.set(
                "count",
                ControlValue::from_blob(ControlType::Int, &42_i32.to_be_bytes()).unwrap(),
                1.,
                Interpolation::Linear,
                0.
            )
            .is_err()
        );
        assert_eq!(c.get("count", 1.).unwrap().blob(), [0; 4]);
        assert!(
            c.set(
                "absent",
                ControlValue::float(1.),
                0.,
                Interpolation::Step,
                0.
            )
            .is_err()
        );
    }

    #[test]
    fn changed_layout_preserves_by_name_not_offset_and_resets_new_types() {
        let mut old = Controls::reflect(
            "struct C { gain: f32, count: i32 }; @group(0) @binding(4) var<uniform> c: C;",
            false,
        )
        .unwrap();
        old.set(
            "gain",
            ControlValue::float(0.75),
            0.,
            Interpolation::Step,
            0.,
        )
        .unwrap();
        let mut new = Controls::reflect(
            "struct C { count: u32, gain: f32 }; @group(0) @binding(4) var<uniform> c: C;",
            false,
        )
        .unwrap();
        new.preserve(&mut old, 1.);
        assert_eq!(&new.bytes()[4..8], &0.75_f32.to_le_bytes());
        assert_eq!(new.get("count", 1.).unwrap().blob(), [0; 4]);
    }

    #[test]
    fn smooth_curve_and_extreme_float_endpoints_are_finite() {
        let mut c = Controls::legacy();
        c.set(
            "amount",
            ControlValue::float(0.),
            0.,
            Interpolation::Step,
            0.,
        )
        .unwrap();
        c.set(
            "amount",
            ControlValue::float(1.),
            1.,
            Interpolation::Smooth,
            0.,
        )
        .unwrap();
        assert_eq!(c.get("amount", 0.25).unwrap().as_float(), 0.15625);
        c.set(
            "amount",
            ControlValue::float(-f32::MAX),
            0.,
            Interpolation::Step,
            1.,
        )
        .unwrap();
        c.set(
            "amount",
            ControlValue::float(f32::MAX),
            1.,
            Interpolation::Linear,
            1.,
        )
        .unwrap();
        assert_eq!(c.get("amount", 1.5).unwrap().as_float(), 0.);
        assert_eq!(c.get("amount", 2.).unwrap().as_float(), f32::MAX);
    }

    #[test]
    fn glide_retarget_and_step_cancel() {
        let mut c = Controls::legacy();
        c.set(
            "amount",
            ControlValue::float(1.),
            2.,
            Interpolation::Linear,
            0.,
        )
        .unwrap();
        assert_eq!(c.get("amount", 1.).unwrap().as_float(), 0.75);
        c.set(
            "amount",
            ControlValue::float(0.),
            1.,
            Interpolation::Smooth,
            1.,
        )
        .unwrap();
        assert_eq!(c.get("amount", 1.5).unwrap().as_float(), 0.375);
        c.set(
            "amount",
            ControlValue::float(0.2),
            0.,
            Interpolation::Step,
            1.5,
        )
        .unwrap();
        assert_eq!(c.get("amount", 8.).unwrap().as_float(), 0.2);
    }

    #[test]
    fn reload_preserves_matching_controls_and_live_ramps() {
        let mut old = Controls::legacy();
        old.set(
            "amount",
            ControlValue::float(1.),
            2.,
            Interpolation::Linear,
            0.,
        )
        .unwrap();
        let mut new = Controls::legacy();
        new.preserve(&mut old, 1.);
        assert_eq!(new.get("amount", 1.5).unwrap().as_float(), 0.875);
        assert_eq!(new.get("amount", 2.).unwrap().as_float(), 1.);
    }
}
