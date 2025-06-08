use bevy::color::palettes::tailwind::*;
use bevy::ecs::spawn::SpawnWith;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::camera_controller::UI_RENDER_LAYER;
use crate::player::player_mark::PlayerMark;

use super::Screen;
use super::widgets::button::{ButtonBackground, LabelButton};

pub(super) struct GameOverUiPlugin;

impl Plugin for GameOverUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(Screen::GameOver),
            spawn_game_over_ui,
        );
    }
}

fn spawn_game_over_ui(
    mut commands: Commands,
    player_mark: Res<PlayerMark>,
) {
    commands.spawn((
        UI_RENDER_LAYER,
        StateScoped(Screen::GameOver),
        // Root.
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::VMin(10.0)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
        FocusPolicy::Pass,
        Children::spawn(Spawn((
            Node {
                flex_direction: FlexDirection::Column,
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
            BorderRadius::all(Val::Px(8.0)),
            Children::spawn((
                Spawn((
                    Node {
                        padding: UiRect::all(Val::Px(80.0)),
                        ..default()
                    },
                    if player_mark.0 > 0 {
                        (
                            Text::new("Congrats, you win!"),
                            TextColor(GREEN_400.into()),
                        )
                    } else {
                        (
                            Text::new("Lose..."),
                            TextColor(RED_400.into()),
                        )
                    },
                    TextLayout::new_with_justify(JustifyText::Center),
                    TextFont::from_font_size(64.0),
                    TextShadow::default(),
                )),
                Spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    Children::spawn(SpawnWith(
                        |parent: &mut ChildSpawner| {
                            parent
                                .spawn(
                                    LabelButton::new(
                                        "Return to menu...",
                                    )
                                    .with_background(
                                        ButtonBackground::new(
                                            ORANGE_600
                                                .with_alpha(0.5),
                                        ),
                                    )
                                    .build(),
                                )
                                .observe(return_to_main_menu);
                        },
                    )),
                )),
            )),
        ))),
    ));
}

fn return_to_main_menu(
    _: Trigger<Pointer<Click>>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    next_screen.set(Screen::Menu)
}
