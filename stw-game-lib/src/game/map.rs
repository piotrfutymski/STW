use std::{collections::{HashMap}, rc::{Rc}, ops::{Deref, DerefMut}};

use crate::resource::{ResourceManager, resource_data::OriginFieldData, Resource};
use colored::{Colorize, ColoredString};
use noise::{Perlin};
use noise::utils::{NoiseMapBuilder, PlaneMapBuilder};
use rand::Rng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use super::{tile::*, GameError};

/**
 *    _   _   _   _
 *  _/ \_/ \_/ \_/ \
 * / \_/ \_/ \_/ \_/
 * \_/ \_/ \_/ \_/ \
 * / \_/ \_/ \_/ \_/
 * \_/ \_/ \_/ \_/ \
 * / \_/ \_/ \_/ \_/
 * \_/ \_/ \_/ \_/ \
 */
pub struct GameMap{
    
    size: u32,
    tiles: HashMap<TilePos, Box<GameTile>>,

    resource_manager: Rc<ResourceManager>
}


impl GameMap {

    pub fn new(rm: &Rc<ResourceManager>) -> GameMap{
        GameMap { 
            tiles: HashMap::new(),
            resource_manager: rm.clone(),
            size: 0
        }
    }
    
    pub fn generate(&mut self, size: u32, seed: &str) -> Result<(), GameError> {
        self.size = size;
        let base_tile_types = self.resource_manager.get_resources::<OriginFieldData>();
        let tile_map: Vec<TilePos> = GameMap::get_tile_positions_for_grid(size);
        let noise_map = self.generate_noise_map(&tile_map, seed);
        let mid = self.get_mid_position();
        let map_visible_on_start = self.resource_manager.get_globals().data.map_visible_on_start;
        let enabled_near = &self.resource_manager.get_globals().data.map_near_mid_enable;
        self.tiles = noise_map.iter()
            .map(|e|->Result<_,GameError>{
                Ok((*e.0, Box::new( GameTile::new(e.0, &self.resource_manager, 
                    Self::find_proper_resource(&base_tile_types, e.1.0, e.1.1, e.0.distance(&mid) <= 1, enabled_near), 
                    e.0.distance(&mid) <= map_visible_on_start)?)))
            }).collect::<Result<_,_>>()?;

        self.tiles.get_mut(&mid).unwrap().set_field_content(
            &self.resource_manager.get_globals().data.map_middle
        )
    }

    pub fn get_tile(&self, pos: &TilePos) -> &GameTile{
        self.tiles.get(pos).unwrap()
    }

    pub fn print(&self){
        let cords = GameMap::get_min_max_coords(self.size);
        let mut lines = Vec::new();
        for r in cords.0..cords.1 as i32{
            let text: Vec<ColoredString> = (cords.2..cords.3)
            .map(|q|{
                self.tiles
                    .get(&TilePos{q,r})
                    .map_or_else(||"##".to_string().truecolor(255,255,255), |e|e.get_colored_string())
                }
            )
            .collect();
            lines.push(text)
            
        }
        for l in lines.iter(){
            for e in l.iter(){
                print!("{}",e)
            }
            print!("\n")
        }
    }

    pub fn get_adjacent_tiles(&self, pos: &TilePos) -> Vec<&GameTile> {
        pos.adjacent_positions().iter()
            .filter_map(|e|self.tiles.get(e).map(
                |t|t.as_ref()
            )).collect()
            
    }

    pub fn get_tiles_in_range(&self, pos: &TilePos, length: u32) -> Vec<Vec<&GameTile>> {
        let mut res:Vec<Vec<&GameTile>> = Vec::new();
        res.push(vec![self.tiles.get(pos).expect("Use of not existing position in map.getTilesInRange").as_ref()]);
        
        for i in 1..length+1 {
            let to_add: Vec<&GameTile> = pos.positions_in_distance(i).iter()
                .filter_map(|e|self.tiles.get(&e).map(
                    |t|t.as_ref()
                )).collect();
            res.push(to_add);
        }
        res  
            
    }

