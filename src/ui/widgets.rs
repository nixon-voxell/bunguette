use bevy::prelude::*;

pub mod button;
pub mod progress_bar;

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            button::ButtonPlugin,
            progress_bar::ProgressBarPlugin,
        ));
    }
}
