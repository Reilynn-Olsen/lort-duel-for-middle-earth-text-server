# Duel for Middle-earth Rules Engine

A deterministic Rust rules engine for *The Lord of the Rings: Duel for Middle-earth*. It models authoritative game state, validates player actions, and provides a text-first interface for command-line clients, application backends, and automated players.

This is a rules engine, not a complete game application: it has no graphical interface, network service, matchmaking, persistence, or authentication.

## Features

- Deterministic setup and replay from a seed and accepted action sequence.
- Authoritative action generation and validation through `Game`.
- Three chapters, card layouts, costs, construction, chaining, and discard income.
- Landmarks, alliance tokens, quest bonuses, units, map conflicts, and territorial scoring.
- Victory checks for all implemented victory paths.
- Hidden-information-safe text rendering for each faction.
- A line-oriented local server for frontends and LLM/agent harnesses.
- A deterministic random-play executable for smoke testing and replays.

See [rule coverage](docs/RULE_COVERAGE.md) for tested rule groups and known limitations.

## Requirements

- A current Rust toolchain with Cargo (the crate uses Rust edition 2024).

## Getting Started

Clone the repository and run the test suite:

```sh
cargo test
```

Run a deterministic random game:

```sh
cargo run --bin random_game -- 42 100
```

The optional arguments are the setup seed (default `0`) and maximum number of actions (default `500`). Reusing the same seed and action choices reproduces the same game.

## Text Server

Start a local process that manages one in-memory game:

```sh
cargo run --bin rules_server
```

Send one UTF-8 command per line on standard input. Each reply ends with a line containing only `.`, which is the response delimiter.

```text
new 42
state sauron
choose sauron 0
state fellowship
quit
```

Supported commands:

| Command | Description |
| --- | --- |
| `new [seed]` | Create a game; the default seed is `0`. |
| `state <fellowship|sauron>` | Return that faction's observation and legal actions. |
| `legal <fellowship|sauron>` | Alias for `state`. |
| `choose <fellowship|sauron> <number>` | Apply an action from that faction's current legal-action list. |
| `help` | Show the command summary. |
| `quit` | End the process. |

Action numbers are zero-based and valid only for the state that returned them. Clients must use the `actions:` section as the authoritative set of possible moves rather than deriving moves from displayed state.

Read the [server integration guide](docs/SERVER_API.md) for the full framing protocol, error behavior, browser-backend architecture, and agent integration guidance.

## Library Usage

`Game` is the authoritative state-transition boundary. Generate actions for a faction, then submit one of those actions back to the game:

```rust
use duel_for_middle_earth_rules::{Faction, Game};

let mut game = Game::new(42);
let faction = game.active_player();
let action = game.legal_actions_for(faction)[0].clone();
game.apply_for(faction, action)?;

assert_eq!(faction, Faction::Sauron);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Use `render_observation_for` to produce a faction-safe text observation. Clients should not mutate game state directly or construct actions outside the engine.

## Project Layout

| Path | Contents |
| --- | --- |
| `src/` | Rules engine, catalogue schemas, tests, and executables. |
| `cards.json`, `landmark_tiles.json`, `alliance_tokens.json` | Game component catalogues. |
| `docs/SERVER_API.md` | Text-server protocol and client integration guide. |
| `docs/RULE_COVERAGE.md` | Implemented rule coverage and known limitations. |
| `docs/CATALOG_MANIFEST.md` | Catalogue provenance and component inventory. |
| `docs/rules-decisions.md` | Recorded rules interpretations and unresolved decisions. |
| `rules.pdf`, `helpsheet.pdf` | Official game rulebook and player aid supplied with the project. |

## Rule Sources And Contributions

Rules accuracy, determinism, debuggability, and test coverage take priority over convenience. The official rulebook and player aid are the primary sources of truth; do not infer rules from the original *7 Wonders Duel*.

Before changing rules behavior, review `agents.md`, the relevant tests and data catalogue, and any applicable entries in [rules decisions](docs/rules-decisions.md). Document unresolved ambiguities instead of guessing.

## Limitations

- The text server is a local development boundary, not an HTTP or WebSocket service.
- Games live only in memory. To replay a session, retain the seed and accepted `choose` commands and use the same engine version.
- The text observation is designed for people and language models, not as a versioned typed API. Add a separate structured protocol for production clients that need stable fields.
- Known rule-edge limitations are documented in [rule coverage](docs/RULE_COVERAGE.md).

## Acknowledgments

*The Lord of the Rings: Duel for Middle-earth* is a game by Antoine Bauza and Bruno Cathala, published by Repos Production. This repository is an independent, unofficial rules-engine project and is not affiliated with or endorsed by the game's designers or publisher.
