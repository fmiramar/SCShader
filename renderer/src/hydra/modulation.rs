//! Independently written spatial modulation; pinned behavior in MODULATION.md.
use super::{
    compiler::literal,
    ir::{Argument, Node},
    registry,
};
use std::fmt::Write;
#[derive(Debug)]
enum Kind {
    Repeat,
    RepeatX,
    RepeatY,
    Kaleid,
    ScrollX,
    ScrollY,
    Displace,
    Scale,
    Pixelate,
    Rotate,
    Hue,
}
#[derive(Debug)]
pub struct Modulation {
    kind: Kind,
    receiver: u32,
    other: u32,
    args: Vec<f32>,
}
impl Modulation {
    pub fn lower(node: &Node) -> Result<Self, String> {
        let spec = registry::spec(&node.name).ok_or("unknown modulation")?;
        if spec.category != 4 || node.category != 4 || node.arguments.len() != spec.args.len() {
            return Err("modulation signature mismatch".into());
        }
        let Argument::Expression(other) = node.arguments[0] else {
            return Err("modulation needs an expression".into());
        };
        let args = node.arguments[1..]
            .iter()
            .map(|arg| match arg {
                Argument::Float(value) if value.is_finite() => Ok(*value),
                _ => Err("modulation requires finite scalar literals".to_owned()),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let kind = match node.name.as_str() {
            "modulateRepeat" => Kind::Repeat,
            "modulateRepeatX" => Kind::RepeatX,
            "modulateRepeatY" => Kind::RepeatY,
            "modulateKaleid" => Kind::Kaleid,
            "modulateScrollX" => Kind::ScrollX,
            "modulateScrollY" => Kind::ScrollY,
            "modulate" => Kind::Displace,
            "modulateScale" => Kind::Scale,
            "modulatePixelate" => Kind::Pixelate,
            "modulateRotate" => Kind::Rotate,
            "modulateHue" => Kind::Hue,
            _ => return Err("unknown modulation".into()),
        };
        if matches!(kind, Kind::Kaleid) && args[0] == 0.
            || matches!(kind, Kind::Scale | Kind::Pixelate) && args[0] == 0. && args[1] == 0.
        {
            return Err("constant zero divisor in modulation".into());
        }
        Ok(Self {
            kind,
            receiver: node.receiver,
            other,
            args,
        })
    }
    pub fn emit(&self, source: &mut String) {
        writeln!(source, "let m=n{}(uv);", self.other).unwrap();
        for (i, value) in self.args.iter().enumerate() {
            writeln!(source, "let a{i}={};", literal(*value)).unwrap();
        }
        let coordinates = match self.kind {
            Kind::Displace => "let warped=uv+m.rg*a0;",
            Kind::Scale => {
                "let extent=vec2<f32>(a1)+m.rg*a0; let warped=(uv-vec2<f32>(0.5))*(vec2<f32>(1.0)/extent)+vec2<f32>(0.5);"
            }
            Kind::Pixelate => {
                "let size=vec2<f32>(a1)+m.rg*a0; let warped=(floor(uv*size)+vec2<f32>(0.5))/size;"
            }
            Kind::Rotate => {
                "let angle=a1+m.r*a0; let p=uv-vec2<f32>(0.5); let warped=vec2<f32>(cos(angle)*p.x+sin(angle)*p.y,-sin(angle)*p.x+cos(angle)*p.y)+vec2<f32>(0.5);"
            }
            Kind::Hue => "let warped=uv+vec2<f32>(m.g-m.r,m.b-m.g)*a0*hydra.resolution.zw;",
            Kind::Kaleid => {
                "let p=uv-vec2<f32>(0.5); let sector=6.2832/a0; let angle=abs(h_mod(atan2(p.y,p.x),sector)-sector*0.5); let warped=(m.r+length(p))*vec2<f32>(cos(angle),sin(angle));"
            }
            Kind::ScrollX => "let warped=fract(uv+vec2<f32>(m.r*a0+hydra.clock.x*a1,0.0));",
            Kind::ScrollY => "let warped=fract(uv+vec2<f32>(0.0,m.r*a0+hydra.clock.x*a1));",
            Kind::Repeat => {
                "var p=uv*vec2<f32>(a0,a1); p.x+=select(0.0,1.0,h_mod(p.y,2.0)>=1.0)+m.r*a2; p.y+=select(0.0,1.0,h_mod(p.x,2.0)>=1.0)+m.g*a3; let warped=fract(p);"
            }
            Kind::RepeatX => {
                "var p=uv*vec2<f32>(a0,1.0); p.y+=select(0.0,1.0,h_mod(p.x,2.0)>=1.0)+m.r*a1; let warped=fract(p);"
            }
            Kind::RepeatY => {
                "var p=uv*vec2<f32>(a0,1.0); p.x+=select(0.0,1.0,h_mod(p.y,2.0)>=1.0)+m.r*a1; let warped=fract(p);"
            }
        };
        source.push_str(coordinates);
        writeln!(source, "return n{}(warped);", self.receiver).unwrap();
    }
}
