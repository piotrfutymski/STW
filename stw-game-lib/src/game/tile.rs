use std::{rc::{Rc}, fmt::Debug};

use colored::{ColoredString, Colorize};

use crate::resource::{ResourceManager, Resource, resource_data::{OriginFieldData, FieldTypeData}};

use super::{map::{TilePos}, GameError};

pub struct GameTile{
    position: TilePos,
    base_field_type: Rc<Resource<OriginFieldData>>,
    field_content: Option<Rc<Resource<FieldTypeData>>>,
    visible: bool,
    get_quest_completed_till_last_build: bool,

    resource_manager: Rc<ResourceManager>,
}


impl GameTile {
    pub fn new(p: &TilePos, rm: &Rc<ResourceManager>, bft: &str, visible: bool) -> Result<GameTile, GameError>{
        Ok(GameTile { 
            position: p.clone(), 
            base_field_type: rm.get_resource::<OriginFieldData>(bft)?.clone(),
            resource_manager: rm.clone(),
            field_content: None,
            visible: visible,
            get_quest_completed_till_last_build: false
        })
    }

    pub fn get_position(&self)->TilePos{
        self.position
    }

    pub fn get_quest_completed_till_last_build(&self)->bool{
        self.get_quest_completed_till_last_build
    }

    pub fn is_visible(&self)->bool{
        self.visible
    }

    pub fn get_base_field_type(&self)->Rc<Resource<OriginFieldData>>{
        self.base_field_type.clone()
    }

    pub fn get_field_content(&self)->Option<Rc<Resource<FieldTypeData>>>{
        self.field_content.clone().map_or(None, |e|Some(e.clone()))
    }

    pub fn set_field_content(&mut self, id: &str)-> Result<(), GameError>{
        self.field_content = Some(self.resource_manager.get_resource(id)?.clone());
        self.get_quest_completed_till_last_build = false;
        Ok(())
    }

    pub fn set_visible(&mut self, visible: bool){
        self.visible = visible;
    }

    pub fn get_colored_string(&self) -> ColoredString{
        match &self.field_content {
            Some(content) => {
                let (r,g,b) = self.color_based_on_visibility(content.data.color);
                content.id.chars().take(2).collect::<String>().truecolor(r,g,b)
            },
            None => {
                let (r,g,b) = self.color_based_on_visibility(self.base_field_type.data.color);
                self.base_field_type.id.chars().take(2).collect::<String>().truecolor(r,g,b)
            },
        }
    }

    //

    fn color_based_on_visibility(&self, (r,g,b):(u8,u8,u8)) -> (u8,u8,u8){
        match self.visible {
            true => (r,g,b),
            false => (r/3,g/3,b/3),
        }
    }
    
}

impl Debug for GameTile{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameTile")
        .field("position", &self.position)
        .field("base_field_type", &self.base_field_type)
        .field("field_content", &self.field_content)
        .field("visible", &self.visible)
        .field("get_quest_completed_till_last_build", &self.get_quest_completed_till_last_build)
        .finish()
    }
}