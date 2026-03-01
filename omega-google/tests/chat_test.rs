//! Chat service integration tests.

use omega_google::services::chat::types::*;
use omega_google::services::chat::spaces::*;
use omega_google::services::chat::messages::*;
use omega_google::services::chat::dm::*;

// ---------------------------------------------------------------
// REQ-CHAT-001 (Must): Space deserialization from full JSON
// ---------------------------------------------------------------

// Requirement: REQ-CHAT-001 (Must)
// Acceptance: Full space structure from a realistic Chat API response
#[test]
fn req_chat_001_integration_full_space_from_api() {
    // REQ-CHAT-001
    let api_response = r#"{
        "name": "spaces/AAAABBBBcccc",
        "displayName": "Engineering Team",
        "spaceType": "SPACE",
        "singleUserBotDm": false,
        "threaded": true,
        "spaceThreadingState": "THREADED_MESSAGES",
        "spaceHistoryState": "HISTORY_ON",
        "externalUserAllowed": true,
        "importMode": false
    }"#;

    let space: Space = serde_json::from_str(api_response).unwrap();

    // Verify all standard fields
    assert_eq!(space.name, Some("spaces/AAAABBBBcccc".to_string()));
    assert_eq!(space.display_name, Some("Engineering Team".to_string()));
    assert_eq!(space.space_type, Some("SPACE".to_string()));
    assert_eq!(space.single_user_bot_dm, Some(false));
    assert_eq!(space.threaded, Some(true));
    assert_eq!(space.space_threading_state, Some("THREADED_MESSAGES".to_string()));
    assert_eq!(space.space_history_state, Some("HISTORY_ON".to_string()));

    // Unknown fields preserved via flatten
    assert!(space.extra.contains_key("externalUserAllowed"));
    assert!(space.extra.contains_key("importMode"));
}

// ---------------------------------------------------------------
// REQ-CHAT-001 (Must): Space list response from realistic API
// ---------------------------------------------------------------

// Requirement: REQ-CHAT-001 (Must)
// Acceptance: SpaceListResponse with multiple spaces and pagination
#[test]
fn req_chat_001_integration_space_list_from_api() {
    // REQ-CHAT-001
    let api_response = r#"{
        "spaces": [
            {
                "name": "spaces/AAAA",
                "displayName": "Engineering",
                "spaceType": "SPACE",
                "threaded": true,
                "spaceThreadingState": "THREADED_MESSAGES"
            },
            {
                "name": "spaces/BBBB",
                "displayName": "Marketing",
                "spaceType": "SPACE",
                "threaded": false,
                "spaceHistoryState": "HISTORY_OFF"
            },
            {
                "name": "spaces/CCCC",
                "singleUserBotDm": true,
                "spaceType": "DIRECT_MESSAGE"
            }
        ],
        "nextPageToken": "CAE6BAgBEgE0"
    }"#;

    let resp: SpaceListResponse = serde_json::from_str(api_response).unwrap();

    assert_eq!(resp.spaces.len(), 3);
    assert_eq!(resp.next_page_token, Some("CAE6BAgBEgE0".to_string()));

    // First space: Engineering
    assert_eq!(resp.spaces[0].name, Some("spaces/AAAA".to_string()));
    assert_eq!(resp.spaces[0].display_name, Some("Engineering".to_string()));
    assert_eq!(resp.spaces[0].threaded, Some(true));

    // Second space: Marketing
    assert_eq!(resp.spaces[1].display_name, Some("Marketing".to_string()));
    assert_eq!(resp.spaces[1].threaded, Some(false));

    // Third space: DM
    assert_eq!(resp.spaces[2].space_type, Some("DIRECT_MESSAGE".to_string()));
    assert_eq!(resp.spaces[2].single_user_bot_dm, Some(true));
    assert!(resp.spaces[2].display_name.is_none());
}

// ---------------------------------------------------------------
// REQ-CHAT-004 (Must): Message list response deserialization
// ---------------------------------------------------------------

// Requirement: REQ-CHAT-004 (Must)
// Acceptance: MessageListResponse with full messages from API
#[test]
fn req_chat_004_integration_message_list_from_api() {
    // REQ-CHAT-004
    let api_response = r#"{
        "messages": [
            {
                "name": "spaces/AAAA/messages/msg001",
                "sender": {
                    "name": "users/u001",
                    "displayName": "Alice Chen",
                    "domainId": "company.com",
                    "type": "HUMAN"
                },
                "text": "Good morning, team! Sprint planning at 10am.",
                "createTime": "2024-03-15T08:00:00.000Z",
                "thread": {
                    "name": "spaces/AAAA/threads/thread001",
                    "threadKey": "sprint-planning-q1"
                }
            },
            {
                "name": "spaces/AAAA/messages/msg002",
                "sender": {
                    "name": "users/u002",
                    "displayName": "Bob Smith",
                    "type": "HUMAN"
                },
                "text": "Sounds good! I'll prepare the backlog review.",
                "createTime": "2024-03-15T08:05:00.000Z",
                "thread": {
                    "name": "spaces/AAAA/threads/thread001",
                    "threadKey": "sprint-planning-q1"
                }
            },
            {
                "name": "spaces/AAAA/messages/msg003",
                "sender": {
                    "name": "users/bot001",
                    "displayName": "Standup Bot",
                    "type": "BOT"
                },
                "text": "Reminder: Daily standup in 15 minutes.",
                "createTime": "2024-03-15T09:45:00.000Z"
            }
        ],
        "nextPageToken": "msg_page_2_token"
    }"#;

    let resp: MessageListResponse = serde_json::from_str(api_response).unwrap();

    assert_eq!(resp.messages.len(), 3);
    assert_eq!(resp.next_page_token, Some("msg_page_2_token".to_string()));

    // First message: human with thread
    let msg1 = &resp.messages[0];
    assert_eq!(msg1.name, Some("spaces/AAAA/messages/msg001".to_string()));
    assert_eq!(msg1.text, Some("Good morning, team! Sprint planning at 10am.".to_string()));
    assert_eq!(msg1.create_time, Some("2024-03-15T08:00:00.000Z".to_string()));
    let sender1 = msg1.sender.as_ref().unwrap();
    assert_eq!(sender1.display_name, Some("Alice Chen".to_string()));
    assert_eq!(sender1.type_, Some("HUMAN".to_string()));
    assert_eq!(sender1.domain_id, Some("company.com".to_string()));
    let thread1 = msg1.thread.as_ref().unwrap();
    assert_eq!(thread1.name, Some("spaces/AAAA/threads/thread001".to_string()));
    assert_eq!(thread1.thread_key, Some("sprint-planning-q1".to_string()));

    // Second message: same thread
    let msg2 = &resp.messages[1];
    assert!(msg2.text.as_ref().unwrap().contains("backlog review"));
    let thread2 = msg2.thread.as_ref().unwrap();
    assert_eq!(thread2.name, Some("spaces/AAAA/threads/thread001".to_string()));

    // Third message: bot, no thread
    let msg3 = &resp.messages[2];
    let sender3 = msg3.sender.as_ref().unwrap();
    assert_eq!(sender3.type_, Some("BOT".to_string()));
    assert!(msg3.thread.is_none());
}

