use std::collections::BTreeSet;

use crate::catalog::{DeterministicRng, card_definition, setup_chapter};
use crate::*;

#[test]
fn chapter_two_and_three_layouts_have_printed_row_shapes_and_unique_cards() {
    let mut rng = DeterministicRng::new(7);
    for (chapter, rows) in [(2, vec![6, 5, 4, 3, 2]), (3, vec![2, 3, 4, 2, 4, 3, 2])] {
        let state = setup_chapter(chapter, &mut rng);
        assert_eq!(
            (0..rows.len())
                .map(|row| state
                    .layout
                    .iter()
                    .filter(|slot| slot.row == row as u8)
                    .count())
                .collect::<Vec<_>>(),
            rows
        );
        assert_eq!(state.layout.len(), CARDS_IN_LAYOUT);
        assert_eq!(state.discarded.len(), CARDS_PER_CHAPTER - CARDS_IN_LAYOUT);
        let cards = state
            .layout
            .iter()
            .map(|slot| slot.card.to_string())
            .chain(state.discarded.iter().map(ToString::to_string))
            .collect::<BTreeSet<_>>();
        assert_eq!(cards.len(), CARDS_PER_CHAPTER);
        assert!(
            cards
                .iter()
                .all(|card| card.starts_with(&format!("C{chapter}-")))
        );
    }
}

#[test]
fn catalogue_json_has_complete_unique_landmarks_and_alliance_tokens() {
    let landmarks: Vec<landmark_schema::LandmarkDefinition> =
        serde_json::from_str(include_str!("../landmark_tiles.json")).unwrap();
    assert_eq!(landmarks.len(), REGIONS.len());
    assert_eq!(
        landmarks
            .iter()
            .map(|landmark| &landmark.id)
            .collect::<BTreeSet<_>>()
            .len(),
        landmarks.len()
    );
    assert_eq!(
        landmarks
            .iter()
            .map(|landmark| format!("{:?}", landmark.region))
            .collect::<BTreeSet<_>>()
            .len(),
        REGIONS.len()
    );

    let tokens: Vec<alliance_schema::AllianceTokenDefinition> =
        serde_json::from_str(include_str!("../alliance_tokens.json")).unwrap();
    assert_eq!(tokens.len(), RACES.len() * 3);
    for race in RACES {
        assert_eq!(
            tokens
                .iter()
                .filter(|token| token.race == race.into())
                .map(|token| token.id)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([1, 2, 3])
        );
    }
}

#[test]
fn every_card_record_has_a_unique_stable_id_and_a_source_supported_family_shape() {
    let cards: Vec<card_schema::CardDefinition> =
        serde_json::from_str(include_str!("../cards.json")).unwrap();
    assert_eq!(cards.len(), 69);
    assert_eq!(
        cards
            .iter()
            .map(|card| &card.id)
            .collect::<BTreeSet<_>>()
            .len(),
        69
    );
    for chapter in 1..=3 {
        assert_eq!(
            cards.iter().filter(|card| card.chapter == chapter).count(),
            23
        );
        for number in 1..=23 {
            let card = ChapterCard {
                chapter,
                number,
                copy: 0,
            };
            let definition = card_definition(card);
            assert_eq!(definition.id, format!("chapter{chapter}_card_{number:02}"));
            definition.validate().unwrap();
        }
    }

    // Rulebook p. 5 fixes the card families and their mechanical vocabulary.
    assert_eq!(
        cards
            .iter()
            .map(|card| format!("{}:{:?}", card.chapter, card.color))
            .collect::<Vec<_>>(),
        [
            "1:Grey", "1:Grey", "1:Grey", "1:Grey", "1:Grey", "1:Grey", "1:Grey", "1:Grey",
            "1:Yellow", "1:Yellow", "1:Yellow", "1:Yellow", "1:Blue", "1:Blue", "1:Blue", "1:Blue",
            "1:Green", "1:Green", "1:Green", "1:Green", "1:Red", "1:Red", "1:Red", "2:Grey",
            "2:Grey", "2:Grey", "2:Grey", "2:Grey", "2:Grey", "2:Grey", "2:Yellow", "2:Yellow",
            "2:Blue", "2:Blue", "2:Blue", "2:Blue", "2:Blue", "2:Green", "2:Green", "2:Green",
            "2:Green", "2:Red", "2:Red", "2:Red", "2:Red", "2:Red", "3:Yellow", "3:Yellow",
            "3:Blue", "3:Blue", "3:Blue", "3:Blue", "3:Blue", "3:Green", "3:Green", "3:Green",
            "3:Green", "3:Red", "3:Red", "3:Red", "3:Red", "3:Red", "3:Red", "3:Purple",
            "3:Purple", "3:Purple", "3:Purple", "3:Purple", "3:Purple",
        ]
    );
    assert_eq!(
        cards
            .iter()
            .filter(|card| card.cost.chain.is_some())
            .count(),
        17
    );
    assert_eq!(
        cards
            .iter()
            .filter(|card| card.provides_chain.is_some())
            .count(),
        17
    );
    assert_eq!(
        cards
            .iter()
            .filter(|card| !card.provides.skills_any_of.is_empty())
            .count(),
        2
    );
    assert_eq!(
        cards
            .iter()
            .filter(|card| card.provides.race.is_some())
            .count(),
        12
    );
}

