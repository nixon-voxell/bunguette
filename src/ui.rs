use bevy::prelude::*;
use bevy::ui::FocusPolicy;

pub(super) struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui);
    }
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::VMin(4.0)),
            ..default()
        },
        FocusPolicy::Pass,
        Pickable::IGNORE,
        children![(
            Text::new("Some debug text."),
            FocusPolicy::Pass,
            Pickable::IGNORE,
        )],
    ));
}
