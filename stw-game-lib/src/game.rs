pub mod map;
mod tile;
mod hero;
mod quest;
mod history;

pub mod game_controller;
pub mod game_move;

use std::{collections::HashMap, fmt::Display, rc::Rc};
use rand::{Rng, distributions::Alphanumeric};
use strum::IntoEnumIterator;
use crate::resource::{ResourceManager, resource_data::{OriginFieldData, FieldTypeData, Globals, HeroData, QuestData, ActionData}, enums::GResource, Resource};

use self::{map::*, game_controller::{GameController, GameCallback, hero_controller::HeroController, quest_controller::QuestController}, game_move::{GameMove, PossibleBuilding}, hero::Hero, quest::Quest, history::History};

#[derive(Debug, Clone)]
pub struct GameError{
    pub msg: String
}

impl Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error occured while simulating game: {}", self.msg)
    }
}

impl GameError {
    pub fn new(msg: String) -> GameError{
        GameError { msg }
    }
}


#[derive(Debug, Clone)]
pub struct BadMove{
    pub msg: String
}

impl Display for BadMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bad move: {}", self.msg)
    }
}

impl BadMove {
    pub fn new(msg: String) -> BadMove{
        BadMove { msg }
    }
}


const DEFAULT_MAP_SIZE: u32 = 63;
const DEFAULT_GAME_NAME: &str = "New Game";

pub struct GameConfig{
    resources: Vec<(String, serde_json::Value)>,

    name: String,
    seed: String,
    map_size: u32
}

impl GameConfig {
    pub fn new() -> GameConfig{
        let random = rand::thread_rng();
        let random_seed: String = random
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();
        GameConfig { resources: Vec::new(), name: DEFAULT_GAME_NAME.to_string(), seed: random_seed, map_size: DEFAULT_MAP_SIZE }
    }

    pub fn set_resources(mut self, resources: Vec<(String, serde_json::Value)>) -> Self{
        self.resources = resources;
        self
    }

    pub fn set_name(mut self, name: &str) -> Self{
        self.name = String::from(name);
        self
    }   

    pub fn set_seed(mut self, seed: &str) -> Self{
        self.seed = String::from(seed);
        self
    }

    pub fn set_map_size(mut self, map_size: u32) -> Self{
        self.map_size = map_size;
        self
    }

    pub fn start_game(self) -> Result<STWGame, GameError>{
        if self.resources.len() == 0 {
            return Err(GameError::new(format!("No resources! - add this using builder method set_resources")));
        }
        let mut rm = ResourceManager::new();
        self.resources
            .into_iter()
            .try_for_each(|e| {
                match e.0.as_str() {
                    "Globals" => rm.add_resource::<Globals>(e.1),
                    "OriginFieldData" => rm.add_resource::<OriginFieldData>(e.1),
                    "FieldTypeData" => rm.add_resource::<FieldTypeData>(e.1),
                    "HeroData" => rm.add_resource::<HeroData>(e.1),
                    "QuestData" => rm.add_resource::<QuestData>(e.1),
                    "FieldActionData" => rm.add_resource::<ActionData>(e.1),
                    other => Err(GameError::new(format!("Unknown type of resource in GameConfig.start_game: {}", other)))
                }
            })?;

        let ref_rm: Rc<ResourceManager> = Rc::new(rm);
        let mut res = STWGame::new(&ref_rm)?;
        res.start_game(self.map_size, &self.seed)?;
        Ok(res)
    }
}

pub struct STWGame{
    map: Box<GameMap>,
    heroes: Vec<Box<Hero>>,
    quests: HashMap<TilePos,Box<Quest>>,

    history: Option<History>,

    game_turn: u32,
    game_resources: HashMap<GResource, u32>,

    max_heroes: u32,
    _max_additional_learning_hero: u32,
    max_path_length: u32,
    builded: Vec<String>,

    resource_manager: Rc<ResourceManager>

}

impl STWGame {

