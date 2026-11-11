//! Annotated rewrite pipeline ported from the source-gated TS stage
//! preprocess.ts/SourceMap contract. Units are Unicode scalars; public offsets
//! are translated back to original UTF-8 bytes. Native strict APIs keep their
//! existing preprocessor and preprocessed-coordinate contract.
#[derive(Clone)]
struct Unit {
    value: char,
    start: usize,
    end: usize,
}
#[derive(Clone)]
struct Working {
    units: Vec<Unit>,
}
impl Working {
    fn text(&self) -> String {
        self.units.iter().map(|u| u.value).collect()
    }
    fn rewrite(&self, changes: Vec<(usize, usize, String)>) -> Self {
        let mut out = Vec::new();
        let mut cursor = 0;
        for (start, end, text) in changes {
            out.extend_from_slice(&self.units[cursor..start]);
            let first = self.units[start].start;
            let last = self.units[end - 1].end;
            out.extend(text.chars().map(|value| Unit {
                value,
                start: first,
                end: last,
            }));
            cursor = end;
        }
        out.extend_from_slice(&self.units[cursor..]);
        Self { units: out }
    }
    fn literal(&self, from: &str, to: &str) -> Self {
        let pattern: Vec<char> = from.chars().collect();
        let mut changes = Vec::new();
        let mut i = 0;
        while i + pattern.len() <= self.units.len() {
            if self.units[i..i + pattern.len()]
                .iter()
                .map(|u| u.value)
                .eq(pattern.iter().copied())
            {
                changes.push((i, i + pattern.len(), to.to_string()));
                i += pattern.len();
            } else {
                i += 1;
            }
        }
        self.rewrite(changes)
    }
}
fn groups(units: &[Unit]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = 0;
    let mut close = None;
    while i + 6 <= units.len() {
        let prefix: String = units[i..i + 6].iter().map(|u| u.value).collect();
        if prefix == "\\text{" || prefix == "\\mbox{" {
            let body = i + 6;
            if close.is_none_or(|at| at < body) {
                close = (body..units.len()).find(|at| units[*at].value == '}');
            }
            let Some(end) = close else { break };
            if end > body {
                out.push((i, end + 1));
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }
    out
}
pub struct Prepared {
    pub text: String,
    origins: Vec<usize>,
    source_bytes: Vec<usize>,
}
impl Prepared {
    pub fn to_original_byte(&self, offset: usize) -> usize {
        let index = self
            .text
            .char_indices()
            .take_while(|(at, _)| *at < offset)
            .count();
        let origin = self
            .origins
            .get(index)
            .copied()
            .unwrap_or(self.source_bytes.len() - 1);
        self.source_bytes[origin]
    }
}
pub fn call(input: &str) -> Prepared {
    let mut source_bytes: Vec<usize> = input.char_indices().map(|(i, _)| i).collect();
    source_bytes.push(input.len());
    let mut working = Working {
        units: input
            .chars()
            .enumerate()
            .map(|(i, value)| Unit {
                value,
                start: i,
                end: i + 1,
            })
            .collect(),
    };
    let saved: Vec<String> = groups(&working.units)
        .iter()
        .map(|(a, b)| working.units[*a..*b].iter().map(|u| u.value).collect())
        .collect();
    // Decode only complete named/numeric entity tokens, preserving every
    // unchanged unit rather than attributing an enclosing string to one span.
    let mut changes = Vec::new();
    let mut i = 0;
    while i < working.units.len() {
        if working.units[i].value == '&' {
            let mut end = i + 1;
            while end < working.units.len()
                && (working.units[end].value.is_ascii_alphanumeric()
                    || working.units[end].value == '#')
            {
                end += 1;
            }
            if end < working.units.len() && working.units[end].value == ';' {
                let raw: String = working.units[i..=end].iter().map(|u| u.value).collect();
                let decoded = super::preprocess::decode(&raw);
                if raw != decoded {
                    changes.push((i, end + 1, decoded));
                    i = end + 1;
                    continue;
                }
            }
        }
        i += 1;
    }
    working = working.rewrite(changes);
    for basic in [true, false] {
        let changes = working
            .units
            .iter()
            .enumerate()
            .filter_map(|(i, u)| {
                let encode = if basic {
                    matches!(u.value, '<' | '>' | '\'' | '"' | '&')
                } else {
                    !(' '..='~').contains(&u.value)
                };
                encode.then(|| (i, i + 1, format!("&#x{:x};", u.value as u32)))
            })
            .collect();
        working = working.rewrite(changes);
    }
    let changes = working
        .units
        .iter()
        .enumerate()
        .filter_map(|(i, u)| {
            ((u.value == ' ' && (i == 0 || working.units[i - 1].value != '\\')) || u.value == '\n')
                .then(|| (i, i + 1, String::new()))
        })
        .collect();
    working = working.rewrite(changes);
    working = working
        .literal("\\\\ ", "\\\\")
        .literal("&#x26;", "&")
        .literal("&#x22;", "\"");
    let pattern: Vec<char> = "\\&#xa;".chars().collect();
    let mut changes = Vec::new();
    let mut i = 0;
    while i + pattern.len() <= working.units.len() {
        if working.units[i..i + pattern.len()]
            .iter()
            .map(|u| u.value)
            .eq(pattern.iter().copied())
            && !(i >= 2 && working.units[i - 1].value == '\\' && working.units[i - 2].value == '\\')
        {
            changes.push((i, i + pattern.len(), "\\ ".to_string()));
            i += pattern.len();
        } else {
            i += 1;
        }
    }
    working = working.rewrite(changes).literal("&#xa;", "");
    let changes = groups(&working.units)
        .into_iter()
        .enumerate()
        .map(|(i, (a, b))| (a, b, saved.get(i).cloned().unwrap_or_default()))
        .collect();
    working = working.rewrite(changes);
    let mut origins = Vec::new();
    let mut i = 0;
    while i < working.units.len() {
        let unit = &working.units[i];
        let mut end = i + 1;
        while end < working.units.len()
            && working.units[end].start == unit.start
            && working.units[end].end == unit.end
        {
            end += 1;
        }
        for offset in 0..end - i {
            origins.push(unit.start + offset.min(unit.end.saturating_sub(unit.start + 1)));
        }
        i = end;
    }
    Prepared {
        text: working.text(),
        origins,
        source_bytes,
    }
}

#[cfg(test)]
mod tests {
    use super::call;
    #[test]
    fn rewrites_and_origin_spans_cover_deletion_entities_and_text_restoration() {
        let empty = call("   ");
        assert_eq!(empty.text, "");
        assert_eq!(empty.to_original_byte(0), 3);
        let slash = call("\\\\ ");
        assert_eq!(slash.text, "\\\\");
        assert_eq!(slash.to_original_byte(2), 3);
        let created = "x\\text {y}";
        let deleted = call(created);
        assert_eq!(deleted.text, "x");
        assert_eq!(deleted.to_original_byte(1), created.len());
        let text = "\\text{α b}";
        let restored = call(text);
        assert_eq!(restored.text, text);
        for (offset, _) in text.char_indices() {
            assert_eq!(restored.to_original_byte(offset), offset);
        }
        let entity = call("&amp;\\nosuch");
        assert_eq!(entity.text, "&\\nosuch");
        assert_eq!(entity.to_original_byte(1), 5);
        let greek = call("α\\nosuch");
        assert_eq!(greek.text, "&#x3b1;\\nosuch");
        assert_eq!(greek.to_original_byte(7), 2);
        let newline = call("\\\\\n");
        assert_eq!(newline.text, "\\\\ ");
    }
}
