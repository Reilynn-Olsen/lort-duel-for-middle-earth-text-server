//! Physical chapter-deck construction and layout geometry.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use crate::{
    AllianceToken, CARDS_PER_CHAPTER, CardSlot, CardVisibility, ChapterCard, ChapterState,
    alliance_schema::AllianceTokenDefinition, card_schema::CardDefinition,
    landmark_schema::LandmarkDefinition,
};

pub(crate) fn card_definition(card: ChapterCard) -> &'static CardDefinition {
    let id = format!("chapter{}_card_{:02}", card.chapter, card.number);
    catalogue()
        .iter()
        .find(|definition| definition.id == id)
        .expect("every physical card must have a definition")
}

fn catalogue() -> &'static Vec<CardDefinition> {
    static CATALOGUE: OnceLock<Vec<CardDefinition>> = OnceLock::new();
    CATALOGUE.get_or_init(|| {
        let cards: Vec<CardDefinition> = serde_json::from_str(include_str!("../cards.json"))
            .expect("cards.json must match the schema");
        for card in &cards {
            card.validate()
                .expect("cards.json must respect its schema and card-family semantics");
        }
        assert_eq!(
            cards.len(),
            CARDS_PER_CHAPTER * 3,
            "the game has 69 Chapter cards"
        );
        let mut ids = BTreeSet::new();
        for chapter in 1..=3 {
            let chapter_cards = cards
                .iter()
                .filter(|card| card.chapter == chapter)
                .collect::<Vec<_>>();
            assert_eq!(
                chapter_cards.len(),
                CARDS_PER_CHAPTER,
                "each Chapter has 23 cards"
            );
            for card in chapter_cards {
                assert!(ids.insert(card.id.clone()), "card IDs must be unique");
                assert_eq!(
                    card.id,
                    format!(
                        "chapter{chapter}_card_{:02}",
                        card.id[card.id.len() - 2..]
                            .parse::<u8>()
                            .expect("card ID suffix")
                    ),
                    "card ID must agree with its Chapter"
                );
            }
        }
        cards
    })
}

pub(crate) fn landmark_definition(id: &str) -> &'static LandmarkDefinition {
    landmark_catalogue()
        .iter()
        .find(|definition| definition.id == id)
        .expect("every Landmark ID must have a definition")
}

pub(crate) fn landmark_ids() -> Vec<String> {
    landmark_catalogue()
        .iter()
        .map(|definition| definition.id.clone())
        .collect()
}

fn landmark_catalogue() -> &'static Vec<LandmarkDefinition> {
    static CATALOGUE: OnceLock<Vec<LandmarkDefinition>> = OnceLock::new();
    CATALOGUE.get_or_init(|| {
        let landmarks: Vec<LandmarkDefinition> =
            serde_json::from_str(include_str!("../landmark_tiles.json"))
                .expect("landmark_tiles.json must match the schema");
        assert_eq!(landmarks.len(), 7, "the game has exactly seven Landmarks");
        let mut ids = BTreeSet::new();
        let mut regions = Vec::new();
        for landmark in &landmarks {
            validate_landmark(landmark).expect("landmark_tiles.json must respect its schema");
            assert!(
                ids.insert(landmark.id.clone()),
                "Landmark IDs must be unique"
            );
            assert!(
                !regions.contains(&landmark.region),
                "each Landmark must occupy a different region"
            );
            regions.push(landmark.region);
        }
        landmarks
    })
}

pub(crate) fn alliance_token_definition(token: AllianceToken) -> &'static AllianceTokenDefinition {
    alliance_catalogue()
        .iter()
        .find(|definition| definition.race == token.race.into() && definition.id == token.id)
        .expect("every Alliance token must have a definition")
}

fn alliance_catalogue() -> &'static Vec<AllianceTokenDefinition> {
    static CATALOGUE: OnceLock<Vec<AllianceTokenDefinition>> = OnceLock::new();
    CATALOGUE.get_or_init(|| {
        let tokens: Vec<AllianceTokenDefinition> =
            serde_json::from_str(include_str!("../alliance_tokens.json"))
                .expect("alliance_tokens.json must match the schema");
        assert_eq!(tokens.len(), 18, "the game has eighteen Alliance tokens");
        let mut ids = Vec::new();
        for token in &tokens {
            validate_alliance_token(token).expect("alliance_tokens.json must respect its schema");
            assert!(
                (1..=3).contains(&token.id),
                "Alliance token ID must be 1, 2, or 3"
            );
            assert!(
                !ids.contains(&(token.race, token.id)),
                "Alliance token IDs must be unique per Race"
            );
            ids.push((token.race, token.id));
        }
        tokens
    })
}

