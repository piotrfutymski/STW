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
    if new_game_result.is_err() {
        println!("{:?}",new_game_result.err());
        panic!();
    }
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
    assert!(res.contains(&GameCallback::ChangedResource{ resource: GResource::Gold, new_value: 3}));
    assert!(res.contains(&GameCallback::NewTileContent{position: TilePos { q: 8, r: 18 }, field_type_id: "cottage".to_string()}));


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


    let possible_moves = game.get_possible_hero_moves();
    assert!(possible_moves.len() == 3);
    assert!(possible_moves.contains(&(TilePos{q:8, r:18}, "collect_info_village".to_string())));
    assert!(possible_moves.contains(&(TilePos{q:9, r:18}, "collect_info_village".to_string())));
    assert!(possible_moves.contains(&(TilePos{q:9, r:18}, "shopping_village".to_string())));

    let move_play = game.perform_move(&GameMove::PlayMove(TilePos{q:9, r:18}, "shopping_village".to_string()));

    assert!(move_play.expect("Cant move in test").iter().find(|e|match e {
        GameCallback::HeroMoved { .. } => true,
        _ => false
        }).is_some());

    game.get_possible_hero_moves();

    assert!(game.perform_move(&GameMove::PlayMove(TilePos{q:9, r:17}, "gathering_meadow".to_string())).is_ok());
    game.get_possible_hero_moves();

    assert!(game.perform_move(&GameMove::PlayMove(TilePos{q:8, r:17}, "gathering_meadow".to_string())).is_ok());

    let possible_moves = game.get_possible_hero_moves();
    assert!(possible_moves.len() == 0);

    let move_play = game.perform_move(&GameMove::PlayMove(TilePos{q:8, r:17}, "gathering_meadow".to_string()));
    assert!(move_play.is_err());

    game.print();

    // decision


}