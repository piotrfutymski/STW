use super::map::TilePos;

pub enum GameMove{
    Build(TilePos, String),
    Wait,
    StartHistory(TilePos, usize),
    PlayMove(TilePos, String),
    MakeDecision(String),
    RenameHero(usize, String)
}

#[derive(Debug)]
pub struct PossibleBuilding{
    pub id: String,
    pub not_enought_resources: bool
}