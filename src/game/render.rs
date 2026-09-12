use crate::card_schema::Effect;
use crate::catalog::{alliance_token_definition, card_definition, landmark_definition};
use crate::{
    Action, CardVisibility, Faction, PendingDecision, PlayerState, QUEST_VICTORY_POSITION,
    alliance_token_description,
};

use super::Game;

impl Game {
    /// Explains a legal action using the current viewer-safe game state.
    pub fn describe_action(&self, action: &Action) -> String {
        let summary = match action {
            Action::TakeChapterCard { slot_id } => {
                self.chapter_card_selection_description(*slot_id)
            }
            Action::PlaySelectedCard
            | Action::PlaySelectedCardNormally
            | Action::PlaySelectedCardUsingAnySkill => self.selected_card_play_description(action),
            Action::DiscardSelectedCard => self.selected_card_discard_description(),
            Action::TakeLandmark { id } | Action::TakeLandmarkUsingAnySkill { id } => {
                let landmark = landmark_definition(id);
                let payment = if matches!(action, Action::TakeLandmarkUsingAnySkill { .. }) {
                    self.landmark_payment_using_any_skill(id)
                } else {
                    self.landmark_payment(id)
                };
                format!(
                    "Take Landmark {} ({id}) for {} coins. Printed cost: {:?} Skills plus 1 coin per existing fortress. Effects: {:?}.",
                    landmark.name,
                    payment.map_or_else(
                        || "an unavailable payment".into(),
                        |coins| coins.to_string()
                    ),
                    landmark.cost.skills,
                    landmark.effects,
                )
            }
            Action::PlaceCardUnits { region } => {
                format!("Place the selected card's required Units in {region}.")
            }
            Action::ResolveEmptyCardUnitPlacement => {
                "Resolve the required Unit placement with no Units left in supply.".into()
            }
            Action::MoveCardUnit { from, to } => format!(
                "Move one {} Unit from {from} to adjacent {to}; conflicts resolve immediately.",
                self.decision_player()
                    .expect("legal move has a decision player")
            ),
            Action::RemoveCardEnemyUnit { region } => {
                format!("Remove one enemy Unit from {region}; it returns to the opponent's supply.")
            }
            Action::ResolveUnavailableManeuver => {
                "Resolve this maneuver without effect because it has no legal target.".into()
            }
            Action::PlaceQuestBonusUnit { region } => {
                format!("Use the quest bonus to place one Unit in {region}.")
            }
            Action::RemoveQuestBonusFortress { region } => {
                format!("Use the quest bonus to remove the enemy fortress in {region}.")
            }
            Action::DiscardOpponentGreyCard { card } => {
                format!(
                    "Discard the opponent's Grey card {}; its provided Skills are lost.",
                    self.card_name(*card)
                )
            }
            Action::PlayDiscardedCardForFree { card } => format!(
                "Play discarded card {} for free. Effects: {}.",
                self.card_name(*card),
                self.card_effects_description(*card)
            ),
            Action::ResolveAllianceTrigger { trigger } => {
                format!("Resolve the Alliance reward for {trigger}.")
            }
            Action::ChooseAllianceRace { race } => {
                format!("Reveal up to two Alliance tokens from the {race} stack, then choose one.")
            }
            Action::TakeAllianceToken { token } => format!(
                "Take revealed {} token {} ({}). Effects: {}.",
                token.race,
                token.id,
                alliance_token_description(*token),
                self.alliance_token_effects_description(*token)
            ),
            Action::ChooseEntManeuver { maneuver } => {
                format!("Choose the Ent maneuver to {maneuver}.")
            }
            Action::PlaceAllianceUnits { region } => {
                format!("Place one Wizard Alliance-token Unit in {region}.")
            }
            Action::AcceptQuestBonus => {
                "Accept the optional quest bonus described in the pending decision.".into()
            }
            Action::SkipQuestBonus => {
                "Skip the optional quest bonus; no bonus effect is applied.".into()
            }
        };
        let mut preview = self.clone();
        preview
            .apply(action.clone())
            .expect("descriptions are generated only for legal actions");
        format!("{summary} {}", self.preview_consequences(&preview))
    }

