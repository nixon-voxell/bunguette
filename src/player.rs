use bevy::color::palettes::tailwind::*;
use bevy::prelude::*;

use crate::ui::world_space::WorldUi;

pub(super) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(setup_name_ui_for_player)
            .add_observer(setup_player_tag);

        app.register_type::<PlayerType>();
    }
}

/// Setup world space name ui for players.
fn setup_name_ui_for_player(
    trigger: Trigger<OnAdd, PlayerType>,
    mut commands: Commands,
    q_players: Query<&PlayerType>,
) -> Result {
    let entity = trigger.target();

    let player_type = q_players.get(entity)?;

    let world_ui =
        WorldUi::new(entity).with_world_offset(Vec3::Y * 0.5);
    let ui_bundle = move |name: &str| {
        (
            world_ui,
            Node {
                padding: UiRect::all(Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(ZINC_900.with_alpha(0.5).into()),
            BoxShadow::new(
                ZINC_900.into(),
                Val::Px(4.0),
                Val::Px(4.0),
                Val::Px(14.0),
                Val::Px(12.0),
            ),
            Children::spawn(Spawn((
                Text::new(name),
                TextLayout::new_with_justify(JustifyText::Center),
            ))),
        )
    };

    match player_type {
        PlayerType::A => {
            commands.spawn(ui_bundle("Player A"));
        }
        PlayerType::B => {
            commands.spawn(ui_bundle("Player B"));
        }
    }

    Ok(())
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
