use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub tags: Vec<String>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip)]
    pub bypass_tag_filter: bool,
    #[serde(skip)]
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct TagsConfig {
    pub tags: Vec<String>,
}

fn decode_entities(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '&' {
            let mut entity = String::new();
            let mut found_end = false;
            while let Some(&c) = chars.peek() {
                chars.next();
                if c == ';' {
                    found_end = true;
                    break;
                }
                entity.push(c);
            }
            if found_end {
                match entity.as_str() {
                    "amp" => result.push('&'),
                    "lt" => result.push('<'),
                    "gt" => result.push('>'),
                    "quot" => result.push('"'),
                    "apos" | "#39" => result.push('\''),
                    "nbsp" => result.push(' '),
                    "mdash" | "#8212" => result.push('\u{2014}'),
                    "ndash" | "#8211" => result.push('\u{2013}'),
                    "lsquo" | "#8216" => result.push('\u{2018}'),
                    "rsquo" | "#8217" => result.push('\u{2019}'),
                    "ldquo" | "#8220" => result.push('\u{201C}'),
                    "rdquo" | "#8221" => result.push('\u{201D}'),
                    "hellip" | "#8230" => result.push('\u{2026}'),
                    e if e.starts_with('#') && e.starts_with("#x") => {
                        let hex = &e[2..];
                        if let Ok(cp) = u32::from_str_radix(hex, 16) {
                            if let Some(c) = char::from_u32(cp) {
                                result.push(c);
                            } else {
                                result.push('&');
                                result.push_str(e);
                                result.push(';');
                            }
                        } else {
                            result.push('&');
                            result.push_str(e);
                            result.push(';');
                        }
                    }
                    e if e.starts_with('#') => {
                        let num = &e[1..];
                        if let Ok(cp) = num.parse::<u32>() {
                            if let Some(c) = char::from_u32(cp) {
                                result.push(c);
                            } else {
                                result.push('&');
                                result.push_str(e);
                                result.push(';');
                            }
                        } else {
                            result.push('&');
                            result.push_str(e);
                            result.push(';');
                        }
                    }
                    _ => {
                        result.push('&');
                        result.push_str(&entity);
                        result.push(';');
                    }
                }
            } else {
                result.push('&');
                result.push_str(&entity);
            }
        } else {
            result.push(ch);
        }
    }
    result
}

pub fn sanitize(text: &str, max_chars: usize) -> String {
    let decoded = decode_entities(text);

    let mut out = String::with_capacity(decoded.len());
    let mut in_tag = false;
    for ch in decoded.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }

    let collapsed = out
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");

    if collapsed.len() > max_chars {
        let truncated: String = collapsed.chars().take(max_chars).collect();
        format!("{}...", truncated)
    } else {
        collapsed
    }
}
