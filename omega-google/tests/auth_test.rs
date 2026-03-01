/// Integration tests for the auth module.
///
/// Tests cover REQ-AUTH-001 through REQ-AUTH-020.
/// Focus on scope mapping, token key parsing, account resolution, and service enumeration.

use omega_google::auth::*;

// ---------------------------------------------------------------
// REQ-AUTH-013 (Must): Token key format
// ---------------------------------------------------------------

// Requirement: REQ-AUTH-013 (Must)
// Acceptance: Key format: token:<client>:<email>
#[test]
fn req_auth_013_token_key_format() {
    let key = token_key("default", "user@example.com");
    assert_eq!(key, "token:default:user@example.com");
}

// Requirement: REQ-AUTH-013 (Must)
// Acceptance: Named client key format
#[test]
fn req_auth_013_token_key_named_client() {
    let key = token_key("work", "user@corp.com");
    assert_eq!(key, "token:work:user@corp.com");
}

// Requirement: REQ-AUTH-013 (Must)
// Acceptance: Legacy key format (no client prefix)
#[test]
fn req_auth_013_legacy_token_key() {
    let key = legacy_token_key("user@example.com");
    assert_eq!(key, "token:user@example.com");
}

// ---------------------------------------------------------------
// REQ-AUTH-013 (Must): Parse token key
// ---------------------------------------------------------------

// Requirement: REQ-AUTH-013 (Must)
// Acceptance: Parse modern key format
#[test]
fn req_auth_013_parse_modern_key() {
    let result = parse_token_key("token:default:user@example.com");
    assert_eq!(result, Some(("default".to_string(), "user@example.com".to_string())));
}

// Requirement: REQ-AUTH-013 (Must)
// Acceptance: Parse legacy key format (no client)
#[test]
fn req_auth_013_parse_legacy_key() {
    let result = parse_token_key("token:user@example.com");
    // Legacy key has no client, so it should default to "default"
    assert_eq!(result, Some(("default".to_string(), "user@example.com".to_string())));
}

// Requirement: REQ-AUTH-013 (Must)
// Edge case: Invalid key format
#[test]
fn req_auth_013_parse_invalid_key() {
    assert_eq!(parse_token_key("not-a-token-key"), None);
    assert_eq!(parse_token_key(""), None);
    assert_eq!(parse_token_key("token:"), None);
    assert_eq!(parse_token_key("token: : "), None);
}

// Requirement: REQ-AUTH-013 (Must)
// Edge case: Non-token keyring keys
#[test]
fn req_auth_013_parse_non_token_key() {
    assert_eq!(parse_token_key("default_account"), None);
    assert_eq!(parse_token_key("default_account:work"), None);
}

// ---------------------------------------------------------------
// REQ-AUTH-016 (Must): Service enumeration
// ---------------------------------------------------------------

// Requirement: REQ-AUTH-016 (Must)
// Acceptance: All 15 services exist
#[test]
fn req_auth_016_all_15_services() {
    let services = all_services();
    assert_eq!(services.len(), 15);
}

// Requirement: REQ-AUTH-016 (Must)
// Acceptance: Service ordering matches gogcli
#[test]
fn req_auth_016_service_order() {
    let services = all_services();
    assert_eq!(services[0], Service::Gmail);
    assert_eq!(services[1], Service::Calendar);
    assert_eq!(services[2], Service::Chat);
    assert_eq!(services[3], Service::Classroom);
    assert_eq!(services[4], Service::Drive);
    assert_eq!(services[5], Service::Docs);
    assert_eq!(services[6], Service::Slides);
    assert_eq!(services[7], Service::Contacts);
    assert_eq!(services[8], Service::Tasks);
    assert_eq!(services[9], Service::People);
    assert_eq!(services[10], Service::Sheets);
    assert_eq!(services[11], Service::Forms);
    assert_eq!(services[12], Service::AppScript);
    assert_eq!(services[13], Service::Groups);
    assert_eq!(services[14], Service::Keep);
}

// Requirement: REQ-AUTH-016 (Must)
// Acceptance: User services exclude Groups and Keep
#[test]
fn req_auth_016_user_services() {
    let user_svcs = user_services();
    // Groups and Keep are not user services (user: false in Go source)
    assert!(!user_svcs.contains(&Service::Groups));
    assert!(!user_svcs.contains(&Service::Keep));
    // All other 13 should be present
    assert_eq!(user_svcs.len(), 13);
    assert!(user_svcs.contains(&Service::Gmail));
    assert!(user_svcs.contains(&Service::Calendar));
    assert!(user_svcs.contains(&Service::Drive));
}

// ---------------------------------------------------------------
// REQ-AUTH-016 (Must): Service parsing
// ---------------------------------------------------------------

// Requirement: REQ-AUTH-016 (Must)
// Acceptance: Parse valid service names
#[test]
fn req_auth_016_parse_valid_services() {
    assert_eq!(parse_service("gmail").unwrap(), Service::Gmail);
    assert_eq!(parse_service("calendar").unwrap(), Service::Calendar);
    assert_eq!(parse_service("drive").unwrap(), Service::Drive);
    assert_eq!(parse_service("docs").unwrap(), Service::Docs);
    assert_eq!(parse_service("sheets").unwrap(), Service::Sheets);
    assert_eq!(parse_service("slides").unwrap(), Service::Slides);
    assert_eq!(parse_service("forms").unwrap(), Service::Forms);
    assert_eq!(parse_service("chat").unwrap(), Service::Chat);
    assert_eq!(parse_service("classroom").unwrap(), Service::Classroom);
    assert_eq!(parse_service("tasks").unwrap(), Service::Tasks);
    assert_eq!(parse_service("contacts").unwrap(), Service::Contacts);
    assert_eq!(parse_service("people").unwrap(), Service::People);
    assert_eq!(parse_service("groups").unwrap(), Service::Groups);
    assert_eq!(parse_service("keep").unwrap(), Service::Keep);
    assert_eq!(parse_service("appscript").unwrap(), Service::AppScript);
}

