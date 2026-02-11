use std::collections::BTreeMap;

fn replace_colon_emoji(s: &str) -> String {
    let mut result = String::new();
    let mut rest = s;

    while let Some((i, m, n, j)) = rest
        .find(':')
        .map(|i| (i, i + 1))
        .and_then(|(i, m)| rest[m..].find(':').map(|x| (i, m, m + x, m + x + 1)))
    {
        match emojis::get_by_shortcode(&rest[m..n]) {
            Some(emoji) => {
                result.push_str(&rest[..i]);
                result.push_str(emoji.as_str());
                rest = &rest[j..];
            }
            None => {
                result.push_str(&rest[..n]);
                rest = &rest[n..];
            }
        }
    }
    result.push_str(rest);
    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Temp,
    Permanent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Pane {
        pane_id: Option<u32>,
    },
    Tab {
        tab_index: Option<usize>,
        pane_id: Option<u32>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub target: Target,
    pub emojis: String,
    pub mode: Mode,
}

pub fn parse_args(args: &BTreeMap<String, String>) -> Result<Command, String> {
    let target = args
        .get("target")
        .ok_or_else(|| "missing required arg: target".to_string())?;
    let emojis = replace_colon_emoji(
        args.get("emojis")
            .ok_or_else(|| "missing required arg: emojis".to_string())?
            .trim(),
    );
    if emojis.is_empty() {
        return Err("emojis must not be empty".to_string());
    }

    let mode = match args.get("mode").map(|m| m.as_str()).unwrap_or("temp") {
        "temp" => Mode::Temp,
        "permanent" => Mode::Permanent,
        other => return Err(format!("unsupported mode: {other}")),
    };

    let command_target = match target.as_str() {
        "pane" => {
            let pane_id = parse_optional_u32(args.get("pane_id"), "pane_id")?;
            if args.contains_key("tab_index") {
                return Err("tab_index is not allowed when target=pane".to_string());
            }
            Target::Pane { pane_id }
        }
        "tab" => {
            let pane_id = parse_optional_u32(args.get("pane_id"), "pane_id")?;
            let tab_index = parse_optional_usize(args.get("tab_index"), "tab_index")?;
            if pane_id.is_some() && tab_index.is_some() {
                return Err(
                    "pane_id and tab_index cannot be set together when target=tab".to_string(),
                );
            }
            Target::Tab { tab_index, pane_id }
        }
        other => return Err(format!("unsupported target: {other}")),
    };

    Ok(Command {
        target: command_target,
        emojis,
        mode,
    })
}

fn parse_optional_u32(value: Option<&String>, key: &str) -> Result<Option<u32>, String> {
    match value {
        None => Ok(None),
        Some(v) => v
            .parse::<u32>()
            .map(Some)
            .map_err(|_| format!("{key} must be an unsigned integer")),
    }
}

fn parse_optional_usize(value: Option<&String>, key: &str) -> Result<Option<usize>, String> {
    match value {
        None => Ok(None),
        Some(v) => v
            .parse::<usize>()
            .map(Some)
            .map_err(|_| format!("{key} must be an unsigned integer")),
    }
}

#[cfg(test)]
mod replace_tests {
    use super::*;

    #[test]
    fn replace_colon_emoji_with_shortcode() {
        assert_eq!(replace_colon_emoji("launch :rocket:"), "launch 🚀");
    }

    #[test]
    fn replace_colon_emoji_with_multiple() {
        assert_eq!(replace_colon_emoji(":rocket::book:"), "🚀📚");
    }

    #[test]
    fn replace_colon_emoji_with_unknown() {
        assert_eq!(replace_colon_emoji(":unknown:"), ":unknown:");
    }

    #[test]
    fn replace_colon_emoji_with_mixed() {
        assert_eq!(replace_colon_emoji("🚀:rocket:"), "🚀🚀");
    }

    #[test]
    fn replace_colon_emoji_no_change() {
        assert_eq!(replace_colon_emoji("launch nothing"), "launch nothing");
    }

    #[test]
    fn replace_colon_emoji_edge_cases() {
        assert_eq!(replace_colon_emoji(":maybe:rocket:"), ":maybe🚀");
        assert_eq!(replace_colon_emoji(":rocket::rocket:"), "🚀🚀");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn parse_pane_temp_command() {
        let args = map(&[("target", "pane"), ("emojis", "🚀"), ("mode", "temp")]);
        let cmd = parse_args(&args).unwrap();
        assert_eq!(
            cmd,
            Command {
                target: Target::Pane { pane_id: None },
                emojis: "🚀".to_string(),
                mode: Mode::Temp,
            }
        );
    }

    #[test]
    fn parse_tab_command_with_pane_id() {
        let args = map(&[
            ("target", "tab"),
            ("pane_id", "77"),
            ("emojis", "📚"),
            ("mode", "permanent"),
        ]);
        let cmd = parse_args(&args).unwrap();
        assert_eq!(
            cmd,
            Command {
                target: Target::Tab {
                    tab_index: None,
                    pane_id: Some(77),
                },
                emojis: "📚".to_string(),
                mode: Mode::Permanent,
            }
        );
    }

    #[test]
    fn parse_tab_with_both_ids_fails() {
        let args = map(&[
            ("target", "tab"),
            ("tab_index", "1"),
            ("pane_id", "2"),
            ("emojis", "✅"),
        ]);
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn parse_command_with_colon_emoji() {
        let args = map(&[("target", "pane"), ("emojis", ":rocket:"), ("mode", "temp")]);
        let cmd = parse_args(&args).unwrap();
        assert_eq!(
            cmd,
            Command {
                target: Target::Pane { pane_id: None },
                emojis: "🚀".to_string(),
                mode: Mode::Temp,
            }
        );
    }

    #[test]
    fn parse_command_with_multiple_colon_emojis() {
        let args = map(&[
            ("target", "pane"),
            ("emojis", ":rocket::book:"),
            ("mode", "permanent"),
        ]);
        let cmd = parse_args(&args).unwrap();
        assert_eq!(
            cmd,
            Command {
                target: Target::Pane { pane_id: None },
                emojis: "🚀📚".to_string(),
                mode: Mode::Permanent,
            }
        );
    }

    #[test]
    fn parse_command_with_unknown_colon_emoji() {
        let args = map(&[
            ("target", "pane"),
            ("emojis", ":unknown:"),
            ("mode", "temp"),
        ]);
        let cmd = parse_args(&args).unwrap();
        assert_eq!(
            cmd,
            Command {
                target: Target::Pane { pane_id: None },
                emojis: ":unknown:".to_string(),
                mode: Mode::Temp,
            }
        );
    }

    #[test]
    fn parse_command_with_mixed_emojis() {
        let args = map(&[
            ("target", "pane"),
            ("emojis", "🚀:rocket:"),
            ("mode", "temp"),
        ]);
        let cmd = parse_args(&args).unwrap();
        assert_eq!(
            cmd,
            Command {
                target: Target::Pane { pane_id: None },
                emojis: "🚀🚀".to_string(),
                mode: Mode::Temp,
            }
        );
    }
}
