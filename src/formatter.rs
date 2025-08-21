use clap::ValueEnum;
use serde::Serialize;
use std::str::FromStr;

use crate::{extract_url_components, parse_url};

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum EscapeMode {
    None,
    Shell,
    Csv,
    Json,
    Sql,
}

impl Default for EscapeMode {
    fn default() -> Self {
        EscapeMode::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum SqlDialect {
    Postgres,
    Mysql,
    Sqlite,
    Generic,
}

impl Default for SqlDialect {
    fn default() -> Self {
        SqlDialect::Postgres
    }
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum Format {
    Plain,
    Tsv,
    Csv,
    Json,
    Jsonl,
    Custom,
    Sql,
}

impl Default for Format {
    fn default() -> Self {
        Format::Plain
    }
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "plain" => Ok(Format::Plain),
            "tsv" => Ok(Format::Tsv),
            "csv" => Ok(Format::Csv),
            "json" => Ok(Format::Json),
            "jsonl" => Ok(Format::Jsonl),
            "custom" => Ok(Format::Custom),
            "sql" => Ok(Format::Sql),
            _ => Err(format!(
                "Invalid format: {s}. Valid formats: plain, tsv, csv, json, jsonl, custom, sql"
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UrlRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subdomain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fragment: Option<String>,
}

impl UrlRecord {
    pub fn new() -> Self {
        Self {
            url: None,
            scheme: None,
            username: None,
            host: None,
            hostname: None,
            subdomain: None,
            domain: None,
            port: None,
            path: None,
            query: None,
            fragment: None,
        }
    }

    pub fn get_field(&self, field: &str) -> Option<&str> {
        match field {
            "url" => self.url.as_deref(),
            "scheme" => self.scheme.as_deref(),
            "username" => self.username.as_deref(),
            "host" => self.host.as_deref(),
            "hostname" => self.hostname.as_deref(),
            "subdomain" => self.subdomain.as_deref(),
            "domain" => self.domain.as_deref(),
            "port" => self.port.as_deref(),
            "path" => self.path.as_deref(),
            "query" => self.query.as_deref(),
            "fragment" => self.fragment.as_deref(),
            _ => None,
        }
    }
}

fn select_fields(record: &UrlRecord, fields: &[&str], null_value: &str) -> Vec<String> {
    fields
        .iter()
        .map(|field| {
            record
                .get_field(field)
                .map(|v| v.to_string())
                .unwrap_or_else(|| null_value.to_string())
        })
        .collect()
}

pub fn print_plain(records: &[UrlRecord], fields: &[&str], null_value: &str, no_newline: bool) {
    for (i, record) in records.iter().enumerate() {
        let row = select_fields(record, fields, null_value);
        let line = row.join(" ");
        if no_newline && i == records.len() - 1 {
            print!("{line}");
        } else {
            println!("{line}");
        }
    }
}

pub fn print_tabular(
    records: &[UrlRecord],
    fields: &[&str],
    header: bool,
    separator: char,
    null_value: &str,
    no_newline: bool,
) {
    if header {
        let header_line = fields.join(&separator.to_string());
        println!("{header_line}");
    }

    for (i, record) in records.iter().enumerate() {
        let row = select_fields(record, fields, null_value);
        let line = row.join(&separator.to_string());
        if no_newline && i == records.len() - 1 {
            print!("{line}");
        } else {
            println!("{line}");
        }
    }
}

pub fn print_json(
    records: &[UrlRecord],
    fields: &[&str],
    pretty: bool,
    no_newline: bool,
) -> Result<(), serde_json::Error> {
    #[derive(Serialize)]
    struct UrlsWrapper {
        urls: Vec<serde_json::Value>,
    }

    let urls: Vec<serde_json::Value> = records
        .iter()
        .map(|record| {
            let mut map = serde_json::Map::new();
            for field in fields {
                if let Some(value) = record.get_field(field) {
                    map.insert(
                        field.to_string(),
                        serde_json::Value::String(value.to_string()),
                    );
                }
            }
            serde_json::Value::Object(map)
        })
        .collect();

    let wrapper = UrlsWrapper { urls };

    let output = if pretty {
        serde_json::to_string_pretty(&wrapper)?
    } else {
        serde_json::to_string(&wrapper)?
    };

    if no_newline {
        print!("{output}");
    } else {
        println!("{output}");
    }

    Ok(())
}

pub fn print_jsonl(
    records: &[UrlRecord],
    fields: &[&str],
    no_newline: bool,
) -> Result<(), serde_json::Error> {
    for (i, record) in records.iter().enumerate() {
        let mut map = serde_json::Map::new();
        for field in fields {
            if let Some(value) = record.get_field(field) {
                map.insert(
                    field.to_string(),
                    serde_json::Value::String(value.to_string()),
                );
            }
        }

        let line = serde_json::to_string(&serde_json::Value::Object(map))?;
        if no_newline && i == records.len() - 1 {
            print!("{line}");
        } else {
            println!("{line}");
        }
    }

    Ok(())
}

pub fn print_custom(
    records: &[UrlRecord],
    template: &str,
    escape_mode: EscapeMode,
    no_newline: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_template = parse_template(template)?;

    for (i, record) in records.iter().enumerate() {
        let output = render_template(&parsed_template, record, escape_mode);

        if no_newline && i == records.len() - 1 {
            print!("{output}");
        } else {
            println!("{output}");
        }
    }

    Ok(())
}

pub fn print_sql(
    records: &[UrlRecord],
    fields: &[&str],
    table_name: &str,
    dialect: SqlDialect,
    create_table: bool,
    no_newline: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if fields.is_empty() {
        return Err("SQL format requires at least one field to be specified".into());
    }

    if create_table {
        let create_sql = generate_create_table(table_name, fields, dialect);
        println!("{create_sql}");
    }

    for (i, record) in records.iter().enumerate() {
        let insert_sql = generate_insert_statement(record, fields, table_name, dialect);

        if no_newline && i == records.len() - 1 {
            print!("{insert_sql}");
        } else {
            println!("{insert_sql}");
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct TemplateToken {
    text: String,
    is_field: bool,
    field_name: Option<String>,
    default_value: Option<String>,
    conditional_present: Option<String>,
    conditional_missing: Option<String>,
}

fn parse_template(template: &str) -> Result<Vec<TemplateToken>, Box<dyn std::error::Error>> {
    let mut tokens = Vec::new();
    let mut current_text = String::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            if !current_text.is_empty() {
                tokens.push(TemplateToken {
                    text: current_text.clone(),
                    is_field: false,
                    field_name: None,
                    default_value: None,
                    conditional_present: None,
                    conditional_missing: None,
                });
                current_text.clear();
            }

            let mut field_spec = String::new();
            let mut brace_count = 1;

            while let Some(ch) = chars.next() {
                if ch == '{' {
                    brace_count += 1;
                    field_spec.push(ch);
                } else if ch == '}' {
                    brace_count -= 1;
                    if brace_count == 0 {
                        break;
                    }
                    field_spec.push(ch);
                } else {
                    field_spec.push(ch);
                }
            }

            let token = parse_field_spec(&field_spec)?;
            tokens.push(token);
        } else {
            current_text.push(ch);
        }
    }

    if !current_text.is_empty() {
        tokens.push(TemplateToken {
            text: current_text,
            is_field: false,
            field_name: None,
            default_value: None,
            conditional_present: None,
            conditional_missing: None,
        });
    }

    Ok(tokens)
}

fn parse_field_spec(spec: &str) -> Result<TemplateToken, Box<dyn std::error::Error>> {
    let field_name;
    let mut default_value = None;
    let mut conditional_present = None;
    let mut conditional_missing = None;

    if let Some((field_part, rest)) = spec.split_once(':') {
        field_name = field_part.to_string();
        default_value = Some(rest.to_string());
    } else if let Some((field_part, rest)) = spec.split_once('?') {
        field_name = field_part.to_string();
        conditional_present = Some(rest.to_string());
    } else if let Some((field_part, rest)) = spec.split_once('!') {
        field_name = field_part.to_string();
        conditional_missing = Some(rest.to_string());
    } else {
        field_name = spec.to_string();
    }

    if !is_valid_field_name(&field_name) {
        return Err(format!("Invalid field name: {field_name}").into());
    }

    Ok(TemplateToken {
        text: String::new(),
        is_field: true,
        field_name: Some(field_name),
        default_value,
        conditional_present,
        conditional_missing,
    })
}

fn is_valid_field_name(name: &str) -> bool {
    matches!(
        name,
        "url"
            | "scheme"
            | "username"
            | "host"
            | "hostname"
            | "subdomain"
            | "domain"
            | "port"
            | "path"
            | "query"
            | "fragment"
    )
}

fn render_template(
    tokens: &[TemplateToken],
    record: &UrlRecord,
    escape_mode: EscapeMode,
) -> String {
    let mut output = String::new();

    for token in tokens {
        if token.is_field {
            if let Some(field_name) = &token.field_name {
                let field_value = record.get_field(field_name);

                if let Some(value) = field_value {
                    if let Some(conditional_text) = &token.conditional_present {
                        output.push_str(conditional_text);
                    } else {
                        let escaped_value = escape_value(value, escape_mode);
                        output.push_str(&escaped_value);
                    }
                } else if let Some(conditional_text) = &token.conditional_missing {
                    output.push_str(conditional_text);
                } else if let Some(default_value) = &token.default_value {
                    let escaped_default = escape_value(default_value, escape_mode);
                    output.push_str(&escaped_default);
                }
            }
        } else {
            output.push_str(&token.text);
        }
    }

    output
}

fn escape_value(value: &str, mode: EscapeMode) -> String {
    match mode {
        EscapeMode::None => value.to_string(),
        EscapeMode::Shell => shell_escape(value),
        EscapeMode::Csv => csv_escape(value),
        EscapeMode::Json => json_escape(value),
        EscapeMode::Sql => sql_escape(value),
    }
}

fn shell_escape(value: &str) -> String {
    if value
        .chars()
        .any(|c| " \t\n\r\"'\\$`(){}[]|&;<>".contains(c))
    {
        format!("'{}'", value.replace('\'', "'\"'\"'"))
    } else {
        value.to_string()
    }
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn json_escape(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".to_string())
}

fn sql_escape(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn generate_create_table(table_name: &str, fields: &[&str], dialect: SqlDialect) -> String {
    let mut sql = format!("CREATE TABLE IF NOT EXISTS {} (\n", table_name);
    sql.push_str("    id SERIAL PRIMARY KEY,\n");

    for field in fields {
        let column_type = match dialect {
            SqlDialect::Postgres => get_postgres_column_type(field),
            SqlDialect::Mysql => get_mysql_column_type(field),
            SqlDialect::Sqlite => get_sqlite_column_type(field),
            SqlDialect::Generic => get_generic_column_type(field),
        };

        sql.push_str(&format!("    {} {},\n", field, column_type));
    }

    sql.push_str("    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP\n");
    sql.push_str(");");

    sql
}

fn get_postgres_column_type(field: &str) -> &'static str {
    match field {
        "url" => "VARCHAR(2048)",
        "scheme" => "VARCHAR(32)",
        "username" => "VARCHAR(255)",
        "hostname" | "subdomain" | "domain" => "VARCHAR(253)",
        "port" => "INTEGER",
        "path" => "TEXT",
        "query" => "TEXT",
        "fragment" => "VARCHAR(255)",
        _ => "TEXT",
    }
}

fn get_mysql_column_type(field: &str) -> &'static str {
    match field {
        "url" => "VARCHAR(2048)",
        "scheme" => "VARCHAR(32)",
        "username" => "VARCHAR(255)",
        "hostname" | "subdomain" | "domain" => "VARCHAR(253)",
        "port" => "INT",
        "path" => "TEXT",
        "query" => "TEXT",
        "fragment" => "VARCHAR(255)",
        _ => "TEXT",
    }
}

fn get_sqlite_column_type(field: &str) -> &'static str {
    match field {
        "port" => "INTEGER",
        _ => "TEXT",
    }
}

fn get_generic_column_type(field: &str) -> &'static str {
    match field {
        "port" => "INTEGER",
        _ => "TEXT",
    }
}

fn generate_insert_statement(
    record: &UrlRecord,
    fields: &[&str],
    table_name: &str,
    _dialect: SqlDialect,
) -> String {
    let field_names = fields.join(", ");
    let values: Vec<String> = fields
        .iter()
        .map(|field| {
            if let Some(value) = record.get_field(field) {
                sql_escape(value)
            } else {
                "NULL".to_string()
            }
        })
        .collect();

    format!(
        "INSERT INTO {} ({}) VALUES ({});",
        table_name,
        field_names,
        values.join(", ")
    )
}

pub fn to_record(input: &str) -> Result<UrlRecord, crate::UrlParseError> {
    let url = parse_url(input)?;
    let components = extract_url_components(&url);

    fn non_empty_string(s: String) -> Option<String> {
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }

    let path = if components.path.is_empty() || components.path == "/" {
        Some("/".to_string())
    } else {
        Some(components.path)
    };

    Ok(UrlRecord {
        url: Some(input.to_string()),
        scheme: non_empty_string(components.scheme),
        username: non_empty_string(components.username),
        host: non_empty_string(components.hostname.clone()),
        hostname: non_empty_string(components.hostname),
        subdomain: non_empty_string(components.subdomain),
        domain: non_empty_string(components.domain),
        port: non_empty_string(components.port),
        path,
        query: non_empty_string(components.query),
        fragment: non_empty_string(components.fragment),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> UrlRecord {
        UrlRecord {
            url: Some("https://www.example.com/path".to_string()),
            scheme: Some("https".to_string()),
            username: None,
            host: Some("www.example.com".to_string()),
            hostname: Some("www.example.com".to_string()),
            subdomain: Some("www".to_string()),
            domain: Some("example.com".to_string()),
            port: None,
            path: Some("/path".to_string()),
            query: None,
            fragment: None,
        }
    }

    fn create_test_record_with_all_fields() -> UrlRecord {
        UrlRecord {
            url: Some("https://user@api.example.com:8080/v1/users?limit=10#results".to_string()),
            scheme: Some("https".to_string()),
            username: Some("user".to_string()),
            host: Some("api.example.com".to_string()),
            hostname: Some("api.example.com".to_string()),
            subdomain: Some("api".to_string()),
            domain: Some("example.com".to_string()),
            port: Some("8080".to_string()),
            path: Some("/v1/users".to_string()),
            query: Some("limit=10".to_string()),
            fragment: Some("results".to_string()),
        }
    }

    #[test]
    fn test_format_parsing() {
        assert_eq!("plain".parse::<Format>().unwrap(), Format::Plain);
        assert_eq!("tsv".parse::<Format>().unwrap(), Format::Tsv);
        assert_eq!("csv".parse::<Format>().unwrap(), Format::Csv);
        assert_eq!("json".parse::<Format>().unwrap(), Format::Json);
        assert_eq!("jsonl".parse::<Format>().unwrap(), Format::Jsonl);
        assert_eq!("custom".parse::<Format>().unwrap(), Format::Custom);
        assert_eq!("sql".parse::<Format>().unwrap(), Format::Sql);
        assert!("invalid".parse::<Format>().is_err());
    }

    #[test]
    fn test_url_record_get_field() {
        let record = create_test_record();
        assert_eq!(record.get_field("domain"), Some("example.com"));
        assert_eq!(record.get_field("path"), Some("/path"));
        assert_eq!(record.get_field("port"), None);
        assert_eq!(record.get_field("unknown"), None);
    }

    #[test]
    fn test_select_fields() {
        let record = create_test_record();
        let fields = vec!["domain", "path", "port"];
        let result = select_fields(&record, &fields, "\\N");
        assert_eq!(result, vec!["example.com", "/path", "\\N"]);
    }

    #[test]
    fn test_parse_template_basic() {
        let template = "{scheme}://{domain}{path}";
        let tokens = parse_template(template).unwrap();
        assert_eq!(tokens.len(), 4);

        assert!(tokens[0].is_field);
        assert_eq!(tokens[0].field_name, Some("scheme".to_string()));

        assert!(!tokens[1].is_field);
        assert_eq!(tokens[1].text, "://");

        assert!(tokens[2].is_field);
        assert_eq!(tokens[2].field_name, Some("domain".to_string()));

        assert!(tokens[3].is_field);
        assert_eq!(tokens[3].field_name, Some("path".to_string()));
    }

    #[test]
    fn test_parse_template_with_default() {
        let template = "{port:80}";
        let tokens = parse_template(template).unwrap();
        assert_eq!(tokens.len(), 1);

        assert!(tokens[0].is_field);
        assert_eq!(tokens[0].field_name, Some("port".to_string()));
        assert_eq!(tokens[0].default_value, Some("80".to_string()));
    }

    #[test]
    fn test_parse_template_with_conditional() {
        let template = "{query?&found}";
        let tokens = parse_template(template).unwrap();
        assert_eq!(tokens.len(), 1);

        assert!(tokens[0].is_field);
        assert_eq!(tokens[0].field_name, Some("query".to_string()));
        assert_eq!(tokens[0].conditional_present, Some("&found".to_string()));
    }

    #[test]
    fn test_parse_template_invalid_field() {
        let template = "{invalid_field}";
        let result = parse_template(template);
        assert!(result.is_err());
    }

    #[test]
    fn test_render_template_basic() {
        let record = create_test_record();
        let template = "{scheme}://{domain}{path}";
        let tokens = parse_template(template).unwrap();
        let result = render_template(&tokens, &record, EscapeMode::None);
        assert_eq!(result, "https://example.com/path");
    }

    #[test]
    fn test_render_template_with_default() {
        let record = create_test_record();
        let template = "{port:80}";
        let tokens = parse_template(template).unwrap();
        let result = render_template(&tokens, &record, EscapeMode::None);
        assert_eq!(result, "80");
    }

    #[test]
    fn test_render_template_with_conditional() {
        let record = create_test_record_with_all_fields();
        let template = "{query?&found}";
        let tokens = parse_template(template).unwrap();
        let result = render_template(&tokens, &record, EscapeMode::None);
        assert_eq!(result, "&found");

        let record_no_query = create_test_record();
        let result2 = render_template(&tokens, &record_no_query, EscapeMode::None);
        assert_eq!(result2, "");
    }

    #[test]
    fn test_shell_escape() {
        assert_eq!(shell_escape("simple"), "simple");
        assert_eq!(shell_escape("with space"), "'with space'");
        assert_eq!(shell_escape("with'quote"), "'with'\"'\"'quote'");
        assert_eq!(shell_escape("with$dollar"), "'with$dollar'");
    }

    #[test]
    fn test_csv_escape() {
        assert_eq!(csv_escape("simple"), "simple");
        assert_eq!(csv_escape("with,comma"), "\"with,comma\"");
        assert_eq!(csv_escape("with\"quote"), "\"with\"\"quote\"");
        assert_eq!(csv_escape("with\nnewline"), "\"with\nnewline\"");
    }

    #[test]
    fn test_sql_escape() {
        assert_eq!(sql_escape("simple"), "'simple'");
        assert_eq!(sql_escape("with'quote"), "'with''quote'");
    }

    #[test]
    fn test_generate_create_table() {
        let fields = vec!["domain", "path", "port"];
        let sql = generate_create_table("test_table", &fields, SqlDialect::Postgres);

        assert!(sql.contains("CREATE TABLE IF NOT EXISTS test_table"));
        assert!(sql.contains("domain VARCHAR(253)"));
        assert!(sql.contains("path TEXT"));
        assert!(sql.contains("port INTEGER"));
        assert!(sql.contains("created_at TIMESTAMP"));
    }

    #[test]
    fn test_generate_insert_statement() {
        let record = create_test_record();
        let fields = vec!["domain", "path", "port"];
        let sql = generate_insert_statement(&record, &fields, "test_table", SqlDialect::Postgres);

        assert_eq!(
            sql,
            "INSERT INTO test_table (domain, path, port) VALUES ('example.com', '/path', NULL);"
        );
    }

    #[test]
    fn test_mysql_column_types() {
        assert_eq!(get_mysql_column_type("port"), "INT");
        assert_eq!(get_mysql_column_type("domain"), "VARCHAR(253)");
        assert_eq!(get_mysql_column_type("path"), "TEXT");
    }

    #[test]
    fn test_sqlite_column_types() {
        assert_eq!(get_sqlite_column_type("port"), "INTEGER");
        assert_eq!(get_sqlite_column_type("domain"), "TEXT");
        assert_eq!(get_sqlite_column_type("path"), "TEXT");
    }
}
