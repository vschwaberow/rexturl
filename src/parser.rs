use crate::domain::{extract_domain, extract_subdomain};
use crate::url::{Url, UrlParseError};

#[derive(Debug, Clone)]
pub struct UrlComponents {
    pub scheme: String,
    pub username: String,
    pub subdomain: String,
    pub hostname: String,
    pub domain: String,
    pub port: String,
    pub path: String,
    pub query: String,
    pub fragment: String,
}

pub fn parse_url(url_str: &str) -> Result<Url, UrlParseError> {
    Url::parse(url_str)
}

pub fn extract_url_components(url: &Url) -> UrlComponents {
    let hostname = url.host();
    let subdomain = extract_subdomain(hostname);
    let domain = extract_domain(hostname);

    let port = if let Some(port_num) = url.port() {
        port_num.to_string()
    } else {
        String::new()
    };

    let path = {
        let raw_path = url.path();
        if raw_path.is_empty() || !raw_path.starts_with('/') {
            format!("/{raw_path}")
        } else {
            raw_path.to_string()
        }
    };

    let query = if let Some(q) = url.query() {
        if q.is_empty() {
            String::new()
        } else {
            format!("?{q}")
        }
    } else {
        String::new()
    };

    let fragment = if let Some(f) = url.fragment() {
        if f.is_empty() {
            String::new()
        } else {
            format!("#{f}")
        }
    } else {
        String::new()
    };

    UrlComponents {
        scheme: url.scheme().to_string(),
        username: url.username().to_string(),
        subdomain,
        hostname: hostname.to_string(),
        domain,
        port,
        path,
        query,
        fragment,
    }
}

pub fn parse_and_extract_components(url_str: &str) -> Result<UrlComponents, UrlParseError> {
    let url = parse_url(url_str)?;
    Ok(extract_url_components(&url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_components_simple() {
        let url = parse_url("https://www.example.com").unwrap();
        let components = extract_url_components(&url);

        assert_eq!(components.scheme, "https");
        assert_eq!(components.username, "");
        assert_eq!(components.subdomain, "www");
        assert_eq!(components.hostname, "www.example.com");
        assert_eq!(components.domain, "example.com");
        assert_eq!(components.port, "");
        assert_eq!(components.path, "/");
        assert_eq!(components.query, "");
        assert_eq!(components.fragment, "");
    }

    #[test]
    fn test_extract_components_complex() {
        let url = parse_url(
            "https://user@blog.example.com:8080/path/to/page?param=value&other=test#section",
        )
        .unwrap();
        let components = extract_url_components(&url);

        assert_eq!(components.scheme, "https");
        assert_eq!(components.username, "user");
        assert_eq!(components.subdomain, "blog");
        assert_eq!(components.hostname, "blog.example.com");
        assert_eq!(components.domain, "example.com");
        assert_eq!(components.port, "8080");
        assert_eq!(components.path, "/path/to/page");
        assert_eq!(components.query, "?param=value&other=test");
        assert_eq!(components.fragment, "#section");
    }

    #[test]
    fn test_extract_components_multipart_tld() {
        let url = parse_url("https://www.example.co.uk/path").unwrap();
        let components = extract_url_components(&url);

        assert_eq!(components.scheme, "https");
        assert_eq!(components.subdomain, "www");
        assert_eq!(components.hostname, "www.example.co.uk");
        assert_eq!(components.domain, "example.co.uk");
        assert_eq!(components.path, "/path");
    }

    #[test]
    fn test_extract_components_no_subdomain() {
        let url = parse_url("https://example.com").unwrap();
        let components = extract_url_components(&url);

        assert_eq!(components.scheme, "https");
        assert_eq!(components.subdomain, "");
        assert_eq!(components.hostname, "example.com");
        assert_eq!(components.domain, "example.com");
    }

    #[test]
    fn test_parse_and_extract_integration() {
        let components =
            parse_and_extract_components("https://api.example.com:443/v1/users?limit=10#results")
                .unwrap();

        assert_eq!(components.scheme, "https");
        assert_eq!(components.subdomain, "api");
        assert_eq!(components.hostname, "api.example.com");
        assert_eq!(components.domain, "example.com");
        assert_eq!(components.port, "443");
        assert_eq!(components.path, "/v1/users");
        assert_eq!(components.query, "?limit=10");
        assert_eq!(components.fragment, "#results");
    }

    #[test]
    fn test_edge_cases() {
        let components = parse_and_extract_components("https://example.com").unwrap();
        assert_eq!(components.path, "/");

        let components = parse_and_extract_components("https://example.com/").unwrap();
        assert_eq!(components.path, "/");

        let components = parse_and_extract_components("https://example.com?").unwrap();
        assert_eq!(components.query, "");

        let components = parse_and_extract_components("https://example.com#").unwrap();
        assert_eq!(components.fragment, "");
    }
}
