//! Native byte reference for the focused WASM smoke inputs; run by WASM CI only.
use latexmath::{generated::json_codecs, typed, Parser};
use serde_json::{json, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Value = serde_json::from_str(include_str!("../tests/fixtures/wasm-smoke.json"))?;
    let parser = Parser::new()?;
    let mut rows = Vec::new();
    for case in cases.as_array().unwrap() {
        let input = case["input"].as_str().unwrap();
        let decimal = case["decimal"].as_str().unwrap();
        // Empty input has a valid model, but the raw LaTeX grammar refuses it.
        // Record only the fixture's declared refusal; other errors still fail CI.
        let (raw, raw_error) = match parser.parse_with_decimal(input, decimal) {
            Ok(parsed) => {
                assert_ne!(case["raw_refusal"], true, "expected raw refusal: {input:?}");
                (
                    Some(
                        json!({"preprocessed": parsed.parsed_input, "tree": parsed.canonical()})
                            .to_string(),
                    ),
                    None,
                )
            }
            Err(
                error @ latexmath::Error::Runtime(parsanol::PargError::ParseWire {
                    offset: 0, ..
                }),
            ) if input.is_empty() && case["raw_refusal"] == true => (None, Some(error.to_string())),
            Err(error) => return Err(error.into()),
        };
        let model = typed::to_portable_json(&parser.parse_model(input, decimal)?)?;
        let decoded = json_codecs::read_Latexmath_Model_Document_json(&model)?;
        let serialized = typed::to_portable_json(&decoded)?;
        rows.push(
            json!({"input": input, "decimal": decimal, "raw": raw, "raw_error": raw_error,
            "model": model, "serialized": serialized}),
        );
    }
    println!("{}", serde_json::to_string(&rows)?);
    Ok(())
}
