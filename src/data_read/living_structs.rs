use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
pub struct LivingData {
    pub all: Vec<Living>,
}

/// Used for regular living entities such as monsters, humans, good or bad, anything living that is
/// not static. Dynamic components like position can be added at run time with functions.
#[derive(Deserialize, Debug)]
pub struct Living {
    pub name: String,
    pub sprite: Option<RawSprite>,
    pub view_range: Option<u32>,
    pub breed: Option<String>,
    pub ai: Option<String>,
    pub player: Option<String>,
    pub combatstats: Option<CombatStats>,
}

#[derive(Deserialize, Debug)]
pub struct RawSprite {
    pub glyph: char,
    pub fg: String,
    pub bg: String,
}

#[derive(Deserialize, Debug)]
pub struct CombatStats {
    pub hp: u32,
    pub str: i32,
    pub def: i32,
}
