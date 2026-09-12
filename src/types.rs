use std::fmt;

use crate::card_schema;

pub const TOTAL_COINS: u8 = 30;
pub const QUEST_VICTORY_POSITION: u8 = 15;
pub const CARDS_PER_CHAPTER: usize = 23;
pub const CARDS_IN_LAYOUT: usize = 20;
pub const REGIONS: [Region; 7] = [
    Region::Lindon,
    Region::Arnor,
    Region::Rhovanion,
    Region::Enedwaith,
    Region::Rohan,
    Region::Gondor,
    Region::Mordor,
];
pub const RACES: [Race; 6] = [
    Race::Elves,
    Race::Ents,
    Race::Hobbits,
    Race::Humans,
    Race::Dwarves,
    Race::Wizards,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Faction {
    Fellowship,
    Sauron,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VictoryType {
    Quest,
    RaceSupport,
    Conquest,
    TerritorialComparison,
    SharedTerritorialComparison,
}
impl fmt::Display for VictoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Quest => "Quest of the Ring",
            Self::RaceSupport => "Support of the Races",
            Self::Conquest => "Conquering Middle-earth",
            Self::TerritorialComparison => "final territorial comparison",
            Self::SharedTerritorialComparison => "shared final territorial comparison",
        })
    }
}
impl Faction {
    pub fn other(self) -> Self {
        match self {
            Self::Fellowship => Self::Sauron,
            Self::Sauron => Self::Fellowship,
        }
    }
}
impl fmt::Display for Faction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Fellowship => "Fellowship",
            Self::Sauron => "Sauron",
        })
    }
}
impl std::str::FromStr for Faction {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "fellowship" => Ok(Self::Fellowship),
            "sauron" => Ok(Self::Sauron),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    Lindon,
    Arnor,
    Rhovanion,
    Enedwaith,
    Rohan,
    Gondor,
    Mordor,
}
impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Lindon => "Lindon",
            Self::Arnor => "Arnor",
            Self::Rhovanion => "Rhovanion",
            Self::Enedwaith => "Enedwaith",
            Self::Rohan => "Rohan",
            Self::Gondor => "Gondor",
            Self::Mordor => "Mordor",
        })
    }
}
impl Region {
    /// Connections printed on the central board. The relation is undirected.
    pub fn adjacent_to(self) -> &'static [Region] {
        match self {
            Self::Lindon => &[Self::Arnor, Self::Enedwaith],
            Self::Arnor => &[Self::Lindon, Self::Enedwaith, Self::Rhovanion],
            Self::Rhovanion => &[Self::Arnor, Self::Enedwaith, Self::Rohan, Self::Mordor],
            Self::Enedwaith => &[
                Self::Lindon,
                Self::Arnor,
                Self::Rhovanion,
                Self::Rohan,
                Self::Gondor,
            ],
            Self::Rohan => &[Self::Enedwaith, Self::Rhovanion, Self::Gondor, Self::Mordor],
            Self::Gondor => &[Self::Enedwaith, Self::Rohan, Self::Mordor],
            Self::Mordor => &[Self::Rhovanion, Self::Rohan, Self::Gondor],
        }
    }
}
impl From<card_schema::Region> for Region {
    fn from(region: card_schema::Region) -> Self {
        match region {
            card_schema::Region::Lindon => Self::Lindon,
            card_schema::Region::Arnor => Self::Arnor,
            card_schema::Region::Rhovanion => Self::Rhovanion,
            card_schema::Region::Enedwaith => Self::Enedwaith,
            card_schema::Region::Rohan => Self::Rohan,
            card_schema::Region::Gondor => Self::Gondor,
            card_schema::Region::Mordor => Self::Mordor,
        }
    }
}
impl From<card_schema::Race> for Race {
    fn from(race: card_schema::Race) -> Self {
        match race {
            card_schema::Race::Elves => Self::Elves,
            card_schema::Race::Ents => Self::Ents,
            card_schema::Race::Hobbits => Self::Hobbits,
            card_schema::Race::Humans => Self::Humans,
            card_schema::Race::Dwarves => Self::Dwarves,
            card_schema::Race::Wizards => Self::Wizards,
        }
    }
}
impl From<Race> for card_schema::Race {
    fn from(race: Race) -> Self {
        match race {
            Race::Elves => Self::Elves,
            Race::Ents => Self::Ents,
            Race::Hobbits => Self::Hobbits,
            Race::Humans => Self::Humans,
            Race::Dwarves => Self::Dwarves,
            Race::Wizards => Self::Wizards,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Race {
    Elves,
    Ents,
    Hobbits,
    Humans,
    Dwarves,
    Wizards,
}
impl fmt::Display for Race {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Elves => "Elves",
            Self::Ents => "Ents",
            Self::Hobbits => "Hobbits",
            Self::Humans => "Humans",
            Self::Dwarves => "Dwarves",
            Self::Wizards => "Wizards",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestCharacter {
    FrodoAndSam,
    Nazgul,
}
impl fmt::Display for QuestCharacter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::FrodoAndSam => "Frodo and Sam",
            Self::Nazgul => "Nazgul",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionState {
    pub region: Region,
    pub fellowship_units: u8,
    pub sauron_units: u8,
    pub fortress: Option<Faction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerState {
    pub faction: Faction,
    pub quest_character: QuestCharacter,
    pub coins: u8,
    pub units_in_supply: u8,
    pub fortresses_in_supply: u8,
    pub tableau: Vec<ChapterCard>,
    pub landmarks: Vec<String>,
    pub alliance_tokens: Vec<AllianceToken>,
    pub matched_races: Vec<Race>,
    pub claimed_three_race_alliance: bool,
    /// Persistent hooks for Alliance effects that will be populated when the
    /// Alliance-token rules are enabled.
    pub waives_landmark_fortress_cost: bool,
    pub landmark_grants_extra_turn: bool,
    pub used_any_skill_this_turn: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestTrack {
    /// Each faction advances independently from 0 to the quest victory position.
    pub fellowship_position: u8,
    pub sauron_position: u8,
    pub bonuses: Vec<QuestBonus>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestBonus {
    pub position: u8,
    pub effect: QuestBonusEffect,
    pub claimed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestBonusEffect {
    GainCoin,
    PlaceUnit,
    TakeExtraTurn,
    RemoveEnemyFortress,
}
impl fmt::Display for QuestBonusEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::GainCoin => "gain 1 coin",
            Self::PlaceUnit => "place 1 unit in any region",
            Self::TakeExtraTurn => "take another turn",
            Self::RemoveEnemyFortress => "remove 1 enemy fortress",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllianceStack {
    pub race: Race,
    pub token_order: Vec<u8>,
}

/// A token is public once acquired. Its numeric ID is stable within its Race
/// stack and maps to the Player Aid's listed effect for that Race.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllianceToken {
    pub race: Race,
    pub id: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllianceTrigger {
    MatchingRace(Race),
    ThreeDifferentRaces,
}
impl fmt::Display for AllianceTrigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MatchingRace(race) => write!(f, "the matching {race} Race symbols"),
            Self::ThreeDifferentRaces => f.write_str("the three different Race symbols"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardVisibility {
    FaceUp,
    FaceDown,
}

impl fmt::Display for CardVisibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::FaceUp => "face-up",
            Self::FaceDown => "face-down",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChapterCard {
    pub chapter: u8,
    /// Identifies the distinct face within its chapter.
    pub number: u8,
    /// Identifies a physical copy of that face (`a`, `b`, ...).
    pub copy: u8,
}
impl fmt::Display for ChapterCard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = char::from(b'a' + self.copy);
        write!(f, "C{}-{:02}{suffix}", self.chapter, self.number)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardSlot {
    pub id: usize,
    pub row: u8,
    pub column: u8,
    pub card: ChapterCard,
    pub visibility: CardVisibility,
    /// Slot IDs covering this card. An empty list means the card is available.
    pub covering_slots: Vec<usize>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterState {
    pub chapter: u8,
    pub layout: Vec<CardSlot>,
    pub discarded: Vec<ChapterCard>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LandmarkState {
    pub available: Vec<String>,
    pub facedown_deck: Vec<String>,
}

/// A stable, machine-selectable action. The legal set represents the only
/// valid input for the current decision point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    TakeChapterCard { slot_id: usize },
    TakeLandmark { id: String },
    TakeLandmarkUsingAnySkill { id: String },
    PlaySelectedCard,
    PlaySelectedCardNormally,
    PlaySelectedCardUsingAnySkill,
    DiscardSelectedCard,
    PlaceCardUnits { region: Region },
    ResolveEmptyCardUnitPlacement,
    MoveCardUnit { from: Region, to: Region },
    RemoveCardEnemyUnit { region: Region },
    ResolveUnavailableManeuver,
    PlaceQuestBonusUnit { region: Region },
    RemoveQuestBonusFortress { region: Region },
    DiscardOpponentGreyCard { card: ChapterCard },
    PlayDiscardedCardForFree { card: ChapterCard },
    ResolveAllianceTrigger { trigger: AllianceTrigger },
    ChooseAllianceRace { race: Race },
    TakeAllianceToken { token: AllianceToken },
    ChooseEntManeuver { maneuver: EntManeuver },
    PlaceAllianceUnits { region: Region },
    AcceptQuestBonus,
    SkipQuestBonus,
}
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TakeChapterCard { slot_id } => {
                write!(f, "take available chapter card at slot {slot_id}")
            }
            Self::TakeLandmark { id } => write!(f, "take Landmark {id}"),
            Self::TakeLandmarkUsingAnySkill { id } => {
                write!(f, "take Landmark {id}, using the Elf any-Skill effect")
            }
            Self::PlaySelectedCard => f.write_str("play the selected chapter card"),
            Self::PlaySelectedCardNormally => {
                f.write_str("play the selected chapter card without using its chaining symbol")
            }
            Self::PlaySelectedCardUsingAnySkill => {
                f.write_str("play the selected chapter card, using the Elf any-Skill effect")
            }
            Self::DiscardSelectedCard => f.write_str("discard the selected chapter card"),
            Self::PlaceCardUnits { region } => {
                write!(f, "deploy the card's Units in {region}")
            }
            Self::ResolveEmptyCardUnitPlacement => {
                f.write_str("resolve deployment (no Units remain in supply)")
            }
            Self::MoveCardUnit { from, to } => write!(f, "move 1 Unit from {from} to {to}"),
            Self::RemoveCardEnemyUnit { region } => {
                write!(f, "remove 1 enemy Unit from {region}")
            }
            Self::ResolveUnavailableManeuver => f.write_str("resolve maneuver (no legal target)"),
            Self::PlaceQuestBonusUnit { region } => {
                write!(f, "place the quest-bonus unit in {region}")
            }
            Self::RemoveQuestBonusFortress { region } => {
                write!(f, "remove the enemy fortress in {region}")
            }
            Self::DiscardOpponentGreyCard { card } => {
                write!(f, "discard the opponent's Grey card {card}")
            }
            Self::PlayDiscardedCardForFree { card } => {
                write!(f, "play discarded card {card} for free")
            }
            Self::ResolveAllianceTrigger { trigger } => write!(f, "resolve {trigger}"),
            Self::ChooseAllianceRace { race } => {
                write!(f, "reveal Alliance tokens from the {race} stack")
            }
            Self::TakeAllianceToken { token } => {
                write!(f, "take revealed {} token {}", token.race, token.id)
            }
            Self::ChooseEntManeuver { maneuver } => write!(f, "Ent maneuver: {maneuver}"),
            Self::PlaceAllianceUnits { region } => {
                write!(f, "place an Alliance-token Unit in {region}")
            }
            Self::AcceptQuestBonus => f.write_str("accept this optional quest bonus"),
            Self::SkipQuestBonus => f.write_str("skip this optional quest bonus"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntManeuver {
    RemoveEnemyUnit,
    OpponentLosesCoin,
    MoveUnit,
}
impl fmt::Display for EntManeuver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::RemoveEnemyUnit => "remove 1 enemy Unit",
            Self::OpponentLosesCoin => "make the opponent lose 1 Coin",
            Self::MoveUnit => "move 1 Unit",
        })
    }
}

/// A choice pauses automatic effect resolution. The server shows only actions
/// which answer this decision, never normal-turn actions at the same time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingDecision {
    CardDisposition {
        card: ChapterCard,
    },
    CardUnitPlacement {
        player: Faction,
        amount: u8,
        regions: Vec<Region>,
    },
    CardUnitMovement {
        player: Faction,
        remaining: u8,
    },
    CardEnemyUnitRemoval {
        player: Faction,
        remaining: u8,
    },
    QuestBonusUnitPlacement {
        player: Faction,
    },
    QuestBonusFortressRemoval {
        player: Faction,
        optional: bool,
    },
    QuestBonusChoice {
        player: Faction,
        effect: QuestBonusEffect,
    },
    OpponentGreyCardDiscard {
        player: Faction,
    },
    DiscardedCardFreePlay {
        player: Faction,
    },
    AllianceTriggerChoice {
        player: Faction,
        triggers: Vec<AllianceTrigger>,
    },
    AllianceRaceChoice {
        player: Faction,
    },
    AllianceTokenChoice {
        player: Faction,
        revealed: Vec<AllianceToken>,
    },
    EntManeuverChoice {
        player: Faction,
        remaining: u8,
    },
    AllianceUnitPlacement {
        player: Faction,
        remaining: u8,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    GameOver,
    IllegalAction(Action),
}
impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GameOver => f.write_str("the game is already over"),
            Self::IllegalAction(action) => write!(f, "illegal action: {action}"),
        }
    }
}
impl std::error::Error for GameError {}
