mod common;
use stw_game_lib::{self, game::{map::TilePos, game_move::GameMove, game_controller::GameCallback}, resource::enums::GResource};
use crate::common::load_resources;

#[test]
fn game_test() {

    let resources = load_resources();

    //starting game

    let new_game_result = stw_game_lib::GameConfig::new()
        .set_map_size(37)
        .set_name("test game")
        .set_resources(resources)
        .set_seed("test seed")
        .start_game();
    
    assert!(new_game_result.is_ok());

    let mut game = new_game_result.unwrap();
    game.print();

    // getting possible buildings
    assert!(game.get_possible_building_at_position(&TilePos { q: 7, r: 18 }).iter().find(|e|e.id == "wilderness").is_some());
    assert!(game.get_possible_building_at_position(&TilePos { q: 8, r: 18 }).iter().find(|e|e.id == "cottage").is_some());
    assert_eq!(game.get_possible_building_at_position(&TilePos { q: 9, r: 18 })[0].id, "village");
    assert!(game.get_possible_building_at_position(&TilePos { q: 9, r: 18 })[0].not_enought_resources);
    assert_eq!(game.get_possible_building_at_position(&TilePos { q: 4, r: 18 }).len(), 0);


    // building

    let build_res_good = game.perform_move(&GameMove::Build(TilePos { q: 8, r: 18 }, "cottage".to_string()));
    println!("{:?}", build_res_good);

    assert!(build_res_good.is_ok());
    let res = build_res_good.unwrap();
    assert!(res.contains(&GameCallback::ChangedResource(GResource::Gold, 3)));
    assert!(res.contains(&GameCallback::NewTileContent(TilePos { q: 8, r: 18 }, "cottage".to_string())));


    let build_res_bad = game.perform_move(&GameMove::Build(TilePos { q: 9, r: 18 }, "village".to_string()));
    assert!(build_res_bad.is_err());

    game.print();

    // waiting

    let wait_res = game.perform_move(&GameMove::Wait);
    assert!(wait_res.unwrap().len() == 2);
    let wait_res = game.perform_move(&GameMove::Wait);
    assert!(wait_res.unwrap().len() == 0);
    game.print();

    // history

    let can_start_history_bad = game.can_start_history(&TilePos { q: 7, r: 18 });
    assert!(can_start_history_bad.is_err());
    let can_start_history_good = game.can_start_history(&TilePos { q: 8, r: 18 });
    assert!(can_start_history_good.is_ok());

    let start_history = game.perform_move(&GameMove::StartHistory(TilePos { q: 8, r: 18 }, 0));
    assert!(start_history.unwrap().len() == 1);

    let wait_res = game.perform_move(&GameMove::Wait);
    assert!(wait_res.is_err());
    game.print();

    
}