use std::path::PathBuf;
use std::process::Command;

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ctysearch"))
}

fn example_header() -> String {
    format!("{}/example.h", env!("CARGO_MANIFEST_DIR"))
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(binary())
        .args(args)
        .output()
        .expect("failed to run ctysearch")
}

fn stdout_lines(output: &std::process::Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

#[test]
fn exact_signature_match_is_ranked_first() {
    let header = example_header();
    let out = run(&["-f", &header, "uint8_t* (uint8_t)"]);
    assert!(out.status.success());
    let lines = stdout_lines(&out);
    assert!(!lines.is_empty());
    assert!(
        lines[0].contains("test :: uint8_t* (uint8_t)"),
        "unexpected first line: {}",
        lines[0]
    );
}

#[test]
fn void_search_finds_nullary_function_first() {
    let header = example_header();
    let out = run(&["-f", &header, "void ()"]);
    assert!(out.status.success());
    let lines = stdout_lines(&out);
    assert!(lines[0].contains("test14 :: void ()"), "got: {:?}", lines);
}

#[test]
fn limit_flag_caps_number_of_results() {
    let header = example_header();
    let out = run(&["-f", &header, "-l", "3", "Test_t* (Test_t*)"]);
    assert!(out.status.success());
    let lines = stdout_lines(&out);
    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains("test9 :: Test_t* (Test_t*)"));
}

#[test]
fn short_and_long_flags_agree() {
    let header = example_header();
    let short = run(&["-f", &header, "-l", "2", "void ()"]);
    let long = run(&[
        "--header",
        &header,
        "--limit",
        "2",
        "void ()",
    ]);
    assert!(short.status.success());
    assert!(long.status.success());
    assert_eq!(short.stdout, long.stdout);
}

#[test]
fn default_limit_returns_at_most_ten_lines() {
    let header = example_header();
    let out = run(&["-f", &header, "void ()"]);
    assert!(out.status.success());
    assert!(stdout_lines(&out).len() <= 10);
}

#[test]
fn fuzzy_search_still_finds_close_match() {
    // Intentionally slightly off: missing '*' and parens spacing.
    let header = example_header();
    let out = run(&["-f", &header, "Test_t (uint8_t, uint8_t)"]);
    assert!(out.status.success());
    let body = String::from_utf8_lossy(&out.stdout);
    assert!(
        body.contains("test2 :: Test_t (uint8_t, uint8_t*)")
            || body.contains("test6_t :: Test_t (uint8_t, uint8_t*)"),
        "got:\n{body}"
    );
}

#[test]
fn struct_pointer_search_finds_struct_functions() {
    let header = example_header();
    let out = run(&["-f", &header, "struct Test_t* (struct Test_t*)"]);
    assert!(out.status.success());
    let body = String::from_utf8_lossy(&out.stdout);
    assert!(
        body.contains("test12 ::") || body.contains("test10 ::"),
        "got:\n{body}"
    );
}

#[test]
fn missing_header_argument_fails() {
    let out = run(&["void ()"]);
    assert!(!out.status.success());
}

#[test]
fn missing_search_argument_fails() {
    let header = example_header();
    let out = run(&["-f", &header]);
    assert!(!out.status.success());
}

#[test]
fn nonexistent_header_fails() {
    let out = run(&["-f", "does-not-exist.h", "void ()"]);
    assert!(!out.status.success());
}

#[test]
fn help_flag_prints_usage() {
    let out = run(&["--help"]);
    assert!(out.status.success());
    let body = String::from_utf8_lossy(&out.stdout);
    assert!(body.contains("Usage: ctysearch"));
}
