#![cfg(feature = "models")]
use latexmath::{
    generated::json_codecs::read_Latexmath_Model_Document_json, typed::to_portable_json, Parser,
};
#[test]
fn generated_model_symbol_and_codec_contract() {
    let parser = Parser::new().unwrap();
    for (input, decimal) in [
        ("\\alpha+1", "."),
        ("\\sqrt{x}", "."),
        ("\\frac{a}{b}", "."),
        ("1,5", ","),
        ("", "."),
    ] {
        let model = parser.parse_model(input, decimal).unwrap();
        let encoded = to_portable_json(&model).unwrap();
        let decoded = read_Latexmath_Model_Document_json(&encoded).unwrap();
        // The declared writer omits empty/nil members. Neither model identity nor
        // wire idempotence is assumed across that lossy boundary.
        decoded.validate().unwrap();
        assert_eq!(model.decimal, decoded.decimal, "{input} decimal");
        let json: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(json["decimal"], decimal);
        if input.is_empty() {
            assert!(json.get("expression").is_none());
        }
        if input == "\\alpha+1" {
            assert!(encoded.contains('α'));
            assert!(encoded.contains("symbol"));
        }
    }
    assert!(parser.parse_model("x", "bad").is_err());
}

#[test]
fn matrix_row_break_typed_model_has_one_row_break() {
    // Frozen stage rows latex-matrix-parens/-bars (testsuite 337262e, latex gen 3): the recorded
    // model has table_data.expression = {sequence: {row_break: "\\\\"}, expression: {symbols: b}}.
    let parser = Parser::new().unwrap();
    for env in ["pmatrix", "vmatrix"] {
        let input = format!("\\begin{{{env}}} a \\\\ b \\end{{{env}}}");
        let strict = parser.parse_with_decimal(&input, ".").unwrap();
        let legacy = parser.parse_legacy_with_decimal(&input, ".").unwrap();
        assert_eq!(
            strict.parsed_input, legacy.parsed_input,
            "{env} strict vs legacy preprocessing"
        );
        assert_eq!(
            strict.parsed_input,
            format!("\\begin{{{env}}}a\\\\b\\end{{{env}}}")
        );
        let json: serde_json::Value = serde_json::from_str(
            &to_portable_json(&parser.parse_model(&input, ".").unwrap()).unwrap(),
        )
        .unwrap();
        let rows = &json["expression"]["environment"]["table_data"];
        assert_eq!(rows["sequence"]["symbols"]["spelling"], "a");
        assert_eq!(rows["expression"]["sequence"]["row_break"], "\\\\");
        assert_eq!(rows["expression"]["expression"]["symbols"]["spelling"], "b");
        assert!(
            rows["expression"]["expression"].get("sequence").is_none(),
            "{env}: duplicated row break"
        );
    }
}
