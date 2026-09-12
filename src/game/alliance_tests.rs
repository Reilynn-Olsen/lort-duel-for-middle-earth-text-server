use crate::*;

use super::*;

#[test]
fn matching_and_three_different_race_rewards_are_detected_independently() {
    let mut game = Game::new(0);
    let humans = ChapterCard {
        chapter: 1,
        number: 17,
        copy: 0,
    };
    game.sauron.tableau.extend([humans, humans]);
    game.effect_queue
        .push_back(Effect::ResolveRaceAllianceTriggers {
            player: Faction::Sauron,
        });
    game.resolve_effects();
    assert!(matches!(
        game.pending_decision,
        Some(PendingDecision::AllianceTokenChoice { ref revealed, .. })
            if revealed.iter().all(|token| token.race == Race::Humans)
    ));

    let mut game = Game::new(0);
    game.sauron.tableau.extend([
        humans,
        ChapterCard {
            chapter: 1,
            number: 18,
            copy: 0,
        },
        ChapterCard {
            chapter: 1,
            number: 19,
            copy: 0,
        },
    ]);
    game.effect_queue
        .push_back(Effect::ResolveRaceAllianceTriggers {
            player: Faction::Sauron,
        });
    game.resolve_effects();
    assert!(matches!(
        game.pending_decision,
        Some(PendingDecision::AllianceTokenChoice { ref revealed, .. })
            if revealed.len() == 3
    ));
    assert!(game.sauron.claimed_three_race_alliance);
}
#[test]
fn immediate_alliance_tokens_queue_their_required_follow_up_choices() {
    let mut game = Game::new(0);
    game.effect_queue.push_back(Effect::ResolveAllianceToken {
        player: Faction::Sauron,
        token: AllianceToken {
            race: Race::Wizards,
            id: 2,
        },
    });
    game.resolve_effects();
    assert!(matches!(
        game.pending_decision,
        Some(PendingDecision::AllianceUnitPlacement { remaining: 2, .. })
    ));

    let mut game = Game::new(0);
    game.effect_queue.push_back(Effect::ResolveAllianceToken {
        player: Faction::Sauron,
        token: AllianceToken {
            race: Race::Ents,
            id: 3,
        },
    });
    game.resolve_effects();
    assert!(matches!(
        game.pending_decision,
        Some(PendingDecision::EntManeuverChoice { remaining: 3, .. })
    ));
}
