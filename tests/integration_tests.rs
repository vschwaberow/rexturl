use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_url_file(urls: &[&str]) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    for url in urls {
        writeln!(file, "{}", url).unwrap();
    }
    file
}

#[test]
fn test_extract_host() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();
    
    cmd.arg("--urls")
       .arg("https://www.example.com")
       .arg("--host");
    
    cmd.assert()
       .success()
       .stdout(predicate::str::contains("www"));
}

#[test]
fn test_extract_domain() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();
    
    cmd.arg("--urls")
       .arg("https://www.example.com")
       .arg("--domain");
    
    cmd.assert()
       .success()
       .stdout(predicate::str::contains("example.com"));
}

#[test]
fn test_extract_multiple_components() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();
    
    cmd.arg("--urls")
       .arg("https://www.example.com")
       .arg("--scheme")
       .arg("--domain");
    
    cmd.assert()
       .success()
       .stdout(predicate::str::contains("https"))
       .stdout(predicate::str::contains("example.com"));
}

#[test]
fn test_custom_format() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();
    
    cmd.arg("--urls")
       .arg("https://www.example.com/path")
       .arg("--format")
       .arg("{scheme}://{domain}{path}");
    
    cmd.assert()
       .success()
       .stdout(predicate::str::contains("https://example.com/path"));
}

#[test]
fn test_json_output() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();
    
    cmd.arg("--urls")
       .arg("https://www.example.com")
       .arg("--json")
       .arg("--domain");
    
    cmd.assert()
       .success()
       .stdout(predicate::str::contains("\"urls\""))
       .stdout(predicate::str::contains("example.com"));
}

#[test]
fn test_sort_and_deduplicate() {
    let urls = ["https://b.example.com", "https://a.example.com", "https://b.example.com"];
    let file = create_url_file(&urls);
    
    let mut cmd = Command::cargo_bin("rexturl").unwrap();
    
    cmd.pipe_stdin(file.path()).unwrap()
       .arg("--host")
       .arg("--sort")
       .arg("--unique");
    
    let mut debug_cmd = Command::cargo_bin("rexturl").unwrap();
    debug_cmd.pipe_stdin(file.path()).unwrap()
        .arg("--host");
    let debug_output = debug_cmd.output().unwrap();
    let debug_str = String::from_utf8_lossy(&debug_output.stdout);
    println!("Debug host-only output: {:?}", debug_str);
    
    let output = cmd.output().unwrap();
    let output_str = String::from_utf8_lossy(&output.stdout);
    println!("Actual output: {:?}", output_str);
    
    cmd.assert().success();
    
    let lines: Vec<_> = output_str.lines().filter(|l| !l.is_empty()).collect();
    println!("Filtered lines: {:?}", lines);
    
    assert_eq!(lines.len(), 2, "Should have 2 lines after deduplication");
    assert!(lines[0].contains("a.example.com"), "First line should contain a.example.com");
    assert!(lines[1].contains("b.example.com"), "Second line should contain b.example.com");
}

#[test]
fn test_stdin_processing() {
    let urls = ["https://www.example.com", "https://blog.example.org"];
    let file = create_url_file(&urls);
    
    let mut cmd = Command::cargo_bin("rexturl").unwrap();
    
    cmd.pipe_stdin(file.path()).unwrap()
       .arg("--domain");
    
    cmd.assert()
       .success()
       .stdout(predicate::str::contains("example.com"))
       .stdout(predicate::str::contains("example.org"));
}

#[test]
fn test_multipart_tld() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();
    
    cmd.arg("--urls")
       .arg("https://www.example.co.uk")
       .arg("--domain");
    
    cmd.assert()
       .success()
       .stdout(predicate::str::contains("example.co.uk"));
}

#[test]
fn test_all_flag() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();
    
    cmd.arg("--urls")
       .arg("https://user@www.example.com:8080/path?q=1#f")
       .arg("--all");
    
    let output = cmd.output().unwrap();
    let output_str = String::from_utf8_lossy(&output.stdout);
    println!("All flag output: {:?}", output_str);
    
    cmd.assert()
       .success()
       .stdout(predicate::function(|s: &str| {
           s.contains("user") && 
           s.contains("8080") && 
           s.contains("www.example.com")
       }));
}
