use bevy::prelude::*;

pub(super) struct GrabPlugin;

impl Plugin for GrabPlugin {
    fn build(&self, app: &mut App) {
        //
    }
}

fn grab_movement(
    evr_cursor: EventReader<CursorMoved>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut q_grabs: Query<&mut Transform, With<Grabbed>>,
) {
}

/// Tag component for items that are being grabbed.
#[derive(Component)]
pub struct Grabbed;
