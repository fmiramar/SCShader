//! Bounded native Hydra IR. No shader text or executable code crosses this parser.
use std::collections::HashMap;

pub const MAX_BYTES: usize = 65_536;
pub const MAX_NODES: usize = 256;

#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    Float(f32),
    Parameter(u32),
    Output(u8),
    Vector([f32; 4]),
    Expression(u32),
}
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: u32,
    pub name: String,
    pub category: u8,
    pub receiver: u32,
    pub arguments: Vec<Argument>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub parameters: Vec<super::parameters::Parameter>,
    pub nodes: Vec<Node>,
    pub root: u32,
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], String> {
        let end = self.offset.checked_add(count).ok_or("IR length overflow")?;
        let result = self.bytes.get(self.offset..end).ok_or("truncated IR")?;
        self.offset = end;
        Ok(result)
    }
    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn short(&mut self) -> Result<u16, String> {
        Ok(u16::from_be_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn word(&mut self) -> Result<u32, String> {
        Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }
}

impl Program {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_BYTES {
            return Err("IR exceeds 64 KiB".into());
        }
        let mut input = Reader { bytes, offset: 0 };
        if input.take(4)? != b"HYIR" || input.short()? != 1 {
            return Err("unsupported IR version".into());
        }
        let minor = input.short()?;
        if minor > 1 {
            return Err("unsupported IR minor".into());
        }
        let count = input.word()? as usize;
        if count == 0 || count > MAX_NODES {
            return Err("IR node count outside 1..256".into());
        }
        let parameter_count = input.word()? as usize;
        if parameter_count > 256 || (minor == 0 && parameter_count != 0) {
            return Err("parameter limit".into());
        }
        let root = input.word()?;
        let mut parameters = Vec::new();
        for _ in 0..parameter_count {
            let id = input.word()?;
            let kind = input.byte()?;
            let count = input.byte()? as usize;
            if count > 68 {
                return Err("numeric source size".into());
            }
            let mut values = Vec::new();
            for _ in 0..count {
                values.push(f32::from_bits(input.word()?));
            }
            let p = super::parameters::Parameter { id, kind, values };
            p.validate()?;
            if parameters
                .iter()
                .any(|q: &super::parameters::Parameter| q.id == id)
            {
                return Err("duplicate parameter".into());
            }
            parameters.push(p);
        }
        let mut nodes = Vec::with_capacity(count);
        let mut known: HashMap<u32, (usize, usize)> = HashMap::new();
        for _ in 0..count {
            let id = input.word()?;
            if id == 0 || id > i32::MAX as u32 || known.contains_key(&id) {
                return Err("invalid or duplicate node ID".into());
            }
            let category = input.byte()?;
            let argc = usize::from(input.byte()?);
            let length = usize::from(input.short()?);
            if length == 0 || length > 48 || argc > 16 {
                return Err("operation metadata exceeds limits".into());
            }
            let name = std::str::from_utf8(input.take(length)?)
                .map_err(|_| "operation name must be ASCII")?
                .to_owned();
            if !name.is_ascii() {
                return Err("operation name must be ASCII".into());
            }
            let receiver = input.word()?;
            use super::registry::ValueType;
            let spec = super::registry::spec(&name)
                .ok_or_else(|| format!("unsupported operation {name}"))?;
            let (expected_category, types) = (spec.category, spec.args);
            if category != expected_category || argc != types.len() {
                return Err("operation category/arity mismatch".into());
            }
            let mut dependencies = Vec::new();
            if category == 0 {
                if receiver != 0 {
                    return Err("source must not have a receiver".into());
                }
            } else {
                dependencies.push(receiver);
            }
            let mut arguments = Vec::with_capacity(argc);
            for expected_type in types {
                match input.byte()? {
                    1 if *expected_type == ValueType::Float => {
                        let id = input.word()?;
                        if !parameters.iter().any(|p| p.id == id) {
                            return Err("unknown parameter".into());
                        }
                        arguments.push(Argument::Parameter(id));
                    }
                    4 if *expected_type == ValueType::Output => {
                        let slot = input.byte()?;
                        if slot > 3 {
                            return Err("invalid output slot".into());
                        }
                        arguments.push(Argument::Output(slot));
                    }
                    0 if *expected_type == ValueType::Float => {
                        let value = f32::from_bits(input.word()?);
                        if !value.is_finite() {
                            return Err("nonfinite argument".into());
                        }
                        arguments.push(Argument::Float(value));
                    }
                    2 if *expected_type == ValueType::Expression => {
                        let reference = input.word()?;
                        dependencies.push(reference);
                        arguments.push(Argument::Expression(reference));
                    }
                    5 if *expected_type == ValueType::Vector => {
                        let mut values = [0.; 4];
                        for value in &mut values {
                            *value = f32::from_bits(input.word()?);
                            if !value.is_finite() {
                                return Err("nonfinite vector argument".into());
                            }
                        }
                        arguments.push(Argument::Vector(values));
                    }
                    _ => return Err("argument type mismatch or unsupported tag".into()),
                }
            }
            let mut depth = 1;
            let mut expanded = 1usize;
            for dependency in dependencies {
                let &(child_depth, child_cost) = known
                    .get(&dependency)
                    .ok_or("reference must name an earlier node")?;
                depth = depth.max(child_depth + 1);
                expanded = expanded
                    .checked_add(child_cost)
                    .ok_or("expansion overflow")?;
            }
            if depth > 64 || expanded > 4096 {
                return Err("IR depth or expansion limit exceeded".into());
            }
            known.insert(id, (depth, expanded));
            nodes.push(Node {
                id,
                name,
                category,
                receiver,
                arguments,
            });
        }
        if input.offset != bytes.len() || !known.contains_key(&root) {
            return Err("trailing bytes or missing root".into());
        }
        let by_id: HashMap<_, _> = nodes.iter().map(|node| (node.id, node)).collect();
        let mut stack = vec![root];
        let mut reachable = std::collections::HashSet::new();
        while let Some(id) = stack.pop() {
            if !reachable.insert(id) {
                continue;
            }
            let node = by_id[&id];
            if node.receiver != 0 {
                stack.push(node.receiver);
            }
            for arg in &node.arguments {
                if let Argument::Expression(id) = arg {
                    stack.push(*id);
                }
            }
        }
        if reachable.len() != nodes.len() {
            return Err("unreachable IR node".into());
        }
        Ok(Self {
            nodes,
            root,
            parameters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source() -> Vec<u8> {
        let mut bytes = b"HYIR\0\x01\0\0".to_vec();
        for n in [1u32, 0, 1, 1] {
            bytes.extend(n.to_be_bytes());
        }
        bytes.extend([0, 3, 0, 3]);
        bytes.extend(b"osc");
        bytes.extend(0u32.to_be_bytes());
        for f in [10f32, 0.1, 0.] {
            bytes.push(0);
            bytes.extend(f.to_be_bytes());
        }
        bytes
    }
    #[test]
    fn valid_and_truncated_ir() {
        let bytes = source();
        assert_eq!(Program::decode(&bytes).unwrap().root, 1);
        for end in 0..bytes.len() {
            assert!(Program::decode(&bytes[..end]).is_err());
        }
        let mut trailing = bytes;
        trailing.push(0);
        assert!(Program::decode(&trailing).is_err());
    }
    #[test]
    fn nonfinite_and_bad_references() {
        let mut bytes = source();
        let end = bytes.len();
        bytes[end - 4..].copy_from_slice(&f32::NAN.to_be_bytes());
        assert!(Program::decode(&bytes).is_err());
        let mut bytes = source();
        bytes[16..20].copy_from_slice(&2u32.to_be_bytes());
        assert!(Program::decode(&bytes).is_err());
    }
}
