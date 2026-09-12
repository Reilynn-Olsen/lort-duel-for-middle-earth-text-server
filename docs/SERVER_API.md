# Rules Server Integration Guide

`rules_server` is a local, line-oriented process for a single game. It is
intended for a browser backend, desktop app, CLI client, or automated/LLM
player. It does not expose HTTP, WebSockets, JSON, authentication, persistence,
or matchmaking. For evaluation clients, use its versioned JSON-lines mode:
`cargo run --bin rules_server -- --jsonl`. The adjacent Rust project's
`docs/SERVER_API.md` is the authoritative protocol reference.

The default is the legacy text protocol described below. Pass `--jsonl` to use
the versioned JSON-lines protocol; the two formats are separate and must not be
mixed in one process.

## Start A Server

```sh
cargo run --bin rules_server
```

The process accepts UTF-8 commands on standard input and writes UTF-8 replies
to standard output. Keep one process alive for one game session. Start a new
process for an independent concurrent game.

## Framing

Send exactly one command per input line. Every reply, including errors, ends
with a line containing only `.`. Read until that terminator before sending the
next command.

```text
new 42
game created; use `state <fellowship|sauron>`
.
```

Do not treat a blank line as a reply boundary. The response body is multiline.
The `.` terminator is part of the transport protocol and is not part of the
observation.

## Commands

| Command | Result |
| --- | --- |
| `new [seed]` | Starts a game. `seed` is an optional unsigned 64-bit integer; omitted means `0`. A seed fully determines setup and randomization. |
| `state <fellowship\|sauron>` | Returns that faction's current observation and legal actions. |
| `legal <fellowship\|sauron>` | Alias for `state`. |
| `choose <fellowship\|sauron> <action-number>` | Applies an action from that faction's latest legal-action list, then returns that faction's new observation. |
| `help` | Returns the server's brief command summary. |
| `quit` | Ends the process without a reply. |

Faction values are lowercase: `fellowship` and `sauron`.

Example session:

```text
new 42
state sauron
choose sauron 0
state sauron
quit
```

## Observation Contract

A successful `state` or `choose` response is human-readable text with two
ordered sections:

```text
state for Sauron:
<public game state>
actions:
0. <legal action description>
1. <legal action description>
```

The state includes the active player, turn/chapter, outcome, player resources,
tableaus, alliance tokens, map, quest, landmarks, and visible chapter-card
layout. It deliberately withholds facedown card identities and the order of
facedown alliance tokens.

`actions:` is authoritative. The client must not construct actions from the
visible state, predict costs, or assume a phase has completed. A turn can
require several successive choices, such as selecting a card, choosing whether
to play or discard it, then choosing a Unit destination.

The state/action presentation is text intended for people and language models,
not a versioned machine-readable schema. A UI should render it as text or use
its own tolerant parser. For a production frontend that needs stable typed
fields, add a separate JSON protocol rather than parsing the prose format.

## Action Numbers And Errors

Action numbers are zero-based. They are only valid for the exact state from
which they were read. After every successful `choose`, discard the old action
list and use the list from the new response.

The engine validates the faction and action again when `choose` is received.
Typical errors are:

```text
error: action number is not legal in the current state
.
```

```text
error: choose requires fellowship or sauron
.
```

On an error, the game remains unchanged. Request `state` again before retrying.
Clients should display the error, not silently substitute another action.

An observation with `(no legal actions)` means that faction cannot currently
act. Usually the other faction should be queried. An in-progress game with no
legal actions for either faction is a stalled rules state and should be
reported with its seed and final observations.

## Frontend Integration

A browser should not connect directly to `rules_server`; browsers cannot spawn
local processes safely. Use a backend or desktop host as the adapter:

1. Spawn one `rules_server` child process per game.
2. Write a command plus `\n` to its stdin.
3. Buffer stdout lines until `.`.
4. Forward the response body to the browser and show it as the current game
   view.
5. Render the numbered actions as buttons only for the faction that requested
   the observation.