    pub fn perform_move(&mut self, game_move: &GameMove) -> Result<Vec<GameCallback>, BadMove>{
        match game_move {
            GameMove::Build(position, id) => self.build(position, id),
            GameMove::Wait => self.wait(),
            GameMove::StartHistory(position, hero_index) => self.start_history(position, *hero_index),
            GameMove::PlayMove(_, _) => todo!(),
            GameMove::MakeDecision(_) => todo!(),
            GameMove::RenameHero(_, _) => todo!(),
        }
    }

    pub fn get_possible_building_at_position(&self, pos: &TilePos) -> Vec<PossibleBuilding>{
        if let Some(tile) = self.map.get(pos){
            if !tile.is_visible() {
                return vec![];
            }

            let adjacent: Vec<Vec<Rc<Resource<FieldTypeData>>>> = self.map.get_tiles_in_range(pos, 5)
                .iter()
                .map(|v|v.iter()
                    .filter_map(|e|e.get_field_content())
                    .collect()
                ).collect();
            let res: Vec<PossibleBuilding> =self.resource_manager
            .get_resources::<FieldTypeData>()
            .iter()
            .filter(|f|{
                f.1.is_possible_to_be_build(tile.get_base_field_type().as_ref(), &tile.get_field_content(), &adjacent, tile.get_quest_completed_till_last_build())
            })
            .map(|f|
                PossibleBuilding {
                    id: f.1.id.clone(),
                    not_enought_resources: !f.1.has_enough_resources(&self.game_resources),
                }
            )
            .collect();
            return res;
        }
        vec![]
    }

    pub fn can_wait(&self) -> Result<(), BadMove>{
        self.is_playing_history()?;
        if self.map.check_if_exists_tile_with_field_path(&self.resource_manager.get_globals().data.win_cond_build_path) {
                Ok(())
            } else {
                Err(BadMove::new(format!("Can not perform game step because there are no required buildings")))
            }
    }

    pub fn can_start_history(&self, pos: &TilePos) -> Result<(), BadMove>{
        History::can_start_new(pos, 0, &self)
    }

    pub fn print(&self){
        self.map.print();
        println!("Resources: {:?}", self.game_resources);
        println!("Map center: {:?}", self.map.get_mid_position());
        println!("Heroes: {:?}", self.heroes);
        println!("Quests: {:?}", self.quests);
        print!("History {:?}", self.history);
    }

    pub fn get_heroes(&self) -> &Vec<Box<Hero>>{
        &self.heroes
    }

    pub fn get_quests(&self) -> &HashMap<TilePos,Box<Quest>>{
        &self.quests
    }

    //

    pub(crate) fn _get_heroes_mut(&mut self) -> &Vec<Box<Hero>>{
        &mut self.heroes
    }

    pub(crate) fn _get_quests_mut(&mut self) -> &HashMap<TilePos,Box<Quest>>{
        &mut self.quests
    }


    pub(crate) fn new(rm: &Rc<ResourceManager>) -> Result<STWGame, GameError>{
        Ok(STWGame { 
            map: Box::new(GameMap::new(rm)),
            heroes: Vec::new(),
            quests: HashMap::new(),
            game_resources: HashMap::new(),
            game_turn: 0,
            max_heroes: 1,
            _max_additional_learning_hero: 0,
            max_path_length: rm.get_globals().data.init_path_length_per_hero,
            builded: Vec::new(),
            resource_manager: rm.clone(),
            history: None,
        })
    }

    pub(crate) fn start_game(&mut self, size: u32, seed: &str) -> Result<(), GameError>{
        self.map.generate(size, seed)?;
        self.game_resources = self.resource_manager.get_globals().data.start_game_resources.clone();
        GResource::iter()
            .for_each(|e|{
                self.game_resources.entry(e).or_insert(0);
        });
        Ok(())
    }

    pub(crate) fn _load_game(&mut self, _game_save_file: &str){
        todo!()
    }

    

    //


