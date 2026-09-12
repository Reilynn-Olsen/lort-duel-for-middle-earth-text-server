use duel_for_middle_earth_rules::{Action, Faction, Game};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};

const JSONL_PROTOCOL_VERSION: u32 = 1;

fn main() {
    if std::env::args()
        .skip(1)
        .any(|argument| argument == "--jsonl")
    {
        run_jsonl();
    } else {
        run_text();
    }
}

fn run_text() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut game: Option<Game> = None;
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                reply(&mut stdout, format!("error: input: {error}"));
                continue;
            }
        };
        let mut words = line.split_whitespace();
        let command = match words.next() {
            Some(command) => command,
            None => continue,
        };
        let response = match command {
            "new" => match words.next().map(str::parse).transpose() {
                Ok(seed) => {
                    game = Some(Game::new(seed.unwrap_or(0)));
                    "game created; use `state <fellowship|sauron>`".into()
                }
                Err(_) => "error: seed must be an unsigned integer".into(),
            },
            "state" | "legal" => observation(&game, words.next()),
            "choose" => choose(&mut game, words.next(), words.next()),
            "help" => "commands: new [seed] | state <fellowship|sauron> | choose <fellowship|sauron> <action-number> | help | quit".into(),
            "quit" => break,
            _ => "error: unknown command (use 'help')".into(),
        };
        reply(&mut stdout, response);
    }
}

