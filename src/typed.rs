//! Stage-specific adapter into upstream generated Lutaml models and codecs.
use crate::{
    generated::{json_codecs, models::Latexmath_Model_Document},
    Error, Parser,
};
use serde_json::{json, Value};
use std::{collections::HashMap, fmt};
#[derive(Debug)]
pub enum ModelError {
    Parse(Error),
    Codec(Box<lutaml_rs::contracts::ContractError>),
}
impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "{e}"),
            Self::Codec(e) => write!(f, "{e:?}"),
        }
    }
}
impl std::error::Error for ModelError {}
fn strings(value: &Value, source: &str, catalog: &HashMap<String, Value>) -> Value {
    match value {
        Value::Array(a) => Value::Array(a.iter().map(|v| strings(v, source, catalog)).collect()),
        Value::Object(map) if crate::replay::is_input_ref(map) => {
            if let Some(text) = map.get("value").and_then(Value::as_str) {
                return Value::String(text.into());
            }
            let offset = map["offset"].as_u64().unwrap() as usize;
            let length = map["length"].as_u64().unwrap() as usize;
            Value::String(source[offset..].chars().take(length).collect())
        }
        Value::Object(map) => {
            let mut object = map
                .iter()
                .map(|(k, v)| (k.clone(), strings(v, source, catalog)))
                .collect::<serde_json::Map<String, Value>>();
            if let Some(spelling) = object
                .get("spelling")
                .and_then(Value::as_str)
                .map(str::to_string)
            {
                let stripped = spelling.trim_matches(|c| {
                    matches!(c, '\0' | '\t' | '\n' | '\x0b' | '\x0c' | '\r' | ' ')
                });
                let found = catalog.get(stripped);
                object.insert(
                    "value".into(),
                    found
                        .and_then(|v| v.get("value"))
                        .cloned()
                        .unwrap_or_else(|| Value::String(stripped.to_string())),
                );
                object.insert(
                    "kind".into(),
                    found
                        .and_then(|v| v.get("kind"))
                        .cloned()
                        .unwrap_or_else(|| Value::String("symbol".into())),
                );
            }
            Value::Object(object)
        }
        _ => value.clone(),
    }
}
impl Parser {
    pub fn parse_model(
        &self,
        input: &str,
        decimal: &str,
    ) -> Result<Latexmath_Model_Document, ModelError> {
        if crate::preprocess::call(input).is_empty() {
            if ![".", ",", "٫"].contains(&decimal) {
                return Err(ModelError::Parse(Error::UnsupportedDecimal(decimal.into())));
            }
            return json_codecs::read_Latexmath_Model_Document_json(
                &json!({"decimal":decimal}).to_string(),
            )
            .map_err(ModelError::Codec);
        }
        let parsed = self
            .parse_with_decimal(input, decimal)
            .map_err(ModelError::Parse)?;
        let catalog = serde_json::from_str(include_str!("../artifacts/catalog.json"))
            .expect("embedded stage catalog");
        let mut wire =
            json!({"expression":strings(&parsed.bound["model"],&parsed.parsed_input,&catalog)});
        wire.as_object_mut()
            .expect("declared bindings object")
            .insert("decimal".into(), Value::String(decimal.into()));
        json_codecs::read_Latexmath_Model_Document_json(&wire.to_string())
            .map_err(ModelError::Codec)
    }
}
pub fn to_portable_json(model: &Latexmath_Model_Document) -> Result<String, ModelError> {
    json_codecs::write_Latexmath_Model_Document_json(model).map_err(ModelError::Codec)
}