    pub fn get_tiles_in_range_together(&self, pos: &TilePos, length: u32) -> Vec<&GameTile> {
        self.get_tiles_in_range(pos, length)
            .into_iter()
            .flat_map(|e|e.into_iter())
            .collect()       
    }

    pub fn get_mid_position(&self) -> TilePos{
        TilePos { q: (self.size/4) as i32, r: (self.size/2) as i32}
    }

    pub fn check_if_exists_tile_with_field_content(&self, id: &str) ->bool{
        self.iter()
            .any(|e|{
                match e.1.get_field_content(){
                    Some(content) => content.id == id,
                    None => false,
                }
            })
    }

    pub fn check_if_exists_tile_with_field_path(&self, path: &str) ->bool{
        self.iter()
            .any(|e|{
                match e.1.get_field_content(){
                    Some(content) => content.data.build_path == path,
                    None => false,
                }
            })
    }

    //priv

    fn find_proper_resource<'a>(btt: &HashMap<&'a str, Rc<Resource<OriginFieldData>>>, h: f32, v:f32, is_near: bool, enabled_near: &[String]) -> &'a str{
        btt.iter()
            .filter(|e| !is_near || enabled_near.contains(&e.0.to_string()))
            .filter(|e|e.1.data.height > h && e.1.data.vegetation > v)
            .map(|e|(e.0, e.1.data.height + e.1.data.vegetation))
            .min_by(|a,b| a.1.total_cmp(&b.1))
            .map(|e|*e.0)
            .unwrap_or(btt.keys().next().unwrap())
    }

    fn get_min_max_coords(size: u32) ->(i32,i32,i32,i32) {
        (0, size as i32,-(size as i32-1)/2, size as i32)
    }
    
    fn get_tile_positions_for_grid(size: u32) -> Vec<TilePos>{
        let cords = GameMap::get_min_max_coords(size);
        let mut res: Vec<TilePos> = Vec::new();
        for r in cords.0..cords.1 as i32{
            for q in 0-(r/2)..cords.3-(r/2){
                res.push(TilePos { q,r })
            }
        }
        res
    }

    fn generate_noise_map(&self, positions: &Vec<TilePos>, seed: &str) -> HashMap<TilePos, (f32, f32)>{
        let mut rng: Pcg64 = Seeder::from(seed).make_rng();
        let u32_seed: u32 = rng.gen();
        let perlin = Perlin::new(u32_seed);

        let map_frequency = self.resource_manager.get_globals().data.map_frequency as f64;

        let noise_map = PlaneMapBuilder::<_, 2>::new(perlin)
            .set_size((self.size + (self.size-1)/2) as usize, (self.size * 2) as usize)
            .set_x_bounds(0.0,  self.size as f64 * map_frequency)
            .set_y_bounds(0.0, self.size as f64 * map_frequency)
            .build();

        positions
            .iter()
            .map(|e|(*e, 
                (((noise_map.get_value((e.q + (self.size as i32-1)/2) as usize, e.r as usize) as f32).abs()-0.001), 
                ((noise_map.get_value((e.q + (self.size as i32-1)/2) as usize, (e.r + self.size as i32) as usize) as f32).abs()-0.001)))
            )
            .collect()

    }

}


impl Deref for GameMap {
    type Target = HashMap<TilePos, Box<GameTile>>;

    fn deref(&self) -> &Self::Target {
        &self.tiles
    }
}

impl DerefMut for GameMap {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tiles
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TilePos{
    pub q: i32,
    pub r: i32,
}

impl TilePos {

    pub fn neightbour(&self, direction: u32)->TilePos{
        match direction {
            0 => self.moved(1,0),
            1 => self.moved(1,-1),
            2 => self.moved(0,-1),
            3 => self.moved(-1,0),
            4 => self.moved(-1,1),
            5 => self.moved(0,1),
            _ => panic!("Not good direction for TilePos neightbour")
        }
    }
    
