use std::rc::Rc;

use rand::seq::SliceRandom;

use super::GameController;
use crate::{game::{GameCallback, GameError, map::TilePos, quest::Quest}, resource::{resource_data::QuestData, Resource}};

pub(crate) struct QuestController{

}

impl GameController for QuestController {
    fn process_game_step(game: &mut crate::game::STWGame) -> Result<Vec<GameCallback>, GameError> {
        let mut sum_tile_levels = 0.0;
        let mut prob: Vec<(TilePos, f32)> = vec![];

        let mut tiles_with_quests: Vec<TilePos> = game.quests.keys().into_iter()
            .map(|e|*e).collect();

        for (pos, tile) in game.map.iter(){
            if let Some(content) = tile.get_field_content() {
                let f_t = tiles_with_quests.iter()
                    .enumerate()
                    .find(|e|{
                        tile.get_position() == *e.1
                    }).map(|e|e.0);
                if let Some(ind) = f_t {
                    tiles_with_quests.remove(ind);
                }else{
                    prob.push((*pos, sum_tile_levels + content.data.quest_levels.len() as f32));
                    sum_tile_levels +=  content.data.quest_levels.len() as f32;
                }
            }
        }

        if sum_tile_levels == 0.0 {
            return Ok(vec![]);
        }

        let num: f32 = rand::random::<f32>() * sum_tile_levels;
        let mut index = 0;
        while index < prob.len() && prob[index].1 <= num {
            index+=1;
        }

        let choosen_pos = prob[index].0;
        let choosen_tile = game.map.get(&choosen_pos).unwrap();
        let choosen_quest_family = &choosen_tile.get_field_content().unwrap().data.quest_family; 

        let prob = &choosen_tile.get_field_content().unwrap().data.quest_levels;
            
        let sum_tile_levels: f32 = prob.iter().sum();
            
        let num: f32 = rand::random::<f32>() * sum_tile_levels;
        let mut index = 0;
        let mut act_sum = 0.0;
        while index < prob.len() && prob[index] + act_sum <= num {
            act_sum += prob[index];
            index+=1; 
        }

        let choosen_quest_level = index as u32;

        let quest_id = &game.resource_manager.get_resources::<QuestData>()
            .iter()
            .filter(|e|&e.1.data.quest_family == choosen_quest_family && e.1.data.quest_level == choosen_quest_level)
            .map(|e|e.1)
            .collect::<Vec<&Rc<Resource<QuestData>>>>()
            .choose(&mut rand::thread_rng())
            .ok_or_else(||GameError::new(format!("Game badly configured, there are lack of quest [level:{}, family:{}] that shoul be in resources", choosen_quest_level, choosen_quest_family)))?
            .id.to_string();
            
        game.quests.insert(choosen_pos, Box::new(Quest::new(&game.resource_manager, quest_id.as_str(), game.game_turn, &choosen_pos)?));
        Ok(vec![
            GameCallback::NewQuest(choosen_pos, quest_id.to_string())
        ])      

    }
}