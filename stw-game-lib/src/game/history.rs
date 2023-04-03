use std::collections::HashMap;

use crate::resource::enums::{HeroSkill, FieldCharacteristic};

use super::{map::TilePos, STWGame, BadMove};

#[derive(Debug)]
pub struct History{
    pub hero_index: usize,
    pub quest_pos: TilePos,
    pub steps: Vec<(TilePos, String)>,
    pub points_got: Vec<HashMap<HeroSkill, f32>>,
    pub current_modificators: HashMap<HeroSkill, f32>
}

impl History{

    pub fn new(quest_pos: &TilePos, hero_index: usize, game: &STWGame) -> Result<History, BadMove> {
        History::can_start_new(quest_pos, hero_index, game)?;

        Ok(History{
            hero_index : hero_index,
            quest_pos: *quest_pos,
            steps: Vec::new(),
            points_got: Vec::new(),
            current_modificators: game.heroes[hero_index].get_skills().clone()
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

}