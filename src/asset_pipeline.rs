use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub(super) struct AssetPipeline;

impl Plugin for AssetPipeline {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_loading_state(
                LoadingState::new(GameState::Loading)
                    .continue_to_state(GameState::Next)
                    .load_collection::<SceneAssets>(),
            )
            .add_systems(OnEnter(GameState::Next), setup_animations);
    }
}

#[derive(AssetCollection, Resource)]
pub struct SceneAssets {
    #[asset(path = "scenes/animation_test.glb")]
    pub animation_test: Handle<Gltf>,
}

fn setup_animations(
    scenes: Res<SceneAssets>,
    gltfs: Res<Assets<Gltf>>,
) -> Result {
    let gltf = gltfs
        .get(&scenes.animation_test)
        .ok_or("Scene should have been loaded.")?;

    info!("{:?}", gltf.named_animations);

    Ok(())
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Next,
}
