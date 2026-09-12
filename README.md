# Duel for Middle-earth rules engine

An in-progress deterministic Rust rules engine for *The Lord of the Rings: Duel for Middle-earth*. Its boundary is deliberately text-only so automated players, including LLM harnesses, can use it without browser/UI dependencies.

## Run the text server

```sh
cargo run --bin rules_server
```

It reads one command per line and returns a response terminated by a line containing `.`. This makes multi-line responses unambiguous for a harness. Every successful game response has exactly two ordered sections: `state for <faction>:` followed by `actions:`. The action list is numbered; an agent submits the selected number with `choose <faction> <number>`.

```text
new 42
state sauron
choose sauron 0
quit
```

`state <faction>` returns the complete observation (both sections), rather than only state. `legal` remains a compatibility alias. Action numbers are valid only for the most recent observation; the engine recomputes and validates them on `choose`.

Current commands: `new [seed]`, `state <fellowship|sauron>`, `legal <fellowship|sauron>`, `choose <fellowship|sauron> <number>`, `help`, and `quit`. See [the server integration guide](docs/SERVER_API.md) for frontend and LLM-harness guidance.

## Watch random bots

Run two deterministic bots that select only engine-provided legal actions:

```sh
cargo run --bin random_game -- 42 100
```

The optional arguments are `seed` (default `0`) and maximum actions (default
`500`). The output prints the acting faction, selected action, and current
observation after every state transition. Re-run the same seed to replay it.

## Turn-resolution model

The engine uses a FIFO effect queue and a `pending_decision` state. An automatic sequence runs until it ends or a player choice is needed; the action list then contains only actions that answer that choice. A chapter-card turn currently works as follows: choose an available face-up card, choose to play or discard it, then resolve effects, immediate victory checks, newly-uncovered-card reveals, post-reveal checks, and turn passing. Discard income is implemented (1/2/3 coins by chapter).

Card costs/effects, landmark costs/effects, victory checks, extra turns, and chapter transitions are the next additions. Landmark actions are intentionally not offered until their costs can be validated correctly.

## Implemented setup

`new <seed>` deterministically creates the complete initial configuration:

- Fellowship/Sauron supplies, quest characters, starting coins, units, and Sauron first turn;
- the seven-region map and its adjacency graph;
- Quest of the Ring positions and bonus-space locations;
- independently shuffled three-token race stacks;
- seven shuffled landmark placeholders, with three revealed;
- three independently shuffled 23-card chapter decks, each with a 20-card overlapping layout and three facedown discards.

The card and landmark IDs are deliberate placeholders until their official names, costs, and effects are encoded. Public text state never exposes a facedown card's identity or an unrevealed token order.

## Architectural commitments

- `Game` owns all authoritative state and uses a seed for reproducible setup.
- `legal_actions` is the single source of truth for what an agent may do.
- `apply` validates actions; clients cannot mutate state directly.
- Rendering is separate from rules and remains plain text.

## Next rules milestones

1. Encode the published components/card catalogue from a verified source.
2. Model chapter layouts, face-down information, and valid card selection.
3. Implement resources, construction, landmarks, and the three victory paths.
4. Add setup/action golden tests and replay fixtures before training agents.
