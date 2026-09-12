# Rule Coverage

The automated suite covers these implemented rule groups:

- Deterministic setup, hidden-card visibility, chapter layouts and transitions, including the Chapter 2 and 3 row geometry.
- Action authorization through `legal_actions_for` and `apply_for`, illegal-action rejection, and direct game-over rejection.
- Card costs, chained free plays, alternative Skill production, Elf any-Skill payment, Landmark Fortress surcharges, and the Dwarf waiver.
- Coin reserve caps, coin loss caps, discard income, and the Human discard multiplier.
- Quest movement, both quest victories, and all four quest-bonus variants (coin, unit, extra turn, and Fortress removal).
- Unit conflicts, placement, movement, removal, map domination, explicit empty/unavailable fallbacks, and final territorial scoring including Fortress presence.
- Alliance trigger ordering, token selection/return, immediate Ent/Wizard effects, and persistent Elf, Human, Dwarf, and Hobbit effects represented by the current catalog.
- Landmark resolution paths for Fortress, unit, quest, Grey-card discard, Alliance-token, and discarded-card effects.
- Card, Landmark, and Alliance-token JSON schema/catalogue cardinality, uniqueness, family invariants, component totals, quest spaces, layouts, regions, and chaining aggregates. See `CATALOG_MANIFEST.md` and `CATALOG_AMBIGUITIES.md` for source status.

## Known Limitations

- Mandatory Ent Fortress removal has no defined behavior when no enemy Fortress exists. The ignored regression `regression_mandatory_ent_fortress_removal_without_enemy_fortress_deadlocks` documents the resulting zero-action state until the rule semantics are specified.
- `CheckImmediateVictoryConditions` and `CheckPostRevealEffects` are currently intentional no-op queue markers; implemented victories are checked by their effect resolvers.
- `choose` fields in card and Landmark data are represented, but placement resolution currently selects one legal region and places the full listed amount there.
