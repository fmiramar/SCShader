//! Bounded native numeric sources, evaluated at the common Hydra frame time.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub id: u32,
    pub kind: u8,
    pub values: Vec<f32>,
}
impl Parameter {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=256).contains(&self.id)
            || self.values.iter().any(|v| !v.is_finite() || v.abs() > 1e6)
        {
            return Err("invalid numeric source".into());
        }
        let n = self.values.len();
        if !matches!((self.kind, n), (0, 1) | (1, 2) | (2, 4) | (3, 5..=68)) {
            return Err("numeric source shape".into());
        }
        if self.kind == 3
            && (self.values[2] < 0. || self.values[2] > 1. || ![0., 1.].contains(&self.values[3]))
        {
            return Err("invalid smoothing/easing".into());
        }
        Ok(())
    }
    #[allow(dead_code)] // Some compiler-only fixtures validate without rendering.
    pub fn value(&self, time: f32) -> f32 {
        let v = &self.values;
        match self.kind {
            0 => v[0],
            1 => time * v[0] + v[1],
            2 => v[2] + (v[3] - v[2]) * (0.5 + 0.5 * (time * v[0] + v[1]).sin()),
            3 => {
                let data = &v[4..];
                let speed = if v[0] == 0. { 1. } else { v[0] };
                let index = time * speed * 0.5 + v[1]; // Fixed initial BPM 30; Step 16 adds transport controls.
                if v[2] == 0. {
                    return data[(index.floor() as i64).rem_euclid(data.len() as i64) as usize];
                }
                let shifted = index - v[2] / 2.;
                let i = shifted.floor() as i64;
                let mut t = (shifted.rem_euclid(1.) / v[2]).min(1.);
                if v[3] == 1. {
                    t = if t < 0.5 {
                        2. * t * t
                    } else {
                        -1. + (4. - 2. * t) * t
                    };
                }
                data[i.rem_euclid(data.len() as i64) as usize] * (1. - t)
                    + data[(i + 1).rem_euclid(data.len() as i64) as usize] * t
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn centered_sequence_and_clock() {
        let p = Parameter {
            id: 1,
            kind: 3,
            values: vec![1., 0., 1., 0., 0., 1.],
        };
        assert_eq!(p.value(0.), 0.5);
        assert_eq!(p.value(1.), 0.);
        assert_eq!(p.value(2.), 0.5);
        assert_eq!(p.value(3.), 1.);
        assert_eq!(p.value(4.), 0.5);
        assert_eq!(p.value(-1.), 1.);
        let q = Parameter {
            id: 1,
            kind: 3,
            values: vec![1., 0., 1., 1., 0., 1.],
        };
        assert_eq!(q.value(1.5), 0.125);
        let p = Parameter {
            id: 1,
            kind: 1,
            values: vec![2., 3.],
        };
        assert_eq!(p.value(4.), 11.);
    }
    #[test]
    fn bounds_and_invalid_sources() {
        for p in [
            Parameter {
                id: 0,
                kind: 0,
                values: vec![0.],
            },
            Parameter {
                id: 1,
                kind: 0,
                values: vec![f32::NAN],
            },
            Parameter {
                id: 1,
                kind: 3,
                values: vec![1., 0., 2., 0., 0.],
            },
        ] {
            assert!(p.validate().is_err());
        }
    }
}
