# Board Test Constraints

## Two Fortresses In One Region

- Status: impossible under the supplied board.
- Source: Rulebook p. 7 depicts one Fortress space per region.
- Decision: a region has one `fortress: Option<Faction>` and cannot contain both
  players' Fortresses. Tests cover friendly and enemy Fortress interactions with
  Units, plus both players having presence through a Fortress for one player and
  Units for the other.
