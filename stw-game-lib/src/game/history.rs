use std::{collections::HashMap, rc::Rc, fmt::Debug};

use crate::resource::{enums::{HeroSkill, FieldCharacteristic, ActionRequirment}, resource_data::ActionData, Resource, ResourceManager};

use super::{map::TilePos, STWGame, BadMove, tile::GameTile, game_controller::GameCallback, hero::Hero};

pub struct History{
    pub hero_index: usize,
    pub quest_pos: TilePos,
    pub steps: Vec<(TilePos, Option<Rc<Resource<ActionData>>>)>,
    pub points_got: HashMap<HeroSkill, f32>,
    pub current_modificators: HashMap<HeroSkill, f32>,
    pub current_pos: Option<TilePos>,
    pub path_left: u32
}

impl Debug for History{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("History")
        .field("hero_index", &self.hero_index)
        .field("quest_pos", &self.quest_pos)
        .field("points_got", &self.points_got)
        .field("current_modificators", &self.current_modificators)
        .field("current_pos", &self.current_pos)
        .field("path_left", &self.path_left)
        .finish()
    }
}

impl History{

    pub fn new(quest_pos: &TilePos, hero_index: usize, game: &STWGame) -> Result<History, BadMove> {
        History::can_start_new(quest_pos, hero_index, game)?;

        Ok(History{
            hero_index : hero_index,
            quest_pos: *quest_pos,
            steps: Vec::new(),
            points_got: HashMap::new(),
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
                    .flat_map(|e|self.get_possible_actions_for_field_content(e, game))
                    .collect()
            },
            None => {
                game.map.get_tiles_in_range_together(&self.quest_pos, self.path_left)
                    .iter()
                    .filter(|e|e.get_field_content().map_or(false, |b|b.data.characteristic == FieldCharacteristic::Habited))
                    .flat_map(|e|self.get_possible_actions_for_field_content(e, game))
                    .collect()
            }
        }
    }

    fn get_possible_actions_for_field_content(&self, game_tile: &GameTile, game: &STWGame) -> Vec<(TilePos, String)>{
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
            .filter(|e|self.steps.iter().find(|s|s.0 == game_tile.get_position() && s.1.as_ref().map_or("", |v|&v.id) == **e).is_none())
            .filter(|e|self.is_action_permited(e, game))
            .map(|e|(game_tile.get_position(), e.to_string()))
            .collect();
        if res.len() == 0 {
            res.push((game_tile.get_position(), String::from("")))
        }
        res
    }

    fn is_action_permited(&self, id: &String, game: &STWGame) ->bool{
        let action = game.resource_manager.get_resource::<ActionData>(id)
            .expect(format!("No action with id {} that should be in resources", id).as_str());

        action.data.requirments.iter()
            .all(|req|Self::action_meats_requirment(&self.steps, req))

    }

    fn action_meats_requirment(steps: &Vec<(TilePos, Option<Rc<Resource<ActionData>>>)>, req: &ActionRequirment)->bool{
        match req {
            crate::resource::enums::ActionRequirment::IsBefore(before) => 
                steps.iter().find(|e|e.1.as_ref().map_or("", |v|&v.id) == before).is_some(),
            crate::resource::enums::ActionRequirment::IsBeforeFamily(before) => 
                steps.iter().find(|e|e.1.as_ref().map_or("", |v|&v.data.action_family) == *before).is_some(),
            crate::resource::enums::ActionRequirment::IsBeforePosition(pos) => steps.len() < *pos,
        }
    }


    pub fn perform_move(&mut self, pos: TilePos, action: String, rm: &ResourceManager, heroes: &mut Vec<Box<Hero>>) -> Vec<GameCallback> {
        if action.len() != 0 {
            let action = rm.get_resource::<ActionData>(&action)
            .expect(format!("No action with id {} that should be in choosen_heroresources", &action).as_str());

            let rand_succes: f32 = rand::random();
            
            action.data.points
                .iter()
                .map(|e|(*e.0, self.current_modificators.get(e.0).unwrap_or(&1.0) * (rand_succes * (e.1.1 -e.1.0) + e.1.0)))
                .for_each(|e|*self.points_got.entry(e.0).or_insert(0.0) += e.1);

            action.data.bonus_points
                .iter()
                .filter(|v|Self::action_meats_requirment(&self.steps, &v.0))
                .flat_map(|v|&v.1)
                .map(|e|(*e.0, self.current_modificators.get(e.0).unwrap_or(&1.0) * (rand_succes * (e.1.1 -e.1.0) + e.1.0)))
                .for_each(|e|*self.points_got.entry(e.0).or_insert(0.0) += e.1);

            action.data.modificators
                .iter()
                .for_each(|e|{
                    self.current_modificators.entry(*e.0).and_modify(|v|*v+=*e.1);
            });

            let mut res = vec![];
            res.push(GameCallback::HeroMoved{dest_position: pos, hero_number: self.hero_index, success: rand_succes, action_performed: action.id.to_string()});

            if 1.0 - action.data.eternal_modificator.probability < rand_succes {
                action.data.eternal_modificator.skills
                    .iter()
                    .for_each(|e|{
                        heroes[self.hero_index]
                            .get_skills_mut()
                            .entry(*e.0).and_modify(|v|*v+=*e.1);
                        res.push(GameCallback::HeroLeveled{hero_number:self.hero_index, skill: *e.0, new_skill_value: *heroes[self.hero_index].get_skills().get(e.0).unwrap()});
                    });
            }

            self.steps.push((pos, Some(action)));
            self.current_pos = Some(pos);
            self.path_left -= 1;
            res
        }else{
            self.steps.push((pos, None));
            vec![GameCallback::HeroMoved{dest_position: pos, hero_number: self.hero_index, success: 0.0, action_performed: String::from("")}]
        }

    }

    
}