    fn build(&mut self, pos: &TilePos, id: &str) ->  Result<Vec<GameCallback>, BadMove>{
        self.can_be_build(pos, id)?;

        self.map.get_mut(pos).expect("Checked in canBeBuild").set_field_content(id).unwrap();

        let before = self.game_resources.clone();

        let ftd = self.resource_manager.get_resource::<FieldTypeData>(id).unwrap();
        ftd.spent_resources(&mut self.game_resources);

        let mut res = self.play_instant_effects(&ftd);

        res.append(&mut self.get_changed_resource_callbacks(&before));
        res.push(GameCallback::NewTileContent(*pos, id.to_string()));
        self.builded.push(id.to_string());
        Ok(res)
    }

    fn get_changed_resource_callbacks(&self, before: &HashMap<GResource, u32>)-> Vec<GameCallback>{
        before.iter()
        .filter(|e|e.1 != self.game_resources.get(e.0).unwrap())
        .map(|e|GameCallback::ChangedResource(*e.0, *self.game_resources.get(e.0).unwrap()))
        .collect()
    }

    fn can_be_build(&self, pos: &TilePos, id: &str) -> Result<(), BadMove>{
        self.is_playing_history()?;
        if self.quests.get(pos).is_some() {
            return Err(BadMove::new(format!("To build here complete quest - position {:?}", pos)));
        }

        match self.get_possible_building_at_position(pos)
            .iter()
            .find(|e|e.id == id) {
                Some(e) => if e.not_enought_resources{
                    return Err(BadMove::new(format!("Not enought resources to build {} at position {:?}", id, pos)));
                }else{
                    Ok(())
                },
                None => Err(BadMove::new(format!("Cant build {} at position {:?}", id, pos))),
            }
    }

    fn play_instant_effects(&mut self, ftd: &Rc<Resource<FieldTypeData>>) -> Vec<GameCallback> {
        ftd.data.instant_effects.iter()
            .map(|e|match e {
                crate::resource::enums::FieldInstantEffect::ShowFields { .. } => {
                    todo!()
                },
                crate::resource::enums::FieldInstantEffect::GiveResources { only_first_time, resource, amount } => {
                    if self.check_if_should_play_effect(ftd, *only_first_time) {
                        self.game_resources.entry(*resource).and_modify(|f| *f += amount);
                        vec![]
                    }else{
                        vec![]
                    }
                },
                crate::resource::enums::FieldInstantEffect::IncreaseSkill { .. } => {
                    todo!()
                },
                crate::resource::enums::FieldInstantEffect::LearnAboutAction { .. } => todo!(),
                crate::resource::enums::FieldInstantEffect::IncreaseMaxHeroes { only_first_time } => {
                    if self.check_if_should_play_effect(ftd, *only_first_time) {
                        self.max_heroes += 1;
                        vec![GameCallback::MaxHeroesIncreased(self.max_heroes)]
                    }else{
                        vec![]
                    }
                },
            })
            .flatten()
            .collect()
    }

    fn check_if_should_play_effect(&self, ftd: &Rc<Resource<FieldTypeData>>, only_first_time: bool) -> bool{
        !only_first_time || self.builded.contains(&ftd.id)
    }

    fn wait(&mut self) -> Result<Vec<GameCallback>, BadMove>{
        self.can_wait()?;
        self.game_turn += 1;
        let mut res: Vec<GameCallback> = HeroController::process_game_step( self).unwrap();
        let mut to_append = QuestController::process_game_step( self).unwrap();
        res.append(&mut to_append);
        Ok(res)
    }

    fn start_history(&mut self, pos: &TilePos, hero_index: usize) -> Result<Vec<GameCallback>, BadMove>{
        self.history = Some(History::new(pos, hero_index, &self)?);
        Ok(vec![GameCallback::StartedHistory(*pos, hero_index)])
    }

    fn _is_waiting_for_decision(&self) ->Result<(), BadMove>{
        Ok(())
        //Err(BadMove::new(format!("Game is waitng for decision")))
    }

    fn is_playing_history(&self) ->Result<(), BadMove>{
        match self.history {
            Some(_) => Err(BadMove::new(format!("Game is playing history"))),
            None => Ok(()),
        }
    }
    
    

}


//DOKOŃCZYĆ DEMO
//TESTY!!!
//GITHUB
//GODOT
//DOKUMENTACJA