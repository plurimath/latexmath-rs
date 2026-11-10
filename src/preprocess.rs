//! The stage's LaTeX preprocessing contract; text/mbox groups retain caller bytes.
use std::collections::HashMap;
fn groups(text: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut pos = 0;
    while pos < text.len() {
        let a = text[pos..].find("\\text{").map(|i| pos + i);
        let b = text[pos..].find("\\mbox{").map(|i| pos + i);
        let start = match (a, b) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) | (None, Some(a)) => a,
            _ => break,
        };
        let body = start + 6;
        let Some(close) = text[body..].find('}').map(|i| body + i) else {
            break;
        };
        if close == body {
            pos = start + 1;
        } else {
            ranges.push((start, close + 1));
            pos = close + 1;
        }
    }
    ranges
}
pub(super) fn decode(input: &str) -> String {
    let entities: HashMap<String, String> =
        serde_json::from_str(include_str!("../artifacts/html4-entities.json"))
            .expect("embedded HTML4 entities");
    let mut out = String::new();
    let mut pos = 0;
    while let Some(start) = input[pos..].find('&').map(|i| pos + i) {
        out.push_str(&input[pos..start]);
        let Some(end) = input[start..].find(';').map(|i| start + i) else {
            out.push_str(&input[start..]);
            return out;
        };
        let key = &input[start + 1..end];
        let scalar = if let Some(hex) = key.strip_prefix("#x").or_else(|| key.strip_prefix("#X")) {
            u32::from_str_radix(hex, 16).ok()
        } else {
            key.strip_prefix('#').and_then(|n| n.parse().ok())
        };
        if let Some(c) = scalar.and_then(char::from_u32) {
            out.push(c);
            pos = end + 1;
        } else if let Some(s) = entities.get(key) {
            out.push_str(s);
            pos = end + 1;
        } else {
            out.push('&');
            pos = start + 1;
        }
    }
    out.push_str(&input[pos..]);
    out
}
pub fn call(input: &str) -> String {
    let saved: Vec<&str> = groups(input).iter().map(|(a, b)| &input[*a..*b]).collect();
    let text: String = decode(input)
        .chars()
        .map(|c| {
            if matches!(c, '<' | '>' | '\'' | '"' | '&') || !(' '..='~').contains(&c) {
                format!("&#x{:x};", c as u32)
            } else {
                c.to_string()
            }
        })
        .collect();
    let mut cleaned = String::new();
    let mut prev = None;
    for c in text.chars() {
        if c != ' ' || prev == Some('\\') {
            cleaned.push(c);
        }
        prev = Some(c);
    }
    let cleaned = cleaned
        .replace("\\\\ ", "\\\\\\\\")
        .replace("&#x26;", "&")
        .replace("&#x22;", "\"");
    let mut text = String::new();
    let mut pos = 0;
    while let Some(at) = cleaned[pos..].find("\\&#xa;").map(|i| pos + i) {
        text.push_str(&cleaned[pos..at]);
        if at == 0 || cleaned.as_bytes()[at - 1] != b'\\' {
            text.push_str("\\ ");
        } else {
            text.push_str("\\&#xa;");
        }
        pos = at + 6;
    }
    text.push_str(&cleaned[pos..]);
    let text = text.replace("&#xa;", "");
    let mut out = String::new();
    let mut pos = 0;
    for ((a, b), original) in groups(&text).into_iter().zip(saved) {
        out.push_str(&text[pos..a]);
        out.push_str(original);
        pos = b;
    }
    out.push_str(&text[pos..]);
    out
}
