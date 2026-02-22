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
        pane_id: Option<u32>,
        tab_index: Option<usize>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub target: Target,
    pub emojis: String,
    pub mode: Mode,
    pub trace: bool,
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

    let mode = mode_from_emojis(&emojis);

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
            if args.contains_key("tab_position") {
                return Err("tab_position is no longer supported; use tab_index".to_string());
            }
            if pane_id.is_some() && tab_index.is_some() {
                return Err(
                    "pane_id and tab_index cannot be set together when target=tab".to_string(),
                );
            }
            Target::Tab { pane_id, tab_index }
        }
        other => return Err(format!("unsupported target: {other}")),
    };

    let trace = args
        .get("trace")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);

    Ok(Command {
        target: command_target,
        emojis,
        mode,
        trace,
    })
}

fn mode_from_emojis(emojis: &str) -> Mode {
    if emojis.starts_with('📌') {
        Mode::Permanent
    } else {
        Mode::Temp
    }
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
