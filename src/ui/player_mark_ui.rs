use bevy::color::palettes::tailwind::*;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::camera_controller::UI_RENDER_LAYER;
use crate::player::player_mark::{PlayerMark, init_player_mark};

use super::Screen;

pub(super) struct PlayerMarkUiPlugin;

impl Plugin for PlayerMarkUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(Screen::EnterLevel),
            spawn_player_mark_ui.after(init_player_mark),
        )
        .add_systems(
            Update,
            update_player_mark_ui.run_if(
                in_state(Screen::EnterLevel)
                    .and(resource_changed::<PlayerMark>),
            ),
        );
    }
}

/// Spawn the player mark UI element
fn spawn_player_mark_ui(
    mut commands: Commands,
    player_mark: Res<PlayerMark>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        UI_RENDER_LAYER,
        StateScoped(Screen::EnterLevel),
        // Root.
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(20.0)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
        FocusPolicy::Pass,
        Children::spawn(Spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            Pickable::IGNORE,
            FocusPolicy::Pass,
            BackgroundColor(ZINC_900.with_alpha(0.4).into()),
            BoxShadow::new(
                ZINC_900.into(),
                Val::ZERO,
                Val::ZERO,
                Val::Px(4.0),
                Val::Px(8.0),
            ),
            BorderRadius::all(Val::Px(8.0)),
            Children::spawn((
                Spawn((
                    Node {
                        width: Val::Px(30.0),
                        height: Val::Px(30.0),
                        margin: UiRect::right(Val::Px(20.0)),
                        ..default()
                    },
                    ImageNode::new(
                        asset_server.load("icons/heart.png"),
                    ),
                )),
                Spawn((
                    Text::new(player_mark.to_string()),
                    PlayerMarkUiText,
                )),
            )),
        ))),
    ));
}

fn update_player_mark_ui(
    player_mark: Res<PlayerMark>,
    mut q_text: Query<&mut Text, With<PlayerMarkUiText>>,
) -> Result {
    q_text.single_mut()?.0 = player_mark.to_string();

    Ok(())
}

#[derive(Component)]
pub struct PlayerMarkUiText;