    fn chapter_card_selection_description(&self, slot_id: usize) -> String {
        let slot = self
            .current_chapter_state()
            .layout
            .iter()
            .find(|slot| slot.id == slot_id)
            .expect("legal chapter-card action has a matching slot");
        let definition = card_definition(slot.card);
        let payment = self.payment_for(slot.card).map_or_else(
            || "currently unavailable".into(),
            |coins| format!("{coins} coins"),
        );
        let unlocks = self
            .current_chapter_state()
            .layout
            .iter()
            .filter(|candidate| candidate.covering_slots.contains(&slot_id))
            .map(|covered| format!("row {}, column {}", covered.row, covered.column))
            .collect::<Vec<_>>();
        format!(
            "Take chapter card {} at row {}, column {}. Cost: {}, {} coins; current payment: {payment}. Effects: {}. Provides: {}. This card is available now and unlocks {}.",
            self.card_name(slot.card),
            slot.row,
            slot.column,
            self.skills_description(&definition.cost.skills),
            definition.cost.coins,
            self.card_effects_description(slot.card),
            self.card_provides_description(slot.card),
            if unlocks.is_empty() {
                "no cards".into()
            } else {
                unlocks.join(" and ")
            },
        )
    }

    fn selected_card_play_description(&self, action: &Action) -> String {
        let Some(PendingDecision::CardDisposition { card }) = self.pending_decision else {
            unreachable!("legal card-play action has a selected card");
        };
        let payment = match action {
            Action::PlaySelectedCard => self.payment_for(card),
            Action::PlaySelectedCardNormally => self.payment_for_with_any_skill(card, false),
            Action::PlaySelectedCardUsingAnySkill => self.payment_for_using_any_skill(card),
            _ => unreachable!(),
        };
        format!(
            "Play selected card {} for {}. Effects: {}. Provides: {}.",
            self.card_name(card),
            payment.map_or_else(
                || "an unavailable payment".into(),
                |coins| format!("{coins} coins")
            ),
            self.card_effects_description(card),
            self.card_provides_description(card),
        )
    }

    fn selected_card_discard_description(&self) -> String {
        let Some(PendingDecision::CardDisposition { card }) = self.pending_decision else {
            unreachable!("legal discard action has a selected card");
        };
        let income = self.discard_income_for(self.active_player);
        format!(
            "Discard selected card {}; gain {income} coin(s) and do not apply its effects.",
            self.card_name(card)
        )
    }

    fn card_name(&self, card: crate::ChapterCard) -> String {
        format!("{} ({card})", card_definition(card).id)
    }

    fn card_effects_description(&self, card: crate::ChapterCard) -> String {
        let definition = card_definition(card);
        let effects = definition
            .effects
            .iter()
            .map(|effect| match effect {
                Effect::GainCoins { amount } => format!("gain {amount} coins"),
                Effect::AdvanceQuest { spaces } => format!("advance your quest {spaces} spaces"),
                Effect::PlaceUnits {
                    amount, regions, ..
                } => format!("place {amount} Units in one of {regions:?}"),
                Effect::MoveUnits { amount } => format!("move {amount} Unit(s)"),
                Effect::RemoveEnemyUnits { amount } => format!("remove {amount} enemy Unit(s)"),
                Effect::OpponentLosesCoins { amount } => {
                    format!("make the opponent lose {amount} coins")
                }
            })
            .collect::<Vec<_>>();
        if effects.is_empty() {
            "no immediate effect".into()
        } else {
            effects.join("; ")
        }
    }

    fn card_provides_description(&self, card: crate::ChapterCard) -> String {
        let provides = &card_definition(card).provides;
        let mut values = Vec::new();
        if !provides.skills.is_empty() {
            values.push(self.skills_description(&provides.skills));
        }
        if !provides.skills_any_of.is_empty() {
            values.push(format!("one Skill set from {:?}", provides.skills_any_of));
        }
        if let Some(race) = provides.race {
            values.push(format!("a {race:?} Race symbol"));
        }
        if values.is_empty() {
            "nothing persistent".into()
        } else {
            values.join(" and ")
        }
    }

