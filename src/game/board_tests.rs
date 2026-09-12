use crate::*;

use super::test_support::{ResolutionEvent, assert_illegal_action_unchanged};
use super::*;

#[test]
fn conflicts_remove_units_pairwise_but_never_remove_a_fortress() {
    let mut game = Game::new(0);
    let arnor = game
        .map
        .iter_mut()
        .find(|state| state.region == Region::Arnor)
        .unwrap();
    arnor.fortress = Some(Faction::Fellowship);

    // Arnor begins with two Fellowship Units. Each Sauron placement causes
    // one pair to return to supply, leaving no mixed-Unit region.
    game.place_units_in_region(Faction::Sauron, Region::Arnor, 1);
    let arnor = game
        .map
        .iter()
        .find(|state| state.region == Region::Arnor)
        .unwrap();
    assert_eq!((arnor.fellowship_units, arnor.sauron_units), (1, 0));
    assert_eq!(arnor.fortress, Some(Faction::Fellowship));

    game.place_units_in_region(Faction::Sauron, Region::Arnor, 1);
    let arnor = game
        .map
        .iter()
        .find(|state| state.region == Region::Arnor)
        .unwrap();
    assert_eq!((arnor.fellowship_units, arnor.sauron_units), (0, 0));
    assert_eq!(arnor.fortress, Some(Faction::Fellowship));
    assert!(game.has_presence(Faction::Fellowship, Region::Arnor));
    assert!(!game.has_presence(Faction::Sauron, Region::Arnor));
}
#[test]
fn unit_movement_requires_adjacency_and_resolves_conflict_at_destination() {
    let mut game = Game::new(0);
    // Move a Fellowship Unit from Arnor to adjacent Rhovanion, then let a
    // Sauron Unit enter from Mordor; the destination conflict removes both.
    game.move_unit(Faction::Fellowship, Region::Arnor, Region::Rhovanion);
    game.move_unit(Faction::Sauron, Region::Mordor, Region::Rhovanion);
    let rhovanion = game
        .map
        .iter()
        .find(|state| state.region == Region::Rhovanion)
        .unwrap();
    assert_eq!((rhovanion.fellowship_units, rhovanion.sauron_units), (0, 0));
}
#[test]
fn red_card_deployment_offers_only_printed_regions_then_resolves_conflict() {
    let mut game = Game::new(0);
    let card = ChapterCard {
        chapter: 1,
        number: 21,
        copy: 0,
    };
    // C1-21 deploys one Unit to Gondor or Rohan. Put an enemy Unit in
    // Rohan first so the test exercises automatic conflict.
    game.map
        .iter_mut()
        .find(|state| state.region == Region::Rohan)
        .unwrap()
        .fellowship_units = 1;
    game.fellowship.units_in_supply -= 1;
    game.chapters[0]
        .discarded
        .retain(|discarded| *discarded != card);
    game.pending_decision = Some(PendingDecision::CardDisposition { card });
    game.apply(Action::PlaySelectedCard).unwrap();
    assert_eq!(
        game.legal_actions(),
        vec![
            Action::PlaceCardUnits {
                region: Region::Gondor
            },
            Action::PlaceCardUnits {
                region: Region::Rohan
            }
        ]
    );
    game.apply(Action::PlaceCardUnits {
        region: Region::Rohan,
    })
    .unwrap();
    let rohan = game
        .map
        .iter()
        .find(|state| state.region == Region::Rohan)
        .unwrap();
    assert_eq!((rohan.fellowship_units, rohan.sauron_units), (0, 0));
    assert_eq!(game.fellowship.units_in_supply, 13);
    assert_eq!(game.sauron.units_in_supply, 13);
}
#[test]
fn red_deployment_respects_the_finite_unit_supply() {
    let mut game = Game::new(0);
    let card = ChapterCard {
        chapter: 2,
        number: 19,
        copy: 0,
    };
    // C2-19 calls for two Units. With one physical Unit remaining, place
    // that one and never synthesize a sixteenth component.
    game.sauron.units_in_supply = 1;
    game.sauron.tableau.push(ChapterCard {
        chapter: 1,
        number: 22,
        copy: 0,
    }); // Helmet chains to C2-19.
    game.pending_decision = Some(PendingDecision::CardDisposition { card });
    game.apply(Action::PlaySelectedCard).unwrap();
    game.apply(Action::PlaceCardUnits {
        region: Region::Lindon,
    })
    .unwrap();
    let lindon = game
        .map
        .iter()
        .find(|state| state.region == Region::Lindon)
        .unwrap();
    assert_eq!(lindon.sauron_units, 1);
    assert_eq!(game.sauron.units_in_supply, 0);
}
#[test]
fn unit_placement_checks_map_domination_immediately() {
    let mut game = Game::new(0);
    for region in REGIONS {
        if region != Region::Lindon {
            game.map
                .iter_mut()
                .find(|state| state.region == region)
                .unwrap()
                .fortress = Some(Faction::Sauron);
        }
    }
    game.place_units_in_region(Faction::Sauron, Region::Lindon, 1);
    assert_eq!(game.winner(), Some(Faction::Sauron));
}
#[test]
fn fortress_placement_is_permanent_presence_and_checks_domination() {
    let mut game = Game::new(0);
    for region in REGIONS {
        if region != Region::Lindon {
            game.map
                .iter_mut()
                .find(|state| state.region == region)
                .unwrap()
                .fellowship_units = 1;
        }
    }
    game.place_fortress(Faction::Fellowship, Region::Lindon);
    assert!(game.has_presence(Faction::Fellowship, Region::Lindon));
    assert_eq!(game.winner(), Some(Faction::Fellowship));
}
#[test]
fn maneuver_movement_is_adjacent_individual_and_resolves_each_destination_conflict() {
    let mut game = Game::new(0);
    // The first move enters an enemy-occupied region and is cancelled;
    // the second movement point may then move a different Unit.
    game.map
        .iter_mut()
        .find(|state| state.region == Region::Rhovanion)
        .unwrap()
        .sauron_units = 1;
    game.sauron.units_in_supply -= 1;
    game.pending_decision = Some(PendingDecision::CardUnitMovement {
        player: Faction::Fellowship,
        remaining: 2,
    });
    assert!(game.legal_actions().contains(&Action::MoveCardUnit {
        from: Region::Arnor,
        to: Region::Rhovanion,
    }));
    assert!(!game.legal_actions().contains(&Action::MoveCardUnit {
        from: Region::Arnor,
        to: Region::Gondor,
    }));
    game.apply(Action::MoveCardUnit {
        from: Region::Arnor,
        to: Region::Rhovanion,
    })
    .unwrap();
    assert!(matches!(
        game.pending_decision,
        Some(PendingDecision::CardUnitMovement { remaining: 1, .. })
    ));
    assert_eq!(
        game.region_unit_count(Faction::Fellowship, Region::Rhovanion),
        0
    );
    assert_eq!(
        game.region_unit_count(Faction::Sauron, Region::Rhovanion),
        0
    );
    game.apply(Action::MoveCardUnit {
        from: Region::Arnor,
        to: Region::Lindon,
    })
    .unwrap();
    assert_eq!(
        game.region_unit_count(Faction::Fellowship, Region::Lindon),
        1
    );
    assert!(game.pending_decision.is_none());
}
#[test]
fn maneuver_removal_and_coin_loss_are_capped_by_available_components() {
    let mut game = Game::new(0);
    game.sauron.coins = 1;
    game.effect_queue.push_back(Effect::ResolveCardEffect {
        player: Faction::Fellowship,
        effect: crate::card_schema::Effect::OpponentLosesCoins { amount: 3 },
    });
    game.resolve_effects();
    assert_eq!(game.sauron.coins, 0);
    assert_eq!(game.coin_reserve, 26);

    game.pending_decision = Some(PendingDecision::CardEnemyUnitRemoval {
        player: Faction::Sauron,
        remaining: 1,
    });
    game.apply(Action::RemoveCardEnemyUnit {
        region: Region::Arnor,
    })
    .unwrap();
    assert_eq!(
        game.region_unit_count(Faction::Fellowship, Region::Arnor),
        1
    );
    assert_eq!(game.fellowship.units_in_supply, 14);
}
#[test]
fn observation_has_state_then_numbered_actions() {
    let observation = Game::new(42).render_observation();
    assert!(observation.starts_with("state for Sauron:\n"));
    assert!(observation.contains("\nactions:\n0. take available chapter card"));
}

