use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_url_file(urls: &[&str]) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    for url in urls {
        writeln!(file, "{url}").unwrap();
    }
    file
}

#[test]
fn test_new_format_plain() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com/path")
        .arg("--fields")
        .arg("domain,path")
        .arg("--format")
        .arg("plain");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("example.com /path"));
}

#[test]
fn test_new_format_tsv_with_header() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com/path")
        .arg("--fields")
        .arg("domain,path")
        .arg("--format")
        .arg("tsv")
        .arg("--header");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("domain\tpath"))
        .stdout(predicate::str::contains("example.com\t/path"));
}

#[test]
fn test_new_format_csv() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com/path")
        .arg("--fields")
        .arg("domain,path")
        .arg("--format")
        .arg("csv")
        .arg("--header");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("domain,path"))
        .stdout(predicate::str::contains("example.com,/path"));
}

#[test]
fn test_new_format_json() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com/path")
        .arg("--fields")
        .arg("domain")
        .arg("--format")
        .arg("json");

    cmd.assert().success().stdout(predicate::str::contains(
        r#"{"urls":[{"domain":"example.com"}]}"#,
    ));
}

#[test]
fn test_new_format_jsonl() {
    let urls = ["https://example.com", "https://test.com"];
    let file = create_url_file(&urls);

    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.pipe_stdin(file.path())
        .unwrap()
        .arg("--fields")
        .arg("domain")
        .arg("--format")
        .arg("jsonl");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"{"domain":"example.com"}"#))
        .stdout(predicate::str::contains(r#"{"domain":"test.com"}"#));
}

#[test]
fn test_fields_auto_detection() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com/path")
        .arg("--domain")
        .arg("--path");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("example.com"));
}

#[test]
fn test_backward_compatibility_json() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com")
        .arg("--domain")
        .arg("--json");

    cmd.assert().success().stdout(predicate::str::contains(
        r#"{"urls":[{"domain":"example.com"}]}"#,
    ));
}

#[test]
fn test_null_empty_values() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://example.com")
        .arg("--fields")
        .arg("domain,port")
        .arg("--format")
        .arg("tsv")
        .arg("--null-empty")
        .arg("NULL");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("example.com\tNULL"));
}

#[test]
fn test_sort_and_unique() {
    let urls = [
        "https://b.example.com",
        "https://a.example.com",
        "https://b.example.com",
    ];
    let file = create_url_file(&urls);

    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.pipe_stdin(file.path())
        .unwrap()
        .arg("--fields")
        .arg("subdomain")
        .arg("--sort")
        .arg("--unique");

    let output = cmd.output().unwrap();
    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<_> = output_str.lines().filter(|l| !l.is_empty()).collect();

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "a");
    assert_eq!(lines[1], "b");
}

#[test]
fn test_strict_mode() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("not-a-url")
        .arg("--strict")
        .arg("--fields")
        .arg("domain");

    cmd.assert().failure().code(2);
}

#[test]
fn test_no_newline() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://example.com")
        .arg("--fields")
        .arg("domain")
        .arg("--no-newline");

    let output = cmd.output().unwrap();
    let output_str = String::from_utf8_lossy(&output.stdout);

    assert!(!output_str.ends_with('\n'));
    assert_eq!(output_str, "example.com");
}

#[test]
fn test_complex_multipart_tld() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://blog.example.co.uk/posts")
        .arg("--fields")
        .arg("subdomain,domain,path")
        .arg("--format")
        .arg("tsv");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("blog\texample.co.uk\t/posts"));
}
