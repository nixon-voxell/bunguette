use bevy::asset::{AssetLoader, io::Reader};
use bevy::asset::{AsyncReadExt, LoadContext};
use bevy::ecs::system::SystemParam;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use serde::Deserialize;

/// Plugin to handle item metadata loading and registry setup
pub(super) struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ItemMetaAsset>()
            .init_asset_loader::<ItemMetaAssetLoader>();

        app.add_systems(PreStartup, load_item_registry);
    }
}

/// Startup system: load "items.item_meta.ron" and insert as a resource.
fn load_item_registry(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(ItemMetaAssetHandle(
        asset_server.load("items.item_meta.ron"),
    ));
}

#[derive(Asset, TypePath, Deref, Debug, Clone, Deserialize)]
pub struct ItemMetaAsset(HashMap<String, ItemMeta>);

/// Metadata for each item type in the game - loaded from RON files.
#[derive(Debug, Clone, Deserialize)]
pub struct ItemMeta {
    pub name: String,
    pub icon_path: Option<String>,
    pub _description: Option<String>,
    pub stackable: bool,
    pub max_stack_size: u32,

    #[serde(skip_serializing, skip_deserializing)]
    pub icon: Option<Handle<Image>>,
}

#[derive(Resource)]
pub struct ItemMetaAssetHandle(Handle<ItemMetaAsset>);

#[derive(SystemParam)]
pub struct ItemRegistry<'w> {
    pub handle: Res<'w, ItemMetaAssetHandle>,
    pub assets: Res<'w, Assets<ItemMetaAsset>>,
}

impl ItemRegistry<'_> {
    pub fn get(&self) -> Option<&ItemMetaAsset> {
        self.assets.get(&self.handle.0)
    }
}

#[derive(Default)]
pub struct ItemMetaAssetLoader;

impl AssetLoader for ItemMetaAssetLoader {
    type Asset = ItemMetaAsset;

    type Settings = ();

    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut ron_str = String::new();
        reader.read_to_string(&mut ron_str).await?;

        let mut asset = ron::from_str::<ItemMetaAsset>(&ron_str)
            .expect("Failed to parse items.ron");

        for item_meta in asset.0.values_mut() {
            if let Some(icon_path) = item_meta.icon_path.as_ref() {
                item_meta.icon = Some(load_context.load(icon_path));
            }
        }

        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["item_meta.ron"]
    }
}
