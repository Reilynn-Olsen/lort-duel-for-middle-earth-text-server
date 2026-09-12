use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

struct Server {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl Server {
    fn jsonl() -> Self {
        Self::start(Some("--jsonl"))
    }

    fn text() -> Self {
        Self::start(None)
    }

    fn start(argument: Option<&str>) -> Self {
        let mut command = Command::new(env!("CARGO_BIN_EXE_rules_server"));
        if let Some(argument) = argument {
            command.arg(argument);
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("server starts");
        Self {
            input: child.stdin.take().expect("server stdin"),
            output: BufReader::new(child.stdout.take().expect("server stdout")),
            child,
        }
    }

    fn request(&mut self, request: Value) -> Value {
        writeln!(self.input, "{request}").expect("request writes");
        self.input.flush().expect("request flushes");
        let mut line = String::new();
        self.output.read_line(&mut line).expect("response reads");
        serde_json::from_str(&line).expect("response is JSON")
    }

    fn text_request(&mut self, request: &str) -> String {
        writeln!(self.input, "{request}").expect("request writes");
        self.input.flush().expect("request flushes");
        let mut response = String::new();
        loop {
            let mut line = String::new();
            self.output.read_line(&mut line).expect("response reads");
            if line == ".\n" {
                return response;
            }
            response.push_str(&line);
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn request(request_id: &str, request_type: &str) -> Value {
    json!({ "protocolVersion": 1, "requestId": request_id, "type": request_type })
}

#[test]
fn hello_advertises_jsonl_protocol() {
    let mut server = Server::jsonl();
    let response = server.request(request("hello-1", "hello"));

    assert_eq!(response["type"], "hello");
    assert_eq!(response["protocolVersion"], 1);
    assert_eq!(response["requestId"], "hello-1");
    assert!(
        response["engineVersion"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
    assert!(
        response["capabilities"]
            .as_array()
            .unwrap()
            .contains(&json!("choose"))
    );
}

#[test]
fn new_and_state_return_revisioned_opaque_actions() {
    let mut server = Server::jsonl();
    let new = server
        .request(json!({ "protocolVersion": 1, "requestId": "new-1", "type": "new", "seed": 42 }));
    let game_id = new["gameId"].as_str().unwrap().to_owned();
    assert_eq!(new["stateRevision"], 0);
    assert_eq!(new["turn"], 0);
    assert_eq!(new["outcome"], "in_progress");

    let state = server.request(json!({ "protocolVersion": 1, "requestId": "state-1", "type": "state", "gameId": game_id, "faction": "sauron" }));
    assert_eq!(state["type"], "state");
    assert!(
        state["observation"]
            .as_str()
            .unwrap()
            .contains("state for Sauron:")
    );
    assert!(!state["legalActions"].as_array().unwrap().is_empty());
    assert!(
        state["stateHash"]
            .as_str()
            .is_some_and(|value| value.len() == 64)
    );
    assert!(
        state["legalActions"][0]["actionId"]
            .as_str()
            .unwrap()
            .starts_with("r0-a")
    );
    assert!(
        state["legalActions"][0]["description"]
            .as_str()
            .is_some_and(|description| {
                description.starts_with("Take chapter card chapter1_card_")
                    && description.contains("Effects:")
                    && description.contains("current payment:")
                    && description.contains("unlocks")
                    && description.contains("Immediate result:")
            })
    );
}

#[test]
fn choose_rejects_stale_revision_and_unissued_actions() {
    let mut server = Server::jsonl();
    let new = server.request(json!({ "protocolVersion": 1, "requestId": "new", "type": "new" }));
    let game_id = new["gameId"].as_str().unwrap();
    let state = server.request(json!({ "protocolVersion": 1, "requestId": "state", "type": "state", "gameId": game_id, "faction": "sauron" }));
    let action_id = state["legalActions"][0]["actionId"].as_str().unwrap();
    let turn = state["turn"].as_u64().unwrap();
    let state_hash = state["stateHash"].as_str().unwrap();
    let accepted = server.request(json!({ "protocolVersion": 1, "requestId": "choose", "type": "choose", "gameId": game_id, "stateRevision": 0, "turn": turn, "stateHash": state_hash, "faction": "sauron", "actionId": action_id }));
    assert_eq!(accepted["type"], "choose");
    assert_eq!(accepted["stateRevision"], 1);

    let stale = server.request(json!({ "protocolVersion": 1, "requestId": "stale", "type": "choose", "gameId": game_id, "stateRevision": 0, "turn": turn, "stateHash": state_hash, "faction": "sauron", "actionId": action_id }));
    assert_eq!(stale["type"], "error");
    assert!(stale["error"].as_str().unwrap().contains("stateRevision"));

    let invalid = server.request(json!({ "protocolVersion": 1, "requestId": "invalid", "type": "choose", "gameId": game_id, "stateRevision": 1, "turn": accepted["turn"], "stateHash": accepted["stateHash"], "faction": "fellowship", "actionId": "not-issued" }));
    assert_eq!(invalid["type"], "error");
    assert!(invalid["error"].as_str().unwrap().contains("actionId"));
}

#[test]
fn choose_rejects_stale_turn_and_state_hash() {
    let mut server = Server::jsonl();
    let new = server.request(json!({ "protocolVersion": 1, "requestId": "new", "type": "new" }));
    let game_id = new["gameId"].as_str().unwrap();
    let state = server.request(json!({ "protocolVersion": 1, "requestId": "state", "type": "state", "gameId": game_id, "faction": "sauron" }));
    let action_id = state["legalActions"][0]["actionId"].as_str().unwrap();
    let stale_turn = server.request(json!({ "protocolVersion": 1, "requestId": "turn", "type": "choose", "gameId": game_id, "stateRevision": state["stateRevision"], "turn": 999, "stateHash": state["stateHash"], "faction": "sauron", "actionId": action_id }));
    assert!(stale_turn["error"].as_str().unwrap().contains("turn"));
    let stale_hash = server.request(json!({ "protocolVersion": 1, "requestId": "hash", "type": "choose", "gameId": game_id, "stateRevision": state["stateRevision"], "turn": state["turn"], "stateHash": "0".repeat(64), "faction": "sauron", "actionId": action_id }));
    assert!(stale_hash["error"].as_str().unwrap().contains("stateHash"));
}

#[test]
fn jsonl_validates_requests() {
    let mut server = Server::jsonl();
    let unsupported = server
        .request(json!({ "protocolVersion": 2, "requestId": "bad-version", "type": "hello" }));
    assert_eq!(unsupported["type"], "error");
    assert_eq!(unsupported["requestId"], "bad-version");
    assert!(
        unsupported["error"]
            .as_str()
            .unwrap()
            .contains("protocolVersion")
    );

    let malformed = server.request(json!({ "protocolVersion": 1, "requestId": "missing-game", "type": "state", "faction": "sauron" }));
    assert_eq!(malformed["type"], "error");
    assert!(
        malformed["error"]
            .as_str()
            .unwrap()
            .contains("invalid request")
    );
}

#[test]
fn legacy_text_protocol_remains_framed_and_unchanged() {
    let mut server = Server::text();
    assert_eq!(
        server.text_request("new 42"),
        "game created; use `state <fellowship|sauron>`\n"
    );
    let state = server.text_request("state sauron");
    assert!(state.starts_with("state for Sauron:\n"));
    assert!(state.contains("actions:\n0. "));
}