// ---------------------------------------------------------------
// REQ-CHAT-001 (Must): URL builder verification - spaces list
// ---------------------------------------------------------------

// Requirement: REQ-CHAT-001 (Must)
// Acceptance: Spaces list URL builds correctly with various params
#[test]
fn req_chat_001_integration_url_builder_spaces() {
    // REQ-CHAT-001
    // No params
    let url = build_spaces_list_url(None, None);
    assert_eq!(url, "https://chat.googleapis.com/v1/spaces");

    // With page size
    let url = build_spaces_list_url(Some(50), None);
    assert!(url.starts_with("https://chat.googleapis.com/v1/spaces?"));
    assert!(url.contains("pageSize=50"));

    // With page token
    let url = build_spaces_list_url(None, Some("CAE6BAgBEgE0"));
    assert!(url.contains("pageToken=CAE6BAgBEgE0"));

    // With both
    let url = build_spaces_list_url(Some(25), Some("next_token"));
    assert!(url.contains("pageSize=25"));
    assert!(url.contains("pageToken=next_token"));
}

// ---------------------------------------------------------------
// REQ-CHAT-004 (Must): URL builder verification - messages list
// ---------------------------------------------------------------

// Requirement: REQ-CHAT-004 (Must)
// Acceptance: Messages list URL builds correctly with various params
#[test]
fn req_chat_004_integration_url_builder_messages() {
    // REQ-CHAT-004
    // Basic messages list
    let url = build_messages_list_url("AAAA", None, None, None, None);
    assert!(url.contains("/spaces/AAAA/messages"));

    // With all parameters
    let url = build_messages_list_url(
        "BBBB",
        Some(50),
        Some("page2"),
        Some("createTime desc"),
        Some("spaces/BBBB/threads/t1"),
    );
    assert!(url.contains("pageSize=50"));
    assert!(url.contains("pageToken=page2"));
    assert!(url.contains("orderBy=createTime+desc"));
    assert!(url.contains("filter="));
}

// ---------------------------------------------------------------
// REQ-CHAT-008 (Must): DM body construction verification
// ---------------------------------------------------------------

// Requirement: REQ-CHAT-008 (Must)
// Acceptance: DM send body constructs correct JSON
#[test]
fn req_chat_008_integration_dm_body_construction() {
    // REQ-CHAT-008
    // Simple text DM
    let body = build_dm_send_body("Hello, how are you?", None);
    assert_eq!(body["text"], "Hello, how are you?");
    assert!(body.get("thread").is_none());

    // DM with thread reply
    let body = build_dm_send_body(
        "Thanks for the update!",
        Some("spaces/DM123/threads/thread456"),
    );
    assert_eq!(body["text"], "Thanks for the update!");
    assert_eq!(body["thread"]["name"], "spaces/DM123/threads/thread456");
}

// ---------------------------------------------------------------
// REQ-CHAT-007 (Must): DM space URL verification
// ---------------------------------------------------------------

// Requirement: REQ-CHAT-007 (Must)
// Acceptance: DM space finder URL is correct
#[test]
fn req_chat_007_integration_dm_space_url() {
    // REQ-CHAT-007
    let url = build_dm_space_url();
    assert_eq!(url, "https://chat.googleapis.com/v1/spaces:findDirectMessage");

    let body = build_dm_space_body("user@company.com");
    assert_eq!(body["name"], "user@company.com");
}

// ---------------------------------------------------------------
// REQ-CHAT-005 (Must): Message send URL and body verification
// ---------------------------------------------------------------

// Requirement: REQ-CHAT-005 (Must)
// Acceptance: Message send URL and body construct correctly
#[test]
fn req_chat_005_integration_message_send() {
    // REQ-CHAT-005
    let url = build_message_send_url("AAAA");
    assert!(url.contains("/spaces/AAAA/messages"));

    // Body with text only
    let body = build_message_send_body("Hello team!", None);
    assert_eq!(body["text"], "Hello team!");
    assert!(body.get("thread").is_none());

    // Body with thread
    let body = build_message_send_body("Thread reply", Some("spaces/AAAA/threads/t1"));
    assert_eq!(body["text"], "Thread reply");
    assert_eq!(body["thread"]["name"], "spaces/AAAA/threads/t1");
}
