//! Google Chat API request/response types.
//! All types use camelCase serde rename and flatten for forward compatibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------
// Space types
// ---------------------------------------------------------------

/// A Google Chat space (room, DM, or group conversation).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Space {
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub space_type: Option<String>,
    pub single_user_bot_dm: Option<bool>,
    pub threaded: Option<bool>,
    pub space_threading_state: Option<String>,
    pub space_history_state: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response for listing spaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceListResponse {
    #[serde(default)]
    pub spaces: Vec<Space>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Message types
// ---------------------------------------------------------------

/// A Google Chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub name: Option<String>,
    pub sender: Option<MessageSender>,
    pub text: Option<String>,
    pub create_time: Option<String>,
    pub thread: Option<Thread>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// The sender of a Chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageSender {
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub domain_id: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response for listing messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageListResponse {
    #[serde(default)]
    pub messages: Vec<Message>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Thread types
// ---------------------------------------------------------------

/// A Google Chat thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    pub name: Option<String>,
    pub thread_key: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response for listing threads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadListResponse {
    #[serde(default)]
    pub threads: Vec<Thread>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Request types
// ---------------------------------------------------------------

/// Request body for creating a space.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSpaceRequest {
    pub display_name: Option<String>,
    pub space_type: Option<String>,
    #[serde(default)]
    pub members: Vec<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Request body for sending a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMessageRequest {
    pub text: Option<String>,
    pub thread: Option<Thread>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ---------------------------------------------------------------
    // REQ-CHAT-001 (Must): Space type deserialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CHAT-001 (Must)
    // Acceptance: Space type deserializes from Chat API JSON
    #[test]
    fn req_chat_001_space_deserialize() {
        let json_str = r#"{
            "name": "spaces/AAAA",
            "displayName": "Engineering",
            "spaceType": "SPACE",
            "singleUserBotDm": false,
            "threaded": true,
            "spaceThreadingState": "THREADED_MESSAGES",
            "spaceHistoryState": "HISTORY_ON"
        }"#;
        let space: Space = serde_json::from_str(json_str).unwrap();
        assert_eq!(space.name, Some("spaces/AAAA".to_string()));
        assert_eq!(space.display_name, Some("Engineering".to_string()));
        assert_eq!(space.space_type, Some("SPACE".to_string()));
        assert_eq!(space.single_user_bot_dm, Some(false));
        assert_eq!(space.threaded, Some(true));
        assert_eq!(space.space_threading_state, Some("THREADED_MESSAGES".to_string()));
        assert_eq!(space.space_history_state, Some("HISTORY_ON".to_string()));
    }

    // Requirement: REQ-CHAT-001 (Must)
    // Acceptance: SpaceListResponse deserializes with pagination
    #[test]
    fn req_chat_001_space_list_response_deserialize() {
        let json_str = r#"{
            "spaces": [
                {"name": "spaces/AAA", "displayName": "Room 1"},
                {"name": "spaces/BBB", "displayName": "Room 2"}
            ],
            "nextPageToken": "token_xyz"
        }"#;
        let resp: SpaceListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.spaces.len(), 2);
        assert_eq!(resp.spaces[0].name, Some("spaces/AAA".to_string()));
        assert_eq!(resp.next_page_token, Some("token_xyz".to_string()));
    }

    // Requirement: REQ-CHAT-001 (Must)
    // Edge case: SpaceListResponse with empty spaces list
    #[test]
    fn req_chat_001_space_list_response_empty() {
        let json_str = r#"{}"#;
        let resp: SpaceListResponse = serde_json::from_str(json_str).unwrap();
        assert!(resp.spaces.is_empty());
        assert!(resp.next_page_token.is_none());
    }

    // Requirement: REQ-CHAT-001 (Must)
    // Edge case: Space with unknown fields preserved via flatten
    #[test]
    fn req_chat_001_space_unknown_fields_preserved() {
        let json_str = r#"{
            "name": "spaces/XXX",
            "futureField": "some_value"
        }"#;
        let space: Space = serde_json::from_str(json_str).unwrap();
        assert_eq!(space.name, Some("spaces/XXX".to_string()));
        assert!(space.extra.contains_key("futureField"));
    }

    // Requirement: REQ-CHAT-001 (Must)
    // Acceptance: Space round-trip serialization
    #[test]
    fn req_chat_001_space_roundtrip() {
        let space = Space {
            name: Some("spaces/AAA".to_string()),
            display_name: Some("Test".to_string()),
            space_type: Some("SPACE".to_string()),
            single_user_bot_dm: Some(false),
            threaded: Some(true),
            space_threading_state: None,
            space_history_state: None,
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&space).unwrap();
        let parsed: Space = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, Some("spaces/AAA".to_string()));
        assert_eq!(parsed.display_name, Some("Test".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-CHAT-004 (Must): Message type deserialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CHAT-004 (Must)
    // Acceptance: Message type deserializes from Chat API JSON
    #[test]
    fn req_chat_004_message_deserialize() {
        let json_str = r#"{
            "name": "spaces/AAA/messages/msg123",
            "sender": {
                "name": "users/user123",
                "displayName": "Alice",
                "domainId": "domain1",
                "type": "HUMAN"
            },
            "text": "Hello, team!",
            "createTime": "2024-01-15T10:30:00Z",
            "thread": {
                "name": "spaces/AAA/threads/thread456",
                "threadKey": "key123"
            }
        }"#;
        let msg: Message = serde_json::from_str(json_str).unwrap();
        assert_eq!(msg.name, Some("spaces/AAA/messages/msg123".to_string()));
        assert_eq!(msg.text, Some("Hello, team!".to_string()));
        assert_eq!(msg.create_time, Some("2024-01-15T10:30:00Z".to_string()));

        let sender = msg.sender.unwrap();
        assert_eq!(sender.name, Some("users/user123".to_string()));
        assert_eq!(sender.display_name, Some("Alice".to_string()));
        assert_eq!(sender.domain_id, Some("domain1".to_string()));
        assert_eq!(sender.type_, Some("HUMAN".to_string()));

        let thread = msg.thread.unwrap();
        assert_eq!(thread.name, Some("spaces/AAA/threads/thread456".to_string()));
        assert_eq!(thread.thread_key, Some("key123".to_string()));
    }

    // Requirement: REQ-CHAT-004 (Must)
    // Acceptance: MessageListResponse deserializes with pagination
    #[test]
    fn req_chat_004_message_list_response_deserialize() {
        let json_str = r#"{
            "messages": [
                {"name": "spaces/AAA/messages/m1", "text": "Hi"},
                {"name": "spaces/AAA/messages/m2", "text": "Hello"}
            ],
            "nextPageToken": "page2"
        }"#;
        let resp: MessageListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.messages.len(), 2);
        assert_eq!(resp.messages[0].text, Some("Hi".to_string()));
        assert_eq!(resp.next_page_token, Some("page2".to_string()));
    }

    // Requirement: REQ-CHAT-004 (Must)
    // Edge case: MessageListResponse with empty messages
    #[test]
    fn req_chat_004_message_list_response_empty() {
        let json_str = r#"{}"#;
        let resp: MessageListResponse = serde_json::from_str(json_str).unwrap();
        assert!(resp.messages.is_empty());
        assert!(resp.next_page_token.is_none());
    }

    // Requirement: REQ-CHAT-004 (Must)
    // Acceptance: Message round-trip serialization
    #[test]
    fn req_chat_004_message_roundtrip() {
        let msg = Message {
            name: Some("spaces/AAA/messages/m1".to_string()),
            sender: Some(MessageSender {
                name: Some("users/u1".to_string()),
                display_name: Some("Bob".to_string()),
                domain_id: None,
                type_: Some("HUMAN".to_string()),
                extra: HashMap::new(),
            }),
            text: Some("Test message".to_string()),
            create_time: Some("2024-01-15T10:00:00Z".to_string()),
            thread: None,
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, Some("spaces/AAA/messages/m1".to_string()));
        assert_eq!(parsed.text, Some("Test message".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-CHAT-006 (Must): Thread type deserialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CHAT-006 (Must)
    // Acceptance: Thread type deserializes from Chat API JSON
    #[test]
    fn req_chat_006_thread_deserialize() {
        let json_str = r#"{
            "name": "spaces/AAA/threads/thread789",
            "threadKey": "mykey"
        }"#;
        let thread: Thread = serde_json::from_str(json_str).unwrap();
        assert_eq!(thread.name, Some("spaces/AAA/threads/thread789".to_string()));
        assert_eq!(thread.thread_key, Some("mykey".to_string()));
    }

    // Requirement: REQ-CHAT-006 (Must)
    // Acceptance: ThreadListResponse deserializes with pagination
    #[test]
    fn req_chat_006_thread_list_response_deserialize() {
        let json_str = r#"{
            "threads": [
                {"name": "spaces/AAA/threads/t1"},
                {"name": "spaces/AAA/threads/t2", "threadKey": "k2"}
            ],
            "nextPageToken": "next_t"
        }"#;
        let resp: ThreadListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.threads.len(), 2);
        assert_eq!(resp.threads[0].name, Some("spaces/AAA/threads/t1".to_string()));
        assert_eq!(resp.next_page_token, Some("next_t".to_string()));
    }

    // Requirement: REQ-CHAT-006 (Must)
    // Edge case: ThreadListResponse with empty threads
    #[test]
    fn req_chat_006_thread_list_response_empty() {
        let json_str = r#"{}"#;
        let resp: ThreadListResponse = serde_json::from_str(json_str).unwrap();
        assert!(resp.threads.is_empty());
    }

    // ---------------------------------------------------------------
    // REQ-CHAT-003 (Must): CreateSpaceRequest serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CHAT-003 (Must)
    // Acceptance: CreateSpaceRequest serializes correctly
    #[test]
    fn req_chat_003_create_space_request_serialize() {
        let req = CreateSpaceRequest {
            display_name: Some("New Space".to_string()),
            space_type: Some("SPACE".to_string()),
            members: vec!["users/user1".to_string()],
            extra: HashMap::new(),
        };
        let json_val: serde_json::Value = serde_json::to_value(&req).unwrap();
        assert_eq!(json_val["displayName"], "New Space");
        assert_eq!(json_val["spaceType"], "SPACE");
        assert_eq!(json_val["members"], json!(["users/user1"]));
    }

    // ---------------------------------------------------------------
    // REQ-CHAT-005 (Must): CreateMessageRequest serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CHAT-005 (Must)
    // Acceptance: CreateMessageRequest serializes correctly
    #[test]
    fn req_chat_005_create_message_request_serialize() {
        let req = CreateMessageRequest {
            text: Some("Hello, world!".to_string()),
            thread: Some(Thread {
                name: Some("spaces/AAA/threads/t1".to_string()),
                thread_key: None,
                extra: HashMap::new(),
            }),
            extra: HashMap::new(),
        };
        let json_val: serde_json::Value = serde_json::to_value(&req).unwrap();
        assert_eq!(json_val["text"], "Hello, world!");
        assert_eq!(json_val["thread"]["name"], "spaces/AAA/threads/t1");
    }

    // Requirement: REQ-CHAT-005 (Must)
    // Acceptance: CreateMessageRequest without thread
    #[test]
    fn req_chat_005_create_message_request_no_thread() {
        let req = CreateMessageRequest {
            text: Some("Simple message".to_string()),
            thread: None,
            extra: HashMap::new(),
        };
        let json_val: serde_json::Value = serde_json::to_value(&req).unwrap();
        assert_eq!(json_val["text"], "Simple message");
        assert!(json_val.get("thread").unwrap().is_null());
    }

    // ---------------------------------------------------------------
    // REQ-CHAT-004 (Must): MessageSender type rename
    // ---------------------------------------------------------------

    // Requirement: REQ-CHAT-004 (Must)
    // Acceptance: MessageSender "type" field renames correctly
    #[test]
    fn req_chat_004_message_sender_type_rename() {
        let json_str = r#"{
            "name": "users/u1",
            "displayName": "Test",
            "type": "HUMAN"
        }"#;
        let sender: MessageSender = serde_json::from_str(json_str).unwrap();
        assert_eq!(sender.type_, Some("HUMAN".to_string()));

        // Round-trip: should serialize back with "type"
        let serialized = serde_json::to_value(&sender).unwrap();
        assert_eq!(serialized["type"], "HUMAN");
    }
}
