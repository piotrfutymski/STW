use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;


#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)] 
pub enum FieldCharacteristic{
    Habited,
    Mysterious
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum FieldInstantEffect{
    ShowFields{only_first_time: bool, amount: u32},
    GiveResources{only_first_time: bool, resource: GResource, amount: u32},
    IncreaseSkill{only_first_time: bool, skill: HeroSkill, amount: f32},
    LearnAboutAction{only_first_time: bool},
    IncreaseMaxHeroes{only_first_time: bool}
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)] 
pub enum GResource {
    Story,
    Gold,
    GreenTrophy,
    RedTrophy,
    BlueTrophy,
    WhiteTrophy,
    RareTrophy,
    EpicTrophy,
    LegendTrophy
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)] 
pub enum HeroSkill {
    Intnteligence,
    Alechemy,
    Magic,

    Strength,
    MleeFight,
    Charisma,

    Dextrity,
    Tracking,
    DistanceFight
    
}

#[derive(Serialize, Deserialize, Debug, EnumIter)] 
pub enum PointRequirment {
   DidAction(String),
   DidActionFamily(String),
   HeroFromFamliy(String)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum FieldRequirment {
    CantBuild,
    Replaces(String),
    QuestCompleted,

    NearFieldWithCharacteristic{ characteristic: FieldCharacteristic, distance: u32 },
    NearFieldWithPath{ build_path: String, distance: u32 },
    NearFieldWithId{ id: String, distance: u32 },
    NoNearFieldWithCharacteristic{ characteristic: FieldCharacteristic, distance: u32 },
    NoNearFieldWithPath{ build_path: String, distance: u32 },
    NoNearFieldWithId{ id: String, distance: u32 },
    HasOrigin(String),
    HasOriginOneOf(Vec<String>)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum QuestPenalty {
    LooseResources{resource: GResource, amount: u32},
    DestroyRandomNotMain,
    DestroyTileWithPath(String),
    DestroyQuestTile,
    KillHero(f32)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ActionRequirment {
    IsBefore(String),
    IsBeforeFamily(String),
    IsBeforePosition(usize),
}