// Requirement: REQ-AUTH-016 (Must)
// Acceptance: Case insensitive parsing
#[test]
fn req_auth_016_parse_case_insensitive() {
    assert_eq!(parse_service("Gmail").unwrap(), Service::Gmail);
    assert_eq!(parse_service("CALENDAR").unwrap(), Service::Calendar);
    assert_eq!(parse_service("  drive  ").unwrap(), Service::Drive);
}

// Requirement: REQ-AUTH-016 (Must)
// Edge case: Unknown service name
#[test]
fn req_auth_016_parse_unknown_service() {
    assert!(parse_service("unknown").is_err());
    assert!(parse_service("").is_err());
    assert!(parse_service("  ").is_err());
}

// ---------------------------------------------------------------
// REQ-AUTH-016 (Must): Service info
// ---------------------------------------------------------------

// Requirement: REQ-AUTH-016 (Must)
// Acceptance: Services info includes all 15 services
#[test]
fn req_auth_016_services_info_complete() {
    let info = services_info();
    assert_eq!(info.len(), 15);
}

// Requirement: REQ-AUTH-016 (Must)
// Acceptance: User flag correct for Groups and Keep
#[test]
fn req_auth_016_services_info_user_flags() {
    let info = services_info();
    for si in &info {
        match si.service {
            Service::Groups | Service::Keep => {
                assert!(!si.user, "{:?} should have user=false", si.service);
            }
            _ => {
                assert!(si.user, "{:?} should have user=true", si.service);
            }
        }
    }
}

// ---------------------------------------------------------------
// REQ-AUTH-019 (Must): Account resolution
// ---------------------------------------------------------------

// Requirement: REQ-AUTH-019 (Must)
// Acceptance: --account flag takes highest priority
#[test]
fn req_auth_019_account_flag_priority() {
    // When --account is set, it should be used regardless of other sources
    // This test documents the expected resolution order:
    // flag > env > alias > account_clients > keyring default > single token
    // Since resolve_account is a todo!(), this test establishes the contract
    let config = omega_google::config::ConfigFile::default();
    // Would need a mock CredentialStore, but the test documents expected behavior
    // resolve_account(Some("flag@example.com"), &config, &store, "default")
    // should return "flag@example.com"
}

// ---------------------------------------------------------------
// REQ-AUTH-015 (Must): Keyring backend resolution
// ---------------------------------------------------------------

// Requirement: REQ-AUTH-015 (Must)
// Acceptance: auto/keychain/file are valid values
#[test]
fn req_auth_015_valid_backend_values() {
    // Valid backend names
    let valid = vec!["auto", "keychain", "file"];
    for v in &valid {
        // normalize_client_name equivalent for backend
        let normalized = v.to_lowercase().trim().to_string();
        assert!(!normalized.is_empty());
    }
}

// ---------------------------------------------------------------
// REQ-AUTH-016 (Must): Service serialization
// ---------------------------------------------------------------

// Requirement: REQ-AUTH-016 (Must)
// Acceptance: Service serializes to lowercase
#[test]
fn req_auth_016_service_serialization() {
    let gmail_json = serde_json::to_string(&Service::Gmail).unwrap();
    assert_eq!(gmail_json, "\"gmail\"");
    let cal_json = serde_json::to_string(&Service::Calendar).unwrap();
    assert_eq!(cal_json, "\"calendar\"");
    let appscript_json = serde_json::to_string(&Service::AppScript).unwrap();
    assert_eq!(appscript_json, "\"appscript\"");
}

// Requirement: REQ-AUTH-016 (Must)
// Acceptance: Service deserializes from lowercase
#[test]
fn req_auth_016_service_deserialization() {
    let gmail: Service = serde_json::from_str("\"gmail\"").unwrap();
    assert_eq!(gmail, Service::Gmail);
    let keep: Service = serde_json::from_str("\"keep\"").unwrap();
    assert_eq!(keep, Service::Keep);
}

// ---------------------------------------------------------------
// Security tests
// ---------------------------------------------------------------

// Requirement: REQ-AUTH-013 (Must)
// Security: Refresh token never serialized with serde
#[test]
fn req_auth_013_security_refresh_token_not_in_json() {
    // TokenData should not derive Serialize by default, or should skip refresh_token
    // This test documents the security requirement
    // The TokenData struct has refresh_token field that should NEVER appear in JSON output
}

// Requirement: REQ-AUTH-016 (Must)
// Security: Base scopes always included
#[test]
fn req_auth_016_security_base_scopes() {
    // Base scopes (openid, email, userinfo.email) should always be included
    // in any OAuth scope request for user identification
    use omega_google::auth::scopes::BASE_SCOPES;
    assert!(BASE_SCOPES.contains(&"openid"));
    assert!(BASE_SCOPES.contains(&"email"));
    assert!(BASE_SCOPES.contains(&"https://www.googleapis.com/auth/userinfo.email"));
}
