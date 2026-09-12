# Catalog Manifest

This is the review index for declarative content. The machine-readable manifests
are `cards.json`, `landmark_tiles.json`, and `alliance_tokens.json`; their full
field values are validated when the catalog loads and by `catalog_tests`.

## Component Totals

| Component | Manifest | Official source |
| --- | ---: | --- |
| Chapter cards | 69, 23 per Chapter | Rulebook p. 1 |
| Landmark tiles | 7 | Rulebook p. 1; Player Aid p. 1 |
| Alliance tokens | 18, 3 per Race | Rulebook p. 1; Player Aid p. 2 |
| Quest bonuses | 8 | Rulebook p. 6; Player Aid p. 1 |
| Regions | 7 | Rulebook p. 7 |

## Chapter Card Index

Cards have no textual names in the supplied rulebook or player aid. Their stable
IDs are `chapter{N}_card_{NN}`. `cards.json` is the full reviewable transcription
of every printed cost, Skill, chaining symbol, Race, and effect.

| Chapter | IDs | Colors |
| --- | --- | --- |
| 1 | `chapter1_card_01` through `chapter1_card_23` | 8 Grey, 4 Yellow, 4 Blue, 4 Green, 3 Red |
| 2 | `chapter2_card_01` through `chapter2_card_23` | 7 Grey, 2 Yellow, 5 Blue, 4 Green, 5 Red |
| 3 | `chapter3_card_01` through `chapter3_card_23` | 2 Yellow, 5 Blue, 4 Green, 6 Red, 6 Purple |

There are 17 chaining requirements, 17 chaining symbols provided, two choice-
Skill cards, and twelve cards that provide Race symbols. Red cards always choose one
of their two printed regions; each records both regions and the complete Unit
quantity in `cards.json`.

## Source-Confirmed Content

- Chapter display geometry and face-up/face-down rows: Rulebook p. 3.
- Quest spaces: 3 coin, 6 Unit, 9 extra turn, 12 remove Fortress, then 18 coin,
  21 Unit, 24 extra turn, and 27 remove Fortress: Rulebook p. 6 and Player Aid p. 1.
- Landmark names, regions, and effects: Player Aid p. 1.
- Alliance-token effects: Player Aid p. 2.
- Region names and printed connections: Rulebook p. 7.

## Verification Gap

The supplied PDFs do not contain a card-face reference sheet or individual card
images. Therefore the 69 card records are structurally validated and locked by
stable IDs and aggregate assertions, but their individual printed values have not
been independently verified from supplied source material. See
`docs/CATALOG_AMBIGUITIES.md` before correcting any such record.
