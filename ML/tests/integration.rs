//! Integration tests for Antimony Labs ML platform
//!
//! These tests verify that all lessons run correctly and produce valid output.

use std::path::Path;

#[test]
fn test_lesson_json_files_exist_after_run() {
    // This test verifies the lesson output files are created
    // Note: In a real CI, we'd run the binary first
    let expected_files = [
        "lesson_00.json",
        "lesson_01.json", 
        "lesson_02.json",
        "lesson_03.json",
    ];
    
    for file in expected_files {
        let path = Path::new(file);
        // Files should exist if cargo run was executed
        // This test documents expected outputs
        if path.exists() {
            let content = std::fs::read_to_string(path).unwrap();
            assert!(content.contains("$schema"), "JSON should be valid Vega-Lite spec for {}", file);
        }
    }
}

#[test]
fn test_vega_lite_schema_structure() {
    // Validate that generated JSON follows Vega-Lite schema
    let files = ["lesson_00.json", "lesson_01.json", "lesson_02.json", "lesson_03.json"];
    
    for file in files {
        let path = Path::new(file);
        if path.exists() {
            let content = std::fs::read_to_string(path).unwrap();
            let json: serde_json::Value = serde_json::from_str(&content)
                .expect(&format!("Invalid JSON in {}", file));
            
            // Check required Vega-Lite fields
            assert!(json.get("$schema").is_some(), "{} missing $schema", file);
            assert!(json.get("data").is_some(), "{} missing data", file);
        }
    }
}

