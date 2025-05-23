use bevy::prelude::*;

pub(super) struct WorldSpaceUiPlugin;

impl Plugin for WorldSpaceUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            update_world_space_ui
                .after(TransformSystem::TransformPropagate),
        );
    }
}

fn update_world_space_ui(
    q_camera_transform: Query<
        (&GlobalTransform, &Camera),
        With<WorldSpaceUiCamera>,
    >,
    q_global_transforms: Query<
        &GlobalTransform,
        Without<WorldSpaceUiCamera>,
    >,
    mut q_world_space_uis: Query<(&WorldSpaceUi, &mut Node)>,
) {
    let Ok((camera_transform, camera)) = q_camera_transform.single()
    else {
        if q_camera_transform.iter().len() != 0 {
            warn!(
                "There is more than 1 camera with `WorldSpaceUiCamera` component attached to them!"
            );
        }

        // It's fine if there's no world space ui camera.
        return;
    };

    for (world_space_ui, mut node) in q_world_space_uis.iter_mut() {
        let Ok(target_transform) =
            q_global_transforms.get(world_space_ui.target)
        else {
            warn!(
                "Unable to find WorldSpaceUi target: {}",
                world_space_ui.target
            );
            continue;
        };

        match camera.world_to_viewport(
            camera_transform,
            target_transform.translation()
                + world_space_ui.world_offset,
        ) {
            Ok(viewport) => {
                let viewport = viewport + world_space_ui.ui_offest;
                node.top = Val::Px(viewport.y);
                node.left = Val::Px(viewport.x);
            }
            Err(err) => {
                warn!(
                    "Unable to get viewport location for target: {} ({err})",
                    world_space_ui.target
                );
            }
        }
    }

    // camera.world_to_viewport(camera_transform, world_position)
}

/// Component for ui nodes to be transformed into world space
/// based on the target entity's [`GlobalTransform`].
#[derive(Component)]
pub struct WorldSpaceUi {
    pub target: Entity,
    pub ui_offest: Vec2,
    pub world_offset: Vec3,
}

impl WorldSpaceUi {
    pub fn _new(target: Entity) -> Self {
        Self {
            target,
            ui_offest: Vec2::ZERO,
            world_offset: Vec3::ZERO,
        }
    }
}

/// A tag component for camera that will be used to render world space ui.
///
/// Should only be added to one camera!
#[derive(Component)]
pub struct WorldSpaceUiCamera;
