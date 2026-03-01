use super::{DriveScopeMode, ScopeOptions, Service};

/// Base scopes always included in OAuth requests.
pub const BASE_SCOPES: &[&str] = &[
    "openid",
    "email",
    "https://www.googleapis.com/auth/userinfo.email",
];

/// Returns the default scopes for a service.
pub fn scopes_for_service(service: Service) -> Vec<String> {
    match service {
        Service::Gmail => vec![
            "https://www.googleapis.com/auth/gmail.modify".into(),
            "https://www.googleapis.com/auth/gmail.settings.basic".into(),
            "https://www.googleapis.com/auth/gmail.settings.sharing".into(),
        ],
        Service::Calendar => vec![
            "https://www.googleapis.com/auth/calendar".into(),
        ],
        Service::Chat => vec![
            "https://www.googleapis.com/auth/chat.spaces".into(),
            "https://www.googleapis.com/auth/chat.messages".into(),
            "https://www.googleapis.com/auth/chat.memberships".into(),
            "https://www.googleapis.com/auth/chat.users.readstate.readonly".into(),
        ],
        Service::Classroom => vec![
            "https://www.googleapis.com/auth/classroom.courses".into(),
            "https://www.googleapis.com/auth/classroom.rosters".into(),
            "https://www.googleapis.com/auth/classroom.profile.emails".into(),
            "https://www.googleapis.com/auth/classroom.profile.photos".into(),
            "https://www.googleapis.com/auth/classroom.coursework.students".into(),
            "https://www.googleapis.com/auth/classroom.courseworkmaterials".into(),
            "https://www.googleapis.com/auth/classroom.topics".into(),
            "https://www.googleapis.com/auth/classroom.announcements".into(),
            "https://www.googleapis.com/auth/classroom.guardianlinks.students".into(),
            "https://www.googleapis.com/auth/classroom.push-notifications".into(),
        ],
        Service::Drive => vec![
            "https://www.googleapis.com/auth/drive".into(),
        ],
        Service::Docs => vec![
            "https://www.googleapis.com/auth/drive".into(),
            "https://www.googleapis.com/auth/documents".into(),
        ],
        Service::Slides => vec![
            "https://www.googleapis.com/auth/drive".into(),
            "https://www.googleapis.com/auth/presentations".into(),
        ],
        Service::Contacts => vec![
            "https://www.googleapis.com/auth/contacts".into(),
            "https://www.googleapis.com/auth/contacts.other.readonly".into(),
            "https://www.googleapis.com/auth/directory.readonly".into(),
        ],
        Service::Tasks => vec![
            "https://www.googleapis.com/auth/tasks".into(),
        ],
        Service::People => vec![
            "profile".into(),
        ],
        Service::Sheets => vec![
            "https://www.googleapis.com/auth/drive".into(),
            "https://www.googleapis.com/auth/spreadsheets".into(),
        ],
        Service::Forms => vec![
            "https://www.googleapis.com/auth/forms.body".into(),
            "https://www.googleapis.com/auth/forms.responses.readonly".into(),
        ],
        Service::AppScript => vec![
            "https://www.googleapis.com/auth/script.projects".into(),
            "https://www.googleapis.com/auth/script.deployments".into(),
            "https://www.googleapis.com/auth/script.processes".into(),
        ],
        Service::Groups => vec![
            "https://www.googleapis.com/auth/cloud-identity.groups.readonly".into(),
        ],
        Service::Keep => vec![
            "https://www.googleapis.com/auth/keep.readonly".into(),
        ],
    }
}

/// Helper: resolve the drive scope string based on options.
fn resolve_drive_scope(opts: &ScopeOptions) -> String {
    if opts.readonly {
        "https://www.googleapis.com/auth/drive.readonly".into()
    } else {
        match opts.drive_scope {
            DriveScopeMode::Full => "https://www.googleapis.com/auth/drive".into(),
            DriveScopeMode::Readonly => "https://www.googleapis.com/auth/drive.readonly".into(),
            DriveScopeMode::File => "https://www.googleapis.com/auth/drive.file".into(),
        }
    }
}