    fn skills_description(
        &self,
        skills: &std::collections::BTreeMap<crate::card_schema::Skill, u8>,
    ) -> String {
        if skills.is_empty() {
            "no Skills".into()
        } else {
            skills
                .iter()
                .map(|(skill, amount)| format!("{amount} {skill:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        }
    }

    fn alliance_token_effects_description(&self, token: crate::AllianceToken) -> String {
        use crate::alliance_schema::{AllianceEffect, PersistentEffect};

        let definition = alliance_token_definition(token);
        let immediate = definition
            .immediate
            .iter()
            .map(|effect| match effect {
                AllianceEffect::GainCoins { amount } => format!("gain {amount} coins"),
                AllianceEffect::AdvanceQuest { spaces } => {
                    format!("advance your quest {spaces} spaces")
                }
                AllianceEffect::PlaceUnitsAnywhere { amount } => {
                    format!("place {amount} Units in any region")
                }
                AllianceEffect::MoveUnits { amount } => format!("move {amount} Unit(s)"),
                AllianceEffect::RemoveEnemyUnits { amount } => {
                    format!("remove {amount} enemy Unit(s)")
                }
                AllianceEffect::OpponentLosesCoins { amount } => {
                    format!("make the opponent lose {amount} coins")
                }
                AllianceEffect::TakeExtraTurn => "take an extra turn".into(),
                AllianceEffect::RemoveEnemyFortress => "remove one enemy fortress".into(),
                AllianceEffect::PlayDiscardedCardForFree => {
                    "play one discarded card for free".into()
                }
                AllianceEffect::ChooseManeuvers { times } => {
                    format!("choose {times} Ent maneuvers")
                }
            })
            .collect::<Vec<_>>();
        let persistent = definition
            .persistent
            .iter()
            .map(|effect| match effect {
                PersistentEffect::OnCardPlayed { color, effect } => {
                    format!("when playing a {color:?} card: {effect:?}")
                }
                PersistentEffect::OnChainedCardPlayed { effect } => {
                    format!("when playing a chained card: {effect:?}")
                }
                PersistentEffect::OnLandmarkTaken { effect } => {
                    format!("when taking a Landmark: {effect:?}")
                }
                PersistentEffect::DiscardIncomeMultiplier { multiplier } => {
                    format!("multiply discard income by {multiplier}")
                }
                PersistentEffect::WaiveLandmarkFortressCost => {
                    "waive Landmark fortress cost".into()
                }
                PersistentEffect::AnySkillOncePerTurn => "use any one Skill once per turn".into(),
                PersistentEffect::RedDeploymentAnywhere => {
                    "deploy Red-card Units in any region".into()
                }
                PersistentEffect::AdditionalRedUnits { amount } => {
                    format!("deploy {amount} additional Red-card Unit(s)")
                }
                PersistentEffect::EagleRaceSymbol => "gain an Eagle Race symbol".into(),
            })
            .collect::<Vec<_>>();
        [immediate, persistent]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("; ")
    }

    fn preview_consequences(&self, preview: &Self) -> String {
        let mut consequences = Vec::new();
        for faction in [Faction::Fellowship, Faction::Sauron] {
            let before = self.player(faction);
            let after = preview.player(faction);
            if before.coins != after.coins {
                consequences.push(format!(
                    "{faction} coins: {} to {}",
                    before.coins, after.coins
                ));
            }
            let before_quest = match faction {
                Faction::Fellowship => self.quest.fellowship_position,
                Faction::Sauron => self.quest.sauron_position,
            };
            let after_quest = match faction {
                Faction::Fellowship => preview.quest.fellowship_position,
                Faction::Sauron => preview.quest.sauron_position,
            };
            if before_quest != after_quest {
                consequences.push(format!("{faction} quest: {before_quest} to {after_quest}"));
            }
        }
        for (before, after) in self.map.iter().zip(&preview.map) {
            if before != after {
                consequences.push(format!(
                    "{} now has Fellowship {} Units, Sauron {} Units, fortress {}",
                    after.region,
                    after.fellowship_units,
                    after.sauron_units,
                    after
                        .fortress
                        .map_or_else(|| "none".into(), |faction| faction.to_string())
                ));
            }
        }
        if let Some(winner) = preview.winner {
            consequences.push(format!(
                "immediate victory: {winner} wins by {}",
                preview.victory_type.expect("winner has victory type")
            ));
        }
        if preview.shared_territory_victory {
            consequences.push("immediate shared territorial victory".into());
        }
        if preview.pending_decision.is_some() {
            consequences
                .push("a further legal-action response supplies the required next choice".into());
        }
        if consequences.is_empty() {
            "Immediate result: no public state change until a subsequent effect or choice.".into()
        } else {
            format!("Immediate result: {}.", consequences.join("; "))
        }
    }

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
                            "{card}({:?}; skills={:?}; any_of={:?}; race={:?}; effects={:?}; chain={:?})",
                            definition.color,
                            definition.provides.skills,
                            definition.provides.skills_any_of,
                            definition.provides.race,
                            definition.effects,
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
            "game\nturn: {}\nactive_player: {}\ncurrent_chapter: {}\noutcome: {}\nplayers: Fellowship={}, Sauron={}\ncoins: Fellowship={}, Sauron={}, reserve={}\ntableaus: Fellowship=[{}], Sauron=[{}]\nalliance_tokens: Fellowship=[{}], Sauron=[{}]\nunits_in_supply: Fellowship={}, Sauron={}\nquest (first to {}): Nazgul={}, Frodo_and_Sam={}\nquest_bonuses: {}\nlandmarks: available={:?}, facedown_remaining={}\nalliance_stacks:\n{}\nmap:\n{}\nchapter_layout:\n{}{}",
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
            QUEST_VICTORY_POSITION,
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
