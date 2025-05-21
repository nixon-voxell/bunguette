use avian3d::prelude::*;
use bevy::prelude::*;

pub(super) struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CollisionLayerConstructor>();
    }
}

fn setup_game_layer(
    trigger: Trigger<OnAdd, CollisionLayerConstructor>,
    mut commands: Commands,
) {
    // commands.entity(trigger.target()).insert(CollisionLayers::new(, filters))
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component, Default)]
pub struct CollisionLayerConstructor {
    pub memberships: Vec<GameLayer>,
    pub filters: Vec<GameLayer>,
}

#[derive(PhysicsLayer, Reflect, Default, Debug)]
#[reflect(Default)]
pub enum GameLayer {
    #[default]
    Default,
    Player,
    Enemy,
    Interactable,
}