/// Returns scopes for a service with options (readonly, drive scope mode).
pub fn scopes_for_service_with_options(service: Service, opts: &ScopeOptions) -> anyhow::Result<Vec<String>> {
    if !opts.readonly && opts.drive_scope == DriveScopeMode::Full {
        // No options that change anything -- return defaults
        return Ok(scopes_for_service(service));
    }

    let scopes = match service {
        Service::Gmail => {
            if opts.readonly {
                vec!["https://www.googleapis.com/auth/gmail.readonly".into()]
            } else {
                scopes_for_service(service)
            }
        }
        Service::Calendar => {
            if opts.readonly {
                vec!["https://www.googleapis.com/auth/calendar.readonly".into()]
            } else {
                scopes_for_service(service)
            }
        }
        Service::Chat => {
            if opts.readonly {
                vec![
                    "https://www.googleapis.com/auth/chat.spaces.readonly".into(),
                    "https://www.googleapis.com/auth/chat.messages.readonly".into(),
                    "https://www.googleapis.com/auth/chat.memberships.readonly".into(),
                    "https://www.googleapis.com/auth/chat.users.readstate.readonly".into(),
                ]
            } else {
                scopes_for_service(service)
            }
        }
        Service::Classroom => {
            if opts.readonly {
                vec![
                    "https://www.googleapis.com/auth/classroom.courses.readonly".into(),
                    "https://www.googleapis.com/auth/classroom.rosters.readonly".into(),
                    "https://www.googleapis.com/auth/classroom.profile.emails".into(),
                    "https://www.googleapis.com/auth/classroom.profile.photos".into(),
                    "https://www.googleapis.com/auth/classroom.coursework.students.readonly".into(),
                    "https://www.googleapis.com/auth/classroom.courseworkmaterials.readonly".into(),
                    "https://www.googleapis.com/auth/classroom.topics.readonly".into(),
                    "https://www.googleapis.com/auth/classroom.announcements.readonly".into(),
                    "https://www.googleapis.com/auth/classroom.guardianlinks.students.readonly".into(),
                    "https://www.googleapis.com/auth/classroom.push-notifications".into(),
                ]
            } else {
                scopes_for_service(service)
            }
        }
        Service::Drive => {
            vec![resolve_drive_scope(opts)]
        }
        Service::Docs => {
            let drive = resolve_drive_scope(opts);
            let docs = if opts.readonly {
                "https://www.googleapis.com/auth/documents.readonly".into()
            } else {
                "https://www.googleapis.com/auth/documents".into()
            };
            vec![drive, docs]
        }
        Service::Slides => {
            let drive = resolve_drive_scope(opts);
            let slides = if opts.readonly {
                "https://www.googleapis.com/auth/presentations.readonly".into()
            } else {
                "https://www.googleapis.com/auth/presentations".into()
            };
            vec![drive, slides]
        }
        Service::Contacts => {
            if opts.readonly {
                vec![
                    "https://www.googleapis.com/auth/contacts.readonly".into(),
                    "https://www.googleapis.com/auth/contacts.other.readonly".into(),
                    "https://www.googleapis.com/auth/directory.readonly".into(),
                ]
            } else {
                scopes_for_service(service)
            }
        }
        Service::Tasks => {
            if opts.readonly {
                vec!["https://www.googleapis.com/auth/tasks.readonly".into()]
            } else {
                scopes_for_service(service)
            }
        }
        Service::People => {
            // Profile is already read-only
            vec!["profile".into()]
        }
        Service::Sheets => {
            let drive = resolve_drive_scope(opts);
            let sheets = if opts.readonly {
                "https://www.googleapis.com/auth/spreadsheets.readonly".into()
            } else {
                "https://www.googleapis.com/auth/spreadsheets".into()
            };
            vec![drive, sheets]
        }
        Service::Forms => {
            if opts.readonly {
                vec![
                    "https://www.googleapis.com/auth/forms.body.readonly".into(),
                    "https://www.googleapis.com/auth/forms.responses.readonly".into(),
                ]
            } else {
                scopes_for_service(service)
            }
        }
        Service::AppScript => {
            if opts.readonly {
                vec![
                    "https://www.googleapis.com/auth/script.projects.readonly".into(),
                    "https://www.googleapis.com/auth/script.deployments.readonly".into(),
                ]
            } else {
                scopes_for_service(service)
            }
        }
        Service::Groups => {
            // Already readonly
            scopes_for_service(service)
        }
        Service::Keep => {
            // Already readonly
            scopes_for_service(service)
        }
    };
    Ok(scopes)
}

