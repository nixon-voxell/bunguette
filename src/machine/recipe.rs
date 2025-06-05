use crate::asset_pipeline::PrefabName;
use crate::inventory::item::{ItemRegistry, ItemType};
use bevy::asset::{AssetLoader, io::Reader};
use bevy::asset::{AsyncReadExt, LoadContext};
use bevy::ecs::system::SystemParam;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use serde::Deserialize;

/// Plugin to handle recipe metadata loading and registry setup
pub(super) struct RecipePlugin;

impl Plugin for RecipePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<RecipeMetaAsset>()
            .init_asset_loader::<RecipeMetaAssetLoader>();

        app.add_systems(PreStartup, load_recipe_registry)
            .add_systems(Update, validate_recipes_against_items);
    }
}

/// Startup system: load "machines.recipe_meta.ron" and insert as a resource
fn load_recipe_registry(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(RecipeMetaAssetHandle(
        asset_server.load("machines.recipe_meta.ron"),
    ));
}

/// System to validate that all recipe ingredients and outputs exist in the item registry
fn validate_recipes_against_items(
    recipe_registry: RecipeRegistry,
    item_registry: ItemRegistry,
    mut validation_done: Local<bool>,
) {
    // Only run validation once, and only after both registries are loaded
    if *validation_done {
        return;
    }

    let Some(recipes) = recipe_registry.get() else {
        return;
    };

    let Some(items) = item_registry.get() else {
        return;
    };

    // Mark validation as done so we don't run it again
    *validation_done = true;

    info!("Validating recipes against item registry...");

    for (recipe_id, recipe) in recipes.iter() {
        // Validate output item exists and is correct type
        if let Some(output_item) = items.get(&recipe.output_id) {
            match output_item.item_type {
                ItemType::Tower => {
                    // Recipes should produce towers, no warning needed
                }
                ItemType::Ingredient => {
                    warn!(
                        "Recipe '{}' produces ingredient '{}' - consider if this is intended",
                        recipe_id, recipe.output_id
                    );
                }
            }
        } else {
            error!(
                "Recipe '{}' output '{}' not found in item registry!",
                recipe_id, recipe.output_id
            );
        }

        // Validate all ingredient items exist and are ingredients
        for ingredient in &recipe.ingredients {
            if let Some(item_meta) = items.get(&ingredient.item_id) {
                if item_meta.item_type != ItemType::Ingredient {
                    warn!(
                        "Recipe '{}' uses '{}' as ingredient, but it's marked as {:?} in item registry",
                        recipe_id,
                        ingredient.item_id,
                        item_meta.item_type
                    );
                }
            } else {
                error!(
                    "Recipe '{}' ingredient '{}' not found in item registry!",
                    recipe_id, ingredient.item_id
                );
            }
        }
    }

    info!("Recipe validation completed!");
}

#[derive(Asset, TypePath, Deref, Debug, Clone, Deserialize)]
pub struct RecipeMetaAsset(HashMap<String, RecipeMeta>);

/// Recipe metadata loaded from RON files
#[derive(Debug, Clone, Deserialize)]
pub struct RecipeMeta {
    pub ingredients: Vec<RecipeIngredient>,
    pub output_id: String,
    pub output_quantity: u32,
    pub cooking_duration: f32,
    prefab_name: String,
}

impl RecipeMeta {
    pub fn prefab_name(&self) -> PrefabName {
        PrefabName::FileName(&self.prefab_name)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecipeIngredient {
    pub item_id: String,
    pub quantity: u32,
}

#[derive(Resource)]
pub struct RecipeMetaAssetHandle(Handle<RecipeMetaAsset>);

#[derive(SystemParam)]
pub struct RecipeRegistry<'w> {
    pub handle: Res<'w, RecipeMetaAssetHandle>,
    pub assets: Res<'w, Assets<RecipeMetaAsset>>,
}

impl RecipeRegistry<'_> {
    pub fn get(&self) -> Option<&RecipeMetaAsset> {
        self.assets.get(&self.handle.0)
    }

    pub fn get_recipe(&self, recipe_id: &str) -> Option<&RecipeMeta> {
        self.get()?.get(recipe_id)
    }
}

#[derive(Default)]
pub struct RecipeMetaAssetLoader;

impl AssetLoader for RecipeMetaAssetLoader {
    type Asset = RecipeMetaAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut ron_str = String::new();
        reader.read_to_string(&mut ron_str).await?;

        let asset = ron::from_str::<RecipeMetaAsset>(&ron_str)
            .expect("Failed to parse recipes.recipe_meta.ron");

        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["recipe_meta.ron"]
    }
}