fn run_jsonl() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut session = JsonlSession::default();
    for line in stdin.lock().lines() {
        let response = match line {
            Ok(line) => match serde_json::from_str::<JsonRequest>(&line) {
                Ok(request) => session.handle(request),
                Err(error) => JsonResponse::error(None, format!("invalid request: {error}")),
            },
            Err(error) => JsonResponse::error(None, format!("input error: {error}")),
        };
        serde_json::to_writer(&mut stdout, &response).expect("JSON response must serialize");
        writeln!(stdout).expect("stdout must be writable");
        stdout.flush().expect("stdout must be flushable");
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum JsonRequest {
    Hello {
        #[serde(rename = "protocolVersion")]
        protocol_version: u32,
        #[serde(rename = "requestId")]
        request_id: String,
    },
    New {
        #[serde(rename = "protocolVersion")]
        protocol_version: u32,
        #[serde(rename = "requestId")]
        request_id: String,
        seed: Option<u64>,
    },
    State {
        #[serde(rename = "protocolVersion")]
        protocol_version: u32,
        #[serde(rename = "requestId")]
        request_id: String,
        #[serde(rename = "gameId")]
        game_id: String,
        faction: String,
    },
    Choose {
        #[serde(rename = "protocolVersion")]
        protocol_version: u32,
        #[serde(rename = "requestId")]
        request_id: String,
        #[serde(rename = "gameId")]
        game_id: String,
        #[serde(rename = "stateRevision")]
        state_revision: u64,
        turn: u32,
        #[serde(rename = "stateHash")]
        state_hash: String,
        faction: String,
        #[serde(rename = "actionId")]
        action_id: String,
    },
}

impl JsonRequest {
    fn protocol_version(&self) -> u32 {
        match self {
            Self::Hello {
                protocol_version, ..
            }
            | Self::New {
                protocol_version, ..
            }
            | Self::State {
                protocol_version, ..
            }
            | Self::Choose {
                protocol_version, ..
            } => *protocol_version,
        }
    }

    fn request_id(&self) -> &str {
        match self {
            Self::Hello { request_id, .. }
            | Self::New { request_id, .. }
            | Self::State { request_id, .. }
            | Self::Choose { request_id, .. } => request_id,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonResponse {
    protocol_version: u32,
    request_id: Option<String>,
    #[serde(rename = "type")]
    response_type: &'static str,
    game_id: Option<String>,
    engine_version: String,
    state_revision: Option<u64>,
    turn: Option<u32>,
    state_hash: Option<String>,
    outcome: Option<Outcome>,
    winner: Option<String>,
    observation: Option<String>,
    legal_actions: Vec<LegalAction>,
    capabilities: Vec<String>,
    error: Option<String>,
}

impl JsonResponse {
    fn error(request_id: Option<String>, error: String) -> Self {
        Self {
            protocol_version: JSONL_PROTOCOL_VERSION,
            request_id,
            response_type: "error",
            game_id: None,
            engine_version: engine_version(),
            state_revision: None,
            turn: None,
            state_hash: None,
            outcome: None,
            winner: None,
            observation: None,
            legal_actions: Vec::new(),
            capabilities: Vec::new(),
            error: Some(error),
        }
    }

    fn with_game_id(mut self, game_id: Option<String>) -> Self {
        self.game_id = game_id;
        self
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum Outcome {
    InProgress,
    Winner,
    SharedVictory,
    Stalled,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LegalAction {
    action_id: String,
    description: String,
}

#[derive(Default)]
struct JsonlSession {
    game: Option<Game>,
    game_id: Option<String>,
    state_revision: u64,
    next_game_id: u64,
    issued_actions: HashMap<String, IssuedAction>,
}

#[derive(Clone)]
struct IssuedAction {
    faction: Faction,
    action: Action,
    turn: u32,
    state_hash: String,
}

impl JsonlSession {
    fn handle(&mut self, request: JsonRequest) -> JsonResponse {
        if request.protocol_version() != JSONL_PROTOCOL_VERSION {
            return JsonResponse::error(
                Some(request.request_id().into()),
                format!("unsupported protocolVersion {}", request.protocol_version()),
            );
        }
        match request {
            JsonRequest::Hello { request_id, .. } => JsonResponse {
                protocol_version: JSONL_PROTOCOL_VERSION,
                request_id: Some(request_id),
                response_type: "hello",
                game_id: None,
                engine_version: engine_version(),
                state_revision: None,
                turn: None,
                state_hash: None,
                outcome: None,
                winner: None,
                observation: None,
                legal_actions: Vec::new(),
                capabilities: vec![
                    "hello".into(),
                    "new".into(),
                    "state".into(),
                    "choose".into(),
                    "state_revision".into(),
                    "turn".into(),
                    "state_hash".into(),
                    "opaque_action_ids".into(),
                ],
                error: None,
            },
            JsonRequest::New {
                request_id, seed, ..
            } => {
                self.next_game_id += 1;
                self.game_id = Some(format!("game-{}", self.next_game_id));
                self.game = Some(Game::new(seed.unwrap_or(0)));
                self.state_revision = 0;
                self.issued_actions.clear();
                self.response(request_id, "new", None)
            }
            JsonRequest::State {
                request_id,
                game_id,
                faction,
                ..
            } => match self.validate_game_and_faction(&game_id, &faction) {
                Ok(faction) => self.response(request_id, "state", Some(faction)),
                Err(error) => JsonResponse::error(Some(request_id), error),
            },
            JsonRequest::Choose {
                request_id,
                game_id,
                state_revision,
                turn,
                state_hash,
                faction,
                action_id,
                ..
            } => {
                let faction = match self.validate_game_and_faction(&game_id, &faction) {
                    Ok(faction) => faction,
                    Err(error) => return JsonResponse::error(Some(request_id), error),
                };
                if state_revision != self.state_revision {
                    return JsonResponse::error(
                        Some(request_id),
                        "stateRevision does not match current state".into(),
                    )
                    .with_game_id(self.game_id.clone());
                }
                let Some(issued) = self.issued_actions.get(&action_id).cloned() else {
                    return JsonResponse::error(
                        Some(request_id),
                        "actionId was not issued for the current state".into(),
                    )
                    .with_game_id(self.game_id.clone());
                };
                if issued.faction != faction {
                    return JsonResponse::error(
                        Some(request_id),
                        "actionId was not issued for this faction".into(),
                    )
                    .with_game_id(self.game_id.clone());
                }
                if turn != issued.turn {
                    return JsonResponse::error(
                        Some(request_id),
                        "turn does not match current state".into(),
                    )
                    .with_game_id(self.game_id.clone());
                }
                if state_hash != issued.state_hash {
                    return JsonResponse::error(
                        Some(request_id),
                        "stateHash does not match current state".into(),
                    )
                    .with_game_id(self.game_id.clone());
                }
                let game = self.game.as_mut().expect("validated game exists");
                if let Err(error) = game.apply_for(faction, issued.action) {
                    return JsonResponse::error(
                        Some(request_id),
                        format!("action rejected: {error}"),
                    )
                    .with_game_id(self.game_id.clone());
                }
                self.state_revision += 1;
                self.issued_actions.clear();
                self.response(request_id, "choose", Some(faction))
            }
        }
    }

    fn validate_game_and_faction(&self, game_id: &str, faction: &str) -> Result<Faction, String> {
        if self.game.is_none() {
            return Err("start a game with a new request".into());
        }
        if self.game_id.as_deref() != Some(game_id) {
            return Err("gameId does not match the current game".into());
        }
        faction
            .parse()
            .map_err(|_| "faction must be fellowship or sauron".into())
    }

    fn response(
        &mut self,
        request_id: String,
        response_type: &'static str,
        faction: Option<Faction>,
    ) -> JsonResponse {
        let game = self.game.as_ref().expect("response requires a game");
        let outcome = outcome_for(game);
        let turn = game.turn();
        let observation = faction.map(|faction| game.render_observation_for(faction));
        let state_hash = observation.as_deref().map(public_state_hash);
        let legal_actions = faction.map_or_else(Vec::new, |faction| {
            game.legal_actions_for(faction)
                .into_iter()
                .enumerate()
                .map(|(index, action)| {
                    let action_id = format!("r{}-a{}", self.state_revision, index);
                    let description = game.describe_action(&action);
                    self.issued_actions.insert(
                        action_id.clone(),
                        IssuedAction {
                            faction,
                            action,
                            turn,
                            state_hash: state_hash
                                .clone()
                                .expect("viewer-scoped legal actions have an observation hash"),
                        },
                    );
                    LegalAction {
                        action_id,
                        description,
                    }
                })
                .collect()
        });
        JsonResponse {
            protocol_version: JSONL_PROTOCOL_VERSION,
            request_id: Some(request_id),
            response_type,
            game_id: self.game_id.clone(),
            engine_version: engine_version(),
            state_revision: Some(self.state_revision),
            turn: Some(turn),
            state_hash,
            outcome: Some(outcome),
            winner: game.winner().map(faction_name),
            observation,
            legal_actions,
            capabilities: Vec::new(),
            error: None,
        }
    }
}

fn public_state_hash(observation: &str) -> String {
    format!("{:x}", Sha256::digest(observation.as_bytes()))
}

fn outcome_for(game: &Game) -> Outcome {
    if game.winner().is_some() {
        Outcome::Winner
    } else if game.is_shared_victory() {
        Outcome::SharedVictory
    } else if game.legal_actions_for(Faction::Fellowship).is_empty()
        && game.legal_actions_for(Faction::Sauron).is_empty()
    {
        Outcome::Stalled
    } else {
        Outcome::InProgress
    }
}

fn faction_name(faction: Faction) -> String {
    match faction {
        Faction::Fellowship => "fellowship".into(),
        Faction::Sauron => "sauron".into(),
    }
}

fn engine_version() -> String {
    let package_version = env!("CARGO_PKG_VERSION");
    let metadata = option_env!("RULES_ENGINE_BUILD")
        .or(option_env!("VERGEN_GIT_SHA"))
        .or(option_env!("GIT_COMMIT_HASH"));
    metadata.filter(|value| !value.is_empty()).map_or_else(
        || package_version.into(),
        |value| format!("{package_version}+{value}"),
    )
}

fn observation(game: &Option<Game>, player: Option<&str>) -> String {
    let Some(game) = game.as_ref() else {
        return no_game();
    };
    let Some(player) = player.and_then(|value| value.parse::<Faction>().ok()) else {
        return "error: state requires fellowship or sauron".into();
    };
    game.render_observation_for(player)
}

fn choose(game: &mut Option<Game>, player: Option<&str>, action_number: Option<&str>) -> String {
    let Some(game) = game.as_mut() else {
        return no_game();
    };
    let Some(player) = player.and_then(|value| value.parse::<Faction>().ok()) else {
        return "error: choose requires fellowship or sauron".into();
    };
    let Some(action_number) = action_number.and_then(|word| word.parse::<usize>().ok()) else {
        return "error: choose requires an action number from that player's latest response".into();
    };
    let Some(action) = game.legal_actions_for(player).get(action_number).cloned() else {
        return "error: action number is not legal in the current state".into();
    };
    match game.apply_for(player, action) {
        Ok(()) => game.render_observation_for(player),
        Err(error) => format!("error: {error}"),
    }
}

fn no_game() -> String {
    "error: start a game with 'new [seed]'".into()
}

fn reply(stdout: &mut impl Write, message: String) {
    writeln!(stdout, "{message}\n.").expect("stdout must be writable");
    stdout.flush().expect("stdout must be flushable");
}