#[test]
fn source_backed_board_quest_landmark_and_alliance_catalogues_are_complete() {
    // Rulebook pp. 3, 6, and 7; Player Aid pp. 1-2.
    assert_eq!(
        REGIONS,
        [
            Region::Lindon,
            Region::Arnor,
            Region::Rhovanion,
            Region::Enedwaith,
            Region::Rohan,
            Region::Gondor,
            Region::Mordor
        ]
    );
    assert_eq!(
        Region::Lindon.adjacent_to(),
        &[Region::Arnor, Region::Enedwaith]
    );
    assert_eq!(
        Region::Arnor.adjacent_to(),
        &[Region::Lindon, Region::Enedwaith, Region::Rhovanion]
    );
    assert_eq!(
        Region::Rhovanion.adjacent_to(),
        &[
            Region::Arnor,
            Region::Enedwaith,
            Region::Rohan,
            Region::Mordor
        ]
    );
    assert_eq!(
        Region::Enedwaith.adjacent_to(),
        &[
            Region::Lindon,
            Region::Arnor,
            Region::Rhovanion,
            Region::Rohan,
            Region::Gondor
        ]
    );
    assert_eq!(
        Region::Rohan.adjacent_to(),
        &[
            Region::Enedwaith,
            Region::Rhovanion,
            Region::Gondor,
            Region::Mordor
        ]
    );
    assert_eq!(
        Region::Gondor.adjacent_to(),
        &[Region::Enedwaith, Region::Rohan, Region::Mordor]
    );
    assert_eq!(
        Region::Mordor.adjacent_to(),
        &[Region::Rhovanion, Region::Rohan, Region::Gondor]
    );
    assert_eq!(
        crate::quest_bonuses()
            .into_iter()
            .map(|bonus| (bonus.position, bonus.effect))
            .collect::<Vec<_>>(),
        vec![
            (4, QuestBonusEffect::GainCoin),
            (7, QuestBonusEffect::PlaceUnit),
            (10, QuestBonusEffect::TakeExtraTurn),
            (13, QuestBonusEffect::RemoveEnemyFortress)
        ]
    );

    let landmarks: Vec<landmark_schema::LandmarkDefinition> =
        serde_json::from_str(include_str!("../landmark_tiles.json")).unwrap();
    assert_eq!(
        landmarks
            .iter()
            .map(|landmark| landmark.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "Helm's Deep",
            "Erebor",
            "Bree",
            "Grey Havens",
            "Minas Tirith",
            "Isengard",
            "Barad-Dûr"
        ]
    );
    assert!(landmarks.iter().all(|landmark| matches!(landmark.effects.first(), Some(landmark_schema::LandmarkEffect::PlaceFortress { location }) if *location == landmark.region)));

    let tokens: Vec<alliance_schema::AllianceTokenDefinition> =
        serde_json::from_str(include_str!("../alliance_tokens.json")).unwrap();
    assert_eq!(tokens.len(), 18);
    assert_eq!(
        tokens
            .iter()
            .filter(|token| !token.immediate.is_empty())
            .count(),
        6
    );
    assert_eq!(
        tokens
            .iter()
            .filter(|token| !token.persistent.is_empty())
            .count(),
        12
    );
}
