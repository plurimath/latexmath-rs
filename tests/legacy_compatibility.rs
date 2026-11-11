use latexmath::Parser;
use serde_json::Value;
#[test]
fn frozen_raw_smoke_legacy_trees_and_offsets() {
    let parser = Parser::new().unwrap();
    let rows: Value = serde_json::from_str(include_str!("fixtures/raw-smoke.json")).unwrap();
    for row in rows.as_array().unwrap() {
        let input = row["input"]["str"].as_str().unwrap();
        let decimal = row["options"]["decimal"].as_str().unwrap_or(".");
        let result = parser.parse_legacy_with_decimal(input, decimal);
        if let Some(refusal) = row["expected"].get("refusal") {
            let error = result.unwrap_err();
            if let Some(offset) = refusal["byte_offset"].as_u64() {
                assert_eq!(error.byte_offset(), Some(offset as usize), "{}", row["id"]);
            }
        } else {
            let parsed = result.unwrap_or_else(|e| panic!("{}: {}", row["id"], e));
            assert_eq!(
                parsed.parsed_input,
                row["expected"]["preprocessed"]["str"].as_str().unwrap(),
                "{}",
                row["id"]
            );
            // Hash pair order and every slice byte/character offset are part
            // of the fixture. Do not sort or normalize away differences.
            assert_eq!(parsed.canonical(), row["expected"]["tree"], "{}", row["id"]);
        }
    }
}

#[test]
fn legacy_caller_offsets_and_preprocessed_offsets_are_distinct() {
    let parser = Parser::new().unwrap();
    let error = parser.parse_legacy("   ").unwrap_err();
    assert_eq!(error.byte_offset(), Some(3));
    assert_eq!(error.grammar_byte_offset(), Some(0));
    assert_eq!(parser.parse("   ").unwrap_err().byte_offset(), Some(0));
}
