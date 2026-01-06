// Test to verify JSON escaping for webhook templates

/// Escape a string for safe inclusion in JSON (copied from webhook.rs for testing)
fn escape_json_string(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '"' => vec!['\\', '"'],
            '\\' => vec!['\\', '\\'],
            '\n' => vec!['\\', 'n'],
            '\r' => vec!['\\', 'r'],
            '\t' => vec!['\\', 't'],
            '\x08' => vec!['\\', 'b'],
            '\x0C' => vec!['\\', 'f'],
            c if c.is_control() => format!("\\u{:04x}", c as u32).chars().collect(),
            c => vec![c],
        })
        .collect()
}

#[test]
fn test_json_escaping_with_quotes() {
    let input = r#"test"with"quotes"#;
    let escaped = escape_json_string(input);

    // Build JSON manually to verify escaping
    let json = format!(r#"{{"name": "{}"}}"#, escaped);
    println!("Result: {}", json);

    // Should be valid JSON
    let parse_result = serde_json::from_str::<serde_json::Value>(&json);
    assert!(
        parse_result.is_ok(),
        "JSON should be valid after escaping quotes"
    );

    let parsed = parse_result.unwrap();
    assert_eq!(parsed["name"].as_str().unwrap(), input);
}

#[test]
fn test_json_escaping_with_newlines() {
    let input = "line1\nline2\nline3";
    let escaped = escape_json_string(input);

    let json = format!(r#"{{"description": "{}"}}"#, escaped);
    println!("Result with newlines: {}", json);

    let parse_result = serde_json::from_str::<serde_json::Value>(&json);
    assert!(
        parse_result.is_ok(),
        "JSON should be valid after escaping newlines"
    );

    let parsed = parse_result.unwrap();
    assert_eq!(parsed["description"].as_str().unwrap(), input);
}

#[test]
fn test_json_escaping_with_backslashes() {
    let input = r#"C:\Users\test\file.txt"#;
    let escaped = escape_json_string(input);

    let json = format!(r#"{{"path": "{}"}}"#, escaped);
    println!("Result with backslashes: {}", json);

    let parse_result = serde_json::from_str::<serde_json::Value>(&json);
    assert!(
        parse_result.is_ok(),
        "JSON should be valid after escaping backslashes"
    );

    let parsed = parse_result.unwrap();
    assert_eq!(parsed["path"].as_str().unwrap(), input);
}

#[test]
fn test_json_escaping_with_tabs() {
    let input = "test\twith\ttabs";
    let escaped = escape_json_string(input);

    let json = format!(r#"{{"value": "{}"}}"#, escaped);
    println!("Result with tabs: {}", json);

    let parse_result = serde_json::from_str::<serde_json::Value>(&json);
    assert!(
        parse_result.is_ok(),
        "JSON should be valid after escaping tabs"
    );

    let parsed = parse_result.unwrap();
    assert_eq!(parsed["value"].as_str().unwrap(), input);
}

#[test]
fn test_json_escaping_with_all_special_chars() {
    let input = "test\nwith\ttabs\"quotes\"and\\backslashes\rand\x08backspace";
    let escaped = escape_json_string(input);

    let json = format!(r#"{{"complex": "{}"}}"#, escaped);
    println!("Result with all special chars: {}", json);

    let parse_result = serde_json::from_str::<serde_json::Value>(&json);
    assert!(
        parse_result.is_ok(),
        "JSON should be valid after escaping all special characters"
    );

    let parsed = parse_result.unwrap();
    assert_eq!(parsed["complex"].as_str().unwrap(), input);
}
