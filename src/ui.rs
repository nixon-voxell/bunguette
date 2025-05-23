use bevy::color::palettes::tailwind::{TEAL_100, TEAL_300};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use widgets::*;

mod widgets;

pub(super) struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(widgets::WidgetsPlugin);

        app.add_systems(Startup, setup_menu);

        app.init_state::<Screen>();
    }
}

fn setup_menu(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::VMin(10.0)),
            justify_content: JustifyContent::End,
            align_items: AlignItems::End,
            ..default()
        },
        FocusPolicy::Pass,
        Pickable::IGNORE,
        Children::spawn(Spawn((
            Node {
                width: Val::VMin(40.0),
                height: Val::VMin(60.0),
                // padding: UiRect::all(Val::VMin())
                justify_content: JustifyContent::SpaceAround,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.2)),
            BorderRadius::all(Val::VMin(4.0)),
            Children::spawn(Spawn((
                Node {
                    width: Val::VMin(16.0),
                    height: Val::VMin(9.0),
                    ..default()
                },
                HoverBackground {
                    over: TEAL_100.into(),
                    out: TEAL_300.into(),
                },
                Button,
            ))),
        ))),
    ));
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum Screen {
    #[default]
    Menu,
    InputPairing,
    EnterLevel,
}
