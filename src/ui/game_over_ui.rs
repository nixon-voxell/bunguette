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
    const FONT_SIZE: f32 = 40.0;

    let bg_color = Srgba::hex("BFB190").unwrap().with_alpha(0.4);
    let red_color = Srgba::hex("FF5757").unwrap();
    let green_color = Srgba::hex("C1FF72").unwrap();
    let font_color = Srgba::hex("342C24").unwrap();

    let win = player_mark.0 > 0;

    commands.spawn((
        UI_RENDER_LAYER,
        StateScoped(Screen::GameOver),
        // Root.
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(40.0)),
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
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            Pickable::IGNORE,
            FocusPolicy::Pass,
            BackgroundColor(bg_color.into()),
            BorderRadius::all(Val::Px(40.0)),
            Children::spawn((
                Spawn((
                    Node {
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    if win {
                        Text::new("Victory!")
                    } else {
                        Text::new("Failed!")
                    },
                    TextColor(font_color.into()),
                    TextLayout::new_with_justify(JustifyText::Center),
                    TextFont::from_font_size(FONT_SIZE * 1.5),
                )),
                SpawnWith(move |parent: &mut ChildSpawner| {
                    parent
                        .spawn(
                            LabelButton::new(if win {
                                "Continue"
                            } else {
                                "Retry"
                            })
                            .with_background(ButtonBackground::new(
                                if win {
                                    green_color
                                } else {
                                    red_color
                                }
                                .with_alpha(0.45),
                            ))
                            .with_text_color(font_color)
                            .with_font_size(FONT_SIZE)
                            .build(),
                        )
                        .observe(return_to_main_menu);
                }),
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
