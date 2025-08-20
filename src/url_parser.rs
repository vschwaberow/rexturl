use crate::domain::{extract_domain, extract_subdomain};
use crate::url::{Url, UrlParseError};

pub struct UrlComponents {
    pub scheme: String,
    pub username: String,
    pub subdomain: String,
    pub domain: String,
    pub hostname: String,
    pub port: String,
    pub path: String,
    pub query: String,
    pub fragment: String,
}

pub fn parse_url(url_str: &str) -> Result<Url, UrlParseError> {
    let url_with_scheme = if !url_str.contains("://") {
        format!("https://{url_str}")
    } else {
        url_str.to_string()
    };

    Url::parse(&url_with_scheme)
}

pub fn extract_url_components(url: &Url) -> UrlComponents {
    let host_str = url.host();
    let domain = if !host_str.is_empty() {
        extract_domain(host_str)
    } else {
        String::new()
    };
    let subdomain = if !host_str.is_empty() {
        extract_subdomain(host_str)
    } else {
        String::new()
    };

    // Handle query formatting to match previous behavior
    let query = if let Some(q) = url.query() {
        q.to_string()
    } else {
        String::new()
    };

    // Handle fragment formatting to match previous behavior
    let fragment = if let Some(f) = url.fragment() {
        f.to_string()
    } else {
        String::new()
    };

    UrlComponents {
        scheme: url.scheme().to_string(),
        username: url.username().to_string(),
        subdomain,
        domain,
        hostname: host_str.to_string(),
        port: url.port().map_or(String::new(), |p| p.to_string()),
        path: url.path().to_string(),
        query,
        fragment,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        assert!(parse_url("example.com").is_ok());
        assert!(parse_url("https://example.com").is_ok());
        assert!(parse_url("http://example.com").is_ok());

        let url = parse_url("example.com").unwrap();
        assert_eq!(url.scheme(), "https");
    }

    #[test]
    fn test_extract_url_components() {
        let url =
            parse_url("https://user@www.example.co.uk:8080/path?query=value#fragment").unwrap();
        let components = extract_url_components(&url);

        assert_eq!(components.scheme, "https");
        assert_eq!(components.username, "user");
        assert_eq!(components.hostname, "www.example.co.uk");
        assert_eq!(components.subdomain, "www");
        assert_eq!(components.domain, "example.co.uk");
        assert_eq!(components.port, "8080");
        assert_eq!(components.path, "/path");
        assert_eq!(components.query, "query=value");
        assert_eq!(components.fragment, "fragment");
    }
}