6. On a button click, send `choose <faction> <number>` and replace the entire
   displayed observation with the reply.

Always serialize commands per game process: wait for a framed reply before
sending another command. If the UI has two human players, refresh both
factions' observations after each successful choice so the inactive player can
see the changed public state. Do not expose an action button when that
faction's response says `(no legal actions)`.

The process is stateful and in-memory. If it exits or the host restarts, the
game is lost unless the host records a seed and complete accepted command log.
Replaying `new <seed>` followed by the accepted `choose` commands restores the
same game, subject to using the same engine version.

## LLM Harness Integration

Treat each legal action line as a closed choice set. The simplest reliable loop
is:

1. Start `rules_server` and send `new <seed>`.
2. Send `state <controlled-faction>`.
3. If the reply has legal actions, give the state and numbered actions to the
   model and require it to return one integer action number only.
4. Validate the model's value is an integer listed in the latest response.
5. Send `choose <controlled-faction> <number>`.
6. Repeat from the returned observation until the outcome is no longer `in progress`.

For self-play, query both factions each cycle. Exactly one should normally have
actions; ask that faction's model to choose. A choice generated by an LLM is
untrusted input: validate it locally and retain the last valid observation on a
malformed or unavailable action number.

Record the seed, engine revision, faction, action number, action text, and full
response after every accepted choice. This makes failures reproducible and lets
you audit that an agent only used engine-authorized actions.

## Determinism

The same seed plus the same ordered `choose` commands yields the same game.
This supports reproducible UI bug reports, LLM evaluations, and replay files.
Different action choices naturally produce different game paths even with an
identical seed.

## Current Scope

The server is a development integration boundary. It has no network service,
accounts, rate limiting, or save/load command. Keep it behind a trusted adapter
if building a multi-user app.

## JSON-Lines Protocol Version 1

Start it with `cargo run --bin rules_server -- --jsonl`. Each UTF-8 input line
is one JSON object and produces exactly one JSON object on standard output; no
`.` framing line is used. Every request contains `protocolVersion: 1`, a client
chosen `requestId`, and a `type` of `hello`, `new`, `state`, or `choose`.

`hello` advertises the supported version and capabilities. `new` accepts an
optional unsigned `seed` and returns a new `gameId` at `stateRevision: 0`.
`state` requires `gameId` and lowercase `faction`. `choose` requires `gameId`,
`stateRevision`, `turn`, `stateHash`, `faction`, and an `actionId` from `legalActions` in a `state`
or successful `choose` response for that exact revision and faction.

```json
{"protocolVersion":1,"requestId":"new-1","type":"new","seed":42}
{"protocolVersion":1,"requestId":"state-1","type":"state","gameId":"game-1","faction":"sauron"}
{"protocolVersion":1,"requestId":"choose-1","type":"choose","gameId":"game-1","stateRevision":0,"turn":0,"stateHash":"<hash from state response>","faction":"sauron","actionId":"r0-a0"}
```

Responses always include `protocolVersion`, `requestId` (or `null` if the
request could not be parsed), `type`, `engineVersion`, `gameId`,
`stateRevision`, `turn`, a SHA-256 `stateHash` of the viewer-scoped public
`observation`, `outcome`, public `observation` text, `legalActions`, and
`error`. Every legal action contains an opaque `actionId` and a human-readable
`description` of its effect. Chapter-card descriptions identify the card,
printed cost, effects, current play cost, and discard income. Submit only the
ID: never construct or infer one.
`winner` is present when the outcome is `winner`. Outcomes are `in_progress`,
`winner`, `shared_victory`, and `stalled`. `stalled` means the game is not over
but neither faction has an action according to the engine.

Errors leave the game unchanged. A successful `choose` increments
`stateRevision`, invalidates all previously issued action IDs, and returns the
new observation and action IDs for the choosing faction. The server validates
the ID against the game’s current legal actions and applies it through the
authoritative `Game` API.