fn validate_landmark(landmark: &LandmarkDefinition) -> Result<(), String> {
    use crate::landmark_schema::LandmarkEffect;

    if landmark.id.is_empty()
        || landmark.name.is_empty()
        || landmark.cost.coins_per_existing_fortress != 1
        || landmark.cost.skills.is_empty()
        || landmark.cost.skills.values().any(|amount| *amount == 0)
    {
        return Err(format!(
            "{} has an invalid Landmark identity or cost",
            landmark.id
        ));
    }
    match landmark.effects.first() {
        Some(LandmarkEffect::PlaceFortress { location }) if *location == landmark.region => {}
        _ => {
            return Err(format!(
                "{} must first place its regional Fortress",
                landmark.id
            ));
        }
    }
    Ok(())
}

fn validate_alliance_token(token: &AllianceTokenDefinition) -> Result<(), String> {
    if !(1..=3).contains(&token.id) || token.name.is_empty() {
        return Err("Alliance tokens require a numbered, named face".into());
    }
    if token.immediate.is_empty() == token.persistent.is_empty() {
        return Err(format!(
            "{} must be either immediate or persistent",
            token.name
        ));
    }
    Ok(())
}

pub(crate) fn setup_chapter(chapter: u8, rng: &mut DeterministicRng) -> ChapterState {
    let mut deck = chapter_cards(chapter);
    rng.shuffle(&mut deck);
    let positions = chapter_layout(chapter);
    let layout = positions
        .iter()
        .enumerate()
        .map(|(id, &(row, column, visibility))| CardSlot {
            id,
            row,
            column,
            visibility,
            card: deck[id],
            covering_slots: covering_slots(id, &positions),
        })
        .collect::<Vec<_>>();
    ChapterState {
        chapter,
        discarded: deck[layout.len()..].to_vec(),
        layout,
    }
}

/// Physical instances. A definition is keyed by `(chapter, number)`; duplicate
/// faces differ only in `copy`.
pub(crate) fn chapter_cards(chapter: u8) -> Vec<ChapterCard> {
    let cards = (1..=CARDS_PER_CHAPTER as u8)
        .map(|number| ChapterCard {
            chapter,
            number,
            copy: 0,
        })
        .collect::<Vec<_>>();
    debug_assert_eq!(cards.len(), CARDS_PER_CHAPTER);
    cards
}

fn covering_slots(slot_id: usize, positions: &[(u8, u8, CardVisibility)]) -> Vec<usize> {
    let (row, column, _) = positions[slot_id];
    let row_count = positions
        .iter()
        .filter(|(candidate_row, _, _)| *candidate_row == row)
        .count() as i16;
    let center = 2 * column as i16 - row_count + 1;
    positions
        .iter()
        .enumerate()
        .filter_map(|(id, &(other_row, other_column, _))| {
            if other_row != row + 1 {
                return None;
            }
            let other_count = positions
                .iter()
                .filter(|(candidate_row, _, _)| *candidate_row == other_row)
                .count() as i16;
            let other_center = 2 * other_column as i16 - other_count + 1;
            (i16::abs(center - other_center) <= 1).then_some(id)
        })
        .collect()
}

fn chapter_layout(chapter: u8) -> Vec<(u8, u8, CardVisibility)> {
    let rows: &[(u8, CardVisibility)] = match chapter {
        1 => &[
            (2, CardVisibility::FaceUp),
            (3, CardVisibility::FaceDown),
            (4, CardVisibility::FaceUp),
            (5, CardVisibility::FaceDown),
            (6, CardVisibility::FaceUp),
        ],
        2 => &[
            (6, CardVisibility::FaceUp),
            (5, CardVisibility::FaceDown),
            (4, CardVisibility::FaceUp),
            (3, CardVisibility::FaceDown),
            (2, CardVisibility::FaceUp),
        ],
        3 => &[
            (2, CardVisibility::FaceUp),
            (3, CardVisibility::FaceDown),
            (4, CardVisibility::FaceUp),
            (5, CardVisibility::FaceDown),
            (4, CardVisibility::FaceUp),
            (2, CardVisibility::FaceUp),
        ],
        _ => unreachable!("only three chapters exist"),
    };
    rows.iter()
        .enumerate()
        .flat_map(|(row, (count, visibility))| {
            (0..*count).map(move |column| (row as u8, column, *visibility))
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeterministicRng(u64);
impl DeterministicRng {
    pub(crate) fn new(seed: u64) -> Self {
        Self(seed)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    pub(crate) fn shuffle<T>(&mut self, values: &mut [T]) {
        for index in (1..values.len()).rev() {
            let swap = (self.next_u64() % (index as u64 + 1)) as usize;
            values.swap(index, swap);
        }
    }
}
