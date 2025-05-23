use bevy::prelude::*;

pub(super) struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(setup_hover_background);
    }
}

fn setup_hover_background(
    trigger: Trigger<OnAdd, HoverBackground>,
    mut commands: Commands,
    q_backgrounds: Query<&HoverBackground>,
) -> Result {
    let entity = trigger.target();

    let background = q_backgrounds.get(entity)?;

    commands
        .entity(entity)
        .insert(BackgroundColor(background.out))
        .observe(over_hover_background)
        .observe(out_hover_background);

    Ok(())
}

fn over_hover_background(
    trigger: Trigger<Pointer<Over>>,
    mut commands: Commands,
    q_backgrounds: Query<&HoverBackground>,
) -> Result {
    let entity = trigger.target();

    let background = q_backgrounds.get(entity)?;

    commands
        .entity(entity)
        .insert(BackgroundColor(background.over));

    Ok(())
}

fn out_hover_background(
    trigger: Trigger<Pointer<Out>>,
    mut commands: Commands,
    q_backgrounds: Query<&HoverBackground>,
) -> Result {
    let entity = trigger.target();

    let background = q_backgrounds.get(entity)?;

    commands
        .entity(entity)
        .insert(BackgroundColor(background.out));

    Ok(())
}

#[derive(Component, Default)]
pub struct HoverBackground {
    pub over: Color,
    pub out: Color,
}

// impl HoverBackground {
//     pub fn with_over(mut self, over: Color) -> Self {
//         self.over = over;
//         self
//     }

//     pub fn with_out(mut self, out: Color) -> Self {
//         self.out = out;
//         self
//     }
// }
