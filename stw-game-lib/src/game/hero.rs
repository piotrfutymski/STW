use std::{rc::Rc, collections::HashMap, fmt::Debug};

use strum::IntoEnumIterator;

use crate::resource::{resource_data::{ HeroData}, Resource, enums::HeroSkill, ResourceManager};

use super::{GameError};

pub struct Hero{
    background: Rc<Resource<HeroData>>,
    name: String,

    skills: HashMap<HeroSkill, f32>,
    learning_count: u32,
    _stories_get: u32,

    _resource_manager: Rc<ResourceManager>,
}

impl Debug for Hero{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hero")
            .field("background", &self.background)
            .field("skills", &self.skills)
            .field("name", &self.name)
            .field("learning_count", &self.learning_count)
            .finish()
    }
}

impl Hero{
    pub fn new(rm: &Rc<ResourceManager>, id: &str)-> Result<Hero, GameError> {
        Ok(Hero{ 
            background: rm.get_resource::<HeroData>(id)?.clone(),
            _resource_manager: rm.clone(),
            learning_count: 0,
            _stories_get: 0,
            name: "Ziom".to_string(),
            skills: {
                let mut res = rm.get_resource::<HeroData>(id)?.data.init_skills.clone();
                HeroSkill::iter()
                    .for_each(|e|{
                        res.entry(e).or_insert(1.0);
                });
                res
            }
            
        })
    }

    pub fn get_skills(&self) -> &HashMap<HeroSkill, f32>{
        &self.skills
    }

    pub fn get_skills_mut(&mut self) -> &mut HashMap<HeroSkill, f32>{
        &mut (self.skills)
    }
}