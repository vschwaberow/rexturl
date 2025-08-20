use serde::Serialize;

use crate::error::AppError;
use crate::url_parser::{extract_url_components, parse_url};

#[derive(Serialize)]
pub struct UrlsOutput {
    pub urls: Vec<String>,
}

pub fn output_json(results: &[String]) -> Result<(), AppError> {
    let output = UrlsOutput {
        urls: results.to_vec(),
    };

    let json_string = serde_json::to_string_pretty(&output)?;
    println!("{json_string}");
    Ok(())
}

pub fn custom_format_url(url_str: &str, format: &str) -> Result<String, AppError> {
    match parse_url(url_str) {
        Ok(url) => {
            let components = extract_url_components(&url);

            let output = format
                .replace("{scheme}", &components.scheme)
                .replace("{username}", &components.username)
                .replace("{subdomain}", &components.subdomain)
                .replace("{host}", &components.hostname)
                .replace("{hostname}", &components.hostname)
                .replace("{domain}", &components.domain)
                .replace("{port}", &components.port)
                .replace("{path}", &components.path)
                .replace("{query}", &components.query)
                .replace("{fragment}", &components.fragment);
            Ok(output)
        }
        Err(err) => {
            eprintln!("Error parsing URL '{url_str}': {err}");
            Ok(String::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_format_url() {
        let format = "{scheme}://{host}{path}";
        let result = custom_format_url("https://www.example.com/path", format).unwrap();
        assert_eq!(result, "https://www.example.com/path");

        let format = "{scheme}://{subdomain}.{domain}{path}";
        let result = custom_format_url("https://www.example.com/path", format).unwrap();
        assert_eq!(result, "https://www.example.com/path");

        let format = "{scheme}://{hostname}{path}?{query}#{fragment}";
        let result = custom_format_url("https://www.example.com/path?q=1#f", format).unwrap();
        assert_eq!(result, "https://www.example.com/path?q=1#f");

        let format = "{scheme}://{username}@{subdomain}.{domain}:{port}{path}?{query}#{fragment}";
        let result =
            custom_format_url("https://user@blog.example.com:8080/path?q=1#f", format).unwrap();
        assert_eq!(result, "https://user@blog.example.com:8080/path?q=1#f");
    }
}
