use super::{key_line, mark, render_plaintext, LineKind, ValueLine, INDENT_STR};
use crate::tf;
use std::collections::{BTreeMap, BTreeSet};
use yaml_rust2::parser::{Event, Parser};
use yaml_rust2::scanner::{Marker, TScalarStyle};

const MAX_DIFF_CELLS: usize = 4_000_000;
const MAX_DOCUMENT_DEPTH: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Format {
    Text,
    Json,
    Yaml,
}

impl Format {
    fn marker(self) -> &'static str {
        match self {
            Self::Text => "|",
            Self::Json => "| json",
            Self::Yaml => "| yaml",
        }
    }
}

/// A plan value as it renders: a JSON or YAML document string and a multi-line string become a block of lines in
/// their format, so values compare by what they show.
#[derive(Clone, Debug, PartialEq)]
enum Value {
    Scalar(tf::Value),
    String(String),
    Block(Format, Vec<String>),
    Object(BTreeMap<String, Value>),
    Array(Vec<Value>),
}

impl Value {
    fn from_tf(value: &tf::Value) -> Self {
        match value {
            tf::Value::String(text) => Self::from_string(text),
            tf::Value::Object(map) => Self::Object(
                map.iter()
                    .map(|(key, value)| (key.clone(), Self::from_tf(value)))
                    .collect(),
            ),
            tf::Value::Array(array) => Self::Array(array.iter().map(Self::from_tf).collect()),
            scalar => Self::Scalar(scalar.clone()),
        }
    }

    fn from_string(text: &str) -> Self {
        match document_lines(text, 0) {
            Some((format, lines)) => Self::Block(format, lines),
            None if text.contains('\n') => {
                Self::Block(Format::Text, text.split('\n').map(str::to_owned).collect())
            }
            None => Self::String(text.to_owned()),
        }
    }

