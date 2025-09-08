use codex_core::ConversationManager;
use codex_core::ModelProviderInfo;
use codex_core::NewConversation;
use codex_core::WireApi;
use codex_core::built_in_model_providers;
use codex_core::protocol::EventMsg;
use codex_core::protocol::InputItem;
use codex_core::spawn::CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR;
use codex_login::CodexAuth;
use core_test_support::load_default_config_for_test;
use core_test_support::load_sse_fixture_with_id_from_str;
use core_test_support::wait_for_event;
use tempfile::TempDir;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path;

/// Ensures that when using ChatGPT auth with store=false, any Reasoning items
/// recorded in conversation history on a previous turn are filtered out of the
/// next turn's `input` payload. This avoids upstream 404s referencing `rs_*`
/// IDs that are not persisted when `store=false`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn chatgpt_store_false_filters_reasoning_from_next_turn() {
    if std::env::var(CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR).is_ok() {
        println!(
            "Skipping test because it cannot execute when network is disabled in a Codex sandbox."
        );
        return;
    }

    // 1) Start a mock server and mount two sequential handlers.
    let server = MockServer::start().await;

    // Turn 1 SSE: emit a reasoning item (with id rs_...) and then completed.
    let sse_raw = r##"[
        {"type":"response.output_item.done","item":{
            "type":"reasoning",
            "id":"rs_abcdef123",
            "summary":[{"type":"summary_text","text":"brief"}]
        }},
        {"type":"response.completed","response":{"id":"__ID__"}}
    ]"##;
    let sse1 = load_sse_fixture_with_id_from_str(sse_raw, "resp1");
    let first_matcher = |req: &wiremock::Request| {
        let body = std::str::from_utf8(&req.body).unwrap_or("");
        body.contains("\"text\":\"hello\"") && !body.contains("\"text\":\"again\"")
    };
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .and(first_matcher)
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(sse1, "text/event-stream"),
        )
        .expect(1)
        .mount(&server)
        .await;

    // Turn 2 SSE: simple completed to finish quickly.
    let sse2 = load_sse_fixture_with_id_from_str(
        r##"[{"type":"response.completed","response":{"id":"__ID__"}}]"##,
        "resp2",
    );
    let second_matcher = |req: &wiremock::Request| {
        let body = std::str::from_utf8(&req.body).unwrap_or("");
        body.contains("\"text\":\"again\"")
    };
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .and(second_matcher)
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(sse2, "text/event-stream"),
        )
        .expect(1)
        .mount(&server)
        .await;

    // 2) Configure provider to use Responses API against our mock.
    let model_provider = ModelProviderInfo {
        base_url: Some(format!("{}/v1", server.uri())),
        wire_api: WireApi::Responses,
        ..built_in_model_providers()["openai"].clone()
    };

    // 3) Start a conversation using ChatGPT auth (dummy tokens for tests).
    let codex_home = TempDir::new().unwrap();
    let mut config = load_default_config_for_test(&codex_home);
    config.model_provider = model_provider;

    let conversation_manager =
        ConversationManager::with_auth(CodexAuth::create_dummy_chatgpt_auth_for_testing());
    let NewConversation {
        conversation: codex,
        ..
    } = conversation_manager
        .new_conversation(config)
        .await
        .expect("create new conversation");

    // 4) Turn 1: send a user message; the mock streams a Reasoning item.
    codex
        .submit(codex_core::protocol::Op::UserInput {
            items: vec![InputItem::Text {
                text: "hello".into(),
            }],
        })
        .await
        .unwrap();
    wait_for_event(&codex, |ev| matches!(ev, EventMsg::TaskComplete(_))).await;

    // 5) Turn 2: send another user message.
    codex
        .submit(codex_core::protocol::Op::UserInput {
            items: vec![InputItem::Text {
                text: "again".into(),
            }],
        })
        .await
        .unwrap();
    wait_for_event(&codex, |ev| matches!(ev, EventMsg::TaskComplete(_))).await;

    // 6) Inspect the second request body: it must NOT contain any `reasoning` items in input.
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 2, "expected two POSTs (two turns)");
    let req2_body = requests[1].body_json::<serde_json::Value>().unwrap();
    let input = req2_body
        .get("input")
        .and_then(|v| v.as_array())
        .cloned()
        .expect("second request missing input array");

    let has_reasoning = input
        .iter()
        .any(|item| item.get("type").and_then(|t| t.as_str()) == Some("reasoning"));
    assert!(
        !has_reasoning,
        "second request should not include reasoning items when store=false + ChatGPT auth"
    );
}
