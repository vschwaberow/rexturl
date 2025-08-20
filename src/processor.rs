use rayon::prelude::*;
use std::io;
use std::io::{BufRead, BufWriter, Write};

use crate::config::Config;
use crate::domain::extract_subdomain;
use crate::error::AppError;
use crate::output::{custom_format_url, output_json};
use crate::url_parser::{extract_url_components, parse_url};

pub fn process_url(config: &Config, url_str: &str) -> Option<String> {
    match parse_url(url_str) {
        Ok(url) => {
            let components = extract_url_components(&url);
            let mut parts = Vec::new();

            if config.host
                && !config.all
                && !config.domain
                && !config.scheme
                && !config.username
                && !config.port
                && !config.path
                && !config.query
                && !config.fragment
            {
                if !components.subdomain.is_empty() {
                    return Some(components.subdomain);
                }
                return None;
            }

            if config.all || config.scheme {
                parts.push(components.scheme);
            }

            if (config.all || config.username) && !components.username.is_empty() {
                parts.push(components.username);
            }

            if config.all && !components.subdomain.is_empty() {
                parts.push(components.subdomain.clone());
            }

            if config.all || config.host {
                if config.host && !config.all {
                    parts.push(components.subdomain);
                } else if config.all {
                    parts.push(components.hostname);
                }
            }

            if (config.all || config.port) && !components.port.is_empty() {
                parts.push(components.port);
            }

            if config.all || config.path {
                parts.push(components.path);
            }

            if (config.all || config.query) && !components.query.is_empty() {
                parts.push(components.query);
            }
            if (config.all || config.fragment) && !components.fragment.is_empty() {
                parts.push(components.fragment);
            }

            if config.all || config.domain {
                parts.push(components.domain);
            }

            if !parts.is_empty() {
                return Some(parts.join("\t"));
            }
        }
        Err(err) => {
            eprintln!("Error parsing URL '{url_str}': {err}");
        }
    }
    None
}

pub fn process_urls_parallel(config: &Config, urls: &[String]) -> Vec<String> {
    urls.par_iter()
        .filter_map(|url_str| process_url(config, url_str))
        .collect()
}

pub fn process_urls_streaming<R: BufRead>(config: &Config, reader: R) -> Result<(), AppError> {
    let mut results = Vec::new();

    for line in reader.lines().map_while(Result::ok) {
        let line = line.trim();
        if !line.is_empty() {
            if config.host
                && !config.all
                && !config.custom
                && !config.json
                && !config.domain
                && !config.scheme
                && !config.username
                && !config.port
                && !config.path
                && !config.query
                && !config.fragment
            {
                if let Ok(url) = parse_url(line) {
                    if let Some(host_str) = url.host_str() {
                        let subdomain = extract_subdomain(host_str);
                        if !subdomain.is_empty() {
                            results.push(subdomain);
                        } else {
                            results.push(host_str.to_string());
                        }
                    }
                }
            } else if config.custom && !config.json {
                if let Ok(output) = custom_format_url(
                    line,
                    config
                        .legacy_format
                        .as_ref()
                        .unwrap_or(&"{scheme}://{host}{path}".to_string()),
                ) {
                    if !output.is_empty() {
                        results.push(output);
                    }
                }
            } else if let Some(result) = process_url(config, line) {
                results.push(result);
            }
        }
    }

    if config.sort {
        results.sort();
    }
    if config.unique {
        results.dedup();
    }

    if config.json {
        output_json(&results)?;
    } else {
        let stdout = io::stdout();
        let mut writer = BufWriter::new(stdout.lock());
        for result in results {
            writeln!(writer, "{result}")?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_process_url() {
        let mut config = Config::parse_from([""]);
        config.host = true;

        let result = process_url(&config, "https://www.example.com");
        assert_eq!(result, Some("www".to_string()));

        config.host = false;
        config.domain = true;
        let result = process_url(&config, "https://www.example.com");
        assert_eq!(result, Some("example.com".to_string()));

        config.domain = false;
        config.all = true;
        let result = process_url(&config, "https://www.example.com");
        assert!(result.is_some());
    }
}
