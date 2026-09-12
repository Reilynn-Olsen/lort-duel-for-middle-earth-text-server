use crate::catalog::{card_definition, setup_chapter};
use crate::{
    Action, AllianceTrigger, CardVisibility, Faction, RACES, REGIONS, Race, Region, VictoryType,
    alliance_schema,
};

use super::{Effect, Game};

impl Game {
    pub(super) fn gain_coins(&mut self, player: Faction, amount: u8) {
        let taken = amount.min(self.coin_reserve);
        self.coin_reserve -= taken;
        self.player_mut(player).coins += taken;
    }
    pub(super) fn lose_coins(&mut self, player: Faction, amount: u8) {
        let lost = amount.min(self.player(player).coins);
        self.player_mut(player).coins -= lost;
        self.coin_reserve += lost;
    }
    /// Places Units, then resolves a normal conflict immediately. Fortresses
    /// neither prevent placement nor participate in a conflict.
    pub(super) fn place_units_in_region(&mut self, player: Faction, region: Region, amount: u8) {
        assert!(self.player(player).units_in_supply >= amount);
        let map_region = self
            .map
            .iter_mut()
            .find(|state| state.region == region)
            .expect("all regions are on the map");
        match player {
            Faction::Fellowship => map_region.fellowship_units += amount,
            Faction::Sauron => map_region.sauron_units += amount,
        }
        self.player_mut(player).units_in_supply -= amount;
        self.resolve_conflict(region);
        self.check_map_domination();
    }
    /// Performs the rulebook's pairwise conflict removal until at least one
    /// faction has no Units in the region. Removed Units return to supply.
    pub(super) fn resolve_conflict(&mut self, region: Region) {
        let (removed_fellowship, removed_sauron) = {
            let map_region = self
                .map
                .iter_mut()
                .find(|state| state.region == region)
                .expect("all regions are on the map");
            let conflicts = map_region.fellowship_units.min(map_region.sauron_units);
            map_region.fellowship_units -= conflicts;
            map_region.sauron_units -= conflicts;
            (conflicts, conflicts)
        };
        self.fellowship.units_in_supply += removed_fellowship;
        self.sauron.units_in_supply += removed_sauron;
    }
    /// Shared Landmark/bonus primitive. A Fortress is a permanent regional
    /// presence; it has no move operation and never enters Unit conflicts.
    #[allow(dead_code)] // Landmark selection is the next board caller.
    pub(super) fn place_fortress(&mut self, player: Faction, region: Region) {
        assert!(self.player(player).fortresses_in_supply > 0);
        let state = self
            .map
            .iter_mut()
            .find(|state| state.region == region)
            .expect("all regions are on the map");
        assert!(state.fortress.is_none());
        state.fortress = Some(player);
        self.player_mut(player).fortresses_in_supply -= 1;
        self.check_map_domination();
    }
    /// Moves one Unit over a printed map connection and resolves a conflict in
    /// the destination. Fortresses have no corresponding move operation.
    #[allow(dead_code)] // Used by the forthcoming Purple maneuver decisions.
    pub(super) fn move_unit(&mut self, player: Faction, from: Region, to: Region) {
        assert!(from.adjacent_to().contains(&to));
        let source = self
            .map
            .iter_mut()
            .find(|state| state.region == from)
            .expect("all regions are on the map");
        match player {
            Faction::Fellowship => {
                assert!(source.fellowship_units > 0);
                source.fellowship_units -= 1;
            }
            Faction::Sauron => {
                assert!(source.sauron_units > 0);
                source.sauron_units -= 1;
            }
        }
        let destination = self
            .map
            .iter_mut()
            .find(|state| state.region == to)
            .expect("all regions are on the map");
        match player {
            Faction::Fellowship => destination.fellowship_units += 1,
            Faction::Sauron => destination.sauron_units += 1,
        }
        self.resolve_conflict(to);
        self.check_map_domination();
    }
    pub(super) fn region_unit_count(&self, faction: Faction, region: Region) -> u8 {
        let state = self
            .map
            .iter()
            .find(|state| state.region == region)
            .expect("all regions are on the map");
        match faction {
            Faction::Fellowship => state.fellowship_units,
            Faction::Sauron => state.sauron_units,
        }
    }
    pub(super) fn legal_unit_moves(&self, player: Faction) -> Vec<Action> {
        self.map
            .iter()
            .filter(|state| self.region_unit_count(player, state.region) > 0)
            .flat_map(|state| {
                self.permitted_movement_destinations(player, state.region)
                    .into_iter()
                    .map(move |to| Action::MoveCardUnit {
                        from: state.region,
                        to,
                    })
            })
            .collect()
    }
    pub(super) fn permitted_movement_destinations(
        &self,
        _player: Faction,
        from: Region,
    ) -> Vec<Region> {
        from.adjacent_to().to_vec()
    }
    /// The current rules have no active modifier at setup, but deployment
    /// effects (for example the Dwarf alliance) alter this single policy hook
    /// rather than rewriting Red-card resolution.
    pub(super) fn permitted_red_destinations(
        &self,
        _player: Faction,
        printed: &[Region],
    ) -> Vec<Region> {
        if self.has_persistent(_player, |effect| {
            matches!(
                effect,
                alliance_schema::PersistentEffect::RedDeploymentAnywhere
            )
        }) {
            REGIONS.to_vec()
        } else {
            printed.to_vec()
        }
    }
    pub(super) fn player_races(&self, player: Faction) -> Vec<Race> {
        RACES
            .into_iter()
            .filter(|race| {
                self.player(player)
                    .tableau
                    .iter()
                    .any(|card| card_definition(*card).provides.race.map(Race::from) == Some(*race))
            })
            .collect()
    }
    pub(super) fn pending_alliance_triggers(&self, player: Faction) -> Vec<AllianceTrigger> {
        let matching = RACES.into_iter().filter_map(|race| {
            let count = self
                .player(player)
                .tableau
                .iter()
                .filter(|card| card_definition(**card).provides.race.map(Race::from) == Some(race))
                .count();
            (count >= 2
                && !self.player(player).matched_races.contains(&race)
                && self.alliance_stack_has_tokens(race))
            .then_some(AllianceTrigger::MatchingRace(race))
        });
        let three_races = self.player_races(player);
        let three_different = (three_races.len() >= 3
            && !self.player(player).claimed_three_race_alliance
            && three_races
                .iter()
                .take(3)
                .any(|race| self.alliance_stack_has_tokens(*race)))
        .then_some(AllianceTrigger::ThreeDifferentRaces);
        matching.chain(three_different).collect()
    }
    pub(super) fn alliance_stack_has_tokens(&self, race: Race) -> bool {
        self.alliance_stacks
            .iter()
            .find(|stack| stack.race == race)
            .is_some_and(|stack| !stack.token_order.is_empty())
    }
    pub(super) fn check_race_victory(&mut self, player: Faction) {
        let mut races = self.player_races(player);
        if self.has_persistent(player, |effect| {
            matches!(effect, alliance_schema::PersistentEffect::EagleRaceSymbol)
        }) {
            // The Hobbit Eagle token is the one Alliance token that adds a Race symbol.
            races.push(Race::Hobbits);
        }
        if races.len() >= 6 {
            self.record_victory(player, VictoryType::RaceSupport);
        }
    }
    pub(super) fn record_victory(&mut self, winner: Faction, victory_type: VictoryType) {
        self.winner = Some(winner);
        self.victory_type = Some(victory_type);
    }
    pub(super) fn check_map_domination(&mut self) {
        for faction in [Faction::Fellowship, Faction::Sauron] {
            if REGIONS
                .into_iter()
                .all(|region| self.has_presence(faction, region))
            {
                self.record_victory(faction, VictoryType::Conquest);
                return;
            }
        }
    }
    pub fn has_presence(&self, faction: Faction, region: Region) -> bool {
        let state = self
            .map
            .iter()
            .find(|state| state.region == region)
            .expect("all regions are on the map");
        state.fortress == Some(faction)
            || match faction {
                Faction::Fellowship => state.fellowship_units > 0,
                Faction::Sauron => state.sauron_units > 0,
            }
    }
    pub(super) fn advance_quest(&mut self, player: Faction, spaces: u8) {
        let old_fellowship = self.quest.fellowship_position;
        let old_sauron = self.quest.sauron_position;
        match player {
            Faction::Fellowship => {
                self.quest.fellowship_position = (old_fellowship + spaces).min(30);
                // The Nazgûl follows the Fellowship marker, preserving their
                // current separation instead of allowing it to grow.
                self.quest.sauron_position = (old_sauron + spaces).min(30);
            }
            Faction::Sauron => {
                self.quest.sauron_position = (old_sauron + spaces).min(old_fellowship);
            }
        }
        if self.quest.fellowship_position == 30 {
            self.record_victory(Faction::Fellowship, VictoryType::Quest);
            return;
        }
        if self.quest.sauron_position >= self.quest.fellowship_position {
            self.record_victory(Faction::Sauron, VictoryType::Quest);
            return;
        }
        let mut crossed = self
            .quest
            .bonuses
            .iter_mut()
            .filter(|bonus| {
                !bonus.claimed
                    && ((bonus.position > old_fellowship
                        && bonus.position <= self.quest.fellowship_position)
                        || (bonus.position > old_sauron
                            && bonus.position <= self.quest.sauron_position))
            })
            .map(|bonus| {
                bonus.claimed = true;
                (bonus.position, bonus.effect)
            })
            .collect::<Vec<_>>();
        crossed.sort_by_key(|(position, _)| *position);
        for (_, effect) in crossed.into_iter().rev() {
            self.effect_queue
                .push_front(Effect::ResolveQuestBonus { player, effect });
        }
    }
    pub(super) fn reveal_newly_available_cards(&mut self) {
        let present_slots = self
            .current_chapter_state()
            .layout
            .iter()
            .map(|slot| slot.id)
            .collect::<Vec<_>>();
        for slot in &mut self.chapters[(self.current_chapter - 1) as usize].layout {
            if slot.visibility == CardVisibility::FaceDown
                && slot
                    .covering_slots
                    .iter()
                    .all(|id| !present_slots.contains(id))
            {
                slot.visibility = CardVisibility::FaceUp;
            }
        }
    }
    /// Chapter setup is already deterministic at game creation; transition only
    /// makes the next stored layout current after the final card is removed.
    pub(super) fn transition_chapter_if_complete(&mut self) {
        if !self.current_chapter_state().layout.is_empty() {
            return;
        }
        if self.current_chapter == 3 {
            self.resolve_final_territorial_comparison();
            return;
        }
        self.current_chapter += 1;
        self.chapters
            .push(setup_chapter(self.current_chapter, &mut self.rng));
        while self.landmarks.available.len() < 3 {
            let Some(landmark) = self.landmarks.facedown_deck.pop() else {
                break;
            };
            self.landmarks.available.push(landmark);
        }
    }
    pub(super) fn resolve_final_territorial_comparison(&mut self) {
        let fellowship = REGIONS
            .into_iter()
            .filter(|region| self.has_presence(Faction::Fellowship, *region))
            .count();
        let sauron = REGIONS
            .into_iter()
            .filter(|region| self.has_presence(Faction::Sauron, *region))
            .count();
        match fellowship.cmp(&sauron) {
            std::cmp::Ordering::Greater => {
                self.record_victory(Faction::Fellowship, VictoryType::TerritorialComparison)
            }
            std::cmp::Ordering::Less => {
                self.record_victory(Faction::Sauron, VictoryType::TerritorialComparison)
            }
            std::cmp::Ordering::Equal => {
                self.shared_territory_victory = true;
                self.victory_type = Some(VictoryType::SharedTerritorialComparison);
            }
        }
    }
}
