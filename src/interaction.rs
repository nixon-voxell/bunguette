use avian3d::prelude::*;
use bevy::color::palettes::tailwind::{EMERALD_500, SKY_300};
use bevy::prelude::*;
use bevy_mod_outline::{
    InheritOutline, OutlineMode, OutlineStencil, OutlineVolume,
};

use crate::physics::GameLayer;

const MARK_COLOR: Color = Color::Srgba(SKY_300);
const GRABBED_COLOR: Color = Color::Srgba(EMERALD_500);

pub(super) struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_mod_outline::OutlinePlugin);

        app.add_systems(Startup, spawn_test_scene)
            .add_observer(setup_interactable_outline);
        // .add_observer(setup_interaction_player);

        app.register_type::<Interactable>()
            .register_type::<InteractionPlayer>();
    }
}

fn spawn_test_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(SceneRoot(
        asset_server.load(
            GltfAssetLabel::Scene(0)
                .from_asset("scenes/interaction_test.glb"),
        ),
    ));
}

fn detect_interactables(
    mut q_players: Query<(
        &InteractionPlayer,
        &mut MarkedItem,
        Entity,
    )>,
    q_global_transforms: Query<&GlobalTransform>,
    spatial_query: SpatialQuery,
) -> Result {
    for (player, mut marked_item, entity) in q_players.iter_mut() {
        let player_transform = q_global_transforms.get(entity)?;

        let item_entities = spatial_query.shape_intersections(
            &Collider::sphere(0.5),
            player_transform.translation(),
            Quat::IDENTITY,
            &SpatialQueryFilter::from_mask(GameLayer::Interactable),
        );

        let player_forward = player_transform.forward();
        let front =
            Vec2::new(player_forward.x, player_forward.z).normalize();

        let closest_entity = 0;
        let closest_dist = f32::MAX;

        for item_entity in item_entities {
            let Ok(item_translation) =
                q_global_transforms.get(item_entity)
            else {
                continue;
            };
        }
    }

    Ok(())
}

fn clear_prev_marked_item() {}

fn setup_interactable_outline(
    trigger: Trigger<OnAdd, Interactable>,
    mut commands: Commands,
    q_meshes: Query<(), With<Mesh3d>>,
    q_children: Query<&Children>,
) {
    let entity = trigger.target();

    const VOLUME: OutlineVolume = OutlineVolume {
        width: 2.0,
        visible: false,
        colour: MARK_COLOR,
    };

    commands.entity(entity).insert(CollisionLayers::new(
        GameLayer::Interactable,
        LayerMask::ALL,
    ));

    if q_meshes.contains(entity) {
        commands
            .entity(entity)
            .insert((VOLUME, OutlineMode::FloodFlat));
    } else {
        commands.entity(entity).insert((
            VOLUME,
            OutlineMode::FloodFlat,
            OutlineStencil::default(),
        ));

        for child in q_children.iter_descendants(entity) {
            if q_meshes.contains(child) {
                commands.entity(child).insert(InheritOutline);
            }
        }
    }
}

/// An entity that can be interacted.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Interactable;

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct PrevMarkedItem(pub Option<Entity>);

#[derive(Component, Default, Debug, Clone, Copy)]
#[require(PrevMarkedItem)]
pub struct MarkedItem(pub Option<Entity>);

/// Entity that can perform interaction.
/// Raycast will happen from this player.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(MarkedItem)]
pub struct InteractionPlayer {
    /// The interaction radius.
    pub range: f32,
    /// The interaction boundary, anything that is
    /// closer than this range will be considered and ranked
    /// based on their direction.
    pub boundary_range: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct TestBundle {
    pub transform: Transform,
    pub visibility: Visibility,
}