    pub fn distance(&self, second: &TilePos)-> u32{
        (
            self.q.abs_diff(second.q) + 
            self.r.abs_diff(second.r) + 
            (self.q + self.r - second.q - second.r).abs() as u32 
        ) / 2
    }

    pub fn moved(&self, q:i32, r:i32) -> TilePos{
        TilePos { q: self.q+q, r: self.r +r }
    }

    pub fn adjacent_positions(&self) -> [TilePos;6]{
        [self.moved(0,1), self.moved(0,-1),self.moved(1,0),self.moved(-1,0),self.moved(-1,1),self.moved(1,-1)]
    }

    pub fn positions_in_distance(&self, distance: u32) -> Vec<TilePos>{
        if distance == 0 {
            panic!("Distance should be grater than 0");
        }
        let mut hex = TilePos{q: self.q - distance as i32, r: self.r + distance as i32};
        let mut res = Vec::new();
        res.reserve(6 * distance as usize);
        for i in 0..6 {
            for _j in 0..distance {
                res.push(hex);
                hex = hex.neightbour(i);
            }
        }
        res
    }

}

#[cfg(test)]
mod tests{
    use std::{rc::Rc};

    use crate::{resource::{ResourceManager, resource_data::{OriginFieldData, Globals, FieldTypeData}}, game::map::TilePos};

    use super::GameMap;


    fn prepare_simple_map() -> GameMap{
        let mut rm = ResourceManager::new();
        rm.add_resource::<OriginFieldData>(serde_json::json!({
            "id": "meadow",
            "data":{
                "height": 0.6,
                "vegetation": 0.4,
                "color": [200,255,0],
                "possible_actions":["exploration_meadow", "gathering_meadow"]
            }
        })).unwrap();
        rm.add_resource::<Globals>(serde_json::json!(serde_json::json!({
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
        }))).unwrap();
        rm.add_resource::<FieldTypeData>(serde_json::json!(serde_json::json!({
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
                "hero_levels": [1.0],

                "possible_actions": [
                    "collect_info_village",
                    "shopping_village"
                ],
                "color": [139,69,19]
            }
        }))).unwrap();

        let mut map = GameMap::new(&Rc::new(rm));
        map.generate(7, "test").unwrap();
        map

    }

    #[test]
    pub fn test_generate() {
        let map = prepare_simple_map();
        assert!(map.size == 7);
        let center = map.get(&TilePos { q: 1, r: 3 }).unwrap();
        assert!(center.get_field_content().is_some());
        assert!(map.iter().count() == 49)

    }

    #[test]
    pub fn test_get_adjacent_tiles() {
        let map = prepare_simple_map();
        let res: Vec<TilePos> = map.get_adjacent_tiles(&super::TilePos { q: 2, r: 4 }).iter().map(|e|e.get_position()).collect();
        assert!(res.len() == 6);
        assert!(res.iter().find(|e|e.q== 2&& e.r == 3).is_some());
        assert!(res.iter().find(|e|e.q== 3&& e.r == 3).is_some());
        assert!(res.iter().find(|e|e.q== 3&& e.r == 4).is_some());
        assert!(res.iter().find(|e|e.q== 2&& e.r == 5).is_some());
        assert!(res.iter().find(|e|e.q== 1&& e.r == 5).is_some());
        assert!(res.iter().find(|e|e.q== 1&& e.r == 4).is_some());
        
        let res: Vec<TilePos> = map.get_adjacent_tiles(&super::TilePos { q: 0, r: 0 }).iter().map(|e|e.get_position()).collect();
        assert!(res.len() == 2);
        assert!(res.iter().find(|e|e.q== 1&& e.r == 0).is_some());
        assert!(res.iter().find(|e|e.q== 0&& e.r == 1).is_some());

    }

