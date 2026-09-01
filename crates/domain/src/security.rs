use std::net::IpAddr;
use crate::validation::ValidationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for SecuritySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "LOW"),
            Self::Medium => write!(f, "MEDIUM"),
            Self::High => write!(f, "HIGH"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

// 1. SSRF Protection: Blocks private IPs, loopback, link-local, and cloud metadata IPs
pub fn validate_public_destination_url(raw_url: &str) -> Result<String, ValidationError> {
    let parsed = url::Url::parse(raw_url.trim())
        .map_err(|_| ValidationError::InvalidFormat("url".to_string(), "Malformed URL".to_string()))?;

    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err(ValidationError::InvalidFormat("url".to_string(), "Only HTTP/HTTPS schemes permitted".to_string()));
    }

    if let Some(host_str) = parsed.host_str() {
        if host_str.eq_ignore_ascii_case("localhost") || host_str.ends_with(".internal") || host_str.ends_with(".local") {
            return Err(ValidationError::InvalidFormat("url".to_string(), "Access to internal hostnames is prohibited (SSRF Guard)".to_string()));
        }

        if let Ok(ip) = host_str.parse::<IpAddr>() {
            if is_private_or_reserved_ip(ip) {
                return Err(ValidationError::InvalidFormat("url".to_string(), "Access to private/reserved IP addresses is prohibited (SSRF Guard)".to_string()));
            }
        }
    }

    Ok(parsed.to_string())
}

fn is_private_or_reserved_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback() || v4.is_private() || v4.is_link_local() || v4.is_broadcast() || v4.is_documentation() || v4 == std::net::Ipv4Addr::new(169, 254, 169, 254) // Cloud metadata
        }
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_multicast(),
    }
}

// 2. Path Traversal & Storage Key Sanitizer
pub fn sanitize_storage_key(raw_key: &str) -> Result<String, ValidationError> {
    let trimmed = raw_key.trim();
    if trimmed.contains("..") || trimmed.contains('\\') || trimmed.starts_with('/') || trimmed.contains('\0') {
        return Err(ValidationError::InvalidFormat("storage_key".to_string(), "Invalid storage key (Path Traversal detected)".to_string()));
    }

    let is_valid = trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '-' || c == '_' || c == '.');
    if !is_valid {
        return Err(ValidationError::InvalidFormat("storage_key".to_string(), "Illegal characters in storage key".to_string()));
    }

    Ok(trimmed.to_string())
}