#[test]
fn conflict_cancellation_matrix_preserves_fortresses_and_returns_each_side_to_supply() {
    for (fellowship, sauron, fortress) in [
        (1, 0, None),
        (1, 1, None),
        (3, 1, None),
        (1, 3, None),
        (3, 3, None),
        (3, 1, Some(Faction::Fellowship)),
        (1, 3, Some(Faction::Sauron)),
    ] {
        let mut game = Game::new(0);
        for state in &mut game.map {
            state.fellowship_units = 0;
            state.sauron_units = 0;
        }
        let state = game
            .map
            .iter_mut()
            .find(|state| state.region == Region::Rohan)
            .unwrap();
        state.fellowship_units = fellowship;
        state.sauron_units = sauron;
        state.fortress = fortress;
        game.fellowship.units_in_supply = 13 - fellowship;
        game.sauron.units_in_supply = 13 - sauron;

        game.resolve_conflict(Region::Rohan);
        let cancelled = fellowship.min(sauron);
        assert_eq!(
            game.region_unit_count(Faction::Fellowship, Region::Rohan),
            fellowship - cancelled
        );
        assert_eq!(
            game.region_unit_count(Faction::Sauron, Region::Rohan),
            sauron - cancelled
        );
        assert_eq!(game.fellowship.units_in_supply, 13 - fellowship + cancelled);
        assert_eq!(game.sauron.units_in_supply, 13 - sauron + cancelled);
        assert_eq!(
            game.map
                .iter()
                .find(|state| state.region == Region::Rohan)
                .unwrap()
                .fortress,
            fortress
        );
    }
}

