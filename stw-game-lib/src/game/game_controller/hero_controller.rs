use std::rc::Rc;

use rand::seq::SliceRandom;

use super::GameController;
use crate::{game::{GameCallback, map::TilePos, hero::Hero, GameError}, resource::{resource_data::HeroData, Resource}};

pub(crate) struct HeroController{

}

impl GameController for HeroController {
    fn process_game_step(game: &mut crate::game::STWGame) -> Result<Vec<GameCallback>, GameError> {
        let try_to_generate_hero = game.get_heroes().len() < game.max_heroes as usize;
        if try_to_generate_hero {
            let mut sum_tile_levels = 0.0;
            let mut prob: Vec<(TilePos, f32)> = vec![];

            for (pos, tile) in game.map.iter(){
                if let Some(content) = tile.get_field_content() {
                    prob.push((*pos, sum_tile_levels + content.data.hero_levels.len() as f32));
                    sum_tile_levels +=  content.data.hero_levels.len() as f32;
                }
            }

            let num: f32 = rand::random::<f32>() * sum_tile_levels;
            let mut index = 0;
            while index < prob.len() && prob[index].1 < num {
                index+=1;
            }        

            let choosen_pos = prob[index].0;
            let choosen_tile = game.map.get(&choosen_pos).unwrap();
            let choosen_hero_family = &choosen_tile.get_field_content().unwrap().data.hero_family;
            
            let prob = &choosen_tile.get_field_content().unwrap().data.hero_levels;
            
            let sum_tile_levels: f32 = prob.iter().sum();
            
            let num: f32 = rand::random::<f32>() * sum_tile_levels;
            let mut index = 0;
            let mut act_sum = 0.0;
            while index < prob.len() && prob[index] + act_sum < num {
                act_sum += prob[index];
                index+=1; 
            }

            let choosen_hero_level = index as u32;
            
            let hero_id = &game.resource_manager.get_resources::<HeroData>()
                .iter()
                .filter(|e|&e.1.data.hero_family == choosen_hero_family && e.1.data.hero_level == choosen_hero_level)
                .map(|e|e.1)
                .collect::<Vec<&Rc<Resource<HeroData>>>>()
                .choose(&mut rand::thread_rng())
                .ok_or_else(||GameError::new(format!("Game badly configured, there are lack of hero [level:{}, family:{}] that shoul be in resources", choosen_hero_level, choosen_hero_family)))?
                .id.to_string();
            
            game.heroes.push(Box::new(Hero::new(&game.resource_manager, hero_id.as_str())?));
            Ok(vec![
                GameCallback::NewHero{where_born: choosen_pos, hero_id: hero_id.to_string()}
            ])
        }else{
            Ok(vec![])
        }
    }
}