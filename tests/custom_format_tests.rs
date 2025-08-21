use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_custom_format_basic() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com/path?query=value#fragment")
        .arg("--format")
        .arg("custom")
        .arg("--template")
        .arg("{scheme}://{domain}{path}");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("https://example.com/path"));
}

#[test]
fn test_custom_format_with_default() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com/path")
        .arg("--format")
        .arg("custom")
        .arg("--template")
        .arg("{scheme}://{domain}:{port:80}{path}");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("https://example.com:80/path"));
}

#[test]
fn test_custom_format_with_conditional() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com/path?query=value")
        .arg("--format")
        .arg("custom")
        .arg("--template")
        .arg("{domain}{query?&found}");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("example.com&found"));
}

#[test]
fn test_custom_format_shell_escape() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com/path with spaces")
        .arg("--format")
        .arg("custom")
        .arg("--template")
        .arg("{url}")
        .arg("--escape")
        .arg("shell");

    cmd.assert().success().stdout(predicate::str::contains(
        "'https://www.example.com/path with spaces'",
    ));
}

#[test]
fn test_custom_format_csv_escape() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com/path,with,commas")
        .arg("--format")
        .arg("custom")
        .arg("--template")
        .arg("{path}")
        .arg("--escape")
        .arg("csv");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"/path,with,commas\""));
}

#[test]
fn test_sql_format_basic() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com/path")
        .arg("--format")
        .arg("sql")
        .arg("--fields")
        .arg("domain,path");

    cmd.assert().success().stdout(predicate::str::contains(
        "INSERT INTO urls (domain, path) VALUES ('example.com', '/path');",
    ));
}

#[test]
fn test_sql_format_with_create_table() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com/path")
        .arg("--format")
        .arg("sql")
        .arg("--fields")
        .arg("domain,path")
        .arg("--sql-create-table");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("CREATE TABLE IF NOT EXISTS urls"))
        .stdout(predicate::str::contains("domain VARCHAR(253)"))
        .stdout(predicate::str::contains("path TEXT"))
        .stdout(predicate::str::contains("INSERT INTO urls"));
}

#[test]
fn test_sql_format_custom_table() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com/path")
        .arg("--format")
        .arg("sql")
        .arg("--fields")
        .arg("domain")
        .arg("--sql-table")
        .arg("my_urls");

    cmd.assert().success().stdout(predicate::str::contains(
        "INSERT INTO my_urls (domain) VALUES ('example.com');",
    ));
}

#[test]
fn test_sql_format_mysql_dialect() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com/path")
        .arg("--format")
        .arg("sql")
        .arg("--fields")
        .arg("port")
        .arg("--sql-create-table")
        .arg("--sql-dialect")
        .arg("mysql");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("port INT"));
}

#[test]
fn test_sql_format_sqlite_dialect() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com/path")
        .arg("--format")
        .arg("sql")
        .arg("--fields")
        .arg("domain")
        .arg("--sql-create-table")
        .arg("--sql-dialect")
        .arg("sqlite");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("domain TEXT"));
}

#[test]
fn test_sql_format_escaping() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com/path's")
        .arg("--format")
        .arg("sql")
        .arg("--fields")
        .arg("path");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("'/path''s'"));
}

#[test]
fn test_custom_format_invalid_field() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com/path")
        .arg("--format")
        .arg("custom")
        .arg("--template")
        .arg("{invalid_field}");

    cmd.assert().failure().stderr(predicate::str::contains(
        "Invalid field name: invalid_field",
    ));
}

#[test]
fn test_multiple_urls_custom_format() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com/path")
        .arg("https://api.test.com/v1")
        .arg("--format")
        .arg("custom")
        .arg("--template")
        .arg("{domain}");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("example.com"))
        .stdout(predicate::str::contains("test.com"));
}

#[test]
fn test_multiple_urls_sql_format() {
    let mut cmd = Command::cargo_bin("rexturl").unwrap();

    cmd.arg("--urls")
        .arg("https://www.example.com/path")
        .arg("https://api.test.com/v1")
        .arg("--format")
        .arg("sql")
        .arg("--fields")
        .arg("domain");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "INSERT INTO urls (domain) VALUES ('example.com');",
        ))
        .stdout(predicate::str::contains(
            "INSERT INTO urls (domain) VALUES ('test.com');",
        ));
}
