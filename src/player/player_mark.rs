use bevy::prelude::*;

use crate::ui::Screen;

pub(super) struct PlayerMarkPlugin;

impl Plugin for PlayerMarkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(Screen::EnterLevel),
            init_player_mark,
        )
        .add_systems(
            Update,
            game_over_condition.run_if(
                in_state(Screen::EnterLevel)
                    .and(resource_changed::<PlayerMark>),
            ),
        );
    }
}

/// Reset [`PlayerMark`] resource.
pub fn init_player_mark(mut commands: Commands) {
    commands.insert_resource(PlayerMark(10));
}

fn game_over_condition(
    player_mark: Res<PlayerMark>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if player_mark.0 == 0 {
        next_screen.set(Screen::GameOver);
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct PlayerMark(pub u32);
