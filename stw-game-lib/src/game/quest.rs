use std::{rc::Rc, fmt::Debug};

use crate::resource::{resource_data::QuestData, Resource, ResourceManager};

use super::{map::TilePos, GameError};

pub struct Quest{
    quest_type: Rc<Resource<QuestData>>,
    position: TilePos,
    creation_turn: u32,



    _resource_manager: Rc<ResourceManager>,
}

impl Debug for Quest{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Quest")
        .field("quest_type", &self.quest_type)
        .field("position", &self.position)
        .field("creation_turn", &self.creation_turn)
        .finish()
    }
}

impl Quest{
    pub fn new(rm: &Rc<ResourceManager>, id: &str, turn: u32, position: &TilePos)-> Result<Quest, GameError> {
        Ok(Quest{ 
            quest_type: rm.get_resource::<QuestData>(id)?.clone(),
            _resource_manager: rm.clone(),
            creation_turn: turn,
            position: *position
        })
    }

    pub fn get_position(&self) -> TilePos{
        self.position
    } 
}