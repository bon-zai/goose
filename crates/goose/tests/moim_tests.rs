use goose::agents::moim;
use goose::conversation::message::Message;
use goose::conversation::Conversation;
use serial_test::serial;

// All async tests that call inject_moim_if_enabled need #[serial] because
// that function accesses Config::global() to check if MOIM is enabled
#[tokio::test]
#[serial]
async fn test_moim_basic_injection() {
    let messages = vec![Message::user().with_text("Only message")];
    let conversation = Conversation::new_unvalidated(messages.clone());

    let result = moim::inject_moim_if_enabled(conversation, &None).await;

    assert_eq!(result.len(), 2);
    assert!(result.messages()[0]
        .as_concat_text()
        .contains("Current date and time:"));
}

#[tokio::test]
#[serial]
async fn test_moim_disabled_no_injection() {
    std::env::set_var("GOOSE_MOIM_ENABLED", "false");

    let messages = vec![
        Message::user().with_text("Test message"),
        Message::assistant().with_text("Test response"),
    ];
    let conversation = Conversation::new_unvalidated(messages.clone());

    let result = moim::inject_moim_if_enabled(conversation, &None).await;

    assert_eq!(result.len(), messages.len());
    assert_eq!(result.messages()[0].as_concat_text(), "Test message");
    assert_eq!(result.messages()[1].as_concat_text(), "Test response");

    std::env::remove_var("GOOSE_MOIM_ENABLED");
}

#[tokio::test]
#[serial]
async fn test_moim_respects_tool_pairs() {
    // Critical test: ensure MOIM doesn't break tool call/response pairs
    use rmcp::model::{CallToolRequestParam, Content};

    // Create a tool request message using the builder method
    let assistant_msg = Message::assistant()
        .with_text("I'll use the tool for you")
        .with_tool_request(
            "tool1",
            Ok(CallToolRequestParam {
                name: "test_tool".into(),
                arguments: None,
            }),
        );

    // Create a tool response message using the builder method
    let user_msg = Message::user().with_tool_response(
        "tool1",
        Ok(vec![Content::text("Tool executed successfully")]),
    );

    let messages = vec![
        Message::user().with_text("Please use the tool"),
        assistant_msg,
        user_msg,
        Message::assistant().with_text("The tool has been executed"),
        Message::user().with_text("Thank you"),
    ];

    let conversation = Conversation::new_unvalidated(messages.clone());

    let result = moim::inject_moim_if_enabled(conversation, &None).await;

    assert_eq!(result.len(), messages.len() + 1);

    // Verify tool call and response are still adjacent
    let msgs = result.messages();
    for i in 0..msgs.len() - 1 {
        if msgs[i].is_tool_call() {
            assert!(
                msgs[i + 1].is_tool_response()
                    || !msgs[i + 1]
                        .as_concat_text()
                        .contains("Current date and time:"),
                "MOIM should not be inserted between tool call and response"
            );
        }
    }
}

#[test]
fn test_find_safe_insertion_point_ending_with_tool_response() {
    // Critical test: when conversation ends with tool response, don't break the pair
    use rmcp::model::{CallToolRequestParam, Content};

    // Create a tool request message using the builder method
    let assistant_msg = Message::assistant().with_tool_request(
        "tool1",
        Ok(CallToolRequestParam {
            name: "test_tool".into(),
            arguments: None,
        }),
    );

    // Create a tool response message using the builder method
    let user_msg = Message::user().with_tool_response("tool1", Ok(vec![Content::text("Result")]));

    let messages = vec![
        Message::user().with_text("Do something"),
        assistant_msg,
        user_msg,
    ];

    // Should insert before the tool call instead (index 1)
    assert_eq!(goose::agents::moim::find_safe_insertion_point(&messages), 1);
}
