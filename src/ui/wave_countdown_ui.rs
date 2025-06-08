use bevy::color::palettes::css::WHITE;
use bevy::color::palettes::tailwind::*;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::camera_controller::UI_RENDER_LAYER;
use crate::enemy::spawner::{SpawnWave, WaveCountdown};
use crate::ui::Screen;

pub(super) struct WaveCountdownUiPlugin;

impl Plugin for WaveCountdownUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(Screen::EnterLevel),
            spawn_wave_countdown_ui,
        )
        .add_systems(
            Update,
            update_wave_countdown_ui
                .run_if(in_state(Screen::EnterLevel))
                .run_if(
                    resource_changed::<WaveCountdown>
                        .or(state_changed::<SpawnWave>),
                ),
        );
    }
}

/// Spawn the wave countdown UI element
fn spawn_wave_countdown_ui(mut commands: Commands) {
    commands.spawn((
        UI_RENDER_LAYER,
        StateScoped(Screen::EnterLevel),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(20.0)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::End,
            ..default()
        },
        Pickable::IGNORE,
        FocusPolicy::Pass,
        Children::spawn(Spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_self: AlignSelf::End,
                justify_self: JustifySelf::End,
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
            Children::spawn(Spawn((
                Text::new("Wave 1 - 00:00"),
                TextFont::from_font_size(24.0),
                TextColor(WHITE.into()),
                WaveCountdownText,
            ))),
        ))),
    ));
}

fn update_wave_countdown_ui(
    countdown: Res<WaveCountdown>,
    current_wave: Res<State<SpawnWave>>,
    mut q_text: Query<
        (&mut Text, &mut TextColor),
        With<WaveCountdownText>,
    >,
) {
    let Ok((mut text, mut text_color)) = q_text.single_mut() else {
        return;
    };

    let wave_name = match current_wave.get() {
        SpawnWave::None => "Waiting",
        SpawnWave::One => "Wave 1",
        SpawnWave::Two => "Wave 2",
        SpawnWave::Three => "Wave 3",
    };

    let remaining = countdown.duration() - countdown.elapsed();
    let remaining_seconds = remaining.as_secs_f32().max(0.0);

    if remaining_seconds <= 0.0 {
        // When countdown finished, just show wave name
        **text = wave_name.to_string();
        text_color.0 = RED_400.into();
    } else {
        // Show countdown timer
        let seconds = remaining_seconds as u32;
        let minutes = seconds / 60;
        let seconds = seconds % 60;
        **text =
            format!("{} - {:02}:{:02}", wave_name, minutes, seconds);

        text_color.0 = if remaining_seconds <= 5.0 {
            RED_400.into()
        } else if remaining_seconds <= 10.0 {
            YELLOW_400.into()
        } else {
            WHITE.into()
        };
    }
}

#[derive(Component)]
pub struct WaveCountdownText;
