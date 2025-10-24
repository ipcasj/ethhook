pub fn format_date(date_str: &str) -> String {
    // In a production app, you might want to use a date formatting library
    // This is a simple implementation that handles ISO strings like "2023-04-15T12:34:56Z"
    if date_str.len() >= 19 {
        // Extract just the date and time part (2023-04-15 12:34:56)
        let date = &date_str[0..10];
        let time = &date_str[11..19];
        format!("{date} {time}")
    } else {
        date_str.to_string()
    }
}

pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

pub fn truncate_hash(hash: &str, display_chars: usize) -> String {
    if hash.len() <= display_chars * 2 {
        return hash.to_string();
    }

    // For typical Ethereum hashes starting with 0x
    if hash.starts_with("0x") {
        return format!(
            "0x{}...{}",
            &hash[2..(2 + display_chars)],
            &hash[(hash.len() - display_chars)..]
        );
    }

    // For other types of hashes
    format!(
        "{}...{}",
        &hash[..display_chars],
        &hash[(hash.len() - display_chars)..]
    )
}

// ============================================================================
// Validation Functions
// ============================================================================

/// Validates that a URL starts with http:// or https://
pub fn is_valid_url(url: &str) -> bool {
    if url.is_empty() {
        return false;
    }
    url.starts_with("http://") || url.starts_with("https://")
}

/// Validates Ethereum address format (0x followed by 40 hexadecimal characters)
pub fn is_valid_eth_address(address: &str) -> bool {
    if address.len() != 42 {
        return false;
    }
    if !address.starts_with("0x") {
        return false;
    }
    // Check that remaining 40 characters are valid hex
    address[2..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Validates chain ID format (must be numeric)
pub fn is_valid_chain_id(chain_id: &str) -> bool {
    if chain_id.is_empty() {
        return false;
    }
    chain_id.chars().all(|c| c.is_ascii_digit())
}

/// Validates event signature format (EventName(type1,type2,...))
/// Examples: Transfer(address,address,uint256), Approval(address,address,uint256)
pub fn is_valid_event_signature(signature: &str) -> bool {
    if signature.is_empty() {
        return false;
    }

    // Must contain parentheses
    if !signature.contains('(') || !signature.contains(')') {
        return false;
    }

    // Event name must start with uppercase letter
    if let Some(first_char) = signature.chars().next() {
        if !first_char.is_ascii_uppercase() {
            return false;
        }
    } else {
        return false;
    }

    // Must end with ')'
    if !signature.ends_with(')') {
        return false;
    }

    // Basic structure: EventName(...)
    let parts: Vec<&str> = signature.split('(').collect();
    if parts.len() != 2 {
        return false;
    }

    // Event name should only contain alphanumeric characters
    let event_name = parts[0];
    if !event_name.chars().all(|c| c.is_alphanumeric()) {
        return false;
    }

    true
}

/// Validates string length is within range
pub fn is_valid_length(s: &str, min: usize, max: usize) -> bool {
    let len = s.len();
    len >= min && len <= max
}

/// Returns error message for invalid URL
pub fn url_error_message(url: &str) -> Option<String> {
    if url.is_empty() {
        return Some("URL is required".to_string());
    }
    if !is_valid_url(url) {
        return Some("URL must start with http:// or https://".to_string());
    }
    None
}

/// Returns error message for invalid Ethereum address
pub fn eth_address_error_message(address: &str) -> Option<String> {
    if address.is_empty() {
        return None; // Addresses are optional in some contexts
    }
    if !is_valid_eth_address(address) {
        return Some(
            "Invalid Ethereum address (must be 0x followed by 40 hex characters)".to_string(),
        );
    }
    None
}

/// Returns error message for invalid chain ID
pub fn chain_id_error_message(chain_id: &str) -> Option<String> {
    if chain_id.is_empty() {
        return Some("Chain ID is required".to_string());
    }
    if !is_valid_chain_id(chain_id) {
        return Some("Chain ID must be numeric (e.g., 1, 137, 11155111)".to_string());
    }
    None
}

/// Returns error message for invalid event signature
pub fn event_signature_error_message(signature: &str) -> Option<String> {
    if signature.is_empty() {
        return None; // Event signatures are optional
    }
    if !is_valid_event_signature(signature) {
        return Some("Invalid format (e.g., Transfer(address,address,uint256))".to_string());
    }
    None
}

/// Returns error message for invalid string length
pub fn length_error_message(field_name: &str, s: &str, min: usize, max: usize) -> Option<String> {
    let len = s.len();
    if len < min {
        return Some(format!("{field_name} must be at least {min} characters"));
    }
    if len > max {
        return Some(format!("{field_name} must be at most {max} characters"));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_url() {
        assert!(is_valid_url("http://example.com"));
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("https://webhook.site/abc-123"));
        assert!(!is_valid_url("ftp://example.com"));
        assert!(!is_valid_url("example.com"));
        assert!(!is_valid_url(""));
    }

    #[test]
    fn test_is_valid_eth_address() {
        assert!(is_valid_eth_address(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEbF"
        ));
        assert!(is_valid_eth_address(
            "0x0000000000000000000000000000000000000000"
        ));
        assert!(!is_valid_eth_address(
            "742d35Cc6634C0532925a3b844Bc9e7595f0bEbF"
        )); // Missing 0x
        assert!(!is_valid_eth_address(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bE"
        )); // Too short
        assert!(!is_valid_eth_address(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEbZ"
        )); // Invalid char
        assert!(!is_valid_eth_address(""));
    }

    #[test]
    fn test_is_valid_chain_id() {
        assert!(is_valid_chain_id("1"));
        assert!(is_valid_chain_id("137"));
        assert!(is_valid_chain_id("11155111"));
        assert!(!is_valid_chain_id("1.5"));
        assert!(!is_valid_chain_id("abc"));
        assert!(!is_valid_chain_id(""));
    }

    #[test]
    fn test_is_valid_event_signature() {
        assert!(is_valid_event_signature(
            "Transfer(address,address,uint256)"
        ));
        assert!(is_valid_event_signature(
            "Approval(address,address,uint256)"
        ));
        assert!(is_valid_event_signature(
            "Swap(address,uint256,uint256,uint256,uint256,address)"
        ));
        assert!(!is_valid_event_signature("transfer(address)")); // Lowercase
        assert!(!is_valid_event_signature("Transfer")); // No parentheses
        assert!(!is_valid_event_signature("Transfer("));
        assert!(!is_valid_event_signature(""));
    }

    #[test]
    fn test_is_valid_length() {
        assert!(is_valid_length("hello", 1, 10));
        assert!(is_valid_length("hello", 5, 5));
        assert!(!is_valid_length("hello", 6, 10));
        assert!(!is_valid_length("hello", 1, 4));
        assert!(is_valid_length("", 0, 10));
    }
}
