//! Key names to virtual-key codes.

#[derive(Clone, Debug)]
pub struct KeySpec {
    pub vk: u16,
    pub label: String,
    pub extended: bool,
}

/// (name, virtual key, needs the extended-key flag)
const TABLE: &[(&str, u16, bool)] = &[
    ("space", 0x20, false),
    ("enter", 0x0D, false),
    ("tab", 0x09, false),
    ("shift", 0x10, false),
    ("ctrl", 0x11, false),
    ("alt", 0x12, false),
    ("esc", 0x1B, false),
    ("backspace", 0x08, false),
    ("up", 0x26, true),
    ("down", 0x28, true),
    ("left", 0x25, true),
    ("right", 0x27, true),
    ("insert", 0x2D, true),
    ("delete", 0x2E, true),
    ("home", 0x24, true),
    ("end", 0x23, true),
    ("pageup", 0x21, true),
    ("pagedown", 0x22, true),
    ("e", 0x45, false),
    ("q", 0x51, false),
    ("f", 0x46, false),
    ("w", 0x57, false),
    ("a", 0x41, false),
    ("s", 0x53, false),
    ("d", 0x44, false),
];

pub fn parse(raw: &str) -> Result<KeySpec, String> {
    let name = raw.trim().to_ascii_lowercase();
    if name.is_empty() {
        return Err("empty key name".to_string());
    }

    if let Some((label, vk, extended)) = TABLE.iter().find(|(label, _, _)| *label == name) {
        return Ok(KeySpec {
            vk: *vk,
            label: (*label).to_string(),
            extended: *extended,
        });
    }

    // Single letters and digits map straight onto their virtual-key code.
    let mut chars = name.chars();
    if let (Some(c), None) = (chars.next(), chars.next()) {
        if c.is_ascii_alphanumeric() {
            return Ok(KeySpec {
                vk: c.to_ascii_uppercase() as u16,
                label: name,
                extended: false,
            });
        }
    }

    Err(format!(
        "unknown key '{raw}' (try: space, enter, shift, w, a, s, d, up, down, ...)"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_names_and_letters() {
        assert_eq!(parse("SPACE").unwrap().vk, 0x20);
        assert_eq!(parse("w").unwrap().vk, 0x57);
        assert_eq!(parse("9").unwrap().vk, 0x39);
        assert_eq!(parse("up").unwrap().vk, 0x26);
        assert!(parse("up").unwrap().extended);
        assert!(parse("nope").is_err());
    }
}
