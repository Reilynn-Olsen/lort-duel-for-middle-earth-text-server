use crate::catalog::{card_definition, landmark_definition};
use crate::{CardVisibility, Faction, PendingDecision, PlayerState, alliance_token_description};

use super::Game;

impl Game {
    /// Public state only. Face-down card identities and token order are withheld.
    pub(super) fn render_state(&self) -> String {
        let regions = self
            .map
            .iter()
            .map(|region| {
                format!(
                    "  {region}: Fellowship units={}, Sauron units={}, fortress={}",
                    region.fellowship_units,
                    region.sauron_units,
                    region
                        .fortress
                        .map(|f| f.to_string())
                        .unwrap_or_else(|| "none".into()),
                    region = region.region
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let stacks = self
            .alliance_stacks
            .iter()
            .map(|stack| format!("  {}: {} facedown", stack.race, stack.token_order.len()))
            .collect::<Vec<_>>()
            .join("\n");
        let available_landmarks = self
            .landmarks
            .available
            .iter()
            .map(|id| {
                let definition = landmark_definition(id);
                format!(
                    "{}: region={:?}; skills={:?}; effects={:?}",
                    definition.name, definition.region, definition.cost.skills, definition.effects
                )
            })
            .collect::<Vec<_>>()
            .join(" | ");
        let chapter = &self.chapters[(self.current_chapter - 1) as usize];
        let cards = chapter
            .layout
            .iter()
            .map(|slot| match slot.visibility {
                CardVisibility::FaceUp => {
                    let definition = card_definition(slot.card);
                    format!("  row {}, column {}: {} — {:?}/{:?}; cost: {:?} skills + {} coins; chain={:?}; provides_skills={:?}; provides_any_of={:?}; provides_race={:?}; provides_chain={:?}; effects: {:?} ({})", slot.row, slot.column, slot.card, definition.color, definition.color.family(), definition.cost.skills, definition.cost.coins, definition.cost.chain, definition.provides.skills, definition.provides.skills_any_of, definition.provides.race, definition.provides_chain, definition.effects, slot.visibility)
                }
                CardVisibility::FaceDown => format!(
                    "  row {}, column {}: unknown ({})",
                    slot.row, slot.column, slot.visibility
                ),
            })
            .collect::<Vec<_>>()
            .join("\n");
        let tableau = |player: &PlayerState| {
            if player.tableau.is_empty() {
                "(none)".to_string()
            } else {
                player
                    .tableau
                    .iter()
                    .map(|card| {
                        let definition = card_definition(*card);
                        format!(
                            "{card}({:?}; skills={:?}; any_of={:?}; race={:?}; chain={:?})",
                            definition.color,
                            definition.provides.skills,
                            definition.provides.skills_any_of,
                            definition.provides.race,
                            definition.provides_chain
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        };
        let alliance_tokens = |player: &PlayerState| {
            if player.alliance_tokens.is_empty() {
                "(none)".to_string()
            } else {
                player
                    .alliance_tokens
                    .iter()
                    .map(|token| alliance_token_description(*token).to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        };
        let pending_card = match self.pending_decision {
            Some(PendingDecision::CardDisposition { card }) => {
                let definition = card_definition(card);
                let play_cost = self
                    .payment_for(card)
                    .map(|coins| coins.to_string())
                    .unwrap_or_else(|| "unaffordable".into());
                format!(
                    "\npending_card: {card}; printed_cost={:?} skills + {} coins; chain_prerequisite={:?}; play_payment={play_cost} coins",
                    definition.cost.skills, definition.cost.coins, definition.cost.chain
                )
            }
            Some(PendingDecision::QuestBonusUnitPlacement { .. }) => {
                "\npending_quest_bonus: place 1 unit in any region".into()
            }
            Some(PendingDecision::CardUnitPlacement {
                player,
                amount,
                ref regions,
            }) => {
                let available = self.player(player).units_in_supply;
                format!(
                    "\npending_red_deployment: place {} unit(s) in one of {:?}; units_available={}",
                    amount, regions, available
                )
            }
            Some(PendingDecision::CardUnitMovement { remaining, .. }) => {
                format!("\npending_maneuver: complete {remaining} movement(s)")
            }
            Some(PendingDecision::CardEnemyUnitRemoval { remaining, .. }) => {
                format!("\npending_maneuver: remove {remaining} enemy Unit(s)")
            }
            Some(PendingDecision::QuestBonusFortressRemoval { .. }) => {
                "\npending_quest_bonus: remove 1 enemy fortress".into()
            }
            Some(PendingDecision::QuestBonusChoice { effect, .. }) => {
                format!("\npending_quest_bonus: optionally {effect}")
            }
            Some(PendingDecision::OpponentGreyCardDiscard { .. }) => {
                "\npending_landmark: discard 1 opponent Grey card".into()
            }
            Some(PendingDecision::DiscardedCardFreePlay { .. }) => {
                "\npending_landmark: play 1 discarded card for free".into()
            }
            Some(PendingDecision::AllianceTriggerChoice { .. }) => {
                "\npending_alliance: choose which Race-alliance reward to resolve".into()
            }
            Some(PendingDecision::AllianceRaceChoice { .. }) => {
                "\npending_alliance: choose a Race stack".into()
            }
            Some(PendingDecision::AllianceTokenChoice { ref revealed, .. }) => {
                format!("\npending_alliance: choose 1 revealed token from {revealed:?}")
            }
            Some(PendingDecision::EntManeuverChoice { remaining, .. }) => {
                format!("\npending_alliance: choose {remaining} Ent maneuver(s)")
            }
            Some(PendingDecision::AllianceUnitPlacement { remaining, .. }) => {
                format!("\npending_alliance: place {remaining} Wizard Unit(s)")
            }
            None => String::new(),
        };
        let quest_bonuses = self
            .quest
            .bonuses
            .iter()
            .map(|bonus| {
                format!(
                    "{}: {} ({})",
                    bonus.position,
                    bonus.effect,
                    if bonus.claimed {
                        "claimed"
                    } else {
                        "available"
                    }
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let outcome = if self.shared_territory_victory {
            format!(
                "shared victory ({})",
                self.victory_type.expect("shared victory has a type")
            )
        } else if let Some(winner) = self.winner {
            format!(
                "{} ({})",
                winner,
                self.victory_type.expect("winner has a victory type")
            )
        } else {
            "in progress".into()
        };
        format!(
            "game\nturn: {}\nactive_player: {}\ncurrent_chapter: {}\noutcome: {}\nplayers: Fellowship={}, Sauron={}\ncoins: Fellowship={}, Sauron={}, reserve={}\ntableaus: Fellowship=[{}], Sauron=[{}]\nalliance_tokens: Fellowship=[{}], Sauron=[{}]\nunits_in_supply: Fellowship={}, Sauron={}\nquest: Nazgul={}, Frodo_and_Sam={}\nquest_bonuses: {}\nlandmarks: available={:?}, facedown_remaining={}\nalliance_stacks:\n{}\nmap:\n{}\nchapter_layout:\n{}{}",
            self.turn,
            self.active_player,
            self.current_chapter,
            outcome,
            self.fellowship.quest_character,
            self.sauron.quest_character,
            self.fellowship.coins,
            self.sauron.coins,
            self.coin_reserve,
            tableau(&self.fellowship),
            tableau(&self.sauron),
            alliance_tokens(&self.fellowship),
            alliance_tokens(&self.sauron),
            self.fellowship.units_in_supply,
            self.sauron.units_in_supply,
            self.quest.sauron_position,
            self.quest.fellowship_position,
            quest_bonuses,
            available_landmarks,
            self.landmarks.facedown_deck.len(),
            stacks,
            regions,
            cards,
            pending_card,
        )
    }
    #[allow(dead_code)] // Convenience for in-crate tests; server uses viewer-scoped observations.
    pub(super) fn render_observation(&self) -> String {
        self.render_observation_for(self.active_player)
    }
    pub fn render_observation_for(&self, viewer: Faction) -> String {
        let actions = self
            .legal_actions_for(viewer)
            .iter()
            .enumerate()
            .map(|(index, action)| format!("{index}. {action}"))
            .collect::<Vec<_>>();
        format!(
            "state for {viewer}:\n{}\nactions:\n{}",
            self.render_state(),
            if actions.is_empty() {
                "(no legal actions)".into()
            } else {
                actions.join("\n")
            }
        )
    }
}
