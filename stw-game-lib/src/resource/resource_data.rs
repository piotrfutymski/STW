use std::collections::HashMap;
use std::rc::Rc;

use serde::Deserialize;
use serde::Serialize;

use super::Resource;
use super::enums::*;

pub trait BaseResource{}
impl BaseResource for OriginFieldData {}
impl BaseResource for FieldTypeData {}
impl BaseResource for Globals {}
impl BaseResource for QuestData {}
impl BaseResource for HeroData {}
impl BaseResource for ActionData {}

//


#[derive(Serialize, Deserialize, Debug)]
pub struct Globals{
    pub init_path_length_per_hero: u32,
    pub start_game_resources: HashMap<GResource, u32>,

    pub map_frequency: f32,
    pub map_middle: String,
    pub map_near_mid_enable: Vec<String>,
    pub map_visible_on_start: u32,

    pub win_cond_build_path: String
}


#[derive(Serialize, Deserialize, Debug)]
pub struct OriginFieldData{
    pub height: f32,
    pub vegetation: f32,

    pub possible_actions: Vec<String>,

    pub color: (u8, u8, u8)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FieldTypeData{
    pub build_path: String,
    pub path_level: u32,
    pub characteristic: FieldCharacteristic,

    pub requirments: Vec<FieldRequirment>,
    pub cost: HashMap<GResource, u32>,

    pub instant_effects: Vec<FieldInstantEffect>,

    pub quest_family: String,
    pub quest_levels: Vec<f32>,

    pub hero_family: String,
    pub hero_levels: Vec<f32>,
    
    pub possible_actions: Vec<String>,

    pub color: (u8, u8, u8)

}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionData{
    pub action_family: String,

    pub modificators: HashMap<HeroSkill, f32>,
    pub points: HashMap<HeroSkill, (f32,f32)>,
    pub bonus_points: Vec<(ActionRequirment, HashMap<HeroSkill, (f32,f32)>)>,
    pub requirments: Vec<ActionRequirment>,

    pub eternal_modificator: EternalModificator,

    pub modificator_info: HashMap<HeroSkill, i32>

}


#[derive(Serialize, Deserialize, Debug)]
pub struct HeroData{
    pub hero_family: String,
    pub hero_level: u32,

    pub init_skills: HashMap<HeroSkill, f32>

}


#[derive(Serialize, Deserialize, Debug)]
pub struct QuestData{
    pub quest_family: String,
    pub quest_level: u32,

    pub quest_decisions: Vec<QuestDecision>,
    pub penalty: Vec<QuestPenalty>,

}

///


#[derive(Serialize, Deserialize, Debug)]
pub struct EternalModificator{
    pub skills: HashMap<HeroSkill, f32>,
    pub probability: f32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QuestDecision{
    pub id: String,
    pub required_skills: HashMap<HeroSkill, f32>,
    pub min_required_points: f32,
    pub points_50p: f32,
    pub additional_points: Vec<PointRequirment>,
    pub required: Vec<Vec<PointRequirment>>,

    pub treasure: HashMap<GResource, (f32, f32)>
}

//

impl Resource<FieldTypeData> {
    
    pub fn is_possible_to_be_build(&self, origin: &Resource<OriginFieldData>, old: &Option<Rc<Resource<FieldTypeData>>>, near: &Vec<Vec<Rc<Resource<FieldTypeData>>>>, quest_completed: bool) -> bool{
        let is_next_in_build_path = if let Some(d) = old{
            d.data.build_path == self.data.build_path && d.data.path_level as i32 == self.data.path_level as i32 - 1
        }else{
            false
        };

        let is_possible_replacement_for_building = self.data.requirments.iter()
            .find(|e|{
                match e {
                    FieldRequirment::Replaces(id) => self.id == *id,
                    _ => false
                }
            }).is_some();

        if old.is_some() && ! (is_next_in_build_path || is_possible_replacement_for_building) {
            return false;
        } else if old.is_none() && self.data.path_level != 0 {
            return false;
        }

        self.data.requirments
            .iter()
            .all(|r|{
                match r {
                    FieldRequirment::CantBuild => false,
                    FieldRequirment::Replaces(_) => true,   //replacement checked in first order

                    FieldRequirment::HasOrigin(id) => origin.id == *id,
                    FieldRequirment::QuestCompleted => quest_completed,
                    FieldRequirment::NearFieldWithCharacteristic { characteristic, distance } => {
                        near.iter().skip(1).take(*distance as usize).flat_map(|e|e.iter())
                        .find(|n|n.data.characteristic == *characteristic).is_some()
                    },
                    FieldRequirment::NearFieldWithPath { build_path, distance } => {
                        near.iter().skip(1).take(*distance as usize).flat_map(|e|e.iter())
                        .find(|n|n.data.build_path == *build_path).is_some()
                    },
                    FieldRequirment::NearFieldWithId { id, distance } => {
                        near.iter().skip(1).take(*distance as usize).flat_map(|e|e.iter())
                        .find(|n|n.id == *id).is_some()
                    },
                    FieldRequirment::NoNearFieldWithCharacteristic { characteristic, distance } =>{
                        near.iter().skip(1).take(*distance as usize).flat_map(|e|e.iter())
                        .find(|n|n.data.characteristic == *characteristic).is_none()
                    },
                    FieldRequirment::NoNearFieldWithPath { build_path, distance } => {
                        near.iter().skip(1).take(*distance as usize).flat_map(|e|e.iter())
                        .find(|n|n.data.build_path == *build_path).is_none()
                    },
                    FieldRequirment::NoNearFieldWithId { id, distance } => {
                        near.iter().skip(1).take(*distance as usize).flat_map(|e|e.iter())
                        .find(|n|n.id == *id).is_none()
                    },
                    FieldRequirment::HasOriginOneOf(v) => {
                        v.iter().find(|e|origin.id == **e).is_some()
                    },
                }
            })
    }



    pub fn has_enough_resources(&self, gres: &HashMap<GResource, u32>) -> bool{
        self.data.cost
            .iter()
            .all(|e|{
                gres.get(e.0).map_or(false, |g| g >= e.1)
            })
    }

    pub fn spent_resources(&self, gres: &mut HashMap<GResource, u32>) {
        self.data.cost
            .iter()
            .for_each(|e|{
                *gres.get_mut(e.0).unwrap() -= e.1;
            })
    }

    //

}