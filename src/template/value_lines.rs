use super::{key_line, mark, render_plaintext, LineKind, ValueLine, INDENT_STR};
use crate::tf;
use serde::Deserializer;
use std::collections::{BTreeMap, BTreeSet};
use yaml_rust2::parser::{Event, Parser};
use yaml_rust2::scanner::TScalarStyle;

const MAX_DIFF_CELLS: usize = 4_000_000;
const MAX_DOCUMENT_DEPTH: usize = 128;

#[derive(Clone, Debug, PartialEq)]
enum Value {
    Scalar(tf::Value),
    String(String, bool),
    Object(BTreeMap<String, Value>, bool),
    Array(Vec<Value>),
}

impl Value {
    fn from_tf(value: &tf::Value, document: bool) -> Self {
        match value {
            tf::Value::String(text) => Self::String(text.clone(), !document),
            tf::Value::Object(map) => Self::Object(
                map.iter()
                    .map(|(key, value)| {
                        let key = if document {
                            document_key(key)
                        } else {
                            key.clone()
                        };
                        (key, Self::from_tf(value, document))
                    })
                    .collect(),
                document,
            ),
            tf::Value::Array(array) => Self::Array(
                array
                    .iter()
                    .map(|value| Self::from_tf(value, document))
                    .collect(),
            ),
            scalar => Self::Scalar(scalar.clone()),
        }
    }