    fn inline(&self) -> String {
        match self {
            Self::Scalar(value) => render_plaintext(value),
            Self::String(text) => serde_json::to_string(text).unwrap(),
            Self::Block(_, lines) => serde_json::to_string(&lines.join("\n")).unwrap(),
            Self::Object(map) => format!(
                "{{{}}}",
                map.iter()
                    .map(|(key, value)| format!("{key}: {}", value.inline()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Array(array) => format!(
                "[{}]",
                array
                    .iter()
                    .map(Self::inline)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }

    /// The lines a text diff compares: a single-line string diffs against a multi-line one.
    fn text_lines(&self) -> Option<(Format, Vec<String>)> {
        match self {
            Self::String(text) => Some((Format::Text, vec![text.clone()])),
            Self::Block(format, lines) => Some((*format, lines.clone())),
            _ => None,
        }
    }

    fn is_null(&self) -> bool {
        matches!(self, Self::Scalar(tf::Value::Null))
    }

    fn contains_block(&self) -> bool {
        match self {
            Self::Block(..) => true,
            Self::Array(values) => values.iter().any(Self::contains_block),
            Self::Object(values) => values.values().any(Self::contains_block),
            Self::Scalar(_) | Self::String(_) => false,
        }
    }
}

/// The lines of a string holding a JSON or YAML document, with the documents its strings hold expanded.
fn document_lines(text: &str, depth: usize) -> Option<(Format, Vec<String>)> {
    match parse_document(text)? {
        Document::Json => Some((Format::Json, pretty_json(text, depth))),
        Document::Yaml(strings) => Some((Format::Yaml, yaml_lines(text, strings, depth))),
    }
}

/// The lines of a document string held in a document `depth` levels deep, rendered one level deeper; past the depth
/// limit the string stays as written.
fn nested_document(text: &str, depth: usize) -> Option<(Format, Vec<String>)> {
    if depth >= MAX_DOCUMENT_DEPTH || !may_be_document(text) {
        return None;
    }
    document_lines(text, depth + 1)
}

/// Whether a string can hold a document: JSON starts with a bracket, YAML spans several lines.
fn may_be_document(text: &str) -> bool {
    text.trim_start().starts_with(['{', '[']) || text.contains('\n')
}

/// The header line of a nested document under `prefix`, its key or item marker, then its lines indented by `indent`.
fn nest(
    lines: &mut Vec<String>,
    prefix: &str,
    (format, nested): (Format, Vec<String>),
    indent: usize,
) {
    lines.push(if prefix.trim().is_empty() {
        format!("{prefix}{}", format.marker())
    } else {
        format!("{} {}", prefix.trim_end(), format.marker())
    });
    let indent = " ".repeat(indent);
    lines.extend(nested.into_iter().map(|line| format!("{indent}{line}")));
}

/// Pretty-prints valid JSON with 2-space indentation, keeping its keys' order and its tokens as written; a string
/// value holding a document renders as its block.
fn pretty_json(text: &str, depth: usize) -> Vec<String> {
    let indent = |level: usize| " ".repeat(2 * level);
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut level = 0;
    // A nested document's block ends its line, so the comma or bracket after it starts the next one.
    let mut after_block = false;
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '"' => {
                let start = line.len();
                line.push(character);
                while let Some(character) = characters.next() {
                    line.push(character);
                    match character {
                        '\\' => line.extend(characters.next()),
                        '"' => break,
                        _ => {}
                    }
                }
                while characters.next_if(char::is_ascii_whitespace).is_some() {}
                let token = &line[start..];
                if characters.peek() == Some(&':') || !token.contains(['{', '[', '\\']) {
                    continue;
                }
                let Some(document) = serde_json::from_str::<String>(token)
                    .ok()
                    .and_then(|text| nested_document(&text, depth))
                else {
                    continue;
                };
                line.truncate(start);
                nest(&mut lines, &line, document, 2 * (level + 1));
                line.clear();
                after_block = true;
            }
            '{' | '[' => {
                line.push(character);
                while characters.next_if(char::is_ascii_whitespace).is_some() {}
                if let Some(close) = characters.next_if(|c| matches!(c, '}' | ']')) {
                    line.push(close);
                } else {
                    level += 1;
                    lines.push(std::mem::replace(&mut line, indent(level)));
                }
            }
            '}' | ']' => {
                level -= 1;
                let previous = std::mem::replace(&mut line, indent(level));
                if !std::mem::take(&mut after_block) {
                    lines.push(previous);
                }
                line.push(character);
            }
            ',' => {
                if !std::mem::take(&mut after_block) {
                    line.push(character);
                    lines.push(line);
                }
                line = indent(level);
            }
            ':' => line.push_str(": "),
            character if character.is_ascii_whitespace() => {}
            character => line.push(character),
        }
    }
    lines.push(line);
    lines
}

/// The lines of a YAML document as written, with each string scalar holding a document replaced by its block.
fn yaml_lines(text: &str, strings: Vec<YamlString>, depth: usize) -> Vec<String> {
    // A YAML document's final line break ends its last line, unlike text, whose every line shows.
    let source: Vec<&str> = text
        .strip_suffix('\n')
        .unwrap_or(text)
        .split('\n')
        .collect();
    let mut lines = Vec::new();
    let mut next = 0;
    for string in strings {
        let Some((header, prefix, end)) = string.span(&source) else {
            continue;
        };
        if header < next {
            continue;
        }
        let Some(document) = nested_document(&string.text, depth) else {
            continue;
        };
        lines.extend(source[next..header].iter().map(|line| (*line).to_owned()));
        // A key's document indents one level past the key, an item's to the item's content.
        let content = prefix.trim_start_matches([' ', '-']);
        let column = prefix.chars().count() - content.chars().count();
        let indent = if content.is_empty() {
            column
        } else {
            column + INDENT_STR.len()
        };
        nest(&mut lines, &prefix, document, indent);
        next = end + 1;
    }
    lines.extend(source[next..].iter().map(|line| (*line).to_owned()));
    lines
}

enum Document {
    Json,
    /// A YAML document with the quoted and block string scalars its block collections hold that may be documents.
    Yaml(Vec<YamlString>),
}

/// A quoted or block string scalar held by a YAML block mapping's key or a block sequence, where its text starts.
struct YamlString {
    text: String,
    style: TScalarStyle,
    line: usize,
    column: usize,
}

impl YamlString {
    /// The line its key or item marker is on, the text before the scalar there and its last line.
    fn span(&self, source: &[&str]) -> Option<(usize, String, usize)> {
        let text_line = *source.get(self.line)?;
        match self.style {
            TScalarStyle::SingleQuoted | TScalarStyle::DoubleQuoted => {
                let prefix = text_line.chars().take(self.column).collect::<String>();
                let end = self.quoted_end(source)?;
                Self::marked(&prefix).then_some((self.line, prefix, end))
            }
            TScalarStyle::Literal | TScalarStyle::Folded => {
                let blank = |line: &str| line.trim().is_empty();
                let header = (0..self.line).rev().find(|&i| !blank(source[i]))?;
                let prefix = block_indicator_prefix(source[header])?;
                let end = (self.line..source.len())
                    .take_while(|&i| {
                        blank(source[i])
                            || source[i].len() - source[i].trim_start_matches(' ').len()
                                >= self.column
                    })
                    .filter(|&i| !blank(source[i]))
                    .last()?;
                Self::marked(&prefix).then_some((header, prefix, end))
            }
            TScalarStyle::Plain => None,
        }
    }

    /// Whether the text before a scalar ends with its key or is only item markers.
    fn marked(prefix: &str) -> bool {
        let content = prefix.trim_start_matches([' ', '-']).trim_end();
        content.is_empty() && prefix.contains('-') || content.ends_with(':')
    }

    /// The line a quoted scalar ends on, when only a comment follows it there.
    fn quoted_end(&self, source: &[&str]) -> Option<usize> {
        let single = self.style == TScalarStyle::SingleQuoted;
        let quote = if single { '\'' } else { '"' };
        let mut characters = source[self.line..]
            .iter()
            .enumerate()
            .flat_map(|(i, text)| {
                text.chars()
                    .chain(Some('\n'))
                    .map(move |character| (self.line + i, character))
            })
            .skip(self.column)
            .peekable();
        if characters.next()?.1 != quote {
            return None;
        }
        let end = loop {
            match characters.next()? {
                (_, '\\') if !single => {
                    characters.next();
                }
                (line, '\'') if single => {
                    if characters
                        .next_if(|(_, character)| *character == '\'')
                        .is_none()
                    {
                        break line;
                    }
                }
                (line, '"') if !single => break line,
                _ => {}
            }
        };
        let rest = characters
            .map_while(|(_, character)| (character != '\n').then_some(character))
            .collect::<String>();
        let rest = rest.trim();
        (rest.is_empty() || rest.starts_with('#')).then_some(end)
    }
}

/// The text before a block scalar's `|` or `>` indicator on its line, unless the indicator sets the indentation.
fn block_indicator_prefix(line: &str) -> Option<String> {
    let characters: Vec<char> = line.strip_suffix('\r').unwrap_or(line).chars().collect();
    (1..characters.len()).find_map(|i| {
        if !matches!(characters[i], '|' | '>') || characters[i - 1] != ' ' {
            return None;
        }
        let rest = characters[i + 1..]
            .iter()
            .skip_while(|character| matches!(character, '+' | '-'))
            .skip_while(|character| **character == ' ')
            .collect::<String>();
        (rest.is_empty() || rest.starts_with('#')).then(|| characters[..i].iter().collect())
    })
}

/// The kind of a string holding a JSON object or array, or a multi-line YAML one that reads without loss, with the
/// YAML one's strings that may be documents.
fn parse_document(text: &str) -> Option<Document> {
    if let Ok(raw) = serde_json::from_str::<Box<serde_json::value::RawValue>>(text) {
        return raw.get().starts_with(['{', '[']).then_some(Document::Json);
    }
    if !text.contains('\n') {
        return None;
    }
    // The event stream retains duplicate keys and tags that a loaded YAML map can discard.
    let mut parser = Parser::new_from_str(text);
    let mut events = Vec::new();
    loop {
        let (event, marker) = parser.next_token().ok()?;
        let end = event == Event::StreamEnd;
        events.push((event, marker));
        if end {
            break;
        }
    }
    let mut reader = YamlReader {
        events: events.into_iter(),
        characters: text.chars().collect(),
        strings: Vec::new(),
    };
    if reader.event()? != Event::StreamStart || reader.event()? != Event::DocumentStart {
        return None;
    }
    let value = reader.node(0, false)?;
    if reader.event()? != Event::DocumentEnd || reader.event()? != Event::StreamEnd {
        return None;
    }
    matches!(value, tf::Value::Object(_) | tf::Value::Array(_))
        .then_some(Document::Yaml(reader.strings))
}

struct YamlReader {
    events: std::vec::IntoIter<(Event, Marker)>,
    /// The source by character, as markers index it.
    characters: Vec<char>,
    strings: Vec<YamlString>,
}

impl YamlReader {
    fn event(&mut self) -> Option<Event> {
        self.events.next().map(|(event, _)| event)
    }

    fn node(&mut self, depth: usize, held: bool) -> Option<tf::Value> {
        let (event, marker) = self.events.next()?;
        self.parse(event, marker, depth, held)
    }

    /// Whether a collection starting at `marker` is in flow style.
    fn flow(&self, marker: Marker) -> bool {
        self.characters
            .get(marker.index())
            .is_some_and(|character| matches!(character, '[' | '{'))
    }

    /// Reads a node; `held` marks a key's value or an item of a block collection.
    fn parse(
        &mut self,
        event: Event,
        marker: Marker,
        depth: usize,
        held: bool,
    ) -> Option<tf::Value> {
        if depth > MAX_DOCUMENT_DEPTH {
            return None;
        }
        match event {
            Event::Scalar(text, style, 0, None) => {
                if held && style != TScalarStyle::Plain && may_be_document(&text) {
                    self.strings.push(YamlString {
                        text: text.clone(),
                        style,
                        line: marker.line() - 1,
                        column: marker.col(),
                    });
                }
                parse_scalar(text, style)
            }
            Event::SequenceStart(0, None) => {
                let held = !self.flow(marker);
                let mut array = Vec::new();
                loop {
                    let (event, marker) = self.events.next()?;
                    if event == Event::SequenceEnd {
                        break;
                    }
                    array.push(self.parse(event, marker, depth + 1, held)?);
                }
                Some(tf::Value::Array(array))
            }
            Event::MappingStart(0, None) => {
                let held = !self.flow(marker);
                let mut map = tf::ValueMap::new();
                loop {
                    let (event, marker) = self.events.next()?;
                    if event == Event::MappingEnd {
                        break;
                    }
                    let tf::Value::String(key) = self.parse(event, marker, depth + 1, false)?
                    else {
                        return None;
                    };
                    if key == "<<" || map.contains_key(&key) {
                        return None;
                    }
                    map.insert(key, self.node(depth + 1, held)?);
                }
                Some(tf::Value::Object(map))
            }
            _ => None,
        }
    }
}

fn parse_scalar(text: String, style: TScalarStyle) -> Option<tf::Value> {
    if style != TScalarStyle::Plain {
        return Some(tf::Value::String(text));
    }
    match text.as_str() {
        "" | "~" | "null" | "Null" | "NULL" => return Some(tf::Value::Null),
        "true" | "True" | "TRUE" => return Some(tf::Value::Boolean(true)),
        "false" | "False" | "FALSE" => return Some(tf::Value::Boolean(false)),
        _ => {}
    }
    if let Ok(number) = text.parse::<i64>() {
        return Some(tf::Value::Integer(number));
    }
    if let Ok(number) = text.parse::<f64>() {
        if !number.is_finite()
            || !text.contains(['.', 'e', 'E'])
            || decimal_identity(&text) != decimal_identity(&number.to_string())
        {
            return None;
        }
        return Some(tf::Value::Float(number));
    }
    if matches!(
        text.to_ascii_lowercase().as_str(),
        ".inf" | "+.inf" | "-.inf" | ".nan"
    ) {
        return None;
    }
    if text.starts_with("0x") || text.starts_with("0o") {
        return None;
    }
    Some(tf::Value::String(text))
}

fn decimal_identity(text: &str) -> Option<(bool, String, i32)> {
    let negative = text.starts_with('-');
    let text = text.trim_start_matches(['+', '-']);
    let (mantissa, exponent) =
        text.split_once(['e', 'E'])
            .map_or(Some((text, 0)), |(mantissa, exponent)| {
                exponent
                    .parse::<i32>()
                    .ok()
                    .map(|exponent| (mantissa, exponent))
            })?;
    let fraction = mantissa
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len());
    let mut exponent = exponent.checked_sub(i32::try_from(fraction).ok()?)?;
    let mut digits = mantissa
        .replace('.', "")
        .trim_start_matches('0')
        .to_string();
    if digits.is_empty() {
        return Some((false, String::new(), 0));
    }
    while digits.ends_with('0') {
        digits.pop();
        exponent = exponent.checked_add(1)?;
    }
    Some((negative, digits, exponent))
}

fn line(text: String, kind: LineKind) -> ValueLine {
    ValueLine { text, kind }
}

fn header(key: &str, format: Format, indent: usize) -> ValueLine {
    ValueLine::unchanged(format!(
        "{}{key}: {}",
        INDENT_STR.repeat(indent),
        format.marker()
    ))
}

fn named(key: &str, value: &Value, indent: usize) -> Vec<ValueLine> {
    let prefix = format!("{}{key}:", INDENT_STR.repeat(indent));
    match value {
        Value::Object(map) if !map.is_empty() => {
            let mut result = vec![key_line(key, indent)];
            for (key, value) in map {
                result.extend(named(key, value, indent + 1));
            }
            result
        }
        Value::Array(array) if !array.is_empty() => {
            let mut result = vec![key_line(key, indent)];
            for value in array {
                result.extend(item(named("", value, indent + 2), indent + 1));
            }
            result
        }
        Value::Block(format, lines) => {
            let mut result = vec![header(key, *format, indent)];
            result.extend(lines.iter().map(|text| {
                ValueLine::unchanged(format!("{}{text}", INDENT_STR.repeat(indent + 1)))
            }));
            result
        }
        _ => vec![ValueLine::unchanged(format!("{prefix} {}", value.inline()))],
    }
}

fn item(mut lines: Vec<ValueLine>, indent: usize) -> Vec<ValueLine> {
    if lines.first().is_some_and(|line| line.text.trim() == ":") {
        lines.remove(0);
        for line in &mut lines {
            if line.text.starts_with(INDENT_STR) {
                line.text.drain(..INDENT_STR.len());
            }
        }
    } else {
        for line in lines.iter_mut().skip(1) {
            if line.text.starts_with(INDENT_STR) {
                line.text.drain(..INDENT_STR.len());
            }
        }
    }
    let nested = INDENT_STR.repeat(indent + 1);
    if let Some(first) = lines.first_mut() {
        let content = first.text.strip_prefix(&nested).unwrap_or(&first.text);
        let content = content
            .strip_prefix(": ")
            .or_else(|| content.strip_prefix(':'))
            .unwrap_or(content);
        first.text = format!("{}- {}", INDENT_STR.repeat(indent), content);
    }
    lines
}

/// A value replaced by one of another kind, such as an object or a document string by `(known after apply)`, when
/// either spans several lines: the old lines marked removed, then the new ones marked added.
fn replaced(
    key: &str,
    before: &Value,
    after: &Value,
    indent: usize,
) -> Option<(Vec<ValueLine>, Vec<ValueLine>)> {
    let same_kind = match (before, after) {
        (Value::Object(_), Value::Object(_)) | (Value::Array(_), Value::Array(_)) => true,
        _ => matches!(
            (before.text_lines(), after.text_lines()),
            (Some((before, _)), Some((after, _))) if before == after
        ),
    };
    if before.is_null() || after.is_null() || same_kind {
        return None;
    }
    let removed = named(key, before, indent);
    let added = named(key, after, indent);
    (removed.len() > 1 || added.len() > 1).then(|| {
        (
            mark(removed, LineKind::Removed),
            mark(added, LineKind::Added),
        )
    })
}

fn object(
    before: &BTreeMap<String, Value>,
    after: &BTreeMap<String, Value>,
    indent: usize,
    show: bool,
) -> Vec<ValueLine> {
    let keys: BTreeSet<_> = before.keys().chain(after.keys()).collect();
    let null = Value::Scalar(tf::Value::Null);
    let mut result = Vec::new();
    for key in keys {
        result.extend(changed(
            key,
            before.get(key).unwrap_or(&null),
            after.get(key).unwrap_or(&null),
            indent,
            show,
        ));
    }
    result
}

fn changed(key: &str, before: &Value, after: &Value, indent: usize, show: bool) -> Vec<ValueLine> {
    if before == after {
        return if !show {
            Vec::new()
        } else if matches!(before, Value::Array(_)) && !before.contains_block() {
            vec![ValueLine::unchanged(format!(
                "{}{key}: {}",
                INDENT_STR.repeat(indent),
                before.inline()
            ))]
        } else {
            named(key, before, indent)
        };
    }
    match (before, after) {
        (Value::Object(before), Value::Object(after)) => {
            let mut result = vec![key_line(key, indent)];
            result.extend(object(before, after, indent + 1, show));
            result
        }
        (Value::Array(before), Value::Array(after)) => {
            let mut result = vec![key_line(key, indent)];
            result.extend(array(before, after, indent + 1, show));
            result
        }
        _ => {
            if let (Some((format, before_lines)), Some((after_format, after_lines))) =
                (before.text_lines(), after.text_lines())
            {
                if format == after_format
                    && (matches!(before, Value::Block(..)) || matches!(after, Value::Block(..)))
                {
                    return text(key, format, &before_lines, &after_lines, indent, show);
                }
            }
            let expanded = if before.is_null() {
                Some(after)
            } else if after.is_null() {
                Some(before)
            } else {
                None
            };
            if let Some(value) = expanded.filter(|value| {
                matches!(value, Value::Object(_) | Value::Array(_) | Value::Block(..))
            }) {
                return mark(named(key, value, indent), LineKind::Changed);
            }
            if let Some((removed, added)) = replaced(key, before, after, indent) {
                return removed.into_iter().chain(added).collect();
            }
            vec![ValueLine::changed(format!(
                "{}{key}: {} -> {}",
                INDENT_STR.repeat(indent),
                before.inline(),
                after.inline()
            ))]
        }
    }
}

fn alignment<T>(
    before: &[T],
    after: &[T],
    equal: impl Fn(&T, &T) -> bool,
) -> Vec<(Option<usize>, Option<usize>)> {
    let prefix = before
        .iter()
        .zip(after)
        .take_while(|(a, b)| equal(a, b))
        .count();
    let suffix = before[prefix..]
        .iter()
        .rev()
        .zip(after[prefix..].iter().rev())
        .take_while(|(a, b)| equal(a, b))
        .count();
    let (left, right) = (
        &before[prefix..before.len() - suffix],
        &after[prefix..after.len() - suffix],
    );
    let mut result: Vec<_> = (0..prefix).map(|i| (Some(i), Some(i))).collect();
    if left.len().saturating_mul(right.len()) > MAX_DIFF_CELLS {
        for i in 0..left.len().max(right.len()) {
            match (left.get(i), right.get(i)) {
                (Some(left), Some(right)) if equal(left, right) => {
                    result.push((Some(prefix + i), Some(prefix + i)));
                }
                (left, right) => {
                    if left.is_some() {
                        result.push((Some(prefix + i), None));
                    }
                    if right.is_some() {
                        result.push((None, Some(prefix + i)));
                    }
                }
            }
        }
    } else {
        let mut table = vec![vec![0; right.len() + 1]; left.len() + 1];
        for i in (0..left.len()).rev() {
            for j in (0..right.len()).rev() {
                table[i][j] = if equal(&left[i], &right[j]) {
                    table[i + 1][j + 1] + 1
                } else {
                    table[i + 1][j].max(table[i][j + 1])
                };
            }
        }
        let (mut i, mut j) = (0, 0);
        while i < left.len() || j < right.len() {
            if i < left.len() && j < right.len() && equal(&left[i], &right[j]) {
                result.push((Some(prefix + i), Some(prefix + j)));
                i += 1;
                j += 1;
            } else if j == right.len() || (i < left.len() && table[i + 1][j] >= table[i][j + 1]) {
                result.push((Some(prefix + i), None));
                i += 1;
            } else {
                result.push((None, Some(prefix + j)));
                j += 1;
            }
        }
    }
    result.extend((0..suffix).map(|i| {
        (
            Some(before.len() - suffix + i),
            Some(after.len() - suffix + i),
        )
    }));
    result
}

fn array(before: &[Value], after: &[Value], indent: usize, show: bool) -> Vec<ValueLine> {
    let mut result = Vec::new();
    let mut removed = Vec::new();
    let mut added = Vec::new();
    let null = Value::Scalar(tf::Value::Null);
    let flush = |result: &mut Vec<ValueLine>, removed: &mut Vec<usize>, added: &mut Vec<usize>| {
        for i in 0..removed.len().max(added.len()) {
            let before = removed.get(i).map_or(&null, |i| &before[*i]);
            let after = added.get(i).map_or(&null, |i| &after[*i]);
            if let Some((old_lines, new_lines)) = replaced("", before, after, indent + 1) {
                result.extend(item(old_lines, indent));
                result.extend(item(new_lines, indent));
                continue;
            }
            let lines = changed("", before, after, indent + 1, show);
            if lines.is_empty() && before == after {
                continue;
            }
            if lines.is_empty() || (lines.len() == 1 && lines[0].text.trim() == ":") {
                result.extend(item(
                    vec![ValueLine::changed(format!(
                        "{}: {} -> {}",
                        INDENT_STR.repeat(indent + 1),
                        before.inline(),
                        after.inline()
                    ))],
                    indent,
                ));
            } else {
                result.extend(item(lines, indent));
            }
        }
        removed.clear();
        added.clear();
    };
    let mut hidden = false;
    for (left, right) in alignment(before, after, PartialEq::eq) {
        match (left, right) {
            (Some(i), Some(_)) => {
                flush(&mut result, &mut removed, &mut added);
                if show {
                    result.extend(item(named("", &before[i], indent + 1), indent));
                } else if !hidden {
                    result.push(ValueLine::unchanged(format!(
                        "{}- ...",
                        INDENT_STR.repeat(indent)
                    )));
                }
                hidden = true;
            }
            (Some(i), None) => {
                removed.push(i);
                hidden = false;
            }
            (None, Some(j)) => {
                added.push(j);
                hidden = false;
            }
            (None, None) => unreachable!(),
        }
    }
    flush(&mut result, &mut removed, &mut added);
    result
}

fn text(
    key: &str,
    format: Format,
    before: &[String],
    after: &[String],
    indent: usize,
    show: bool,
) -> Vec<ValueLine> {
    let mut result = vec![header(key, format, indent)];
    let mut hidden = false;
    for (left, right) in alignment(before, after, PartialEq::eq) {
        let (text, kind) = match (left, right) {
            (Some(i), Some(_)) if show => (before[i].as_str(), LineKind::Same),
            (Some(_), Some(_)) if hidden => continue,
            (Some(_), Some(_)) => {
                hidden = true;
                ("...", LineKind::Same)
            }
            (Some(i), None) => {
                hidden = false;
                (before[i].as_str(), LineKind::Removed)
            }
            (None, Some(j)) => {
                hidden = false;
                (after[j].as_str(), LineKind::Added)
            }
            (None, None) => unreachable!(),
        };
        result.push(line(
            format!("{}{text}", INDENT_STR.repeat(indent + 1)),
            kind,
        ));
    }
    result
}

pub(super) fn render_value_lines(
    before: Option<&tf::ValueMap>,
    after: Option<&tf::ValueMap>,
    show: bool,
) -> Vec<ValueLine> {
    let convert = |map: &tf::ValueMap| {
        map.iter()
            .map(|(key, value)| (key.clone(), Value::from_tf(value)))
            .collect::<BTreeMap<_, _>>()
    };
    match (before.map(convert), after.map(convert)) {
        (Some(before), Some(after)) => object(&before, &after, 0, show),
        (Some(before), None) => mark(
            before
                .iter()
                .flat_map(|(key, value)| named(key, value, 0))
                .collect(),
            LineKind::Removed,
        ),
        (None, Some(after)) => mark(
            after
                .iter()
                .flat_map(|(key, value)| named(key, value, 0))
                .collect(),
            LineKind::Added,
        ),
        (None, None) => Vec::new(),
    }
}
