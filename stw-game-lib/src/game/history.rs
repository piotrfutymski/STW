use std::{collections::HashMap, rc::Rc};

use crate::resource::{enums::{HeroSkill, FieldCharacteristic}, resource_data::FieldTypeData, Resource};

use super::{map::TilePos, STWGame, BadMove, tile::GameTile};

#[derive(Debug)]
pub struct History{
    pub hero_index: usize,
    pub quest_pos: TilePos,
    pub steps: Vec<(TilePos, String)>,
    pub points_got: Vec<HashMap<HeroSkill, f32>>,
    pub current_modificators: HashMap<HeroSkill, f32>,
    pub current_pos: Option<TilePos>,
    pub path_left: u32
}

impl History{

    pub fn new(quest_pos: &TilePos, hero_index: usize, game: &STWGame) -> Result<History, BadMove> {
        History::can_start_new(quest_pos, hero_index, game)?;

        Ok(History{
            hero_index : hero_index,
            quest_pos: *quest_pos,
            steps: Vec::new(),
            points_got: Vec::new(),
            current_modificators: game.heroes[hero_index].get_skills().clone(),
            current_pos: None,
            path_left: game.max_path_length
        })

    }

    pub fn can_start_new(quest_pos: &TilePos, hero_index: usize, game: &STWGame) -> Result<(), BadMove> {
        if let Some(_) =  game.quests.get(quest_pos){
            if game.heroes.len() > hero_index {
                if game.map.get_tiles_in_range_together(quest_pos, game.max_path_length)
                    .iter()
                    .find(|e|e.get_field_content().map_or(false, |b|b.data.characteristic == FieldCharacteristic::Habited))
                    .is_some() {
                        Ok(())
                    } else {    
                        Err(BadMove::new(format!("Hero can't go so many tiles to get to the quest")))
                    }

            } else {
                Err(BadMove::new(format!("No hero with index {:?}", hero_index)))
            }
        }else{
            Err(BadMove::new(format!("No quest at position {:?}", quest_pos)))
        }
    }

    pub fn get_possible_next_move(&self, game: &STWGame) -> Vec<(TilePos, String)> {
        match self.current_pos {
            Some(current_pos) => {
                current_pos.adjacent_positions()
                    .iter()
                    .filter(|e|e.distance(&self.quest_pos) <= self.path_left)
                    .filter_map(|e|game.map.get(e))
                    .flat_map(|e|self.get_possible_actions_for_field_content(e))
                    .collect()
            },
            None => {
                game.map.get_tiles_in_range_together(&self.quest_pos, self.path_left)
                    .iter()
                    .filter(|e|e.get_field_content().map_or(false, |b|b.data.characteristic == FieldCharacteristic::Habited))
                    .flat_map(|e|self.get_possible_actions_for_field_content(e))
                    .collect()
            }
        }
    }

    fn get_possible_actions_for_field_content(&self, game_tile: &GameTile) -> Vec<(TilePos, String)>{
        let base_actions = &game_tile.get_base_field_type().data.possible_actions;
        let mut res: Vec<(TilePos, String)> = match &game_tile.get_field_content() {
            Some(field_content) => {
                &field_content.data.possible_actions
            }
            None => {
                base_actions
            }
        }
        .iter()
            .filter(|e|!self.steps.contains(&(game_tile.get_position(), e.to_string())))
            .map(|e|(game_tile.get_position(), e.to_string()))
            .collect();
        if res.len() == 0 {
            res.push((game_tile.get_position(), String::from("")))
        }
        res
    }

}