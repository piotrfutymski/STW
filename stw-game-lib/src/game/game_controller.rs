use crate::resource::enums::GResource;

use super::{STWGame, map::TilePos, GameError};

pub mod hero_controller;
pub mod quest_controller;

pub(crate) trait GameController {
    fn process_game_step(game: &mut STWGame) -> Result<Vec<GameCallback>, GameError>;
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameCallback {
    ChangedResource(GResource, u32),
    NewTileContent(TilePos, String),
    MaxHeroesIncreased(u32),
    NewHero(TilePos, String),
    NewQuest(TilePos, String),
    StartedHistory(TilePos , usize)
}