#[test]
fn placement_movement_and_removal_recalculate_presence_and_supplies() {
    let mut game = Game::new(0);
    game.map
        .iter_mut()
        .find(|state| state.region == Region::Rohan)
        .unwrap()
        .fellowship_units = 1;
    game.fellowship.units_in_supply -= 1;
    game.place_units_in_region(Faction::Sauron, Region::Rohan, 1);
    assert_eq!(
        (
            game.region_unit_count(Faction::Fellowship, Region::Rohan),
            game.region_unit_count(Faction::Sauron, Region::Rohan)
        ),
        (0, 0)
    );
    assert_eq!(
        (game.fellowship.units_in_supply, game.sauron.units_in_supply),
        (13, 13)
    );
    assert!(!game.has_presence(Faction::Fellowship, Region::Rohan));

    game.place_fortress(Faction::Fellowship, Region::Rohan);
    assert!(game.has_presence(Faction::Fellowship, Region::Rohan));
    game.place_units_in_region(Faction::Sauron, Region::Rohan, 1);
    assert!(game.has_presence(Faction::Fellowship, Region::Rohan));
    assert!(game.has_presence(Faction::Sauron, Region::Rohan));

    game.pending_decision = Some(PendingDecision::CardEnemyUnitRemoval {
        player: Faction::Fellowship,
        remaining: 1,
    });
    game.run_action(Action::RemoveCardEnemyUnit {
        region: Region::Rohan,
    })
    .unwrap();
    assert!(game.has_presence(Faction::Fellowship, Region::Rohan));
    assert!(!game.has_presence(Faction::Sauron, Region::Rohan));
    assert_eq!(game.sauron.units_in_supply, 13);
    assert!(game.resolution_events().is_empty());
}

