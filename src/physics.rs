use avian3d::prelude::*;

#[derive(PhysicsLayer, Default)]
pub enum GameLayer {
    #[default]
    Default,
    Player,
    Enemy,
    Interactable,
}
