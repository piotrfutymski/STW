pub mod resource_data;
pub mod enums;
use std::{rc::{Rc}, collections::{HashMap}, any::{Any, TypeId}};
use serde::{Deserialize, Serialize};
use crate::game::GameError;
use self::resource_data::{Globals, BaseResource};

#[derive(Serialize, Deserialize, Debug)]
pub struct Resource<T>{
    pub id: String,
    pub data: T
}

pub struct ResourceManager{
    resources: HashMap<TypeId, HashMap<String, Rc<dyn Any>>>
}

impl ResourceManager {

    pub fn new() -> ResourceManager{
        ResourceManager { resources: HashMap::new() }
    }

    pub(crate) fn add_resource<T: BaseResource + for<'a> Deserialize<'a> + 'static>(&mut self, json: serde_json::Value) -> Result<(), GameError>{
        let id = json["id"].as_str().ok_or_else(||GameError::new(format!("Resource {} has no id field", json)))?.to_string();
        match serde_json::from_value::<Resource<T>>(json) {
            Ok(resource) => {
                let type_id = TypeId::of::<T>();
            self.resources.entry(type_id)
                .or_insert_with(||HashMap::new())
                .insert(id, Rc::new(resource));
            Ok(())
            },
            Err(e) => {
                Err(GameError::new(format!("Can not read resource of type {} with id {} - error {}", std::any::type_name::<T>(), id, e)))
            }
        }
    }

    pub(crate) fn get_resource<T: BaseResource + 'static>(&self, str: &str) -> Result<Rc<Resource<T>>, GameError>{
        let type_id = TypeId::of::<T>();
        Ok(
            self.resources
            .get(&type_id)
            .expect("Bad type in ResourceManager.get_resource or no resource with given id")
            .get(str)
            .ok_or_else(||GameError::new(format!("Can't ResourceManager.get_resource with id {}. Probably missing resource", str)))?
            .clone()
            .downcast::<Resource<T>>()
            .unwrap()
        )

    }

    pub(crate) fn get_globals(&self) -> Rc<Resource<Globals>>{
        self.get_resources::<Globals>()
                .values()
                .next()
                .expect("Can not read global resources. Add min. one instance")
                .clone()
                
    }

    pub(crate) fn get_resources<T: BaseResource + 'static>(&self) -> HashMap<&str, Rc<Resource<T>>>{
        let type_id = TypeId::of::<T>();
        self.resources
            .get(&type_id)
            .expect("Bad type in ResourceManager.get_resources")
            .iter()
            .map(|e|{(
                e.0.as_str(),
                e.1.clone().downcast::<Resource<T>>().unwrap()
            )})
            .collect()
    }

    pub(crate) fn _get_possible_names<T: BaseResource + 'static>(&self) -> Vec<&str>{
        let type_id = TypeId::of::<T>();
        self.resources
            .get(&type_id)
            .expect("Bad type in ResourceManager.get_possible_names")
            .iter()
            .map(|e|{e.0.as_str()})
            .collect()
    }

}


#[cfg(test)]
mod tests {

    use crate::resource::resource_data::{OriginFieldData, FieldTypeData};

    use super::*;

    pub fn generate_test_resources() -> ResourceManager{

        let mut resource_manager = ResourceManager::new();

            assert_eq!(resource_manager.add_resource::<Globals>(serde_json::json!({
                "id": "globals",
                "data":{
                "init_path_length_per_hero": 3,
                "start_game_resources":  {
                    "Gold": 5
                },
                "map_frequency": 0.3,
                "map_middle": "village_small",
                "map_near_mid_enable": ["meadow", "forest", "hills"],
                "map_visible_on_start": 3,
                "win_cond_build_path": "town"
            }
            })).is_ok(), true);
    
            assert_eq!(resource_manager.add_resource::<OriginFieldData>(serde_json::json!({
                "id": "meadow",
                "data":{
                    "height": 0.6,
                    "vegetation": 0.4,
                    "color": [200,255,0],
                    "possible_actions":["exploration_meadow", "gathering_meadow"]
                }
            })).is_ok(), true);

            assert_eq!(resource_manager.add_resource::<FieldTypeData>(serde_json::json!({
                "id": "village_small",
                "data":{
                    "build_path": "town",
                    "path_level": 0,
                    "characteristic": "Habited",

                    "requirments": [
                        {"NoNearFieldWithCharacteristic": {"characteristic":"Mysterious", "distance": 2}},
                        {"NoNearFieldWithPath": {"build_path":"town", "distance": 5}}
                    ],
                    "cost": {"Gold":10, "RareTrophy":1},

                    "instant_effects": [{"ShowFields": {"only_first_time": false, "amount": 18}}],

                    "quest_family": "",
                    "quest_levels": [],

                    "hero_family": "village",
                    "hero_levels": [0.6,0.4],

                    "possible_actions": [
                        "collect_info_village",
                        "shopping_village"
                    ],
                    "color": [139,69,19]
                }
            })).is_ok(), true);


        resource_manager
    }

    #[test]
    fn test_resource_manager_all_types(){
        let resource_manager = generate_test_resources();
        assert_eq!(resource_manager.get_resources::<Globals>().len(), 1);
        assert_eq!(resource_manager.get_resources::<OriginFieldData>().len(), 1);
        assert_eq!(resource_manager.get_resources::<FieldTypeData>().len(), 1);

    }

}