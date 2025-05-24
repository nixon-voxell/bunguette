use bevy::prelude::*;
use serde::Deserialize;
use std::fs;

/// The root structure for the RON file containing all item definitions
#[derive(Debug, Clone, Deserialize, Resource)]
pub struct ItemList {
    pub items: Vec<ItemMeta>,
}

/// Metadata for each item type in the game - loaded from RON files
#[derive(Debug, Clone, Deserialize)]
pub struct ItemMeta {
    pub id: u32,
    pub name: String,
    pub icon_path: Option<String>,
    pub _description: Option<String>,
    pub stackable: bool,
    pub max_stack_size: u32,
}

/// A registry resource that holds all ItemMeta by their unique id
#[derive(Resource, Default)]
pub struct ItemRegistry {
    pub by_id: std::collections::HashMap<u32, ItemMeta>,
    pub icons: std::collections::HashMap<u32, Handle<Image>>,
    pub loaded: bool,
}

/// Plugin to handle item metadata loading and registry setup
pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ItemRegistry>()
            .add_systems(Startup, load_item_list)
            .add_systems(
                Startup,
                populate_registry.after(load_item_list),
            );
    }
}

/// Startup system: load "assets/items.ron" and insert as a resource
fn load_item_list(mut commands: Commands) {
    let ron_str = fs::read_to_string("assets/items.ron")
        .expect("Failed to read assets/items.ron");
    let item_list: ItemList =
        ron::from_str(&ron_str).expect("Failed to parse items.ron");
    commands.insert_resource(item_list);
    info!("Loaded items.ron into ItemList resource");
}

/// System to populate the ItemRegistry from the ItemList resource
fn populate_registry(
    mut registry: ResMut<ItemRegistry>,
    item_list: Res<ItemList>,
    asset_server: Res<AssetServer>,
) {
    if registry.loaded {
        return;
    }

    let mut new_by_id = std::collections::HashMap::new();
    let mut new_icons = std::collections::HashMap::new();

    for meta in item_list.items.iter() {
        if new_by_id.contains_key(&meta.id) {
            error!("Duplicate item ID {} found in ItemList", meta.id);
            continue;
        }

        if let Some(icon_path) = &meta.icon_path {
            let icon_handle: Handle<Image> =
                asset_server.load(icon_path);
            new_icons.insert(meta.id, icon_handle);
        }

        new_by_id.insert(meta.id, meta.clone());
    }

    registry.by_id = new_by_id;
    registry.icons = new_icons;
    registry.loaded = true;

    info!(
        "Populated ItemRegistry with {} items",
        item_list.items.len()
    );
}