#[test]
fn unit_destination_validation_modifiers_and_movement_limits_are_enforced() {
    let mut red = Game::new(0);
    red.pending_decision = Some(PendingDecision::CardUnitPlacement {
        player: Faction::Sauron,
        amount: 1,
        regions: vec![Region::Gondor, Region::Rohan],
    });
    assert_illegal_action_unchanged(
        &mut red,
        Action::PlaceCardUnits {
            region: Region::Lindon,
        },
    );
    red.sauron.alliance_tokens.push(AllianceToken {
        race: Race::Elves,
        id: 2,
    });
    assert_eq!(red.legal_actions().len(), REGIONS.len());
    red.run_action(Action::PlaceCardUnits {
        region: Region::Lindon,
    })
    .unwrap();

    let mut movement = Game::new(0);
    for state in &mut movement.map {
        state.fellowship_units = 0;
    }
    movement
        .map
        .iter_mut()
        .find(|state| state.region == Region::Arnor)
        .unwrap()
        .fellowship_units = 1;
    movement.fellowship.units_in_supply = 12;
    movement.pending_decision = Some(PendingDecision::CardUnitMovement {
        player: Faction::Fellowship,
        remaining: 2,
    });
    assert_illegal_action_unchanged(
        &mut movement,
        Action::MoveCardUnit {
            from: Region::Arnor,
            to: Region::Gondor,
        },
    );
    movement
        .run_action(Action::MoveCardUnit {
            from: Region::Arnor,
            to: Region::Lindon,
        })
        .unwrap();
    assert_illegal_action_unchanged(
        &mut movement,
        Action::MoveCardUnit {
            from: Region::Arnor,
            to: Region::Lindon,
        },
    );

    let mut fortress_only = Game::new(0);
    for state in &mut fortress_only.map {
        state.fellowship_units = 0;
    }
    fortress_only
        .map
        .iter_mut()
        .find(|state| state.region == Region::Arnor)
        .unwrap()
        .fortress = Some(Faction::Fellowship);
    fortress_only.pending_decision = Some(PendingDecision::CardUnitMovement {
        player: Faction::Fellowship,
        remaining: 1,
    });
    assert_eq!(
        fortress_only.legal_actions(),
        vec![Action::ResolveUnavailableManeuver]
    );
}

#[test]
fn landmarks_require_a_physical_fortress_before_they_are_selectable() {
    let mut game = Game::new(0);
    game.sauron.coins = 10;
    game.sauron.fortresses_in_supply = 0;
    game.landmarks.available = vec!["helms_deep".into()];
    assert_illegal_action_unchanged(
        &mut game,
        Action::TakeLandmark {
            id: "helms_deep".into(),
        },
    );
}

#[test]
fn seventh_presence_wins_but_six_and_lost_presence_do_not() {
    let mut six = Game::new(0);
    for region in REGIONS
        .into_iter()
        .filter(|region| *region != Region::Lindon)
    {
        six.map
            .iter_mut()
            .find(|state| state.region == region)
            .unwrap()
            .fortress = Some(Faction::Sauron);
    }
    six.check_map_domination();
    assert!(!six.is_over());
    six.place_units_in_region(Faction::Sauron, Region::Lindon, 1);
    assert_eq!(six.winner(), Some(Faction::Sauron));

    let mut lost = Game::new(0);
    for state in &mut lost.map {
        state.fellowship_units = 1;
    }
    lost.map
        .iter_mut()
        .find(|state| state.region == Region::Mordor)
        .unwrap()
        .fellowship_units = 0;
    lost.check_map_domination();
    assert!(!lost.is_over());
}

#[test]
fn purple_removal_and_coin_loss_resolve_in_queue_order() {
    let purple = ChapterCard {
        chapter: 3,
        number: 19,
        copy: 0,
    };
    let mut game = Game::new(0);
    game.sauron.coins = 10;
    game.fellowship.coins = 2;
    game.pending_decision = Some(PendingDecision::CardDisposition { card: purple });
    game.run_action(Action::PlaySelectedCard).unwrap();
    assert_eq!(
        game.resolution_events(),
        [
            ResolutionEvent::PlayedCard {
                player: Faction::Sauron,
                card: purple
            },
            ResolutionEvent::CardEffect {
                player: Faction::Sauron
            }
        ]
    );
    game.run_action(Action::RemoveCardEnemyUnit {
        region: Region::Arnor,
    })
    .unwrap();
    assert_eq!(game.fellowship.coins, 0);
    assert_eq!(
        game.resolution_events()[..2],
        [
            ResolutionEvent::CardEffect {
                player: Faction::Sauron
            },
            ResolutionEvent::CardEffect {
                player: Faction::Sauron
            }
        ]
    );
}
