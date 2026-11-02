use super::{key_line, mark, render_plaintext, LineKind, ValueLine, INDENT_STR};
use crate::tf;
use serde::Deserializer;
use std::collections::{BTreeMap, BTreeSet};
use yaml_rust2::parser::{Event, Parser};
use yaml_rust2::scanner::TScalarStyle;

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
        let lines = |text: &str| text.split('\n').map(str::to_owned).collect();
        match parse_document(text) {
            Some(Format::Json) => Self::Block(Format::Json, pretty_json(text)),
            // A YAML document's final line break ends its last line, unlike text, whose every line shows.
            Some(format) => Self::Block(format, lines(text.strip_suffix('\n').unwrap_or(text))),
            None if text.contains('\n') => Self::Block(Format::Text, lines(text)),
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

/// Pretty-prints valid JSON with 2-space indentation, keeping its keys' order and its tokens as written.
fn pretty_json(text: &str) -> Vec<String> {
    let indent = |depth: usize| " ".repeat(2 * depth);
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut depth = 0;
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '"' => {
                line.push(character);
                while let Some(character) = characters.next() {
                    line.push(character);
                    match character {
                        '\\' => line.extend(characters.next()),
                        '"' => break,
                        _ => {}
                    }
                }
            }
            '{' | '[' => {
                line.push(character);
                while characters.next_if(char::is_ascii_whitespace).is_some() {}
                if let Some(close) = characters.next_if(|c| matches!(c, '}' | ']')) {
                    line.push(close);
                } else {
                    depth += 1;
                    lines.push(std::mem::replace(&mut line, indent(depth)));
                }
            }
            '}' | ']' => {
                depth -= 1;
                lines.push(std::mem::replace(&mut line, indent(depth)));
                line.push(character);
            }
            ',' => {
                line.push(character);
                lines.push(std::mem::replace(&mut line, indent(depth)));
            }
            ':' => line.push_str(": "),
            character if character.is_ascii_whitespace() => {}
            character => line.push(character),
        }
    }
    lines.push(line);
    lines
}

/// The format of a string holding a JSON or YAML object or array, as long as its YAML reads without loss.
fn parse_document(text: &str) -> Option<Format> {
    if let Ok(raw) = serde_json::from_str::<Box<serde_json::value::RawValue>>(text) {
        let value = parse_json(raw.get(), 0).ok()?;
        return matches!(value, tf::Value::Object(_) | tf::Value::Array(_)).then_some(Format::Json);
    }
    // The event stream retains duplicate keys and tags that a loaded YAML map can discard.
    let mut parser = Parser::new_from_str(text);
    let mut events = Vec::new();
    loop {
        let (event, _) = parser.next_token().ok()?;
        let end = event == Event::StreamEnd;
        events.push(event);
        if end {
            break;
        }
    }
    let mut events = events.into_iter();
    if events.next()? != Event::StreamStart || events.next()? != Event::DocumentStart {
        return None;
    }
    let value = parse_node(&mut events, 0)?;
    if events.next()? != Event::DocumentEnd || events.next()? != Event::StreamEnd {
        return None;
    }
    matches!(value, tf::Value::Object(_) | tf::Value::Array(_)).then_some(Format::Yaml)
}

fn parse_node(events: &mut impl Iterator<Item = Event>, depth: usize) -> Option<tf::Value> {
    parse_event(events.next()?, events, depth)
}

fn parse_event(
    event: Event,
    events: &mut impl Iterator<Item = Event>,
    depth: usize,
) -> Option<tf::Value> {
    if depth > MAX_DOCUMENT_DEPTH {
        return None;
    }
    match event {
        Event::Scalar(text, style, 0, None) => parse_scalar(text, style),
        Event::SequenceStart(0, None) => {
            let mut array = Vec::new();
            loop {
                let event = events.next()?;
                if event == Event::SequenceEnd {
                    break;
                }
                array.push(parse_event(event, events, depth + 1)?);
            }
            Some(tf::Value::Array(array))
        }
        Event::MappingStart(0, None) => {
            let mut map = tf::ValueMap::new();
            loop {
                let event = events.next()?;
                if event == Event::MappingEnd {
                    break;
                }
                let tf::Value::String(key) = parse_event(event, events, depth + 1)? else {
                    return None;
                };
                if key == "<<" || map.contains_key(&key) {
                    return None;
                }
                map.insert(key, parse_node(events, depth + 1)?);
            }
            Some(tf::Value::Object(map))
        }
        _ => None,
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
            let mut result = vec![ValueLine::unchanged(format!(
                "{prefix} {}",
                format.marker()
            ))];
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
    let mut result = vec![ValueLine::unchanged(format!(
        "{}{key}: {}",
        INDENT_STR.repeat(indent),
        format.marker()
    ))];
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

fn parse_json(text: &str, depth: usize) -> Result<tf::Value, serde_json::Error> {
    struct Visitor {
        depth: usize,
    }
    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = tf::Value;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a supported document value")
        }

        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            let mut values = Vec::new();
            while let Some(raw) = sequence.next_element::<Box<serde_json::value::RawValue>>()? {
                values
                    .push(parse_json(raw.get(), self.depth + 1).map_err(serde::de::Error::custom)?);
            }
            Ok(tf::Value::Array(values))
        }

        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut values = tf::ValueMap::new();
            while let Some(key) = map.next_key::<String>()? {
                if values.contains_key(&key) {
                    return Err(serde::de::Error::custom("duplicate key"));
                }
                let raw = map.next_value::<Box<serde_json::value::RawValue>>()?;
                let value =
                    parse_json(raw.get(), self.depth + 1).map_err(serde::de::Error::custom)?;
                values.insert(key, value);
            }
            Ok(tf::Value::Object(values))
        }
    }
    if depth > MAX_DOCUMENT_DEPTH {
        return Err(serde::de::Error::custom("document too deep"));
    }
    let mut deserializer = serde_json::Deserializer::from_str(text);
    match text.as_bytes()[0] {
        b'{' => deserializer.deserialize_map(Visitor { depth }),
        b'[' => deserializer.deserialize_seq(Visitor { depth }),
        b'"' => serde_json::from_str(text).map(tf::Value::String),
        _ => parse_scalar(text.to_owned(), TScalarStyle::Plain)
            .ok_or_else(|| serde::de::Error::custom("unsupported number")),
    }
}
