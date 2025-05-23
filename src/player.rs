use bevy::prelude::*;

pub(super) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(setup_player_tag);

        app.register_type::<PlayerType>();
    }
}

/// Setup player tag: [`PlayerA`] and [`PlayerB`]
/// based on [`PlayerType`].
fn setup_player_tag(
    trigger: Trigger<OnAdd, PlayerType>,
    mut commands: Commands,
    q_players: Query<&PlayerType>,
) -> Result {
    let entity = trigger.target();

    let player_type = q_players.get(entity)?;

    match player_type {
        PlayerType::A => {
            commands.entity(entity).insert(PlayerA);
        }
        PlayerType::B => {
            commands.entity(entity).insert(PlayerB);
        }
    }

    Ok(())
}

// TODO: Rename these to the character's name!

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub enum PlayerType {
    A,
    B,
}

/// A unique component tag for player A.
#[derive(Component, Debug)]
pub struct PlayerA;

/// A unique component tag for player B.
#[derive(Component, Debug)]
pub struct PlayerB;
