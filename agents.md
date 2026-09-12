# AGENTS.md

## Project overview

This repository implements a deterministic rules engine for **The Lord of the Rings: Duel for Middle-earth**.

Rules accuracy is the highest priority. Correctness, determinism, debuggability, and test coverage are more important than clever abstractions or premature optimization.

This project implements **Duel for Middle-earth**, not the original **7 Wonders Duel**. Never transfer rules from 7 Wonders Duel unless the supplied Duel for Middle-earth rules explicitly support them.

## Sources of truth

Use sources in this order:

1. Official Duel for Middle-earth rulebook (/rules.pdf)
2. Official player aid (/helpsheet.pdf)
3. Physical cards and components or verified scans
4. Explicit project rulings documented in this repository
5. Existing implementation
6. Existing tests

Existing code and tests are not authoritative when they contradict the official rules.

When sources disagree or a rule is unclear:

- Do not guess.
- Do not silently choose the most convenient interpretation.
- Record the ambiguity with the relevant rule reference.
- Preserve current behavior unless the task explicitly requires resolving it.
- Add a test marked or named to reflect the pending ruling when appropriate.

Do not use the original 7 Wonders Duel rules as a substitute.

## Before making changes

Before editing:

1. Read this file and any nested `AGENTS.md` files.
2. Inspect the relevant production code, tests, and game data.
3. Identify the official rule or component being implemented.
4. Determine whether the requested behavior already exists.
5. Check the working tree and preserve unrelated user changes.

For substantial changes, state a short implementation plan before editing.

## Engine model

Treat the engine as a deterministic state transition:

```text
current state + player action -> result
```

Record any rules decision you don't understand or are ambigous in docs/rules-decisions.md

using this template

### RULE-000 — Short title

- Status: proposed | confirmed
- Source:
- Question:
- Decision:
- Reasoning:
- Alternatives:
- Enforced by:
