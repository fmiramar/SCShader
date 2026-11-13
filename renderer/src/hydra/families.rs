//! Typed lowering for the additional procedural source, coordinate and color families.
use super::{
    compiler::literal,
    ir::{Argument, Node},
    registry::{self, ValueType},
};
use std::fmt::Write;
#[derive(Debug)]
pub enum Kind {
    Noise,
    Voronoi,
    Shape,
    Pixelate,
    Posterize,
    Shift,
    Repeat,
    RepeatX,
    RepeatY,
    Kaleid,
    Scroll,
    ScrollX,
    ScrollY,
    Invert,
    Contrast,
    Brightness,
    Luma,
    Thresh,
    Saturate,
    Hue,
    Colorama,
    Sum,
    R,
    G,
    B,
    A,
}
#[derive(Debug)]
pub struct Builtin {
    kind: Kind,
    receiver: u32,
    category: u8,
    args: Vec<f32>,
}
impl Builtin {
    pub fn lower(node: &Node) -> Result<Self, String> {
        let spec = registry::spec(&node.name).ok_or("unsupported compiler operation")?;
        if node.arguments.len() != spec.args.len() || node.category != spec.category {
            return Err("builtin signature mismatch".into());
        }
        let kind = match node.name.as_str() {
            "noise" => Kind::Noise,
            "voronoi" => Kind::Voronoi,
            "shape" => Kind::Shape,
            "pixelate" => Kind::Pixelate,
            "posterize" => Kind::Posterize,
            "shift" => Kind::Shift,
            "repeat" => Kind::Repeat,
            "repeatX" => Kind::RepeatX,
            "repeatY" => Kind::RepeatY,
            "kaleid" => Kind::Kaleid,
            "scroll" => Kind::Scroll,
            "scrollX" => Kind::ScrollX,
            "scrollY" => Kind::ScrollY,
            "invert" => Kind::Invert,
            "contrast" => Kind::Contrast,
            "brightness" => Kind::Brightness,
            "luma" => Kind::Luma,
            "thresh" => Kind::Thresh,
            "saturate" => Kind::Saturate,
            "hue" => Kind::Hue,
            "colorama" => Kind::Colorama,
            "sum" => Kind::Sum,
            "r" => Kind::R,
            "g" => Kind::G,
            "b" => Kind::B,
            "a" => Kind::A,
            _ => return Err("unsupported builtin".into()),
        };
        let mut args = Vec::new();
        for (arg, kind) in node.arguments.iter().zip(spec.args) {
            match (arg, kind) {
                (Argument::Float(v), ValueType::Float) => args.push(*v),
                (Argument::Vector(v), ValueType::Vector) => args.extend(v),
                _ => return Err("builtin argument type mismatch".into()),
            }
        }
        if args.iter().any(|v| !v.is_finite()) {
            return Err("nonfinite builtin argument".into());
        }
        if matches!(kind, Kind::Shape | Kind::Kaleid) && args[0] == 0.
            || matches!(kind, Kind::Pixelate | Kind::Posterize) && (args[0] == 0. || args[1] == 0.)
        {
            return Err("zero divisor in procedural operation".into());
        }
        if matches!(kind, Kind::Luma | Kind::Thresh) && args[1] < 0. {
            return Err("threshold tolerance must be nonnegative".into());
        }
        Ok(Self {
            kind,
            receiver: node.receiver,
            category: node.category,
            args,
        })
    }
    pub fn emit(&self, source: &mut String) {
        for (i, v) in self.args.iter().enumerate() {
            writeln!(source, "let a{i} = {};", literal(*v)).unwrap();
        }
        if self.category == 2 {
            writeln!(source, "let c = n{}(uv);", self.receiver).unwrap();
        }
        let r = self.receiver;
        let body=match self.kind {
            Kind::Noise=>"let v=h_simplex3(vec3<f32>(uv*a0,hydra.clock.x*a1)); return vec4<f32>(vec3<f32>(v),1.0);".into(),
            Kind::Voronoi=>"let v=h_voronoi(uv,a0,a1,a2,hydra.clock.x);return vec4<f32>(vec3<f32>(v),1.0);".into(),
            Kind::Shape=>"let p=uv*2.0-vec2<f32>(1.0);let angle=atan2(p.x,p.y)+3.1416;let sector=6.2832/a0;let distance=cos(floor(0.5+angle/sector)*sector-angle)*length(p);let v=1.0-smoothstep(a1,a1+a2+0.0000001,distance);return vec4<f32>(vec3<f32>(v),1.0);".into(),
            Kind::Pixelate=>format!("let size=vec2<f32>(a0,a1);return n{r}((floor(uv*size)+vec2<f32>(0.5))/size);"),
            Kind::Repeat=>format!("var p=uv*vec2<f32>(a0,a1);p.x+=select(0.0,a2,h_mod(p.y,2.0)>=1.0);p.y+=select(0.0,a3,h_mod(p.x,2.0)>=1.0);return n{r}(fract(p));"),
            Kind::RepeatX=>format!("var p=uv*vec2<f32>(a0,1.0);p.y+=select(0.0,a1,h_mod(p.x,2.0)>=1.0);return n{r}(fract(p));"),
            Kind::RepeatY=>format!("var p=uv*vec2<f32>(1.0,a0);p.x+=select(0.0,a1,h_mod(p.y,2.0)>=1.0);return n{r}(fract(p));"),
            Kind::Kaleid=>format!("let p=uv-vec2<f32>(0.5);let sector=6.2832/a0;let angle=abs(h_mod(atan2(p.y,p.x),sector)-sector*0.5);return n{r}(length(p)*vec2<f32>(cos(angle),sin(angle)));"),
            Kind::Scroll=>format!("return n{r}(fract(uv+(vec2<f32>(a0,a1)+hydra.clock.x*vec2<f32>(a2,a3))));"),
            Kind::ScrollX=>format!("return n{r}(fract(uv+vec2<f32>(a0+hydra.clock.x*a1,0.0)));"),
            Kind::ScrollY=>format!("return n{r}(fract(uv+vec2<f32>(0.0,a0+hydra.clock.x*a1)));"),
            Kind::Posterize=>"let rgb=pow(floor(pow(c.rgb,vec3<f32>(a1))*a0)/a0,vec3<f32>(1.0/a1));return vec4<f32>(rgb,c.a);".into(),
            Kind::Shift=>"return c+fract(vec4<f32>(a0,a1,a2,a3));".into(),
            Kind::Invert=>"return vec4<f32>((vec3<f32>(1.0)-c.rgb)*a0+c.rgb*(1.0-a0),c.a);".into(),
            Kind::Contrast=>"return vec4<f32>((c.rgb-vec3<f32>(0.5))*a0+vec3<f32>(0.5),c.a);".into(),
            Kind::Brightness=>"return vec4<f32>(c.rgb+vec3<f32>(a0),c.a);".into(),
            Kind::Luma=>"let v=smoothstep(a0-(a1+0.0000001),a0+(a1+0.0000001),h_luminance(c.rgb));return vec4<f32>(c.rgb*v,v);".into(),
            Kind::Thresh=>"let v=smoothstep(a0-(a1+0.0000001),a0+(a1+0.0000001),h_luminance(c.rgb));return vec4<f32>(vec3<f32>(v),c.a);".into(),
            Kind::Saturate=>"return vec4<f32>(mix(vec3<f32>(h_luminance(c.rgb)),c.rgb,a0),c.a);".into(),
            Kind::Hue=>"var hsv=h_rgb_hsv(c.rgb);hsv.x+=a0;return vec4<f32>(h_hsv_rgb(hsv),c.a);".into(),
            Kind::Colorama=>"return vec4<f32>(fract(h_hsv_rgb(h_rgb_hsv(c.rgb)+vec3<f32>(a0))),c.a);".into(),
            Kind::Sum=>"let v=c*vec4<f32>(a0,a1,a2,a3);return vec4<f32>(vec3<f32>(v.r+v.g+v.b+v.a),c.a);".into(),
            Kind::R=>"return vec4<f32>(c.r*a0+a1);".into(),Kind::G=>"return vec4<f32>(c.g*a0+a1);".into(),Kind::B=>"return vec4<f32>(c.b*a0+a1);".into(),Kind::A=>"return vec4<f32>(c.a*a0+a1);".into(),
        };
        source.push_str(&body);
        source.push('\n');
    }
}
