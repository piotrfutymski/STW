use crate::resource::enums::{GResource, HeroSkill};

use super::{STWGame, map::TilePos, GameError};

pub mod hero_controller;
pub mod quest_controller;

pub(crate) trait GameController {
    fn process_game_step(game: &mut STWGame) -> Result<Vec<GameCallback>, GameError>;
}


#[derive(Debug, Clone, PartialEq)]
pub enum GameCallback {
    ChangedResource{resource: GResource, new_value: u32},
    NewTileContent{position: TilePos, field_type_id: String},
    MaxHeroesIncreased{current_max_heroes: u32},
    NewHero{where_born: TilePos, hero_id: String},
    NewQuest{where_created: TilePos, quest_id: String},
    StartedHistory{quest_pos: TilePos , choosen_hero: usize},
    HeroLeveled{hero_number: usize, skill: HeroSkill, new_skill_value: f32},
    HeroMoved{dest_position: TilePos, hero_number: usize, success: f32, action_performed: String}
}