    fn inline(&self) -> String {
        match self {
            Self::Scalar(value) => render_plaintext(value),
            Self::String(text, _) => serde_json::to_string(text).unwrap(),
            Self::Object(map, _) => format!(
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

    fn document(&self) -> Option<Self> {
        let Self::String(text, true) = self else {
            return None;
        };
        let value = parse_document(text)?;
        Some(Self::from_tf(&value, true))
    }

    fn structural_identity(&self) -> Self {
        if let Some(document) = self.document() {
            return document;
        }
        match self {
            Self::Array(array) => {
                Self::Array(array.iter().map(Self::structural_identity).collect())
            }
            Self::Object(map, document) => Self::Object(
                map.iter()
                    .map(|(key, value)| (key.clone(), value.structural_identity()))
                    .collect(),
                *document,
            ),
            _ => self.clone(),
        }
    }

    fn is_null(&self) -> bool {
        matches!(self, Self::Scalar(tf::Value::Null))
    }

    fn contains_block_string(&self) -> bool {
        match self {
            Self::String(text, _) => text.contains('\n') || self.document().is_some(),
            Self::Array(values) => values.iter().any(Self::contains_block_string),
            Self::Object(values, _) => values.values().any(Self::contains_block_string),
            Self::Scalar(_) => false,
        }
    }
}

fn document_key(key: &str) -> String {
    if !key.is_empty()
        && key.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/')
        })
    {
        key.to_owned()
    } else {
        serde_json::to_string(key).unwrap()
    }
}

fn equivalent(before: &Value, after: &Value) -> bool {
    if before == after {
        return true;
    }
    match (before, after) {
        (Value::String(..), Value::String(..)) => match (before.document(), after.document()) {
            (Some(before), Some(after)) => before == after,
            _ => false,
        },
        (Value::Array(before), Value::Array(after)) => {
            before.len() == after.len()
                && before
                    .iter()
                    .zip(after)
                    .all(|(before, after)| equivalent(before, after))
        }
        (Value::Object(before, _), Value::Object(after, _)) => {
            before.len() == after.len()
                && before.iter().all(|(key, before)| {
                    after
                        .get(key)
                        .is_some_and(|after| equivalent(before, after))
                })
        }
        _ => false,
    }
}

fn parse_document(text: &str) -> Option<tf::Value> {
    if let Ok(raw) = serde_json::from_str::<Box<serde_json::value::RawValue>>(text) {
        let value = parse_json(raw.get(), 0).ok()?;
        return matches!(value, tf::Value::Object(_) | tf::Value::Array(_)).then_some(value);
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
    matches!(value, tf::Value::Object(_) | tf::Value::Array(_)).then_some(value)
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

fn named(key: &str, value: &Value, indent: usize, parse: bool) -> Vec<ValueLine> {
    if parse {
        if let Some(document) = value.document() {
            return named(key, &document, indent, false);
        }
    }
    let prefix = format!("{}{key}:", INDENT_STR.repeat(indent));
    match value {
        Value::Object(map, _) if !map.is_empty() => {
            let mut result = vec![key_line(key, indent)];
            for (key, value) in map {
                result.extend(named(key, value, indent + 1, parse));
            }
            result
        }
        Value::Array(array) if !array.is_empty() => {
            let mut result = vec![key_line(key, indent)];
            for value in array {
                result.extend(item(named("", value, indent + 2, parse), indent + 1));
            }
            result
        }
        Value::String(text, _) if text.contains('\n') => {
            let mut result = vec![ValueLine::unchanged(format!("{prefix} |"))];
            result.extend(text.split('\n').map(|text| {
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

fn object(
    before: &BTreeMap<String, Value>,
    after: &BTreeMap<String, Value>,
    document: bool,
    indent: usize,
    show: bool,
) -> Vec<ValueLine> {
    let keys: BTreeSet<_> = before.keys().chain(after.keys()).collect();
    let null = Value::Scalar(tf::Value::Null);
    let mut result = Vec::new();
    for key in keys {
        let before = before.get(key);
        let after = after.get(key);
        result.extend(changed(
            key,
            if document {
                before
            } else {
                Some(before.unwrap_or(&null))
            },
            if document {
                after
            } else {
                Some(after.unwrap_or(&null))
            },
            indent,
            show,
            !document,
        ));
    }
    result
}

fn changed(
    key: &str,
    before: Option<&Value>,
    after: Option<&Value>,
    indent: usize,
    show: bool,
    parse: bool,
) -> Vec<ValueLine> {
    let before_document = before.and_then(Value::document);
    let after_document = after.and_then(Value::document);
    let before_present = before.is_some_and(|value| !value.is_null());
    let after_present = after.is_some_and(|value| !value.is_null());
    if parse
        && (!before_present || before_document.is_some())
        && (!after_present || after_document.is_some())
        && (before_document.is_some() || after_document.is_some())
    {
        let lines = changed(
            key,
            before_document.as_ref(),
            after_document.as_ref(),
            indent,
            show,
            false,
        );
        return if before_present && after_present {
            lines
        } else {
            mark(lines, LineKind::Changed)
        };
    }
    if before
        .zip(after)
        .is_some_and(|(before, after)| equivalent(before, after))
    {
        return if show {
            before.map_or_else(Vec::new, |value| {
                if parse
                    && before == after
                    && matches!(value, Value::Array(_))
                    && !value.contains_block_string()
                {
                    vec![ValueLine::unchanged(format!(
                        "{}{key}: {}",
                        INDENT_STR.repeat(indent),
                        value.inline()
                    ))]
                } else {
                    named(key, value, indent, parse)
                }
            })
        } else {
            Vec::new()
        };
    }
    match (before, after) {
        (Some(Value::Object(before, document)), Some(Value::Object(after, _))) => {
            let mut result = vec![key_line(key, indent)];
            result.extend(object(before, after, *document, indent + 1, show));
            result
        }
        (Some(Value::Array(before)), Some(Value::Array(after))) => {
            let mut result = vec![key_line(key, indent)];
            result.extend(array(before, after, indent + 1, show, parse));
            result
        }
        (Some(Value::String(before, _)), Some(Value::String(after, _)))
            if before.contains('\n') || after.contains('\n') =>
        {
            text(key, before, after, indent, show)
        }
        (Some(value), None) if !parse => mark(named(key, value, indent, false), LineKind::Removed),
        (None, Some(value)) if !parse => mark(named(key, value, indent, false), LineKind::Added),
        _ => {
            let expanded = match (before, after) {
                (Some(value), None) | (None, Some(value)) => Some(value),
                (Some(before), Some(after)) if before.is_null() => Some(after),
                (Some(before), Some(after)) if after.is_null() => Some(before),
                _ => None,
            };
            if let Some(value) = expanded {
                if matches!(value, Value::Object(..) | Value::Array(..))
                    || matches!(value, Value::String(text, _) if text.contains('\n'))
                {
                    return mark(named(key, value, indent, parse), LineKind::Changed);
                }
            }
            vec![ValueLine::changed(format!(
                "{}{key}: {} -> {}",
                INDENT_STR.repeat(indent),
                before.map_or_else(|| "(absent)".to_string(), Value::inline),
                after.map_or_else(|| "(absent)".to_string(), Value::inline)
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

fn array(
    before: &[Value],
    after: &[Value],
    indent: usize,
    show: bool,
    parse: bool,
) -> Vec<ValueLine> {
    let mut result = Vec::new();
    let mut removed = Vec::new();
    let mut added = Vec::new();
    let null = Value::Scalar(tf::Value::Null);
    let flush = |result: &mut Vec<ValueLine>, removed: &mut Vec<usize>, added: &mut Vec<usize>| {
        for i in 0..removed.len().max(added.len()) {
            let before_value = removed.get(i).map(|i| &before[*i]);
            let after_value = added.get(i).map(|i| &after[*i]);
            let before = before_value.unwrap_or(&null);
            let after = after_value.unwrap_or(&null);
            let lines = changed(
                "",
                if parse { Some(before) } else { before_value },
                if parse { Some(after) } else { after_value },
                indent + 1,
                show,
                parse,
            );
            if lines.is_empty() && equivalent(before, after) {
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
    let before_identity: Vec<_> = before.iter().map(Value::structural_identity).collect();
    let after_identity: Vec<_> = after.iter().map(Value::structural_identity).collect();
    for (left, right) in alignment(&before_identity, &after_identity, PartialEq::eq) {
        match (left, right) {
            (Some(i), Some(_)) => {
                flush(&mut result, &mut removed, &mut added);
                if show {
                    result.extend(item(named("", &before[i], indent + 1, parse), indent));
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

fn text(key: &str, before: &str, after: &str, indent: usize, show: bool) -> Vec<ValueLine> {
    let mut result = vec![ValueLine::unchanged(format!(
        "{}{key}: |",
        INDENT_STR.repeat(indent)
    ))];
    let before: Vec<_> = before.split('\n').collect();
    let after: Vec<_> = after.split('\n').collect();
    let mut hidden = false;
    for (left, right) in alignment(&before, &after, PartialEq::eq) {
        let (text, kind) = match (left, right) {
            (Some(i), Some(_)) if show => (before[i], LineKind::Same),
            (Some(_), Some(_)) if hidden => continue,
            (Some(_), Some(_)) => {
                hidden = true;
                ("...", LineKind::Same)
            }
            (Some(i), None) => {
                hidden = false;
                (before[i], LineKind::Removed)
            }
            (None, Some(j)) => {
                hidden = false;
                (after[j], LineKind::Added)
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
            .map(|(key, value)| (key.clone(), Value::from_tf(value, false)))
            .collect::<BTreeMap<_, _>>()
    };
    match (before.map(convert), after.map(convert)) {
        (Some(before), Some(after)) => object(&before, &after, false, 0, show),
        (Some(before), None) => mark(
            before
                .iter()
                .flat_map(|(key, value)| named(key, value, 0, true))
                .collect(),
            LineKind::Removed,
        ),
        (None, Some(after)) => mark(
            after
                .iter()
                .flat_map(|(key, value)| named(key, value, 0, true))
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