    #[test]
    pub fn test_get_tiles_in_range() {
        let map = prepare_simple_map();
        let res = map.get_tiles_in_range(&super::TilePos { q: 2, r: 4 },2);
        assert_eq!(res.len(), 3);
        let el0 = res[0][0].get_position();
        assert!(el0.q == 2 && el0.r == 4);
        let elem1: Vec<TilePos> = res[1].iter().map(|e|e.get_position()).collect();
        assert_eq!(elem1.len(), 6);
        assert!(elem1.iter().find(|e|e.q== 2&& e.r == 3).is_some());
        assert!(elem1.iter().find(|e|e.q== 3&& e.r == 3).is_some());
        assert!(elem1.iter().find(|e|e.q== 3&& e.r == 4).is_some());
        assert!(elem1.iter().find(|e|e.q== 2&& e.r == 5).is_some());
        assert!(elem1.iter().find(|e|e.q== 1&& e.r == 5).is_some());
        assert!(elem1.iter().find(|e|e.q== 1&& e.r == 4).is_some());
        let elem2: Vec<TilePos> = res[2].iter().map(|e|e.get_position()).collect();
        assert_eq!(elem2.len(), 12);
        assert!(elem2.iter().find(|e|e.q== 2&& e.r == 2).is_some());
        assert!(elem2.iter().find(|e|e.q== 3&& e.r == 2).is_some());
        assert!(elem2.iter().find(|e|e.q== 4&& e.r == 2).is_some());
        assert!(elem2.iter().find(|e|e.q== 4&& e.r == 3).is_some());
        assert!(elem2.iter().find(|e|e.q== 4&& e.r == 4).is_some());
        assert!(elem2.iter().find(|e|e.q== 3&& e.r == 5).is_some());
        assert!(elem2.iter().find(|e|e.q== 2&& e.r == 6).is_some());
        assert!(elem2.iter().find(|e|e.q== 1&& e.r == 6).is_some());
        assert!(elem2.iter().find(|e|e.q== 0&& e.r == 6).is_some());
        assert!(elem2.iter().find(|e|e.q== 0&& e.r == 5).is_some());
        assert!(elem2.iter().find(|e|e.q== 0&& e.r == 4).is_some());
        assert!(elem2.iter().find(|e|e.q== 1&& e.r == 3).is_some());

        let res = map.get_tiles_in_range(&super::TilePos { q: 0, r: 0 },2);
        assert_eq!(res.len(), 3);
        let el0 = res[0][0].get_position();
        assert!(el0.q == 0 && el0.r == 0);
        let elem1: Vec<TilePos> = res[1].iter().map(|e|e.get_position()).collect();
        assert_eq!(elem1.len(), 2);
        assert!(elem1.iter().find(|e|e.q== 1&& e.r == 0).is_some());
        assert!(elem1.iter().find(|e|e.q== 0&& e.r == 1).is_some());
        let elem2: Vec<TilePos> = res[2].iter().map(|e|e.get_position()).collect();
        assert_eq!(elem2.len(), 4);
        assert!(elem2.iter().find(|e|e.q== 2&& e.r == 0).is_some());
        assert!(elem2.iter().find(|e|e.q== 1&& e.r == 1).is_some());
        assert!(elem2.iter().find(|e|e.q== 0&& e.r == 2).is_some());
        assert!(elem2.iter().find(|e|e.q== -1&& e.r == 2).is_some());
    }

    #[test]
    pub fn test_get_tiles_in_range_together(){
        let map = prepare_simple_map();
        let res = map.get_tiles_in_range_together(&super::TilePos { q: 2, r: 4 },2);
        assert_eq!(res.len(), 19);
        let res = map.get_tiles_in_range_together(&super::TilePos { q: 0, r: 0 },2);
        assert_eq!(res.len(), 7);
    }

    #[test]
    pub fn test_check_if_exists_tile_with_field_content() {
        let map = prepare_simple_map();
        let res = map.check_if_exists_tile_with_field_content("village_small");
        assert!(res);
        let res = map.check_if_exists_tile_with_field_content("not_exist");
        assert!(!res);
    }

    #[test]
    pub fn test_check_if_exists_tile_with_field_path() {
        let map = prepare_simple_map();
        let res = map.check_if_exists_tile_with_field_path("town");
        assert!(res);
        let res = map.check_if_exists_tile_with_field_content("not_exist");
        assert!(!res);
    }

}