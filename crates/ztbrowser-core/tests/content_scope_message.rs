//! Answering messages the content scope script posts to the browser.

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::protection::content_scope_message::ContentScopeMessage;

struct MessageScenario {
    message: Value,
}

impl MessageScenario {
    fn given_page_posted(message: Value) -> Self {
        Self { message }
    }

    fn when_answered(&self) -> Value {
        serde_json::from_str(&ContentScopeMessage::new(&self.message).reply()).expect("reply is JSON")
    }
}

#[test]
fn should_acknowledge_notifications_with_empty_object() {
    let reply = MessageScenario::given_page_posted(json!({
        "context": "contentScopeScripts", "featureName": "breakageReporting", "method": "addDebugFlag", "params": {}
    }))
    .when_answered();

    assert_eq!(reply, json!({}));
}

#[test]
fn should_answer_requests_with_an_error_the_script_can_handle() {
    let reply = MessageScenario::given_page_posted(json!({
        "context": "contentScopeScripts", "featureName": "duckPlayer", "method": "getUserValues", "id": "getUserValues.response"
    }))
    .when_answered();

    assert_eq!(
        reply,
        json!({
            "context": "contentScopeScripts", "featureName": "duckPlayer", "id": "getUserValues.response",
            "error": {"message": "duckPlayer.getUserValues is not supported by ZTBrowser"}
        })
    );
}

#[test]
fn should_ignore_malformed_messages() {
    assert_eq!(MessageScenario::given_page_posted(json!("<script>")).when_answered(), json!({}));
}
