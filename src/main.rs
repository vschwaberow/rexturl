use clap::Parser;
use std::io::{self, BufRead};
use std::process;

use rexturl::formatter::{
    print_custom, print_json, print_jsonl, print_plain, print_sql, print_tabular, to_record,
    Format, UrlRecord,
};
use rexturl::{check_for_stdin, AppError, Config};

fn main() -> Result<(), AppError> {
    let config = Config::parse();

    if config.urls.is_empty() {
        check_for_stdin()?;
    }

    let format = if config.json {
        eprintln!("Warning: --json is deprecated, use --format json");
        Format::Json
    } else {
        config.format
    };

    let fields: Vec<&str> = if let Some(fields_str) = &config.fields {
        fields_str.split(',').map(|s| s.trim()).collect()
    } else if config.all {
        eprintln!("Warning: --all is deprecated, use --fields with specific field names");
        vec![
            "scheme",
            "username",
            "subdomain",
            "hostname",
            "port",
            "path",
            "query",
            "fragment",
            "domain",
        ]
    } else {
        let mut auto_fields = Vec::new();
        if config.scheme {
            auto_fields.push("scheme");
        }
        if config.username {
            auto_fields.push("username");
        }
        if config.host {
            auto_fields.push("subdomain");
        }
        if config.port {
            auto_fields.push("port");
        }
        if config.path {
            auto_fields.push("path");
        }
        if config.query {
            auto_fields.push("query");
        }
        if config.fragment {
            auto_fields.push("fragment");
        }
        if config.domain {
            auto_fields.push("domain");
        }

        if auto_fields.is_empty() {
            auto_fields.push("url");
        }
        auto_fields
    };

    let input_urls: Vec<String> = if !config.urls.is_empty() {
        config.urls
    } else {
        let stdin = io::stdin();
        stdin.lock().lines().filter_map(|line| line.ok()).collect()
    };

    let mut records: Vec<UrlRecord> = Vec::new();
    let mut parse_errors = 0;

    for url_str in input_urls {
        let url_str = url_str.trim();
        if url_str.is_empty() {
            continue;
        }

        match to_record(url_str) {
            Ok(record) => records.push(record),
            Err(_) => {
                parse_errors += 1;
                if config.strict {
                    eprintln!("Error: Failed to parse URL: {url_str}");
                }
            }
        }
    }

    if config.sort {
        if let Some(sort_field) = fields.first() {
            records.sort_by(|a, b| {
                let a_val = a.get_field(sort_field).unwrap_or("");
                let b_val = b.get_field(sort_field).unwrap_or("");
                a_val.cmp(b_val)
            });
        }
    }

    if config.unique {
        let mut seen = std::collections::HashSet::new();
        records.retain(|record| {
            let key: Vec<String> = fields
                .iter()
                .map(|field| record.get_field(field).unwrap_or("").to_string())
                .collect();
            seen.insert(key)
        });
    }

    match format {
        Format::Plain => print_plain(&records, &fields, &config.null_empty, config.no_newline),
        Format::Tsv => print_tabular(
            &records,
            &fields,
            config.header,
            '\t',
            &config.null_empty,
            config.no_newline,
        ),
        Format::Csv => print_tabular(
            &records,
            &fields,
            config.header,
            ',',
            &config.null_empty,
            config.no_newline,
        ),
        Format::Json => {
            if let Err(e) = print_json(&records, &fields, config.pretty, config.no_newline) {
                eprintln!("Error: Failed to serialize JSON: {e}");
                process::exit(1);
            }
        }
        Format::Jsonl => {
            if let Err(e) = print_jsonl(&records, &fields, config.no_newline) {
                eprintln!("Error: Failed to serialize JSONL: {e}");
                process::exit(1);
            }
        }
        Format::Custom => {
            let template = config.template.as_deref().unwrap_or("{url}");
            if let Err(e) = print_custom(&records, template, config.escape, config.no_newline) {
                eprintln!("Error: Failed to render custom format: {e}");
                process::exit(1);
            }
        }
        Format::Sql => {
            if let Err(e) = print_sql(
                &records,
                &fields,
                &config.sql_table,
                config.sql_dialect,
                config.sql_create_table,
                config.no_newline,
            ) {
                eprintln!("Error: Failed to generate SQL: {e}");
                process::exit(1);
            }
        }
    }

    if config.strict && parse_errors > 0 {
        process::exit(2);
    }

    Ok(())
}
