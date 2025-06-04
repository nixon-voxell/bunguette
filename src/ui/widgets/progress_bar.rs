use bevy::prelude::*;

pub(super) struct ProgressBarPlugin;

impl Plugin for ProgressBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_progress_bar)
            .add_observer(setup_progress_bar);
    }
}

#[derive(Component)]
pub struct ProgressBar {
    pub color: Color,
    pub radius: BorderRadius,
    pub progress: f32,
}

impl ProgressBar {
    pub fn new(
        color: impl Into<Color>,
        radius: BorderRadius,
    ) -> Self {
        Self {
            color: color.into(),
            radius,
            progress: 0.0,
        }
    }

    #[allow(dead_code)]
    pub fn with_init_progress(mut self, progress: f32) -> Self {
        self.progress = progress;
        self
    }
}

fn setup_progress_bar(
    trigger: Trigger<OnAdd, ProgressBar>,
    mut commands: Commands,
    q_progress_bars: Query<&ProgressBar>,
) -> Result {
    let entity = trigger.target();

    let progress_bar = q_progress_bars.get(entity)?;

    let foreground = commands
        .spawn((
            Node {
                width: Val::Percent(progress_bar.progress * 100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            progress_bar.radius,
            BackgroundColor(progress_bar.color),
        ))
        .id();

    commands
        .entity(entity)
        .insert(ProgressBarForeground(foreground))
        .add_child(foreground);

    Ok(())
}

fn update_progress_bar(
    q_progress_bars: Query<
        (&ProgressBar, &ProgressBarForeground),
        Changed<ProgressBar>,
    >,
    mut q_nodes: Query<(&mut Node, &mut BackgroundColor)>,
) {
    for (progress_bar, foreground) in q_progress_bars.iter() {
        let Ok((mut node, mut background)) =
            q_nodes.get_mut(foreground.entity())
        else {
            continue;
        };

        node.width = Val::Percent(progress_bar.progress * 100.0);
        background.set_if_neq(BackgroundColor(progress_bar.color));
    }
}

/// The foreground entity that actually displays
/// the progress.
#[derive(Component, Deref)]
pub struct ProgressBarForeground(Entity);
