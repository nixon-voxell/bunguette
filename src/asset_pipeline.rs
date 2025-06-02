use animation_pipeline::AnimationGraphMap;
use bevy::ecs::system::SystemParam;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub mod animation_pipeline;

pub(super) struct AssetPipelinePlugin;

impl Plugin for AssetPipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(animation_pipeline::AnimationPipelinePlugin);

        app.init_state::<AssetState>()
            .init_resource::<CurrentScene>()
            .add_loading_state(
                LoadingState::new(AssetState::LoadingGltf)
                    .continue_to_state(AssetState::LoadingAnimation)
                    .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                        "dynamic_asset.assets.ron",
                    )
                    .load_collection::<PrefabAssets>()
                    .load_collection::<SceneAssets>(),
            )
            .add_systems(
                OnEnter(AssetState::Loaded),
                load_default_scene,
            );

        #[cfg(feature = "dev")]
        app.register_type::<SceneAssets>()
            .register_type::<PrefabAssets>();
    }
}

fn load_default_scene(mut scenes: SceneAssetsLoader) -> Result {
    scenes.load_default_scene()
}

#[derive(SystemParam)]
pub struct SceneAssetsLoader<'w, 's> {
    commands: Commands<'w, 's>,
    scenes: Res<'w, SceneAssets>,
    gltfs: Res<'w, Assets<Gltf>>,
    current_scene: ResMut<'w, CurrentScene>,
}

impl SceneAssetsLoader<'_, '_> {
    pub fn load_default_scene(&mut self) -> Result {
        let gltf = self
            .gltfs
            .get(&self.scenes.default_scene)
            .ok_or("Scene should have been loaded")?;

        self.load_scene(
            gltf.default_scene
                .clone()
                .expect("Should have a default scene."),
        );

        Ok(())
    }

    pub fn load_level1(&mut self) -> Result {
        let gltf = self
            .gltfs
            .get(&self.scenes.level1)
            .ok_or("Scene should have been loaded")?;

        self.load_scene(
            gltf.default_scene
                .clone()
                .expect("Should have a default scene."),
        );

        Ok(())
    }

    /// Despawn the last scene and spawns a new scene,
    /// overwritting the [`CurrentScene`].
    fn load_scene(&mut self, scene: Handle<Scene>) {
        if let Some(last_scene) = self.current_scene.get() {
            self.commands.entity(last_scene).despawn();
        }

        let id = self.commands.spawn(SceneRoot(scene)).id();

        self.current_scene.0 = Some(id);
    }
}

#[derive(AssetCollection, Resource, Debug)]
#[cfg_attr(feature = "dev", derive(Reflect))]
#[cfg_attr(feature = "dev", reflect(Resource))]
pub struct SceneAssets {
    #[asset(key = "scenes.default")]
    default_scene: Handle<Gltf>,
    #[asset(key = "scenes.level1")]
    level1: Handle<Gltf>,
}

#[derive(AssetCollection, Resource, Debug)]
#[cfg_attr(feature = "dev", derive(Reflect))]
#[cfg_attr(feature = "dev", reflect(Resource))]
pub struct PrefabAssets {
    #[asset(key = "prefabs", collection(typed, mapped))]
    pub named_prefabs: HashMap<String, Handle<Gltf>>,
    pub named_animations: HashMap<String, AnimationGraphMap>,
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

    pub fn get_animation(
        &self,
        name: PrefabName,
    ) -> Option<&AnimationGraphMap> {
        self.named_animations.get(&name.cast())
    }
}

#[derive(Debug)]
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

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum AssetState {
    #[default]
    LoadingGltf,
    LoadingAnimation,
    Loaded,
}

/// The current loaded scene instance.
#[derive(Resource, Deref, Default, Debug)]
pub struct CurrentScene(Option<Entity>);

impl CurrentScene {
    pub fn get(&self) -> Option<Entity> {
        self.0
    }
}
