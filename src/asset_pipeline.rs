use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub mod animation_pipeline;

pub(super) struct AssetPipelinePlugin;

impl Plugin for AssetPipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(animation_pipeline::AnimationPipelinePlugin);

        app.init_state::<PrefabState>()
            .add_loading_state(
                LoadingState::new(PrefabState::LoadingGltf)
                    .continue_to_state(PrefabState::LoadingAnimation)
                    .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                        "dynamic_asset.assets.ron",
                    )
                    .load_collection::<PrefabAssets>(),
            ).add_systems(OnEnter(PrefabState::LoadingAnimation), test);

        #[cfg(feature = "dev")]
        app.register_type::<PrefabAssets>();
    }
}

fn test(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(SceneRoot(
        asset_server.load(
            GltfAssetLabel::Scene(0)
                .from_asset("scenes/animation_test.glb"),
        ),
    ));
}

#[derive(AssetCollection, Resource, Debug)]
#[cfg_attr(feature = "dev", derive(Reflect))]
#[cfg_attr(feature = "dev", reflect(Resource))]
pub struct PrefabAssets {
    #[asset(key = "prefabs", collection(typed, mapped))]
    pub named_prefabs: HashMap<String, Handle<Gltf>>,
    pub named_graphs: HashMap<String, Handle<AnimationGraph>>,
}

impl PrefabAssets {
    pub fn get_gltf<'a>(
        &self,
        name: PrefabName,
        gltfs: &'a Assets<Gltf>,
    ) -> Option<&'a Gltf> {
        self.named_prefabs
            .get(&name.cast())
            .and_then(|handle| gltfs.get(handle))
    }
}

pub enum PrefabName<'a> {
    Absolute(&'a str),
    FileName(&'a str),
}

impl PrefabName<'_> {
    pub fn cast(self) -> String {
        match self {
            PrefabName::Absolute(name) => name.to_string(),
            PrefabName::FileName(filename) => {
                let prefix = "prefabs/".to_string();
                prefix + filename + ".glb"
            }
        }
    }
}

// fn test(
//     mut commands: Commands,
//     prefabs: Res<PrefabAssets>,
//     gltfs: Res<Assets<Gltf>>,
// ) -> Result {
//     let rotisserie =
//         gltfs.get(&prefabs.rotisserie).ok_or("Should be loaded.")?;

//     commands.spawn((
//         SceneRoot(rotisserie.default_scene.as_ref().unwrap().clone()),
//         IsAnimatable,
//     ));

//     let open_animation =
//         rotisserie.named_animations.get("Door|Open").unwrap();

//     Ok(())
// }

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum PrefabState {
    #[default]
    LoadingGltf,
    LoadingAnimation,
    Loaded,
}
