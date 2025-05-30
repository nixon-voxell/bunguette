use bevy::prelude::*;
use bevy::ui::UiSystem;

pub(super) struct WorldSpaceUiPlugin;

impl Plugin for WorldSpaceUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            update_world_ui
                .after(UiSystem::Layout)
                .after(TransformSystem::TransformPropagate),
        )
        .add_observer(cleanup_world_ui);
    }
}

fn update_world_ui(
    q_camera_transform: Query<(&GlobalTransform, &Camera)>,
    q_global_transforms: Query<&GlobalTransform, Without<Camera>>,
    mut q_world_space_uis: Query<(
        &WorldUi,
        &mut Node,
        &ComputedNode,
        &UiTargetCamera,
    )>,
) {
    for (world_ui, mut node, computed_node, target_camera) in
        q_world_space_uis.iter_mut()
    {
        let Ok((camera_transform, camera)) =
            q_camera_transform.get(target_camera.entity())
        else {
            warn!(
                "Unable to get WorldUi target camera: {target_camera:?}"
            );
            continue;
        };

        let Ok(target_transform) =
            q_global_transforms.get(world_ui.target)
        else {
            // Hide the node..
            node.display = Display::None;
            warn!(
                "Unable to find WorldSpaceUi target: {}",
                world_ui.target
            );
            continue;
        };

        node.display = Display::DEFAULT;

        let rect = camera.logical_viewport_rect().unwrap_or_default();

        match camera.world_to_viewport(
            camera_transform,
            target_transform.translation() + world_ui.world_offset,
        ) {
            Ok(viewport) => {
                let viewport =
                    viewport + world_ui.ui_offset - rect.min;
                let half_size = computed_node.size * 0.5;

                node.left = Val::Px(viewport.x - half_size.x);
                node.top = Val::Px(viewport.y - half_size.y);
            }
            Err(err) => {
                // Hide the node..
                node.display = Display::None;
                debug!(
                    "Unable to get viewport location for target: {} ({err})",
                    world_ui.target
                );
            }
        }
    }
}

fn cleanup_world_ui(
    trigger: Trigger<OnRemove, RelatedWorldUis>,
    mut commands: Commands,
    q_related_uis: Query<&RelatedWorldUis>,
) -> Result {
    let entity = trigger.target();

    let related_uis = q_related_uis.get(entity)?;

    for ui_entity in related_uis.iter() {
        commands.entity(ui_entity).despawn();
    }

    Ok(())
}

/// Attached to the target entity of [`WorldUi`]s.
#[derive(Component, Deref, Default, Debug)]
#[relationship_target(relationship = WorldUi)]
pub struct RelatedWorldUis(Vec<Entity>);

/// Component for ui nodes to be transformed into world space
/// based on the target entity's [`GlobalTransform`].
#[derive(Component)]
#[relationship(relationship_target = RelatedWorldUis)]
pub struct WorldUi {
    #[relationship]
    pub target: Entity,
    pub ui_offset: Vec2,
    pub world_offset: Vec3,
}

impl WorldUi {
    pub fn new(target: Entity) -> Self {
        Self {
            target,
            ui_offset: Vec2::ZERO,
            world_offset: Vec3::ZERO,
        }
    }

    #[allow(dead_code)]
    pub fn with_world_offset(mut self, offset: Vec3) -> Self {
        self.world_offset = offset;
        self
    }

    #[allow(dead_code)]
    pub fn with_ui_offset(mut self, offset: Vec2) -> Self {
        self.ui_offset = offset;
        self
    }
}