/// Returns combined scopes for multiple services, deduplicated and sorted.
pub fn scopes_for_services(services: &[Service]) -> anyhow::Result<Vec<String>> {
    let opts = ScopeOptions::default();
    let mut all_scopes = Vec::new();
    for &svc in services {
        all_scopes.extend(scopes_for_service_with_options(svc, &opts)?);
    }
    all_scopes.sort();
    all_scopes.dedup();
    Ok(all_scopes)
}

/// Returns combined scopes for multiple services with options, plus base scopes.
pub fn scopes_for_manage(services: &[Service], opts: &ScopeOptions) -> anyhow::Result<Vec<String>> {
    let mut all_scopes: Vec<String> = BASE_SCOPES.iter().map(|s| s.to_string()).collect();
    for &svc in services {
        all_scopes.extend(scopes_for_service_with_options(svc, opts)?);
    }
    all_scopes.sort();
    all_scopes.dedup();
    Ok(all_scopes)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-AUTH-016 (Must): Per-service OAuth scope mapping
    // ---------------------------------------------------------------

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: 15 services with correct default scopes
    #[test]
    fn req_auth_016_gmail_default_scopes() {
        let scopes = scopes_for_service(Service::Gmail);
        assert!(scopes.contains(&"https://www.googleapis.com/auth/gmail.modify".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/gmail.settings.basic".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/gmail.settings.sharing".to_string()));
        assert_eq!(scopes.len(), 3);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Calendar default scope
    #[test]
    fn req_auth_016_calendar_default_scope() {
        let scopes = scopes_for_service(Service::Calendar);
        assert_eq!(scopes, vec!["https://www.googleapis.com/auth/calendar"]);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Chat default scopes
    #[test]
    fn req_auth_016_chat_default_scopes() {
        let scopes = scopes_for_service(Service::Chat);
        assert!(scopes.contains(&"https://www.googleapis.com/auth/chat.spaces".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/chat.messages".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/chat.memberships".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/chat.users.readstate.readonly".to_string()));
        assert_eq!(scopes.len(), 4);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Classroom default scopes (10 scopes)
    #[test]
    fn req_auth_016_classroom_default_scopes() {
        let scopes = scopes_for_service(Service::Classroom);
        assert_eq!(scopes.len(), 10);
        assert!(scopes.contains(&"https://www.googleapis.com/auth/classroom.courses".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/classroom.rosters".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/classroom.profile.emails".to_string()));
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Drive default scope (full access)
    #[test]
    fn req_auth_016_drive_default_scope() {
        let scopes = scopes_for_service(Service::Drive);
        assert_eq!(scopes, vec!["https://www.googleapis.com/auth/drive"]);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Docs requires both drive and documents scopes
    #[test]
    fn req_auth_016_docs_default_scopes() {
        let scopes = scopes_for_service(Service::Docs);
        assert!(scopes.contains(&"https://www.googleapis.com/auth/drive".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/documents".to_string()));
        assert_eq!(scopes.len(), 2);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Slides requires both drive and presentations scopes
    #[test]
    fn req_auth_016_slides_default_scopes() {
        let scopes = scopes_for_service(Service::Slides);
        assert!(scopes.contains(&"https://www.googleapis.com/auth/drive".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/presentations".to_string()));
        assert_eq!(scopes.len(), 2);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Contacts has contacts + other + directory scopes
    #[test]
    fn req_auth_016_contacts_default_scopes() {
        let scopes = scopes_for_service(Service::Contacts);
        assert!(scopes.contains(&"https://www.googleapis.com/auth/contacts".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/contacts.other.readonly".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/directory.readonly".to_string()));
        assert_eq!(scopes.len(), 3);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Tasks default scope
    #[test]
    fn req_auth_016_tasks_default_scope() {
        let scopes = scopes_for_service(Service::Tasks);
        assert_eq!(scopes, vec!["https://www.googleapis.com/auth/tasks"]);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: People uses OIDC profile scope
    #[test]
    fn req_auth_016_people_default_scope() {
        let scopes = scopes_for_service(Service::People);
        assert_eq!(scopes, vec!["profile"]);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Sheets requires drive and spreadsheets scopes
    #[test]
    fn req_auth_016_sheets_default_scopes() {
        let scopes = scopes_for_service(Service::Sheets);
        assert!(scopes.contains(&"https://www.googleapis.com/auth/drive".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/spreadsheets".to_string()));
        assert_eq!(scopes.len(), 2);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Forms default scopes
    #[test]
    fn req_auth_016_forms_default_scopes() {
        let scopes = scopes_for_service(Service::Forms);
        assert!(scopes.contains(&"https://www.googleapis.com/auth/forms.body".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/forms.responses.readonly".to_string()));
        assert_eq!(scopes.len(), 2);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: AppScript default scopes
    #[test]
    fn req_auth_016_appscript_default_scopes() {
        let scopes = scopes_for_service(Service::AppScript);
        assert!(scopes.contains(&"https://www.googleapis.com/auth/script.projects".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/script.deployments".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/script.processes".to_string()));
        assert_eq!(scopes.len(), 3);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Groups scope (cloud identity)
    #[test]
    fn req_auth_016_groups_default_scope() {
        let scopes = scopes_for_service(Service::Groups);
        assert_eq!(scopes, vec!["https://www.googleapis.com/auth/cloud-identity.groups.readonly"]);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Keep scope (readonly)
    #[test]
    fn req_auth_016_keep_default_scope() {
        let scopes = scopes_for_service(Service::Keep);
        assert_eq!(scopes, vec!["https://www.googleapis.com/auth/keep.readonly"]);
    }

    // ---------------------------------------------------------------
    // REQ-AUTH-016 (Must): Readonly variants
    // ---------------------------------------------------------------

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Gmail readonly variant
    #[test]
    fn req_auth_016_gmail_readonly() {
        let opts = ScopeOptions { readonly: true, ..Default::default() };
        let scopes = scopes_for_service_with_options(Service::Gmail, &opts).unwrap();
        assert_eq!(scopes, vec!["https://www.googleapis.com/auth/gmail.readonly"]);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Calendar readonly variant
    #[test]
    fn req_auth_016_calendar_readonly() {
        let opts = ScopeOptions { readonly: true, ..Default::default() };
        let scopes = scopes_for_service_with_options(Service::Calendar, &opts).unwrap();
        assert_eq!(scopes, vec!["https://www.googleapis.com/auth/calendar.readonly"]);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Chat readonly variants
    #[test]
    fn req_auth_016_chat_readonly() {
        let opts = ScopeOptions { readonly: true, ..Default::default() };
        let scopes = scopes_for_service_with_options(Service::Chat, &opts).unwrap();
        assert!(scopes.contains(&"https://www.googleapis.com/auth/chat.spaces.readonly".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/chat.messages.readonly".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/chat.memberships.readonly".to_string()));
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Drive readonly scope
    #[test]
    fn req_auth_016_drive_readonly() {
        let opts = ScopeOptions { readonly: true, ..Default::default() };
        let scopes = scopes_for_service_with_options(Service::Drive, &opts).unwrap();
        assert_eq!(scopes, vec!["https://www.googleapis.com/auth/drive.readonly"]);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Docs readonly variant uses both drive.readonly and documents.readonly
    #[test]
    fn req_auth_016_docs_readonly() {
        let opts = ScopeOptions { readonly: true, ..Default::default() };
        let scopes = scopes_for_service_with_options(Service::Docs, &opts).unwrap();
        assert!(scopes.contains(&"https://www.googleapis.com/auth/drive.readonly".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/documents.readonly".to_string()));
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Tasks readonly variant
    #[test]
    fn req_auth_016_tasks_readonly() {
        let opts = ScopeOptions { readonly: true, ..Default::default() };
        let scopes = scopes_for_service_with_options(Service::Tasks, &opts).unwrap();
        assert_eq!(scopes, vec!["https://www.googleapis.com/auth/tasks.readonly"]);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Contacts readonly variant
    #[test]
    fn req_auth_016_contacts_readonly() {
        let opts = ScopeOptions { readonly: true, ..Default::default() };
        let scopes = scopes_for_service_with_options(Service::Contacts, &opts).unwrap();
        assert!(scopes.contains(&"https://www.googleapis.com/auth/contacts.readonly".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/contacts.other.readonly".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/directory.readonly".to_string()));
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: People has no distinct readonly (profile is already read-only)
    #[test]
    fn req_auth_016_people_readonly_same_as_default() {
        let opts = ScopeOptions { readonly: true, ..Default::default() };
        let scopes = scopes_for_service_with_options(Service::People, &opts).unwrap();
        assert_eq!(scopes, vec!["profile"]);
    }

    // ---------------------------------------------------------------
    // REQ-AUTH-016 (Must): Drive scope variants
    // ---------------------------------------------------------------

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Drive file scope mode
    #[test]
    fn req_auth_016_drive_file_scope() {
        let opts = ScopeOptions {
            readonly: false,
            drive_scope: DriveScopeMode::File,
        };
        let scopes = scopes_for_service_with_options(Service::Drive, &opts).unwrap();
        assert_eq!(scopes, vec!["https://www.googleapis.com/auth/drive.file"]);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Drive readonly scope mode (via DriveScopeMode)
    #[test]
    fn req_auth_016_drive_readonly_scope_mode() {
        let opts = ScopeOptions {
            readonly: false,
            drive_scope: DriveScopeMode::Readonly,
        };
        let scopes = scopes_for_service_with_options(Service::Drive, &opts).unwrap();
        assert_eq!(scopes, vec!["https://www.googleapis.com/auth/drive.readonly"]);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: readonly flag overrides drive_scope mode
    #[test]
    fn req_auth_016_readonly_overrides_drive_scope() {
        let opts = ScopeOptions {
            readonly: true,
            drive_scope: DriveScopeMode::Full,
        };
        let scopes = scopes_for_service_with_options(Service::Drive, &opts).unwrap();
        assert_eq!(scopes, vec!["https://www.googleapis.com/auth/drive.readonly"]);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Docs with drive file scope
    #[test]
    fn req_auth_016_docs_with_drive_file_scope() {
        let opts = ScopeOptions {
            readonly: false,
            drive_scope: DriveScopeMode::File,
        };
        let scopes = scopes_for_service_with_options(Service::Docs, &opts).unwrap();
        assert!(scopes.contains(&"https://www.googleapis.com/auth/drive.file".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/documents".to_string()));
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Sheets with drive file scope
    #[test]
    fn req_auth_016_sheets_with_drive_file_scope() {
        let opts = ScopeOptions {
            readonly: false,
            drive_scope: DriveScopeMode::File,
        };
        let scopes = scopes_for_service_with_options(Service::Sheets, &opts).unwrap();
        assert!(scopes.contains(&"https://www.googleapis.com/auth/drive.file".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/spreadsheets".to_string()));
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Base scopes always included in manage scopes
    #[test]
    fn req_auth_016_base_scopes_always_included() {
        let opts = ScopeOptions::default();
        let scopes = scopes_for_manage(&[Service::Gmail], &opts).unwrap();
        assert!(scopes.contains(&"openid".to_string()));
        assert!(scopes.contains(&"email".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/userinfo.email".to_string()));
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Multiple services combined, deduplicated, sorted
    #[test]
    fn req_auth_016_multiple_services_deduplicated() {
        let opts = ScopeOptions::default();
        // Docs and Sheets both request drive scope -- should be deduplicated
        let scopes = scopes_for_manage(&[Service::Docs, Service::Sheets], &opts).unwrap();
        let drive_count = scopes.iter().filter(|s| *s == "https://www.googleapis.com/auth/drive").count();
        assert_eq!(drive_count, 1, "drive scope should appear exactly once");
        // Scopes should be sorted
        let mut sorted = scopes.clone();
        sorted.sort();
        assert_eq!(scopes, sorted, "scopes should be sorted alphabetically");
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: AppScript readonly variants
    #[test]
    fn req_auth_016_appscript_readonly() {
        let opts = ScopeOptions { readonly: true, ..Default::default() };
        let scopes = scopes_for_service_with_options(Service::AppScript, &opts).unwrap();
        assert!(scopes.contains(&"https://www.googleapis.com/auth/script.projects.readonly".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/script.deployments.readonly".to_string()));
        assert_eq!(scopes.len(), 2);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Groups and Keep have no readonly variants (already readonly)
    #[test]
    fn req_auth_016_groups_keep_no_readonly_change() {
        let opts = ScopeOptions { readonly: true, ..Default::default() };
        let groups_scopes = scopes_for_service_with_options(Service::Groups, &opts).unwrap();
        let keep_scopes = scopes_for_service_with_options(Service::Keep, &opts).unwrap();
        assert_eq!(groups_scopes, vec!["https://www.googleapis.com/auth/cloud-identity.groups.readonly"]);
        assert_eq!(keep_scopes, vec!["https://www.googleapis.com/auth/keep.readonly"]);
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Forms readonly variant
    #[test]
    fn req_auth_016_forms_readonly() {
        let opts = ScopeOptions { readonly: true, ..Default::default() };
        let scopes = scopes_for_service_with_options(Service::Forms, &opts).unwrap();
        assert!(scopes.contains(&"https://www.googleapis.com/auth/forms.body.readonly".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/forms.responses.readonly".to_string()));
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Slides readonly variant
    #[test]
    fn req_auth_016_slides_readonly() {
        let opts = ScopeOptions { readonly: true, ..Default::default() };
        let scopes = scopes_for_service_with_options(Service::Slides, &opts).unwrap();
        assert!(scopes.contains(&"https://www.googleapis.com/auth/drive.readonly".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/presentations.readonly".to_string()));
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Sheets readonly variant
    #[test]
    fn req_auth_016_sheets_readonly() {
        let opts = ScopeOptions { readonly: true, ..Default::default() };
        let scopes = scopes_for_service_with_options(Service::Sheets, &opts).unwrap();
        assert!(scopes.contains(&"https://www.googleapis.com/auth/drive.readonly".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/spreadsheets.readonly".to_string()));
    }

    // Requirement: REQ-AUTH-016 (Must)
    // Acceptance: Classroom readonly variants (10 readonly scopes)
    #[test]
    fn req_auth_016_classroom_readonly() {
        let opts = ScopeOptions { readonly: true, ..Default::default() };
        let scopes = scopes_for_service_with_options(Service::Classroom, &opts).unwrap();
        assert_eq!(scopes.len(), 10);
        assert!(scopes.contains(&"https://www.googleapis.com/auth/classroom.courses.readonly".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/classroom.rosters.readonly".to_string()));
        // profile.emails and profile.photos do NOT have readonly variants
        assert!(scopes.contains(&"https://www.googleapis.com/auth/classroom.profile.emails".to_string()));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/classroom.profile.photos".to_string()));
    }
}
