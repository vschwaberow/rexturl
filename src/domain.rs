pub const MULTI_PART_TLDS: &[&str] = &[
    "co.uk", "org.uk", "ac.uk", "gov.uk", "me.uk", "net.uk", "sch.uk", "com.au", "net.au",
    "org.au", "edu.au", "gov.au", "co.nz", "net.nz", "org.nz", "govt.nz", "co.za", "org.za",
    "com.br", "net.br", "org.br", "co.jp", "com.mx", "com.ar", "com.sg", "com.my", "co.id",
    "com.hk", "co.th", "in.th",
];

pub fn is_multi_part_tld(domain: &str) -> bool {
    MULTI_PART_TLDS
        .iter()
        .any(|tld| domain.ends_with(&format!(".{tld}")))
}

pub fn extract_domain(host: &str) -> String {
    if host.starts_with('[') && host.ends_with(']') {
        return String::new();
    }

    if host.parse::<std::net::Ipv4Addr>().is_ok() {
        return String::new();
    }

    let parts: Vec<&str> = host.split('.').collect();
    let parts_len = parts.len();

    if parts_len <= 2 {
        return host.to_string();
    }

    let potential_domain = parts[(parts_len - 3)..].join(".");
    if parts_len >= 3 && is_multi_part_tld(&potential_domain) {
        return potential_domain;
    }

    parts[(parts_len - 2)..].join(".")
}

pub fn extract_subdomain(host: &str) -> String {
    let domain = extract_domain(host);

    if host == domain {
        return "".to_string();
    }

    host.strip_suffix(&format!(".{domain}"))
        .or_else(|| host.strip_suffix(&domain))
        .unwrap_or(host)
        .trim_end_matches('.')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain_simple() {
        assert_eq!(extract_domain("example.com"), "example.com");
        assert_eq!(extract_domain("www.example.com"), "example.com");
        assert_eq!(extract_domain("blog.example.com"), "example.com");
    }

    #[test]
    fn test_extract_domain_multipart_tld() {
        assert_eq!(extract_domain("example.co.uk"), "example.co.uk");
        assert_eq!(extract_domain("www.example.co.uk"), "example.co.uk");
        assert_eq!(extract_domain("blog.example.co.uk"), "example.co.uk");
    }

    #[test]
    fn test_extract_subdomain() {
        assert_eq!(extract_subdomain("example.com"), "");
        assert_eq!(extract_subdomain("www.example.com"), "www");
        assert_eq!(extract_subdomain("blog.example.com"), "blog");
        assert_eq!(extract_subdomain("blog.dev.example.com"), "blog.dev");
    }

    #[test]
    fn test_extract_subdomain_multipart_tld() {
        assert_eq!(extract_subdomain("example.co.uk"), "");
        assert_eq!(extract_subdomain("www.example.co.uk"), "www");
        assert_eq!(extract_subdomain("blog.example.co.uk"), "blog");
    }
}
