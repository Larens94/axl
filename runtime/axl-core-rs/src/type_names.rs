pub const MAX_TYPE_DEPTH: usize = 16;
const SCALARS: &[&str] = &["int", "string", "bool"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeError {
    Invalid(String),
    TooDeep,
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeError::Invalid(name) => write!(f, "invalid type '{name}'"),
            TypeError::TooDeep => write!(f, "type nesting is too deep ({MAX_TYPE_DEPTH})"),
        }
    }
}

impl std::error::Error for TypeError {}

pub fn validate_type_name(type_name: &str) -> Result<(), TypeError> {
    let pos = parse_type(type_name, 0, 0, true)?;
    if pos != type_name.len() {
        return Err(TypeError::Invalid(type_name.into()));
    }
    Ok(())
}

pub fn is_known_type_name(type_name: &str) -> bool {
    parse_type(type_name, 0, 0, false).ok() == Some(type_name.len())
}

pub fn split_map_type(type_name: &str) -> Option<(&str, &str)> {
    if !type_name.starts_with("map<") {
        return None;
    }
    let mut depth = 0usize;
    let bytes = type_name.as_bytes();
    for pos in 4..type_name.len().saturating_sub(1) {
        match bytes[pos] {
            b'<' => depth += 1,
            b'>' => depth -= 1,
            b',' if depth == 0 && type_name.ends_with('>') => {
                return Some((&type_name[4..pos], &type_name[pos + 1..type_name.len() - 1]));
            }
            _ => {}
        }
    }
    None
}

pub fn split_list_type(type_name: &str) -> Option<&str> {
    if type_name.starts_with("list<") && type_name.ends_with('>') {
        Some(&type_name[5..type_name.len() - 1])
    } else {
        None
    }
}

fn parse_type(source: &str, position: usize, depth: usize, allow_unknown: bool) -> Result<usize, TypeError> {
    for scalar in SCALARS {
        if source[position..].starts_with(scalar) {
            return Ok(position + scalar.len());
        }
    }
    if depth >= MAX_TYPE_DEPTH {
        return Err(TypeError::TooDeep);
    }
    if source[position..].starts_with("list<") {
        let pos = parse_type(source, position + 5, depth + 1, allow_unknown)?;
        if pos >= source.len() || source.as_bytes()[pos] != b'>' {
            return Err(TypeError::Invalid(source.into()));
        }
        return Ok(pos + 1);
    }
    if source[position..].starts_with("map<") {
        let pos = parse_type(source, position + 4, depth + 1, allow_unknown)?;
        if pos >= source.len() || source.as_bytes()[pos] != b',' {
            return Err(TypeError::Invalid(source.into()));
        }
        let pos = parse_type(source, pos + 1, depth + 1, allow_unknown)?;
        if pos >= source.len() || source.as_bytes()[pos] != b'>' {
            return Err(TypeError::Invalid(source.into()));
        }
        return Ok(pos + 1);
    }
    if allow_unknown && position < source.len() {
        let bytes = source.as_bytes();
        if bytes[position].is_ascii_alphabetic() || bytes[position] == b'_' {
            let mut end = position + 1;
            while end < source.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            return Ok(end);
        }
    }
    Err(TypeError::Invalid(source.into()))
}
