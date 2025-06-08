use bevy::color::palettes::tailwind::*;
use bevy::prelude::*;

pub(super) struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(setup_hover_background);
    }
}

#[derive(Default)]
pub struct LabelButton {
    pub background: ButtonBackground,
    pub text_color: Color,
    pub font_size: f32,
    pub label: String,
    pub node: Node,
}

impl LabelButton {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            node: Node {
                padding: UiRect::axes(Val::Px(50.0), Val::Px(12.0)),
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            ..default()
        }
    }

    pub fn with_background(
        mut self,
        background: ButtonBackground,
    ) -> Self {
        self.background = background;
        self
    }

    pub fn with_text_color(
        mut self,
        text_color: impl Into<Color>,
    ) -> Self {
        self.text_color = text_color.into();
        self
    }

    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn build(self) -> impl Bundle {
        (
            self.node,
            self.background,
            BorderRadius::all(Val::Percent(100.0)),
            Children::spawn(Spawn((
                Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Children::spawn(Spawn((
                    Text::new(self.label),
                    TextLayout::new(
                        JustifyText::Center,
                        LineBreak::WordBoundary,
                    ),
                    TextColor(self.text_color),
                    TextFont::from_font_size(self.font_size),
                ))),
            ))),
        )
    }
}

fn setup_hover_background(
    trigger: Trigger<OnAdd, ButtonBackground>,
    mut commands: Commands,
    q_backgrounds: Query<&ButtonBackground>,
) -> Result {
    let entity = trigger.target();

    let background = q_backgrounds.get(entity)?;

    commands
        .entity(entity)
        .insert(BackgroundColor(background.out))
        .observe(over_btn_background)
        .observe(out_btn_background)
        .observe(pressed_btn_background)
        .observe(released_btn_background);

    Ok(())
}

fn over_btn_background(
    trigger: Trigger<Pointer<Over>>,
    mut commands: Commands,
    q_backgrounds: Query<&ButtonBackground>,
) -> Result {
    let entity = trigger.target();

    let background = q_backgrounds.get(entity)?;

    commands
        .entity(entity)
        .insert(BackgroundColor(background.over));

    Ok(())
}

fn out_btn_background(
    trigger: Trigger<Pointer<Out>>,
    mut commands: Commands,
    q_backgrounds: Query<&ButtonBackground>,
) -> Result {
    let entity = trigger.target();

    let background = q_backgrounds.get(entity)?;

    commands
        .entity(entity)
        .insert(BackgroundColor(background.out));

    Ok(())
}

fn pressed_btn_background(
    trigger: Trigger<Pointer<Pressed>>,
    mut commands: Commands,
    q_backgrounds: Query<&ButtonBackground>,
) -> Result {
    let entity = trigger.target();

    let background = q_backgrounds.get(entity)?;

    commands
        .entity(entity)
        .insert(BackgroundColor(background.pressed));

    Ok(())
}

fn released_btn_background(
    trigger: Trigger<Pointer<Released>>,
    mut commands: Commands,
    q_backgrounds: Query<&ButtonBackground>,
) -> Result {
    let entity = trigger.target();

    let background = q_backgrounds.get(entity)?;

    commands
        .entity(entity)
        .insert(BackgroundColor(background.out));

    Ok(())
}

#[derive(Component)]
pub struct ButtonBackground {
    pub out: Color,
    pub over: Color,
    pub pressed: Color,
}

impl ButtonBackground {
    pub fn new(color: impl Into<Color>) -> Self {
        let color = color.into();

        Self {
            out: color,
            over: color.lighter(0.1),
            pressed: color.darker(0.1),
        }
    }

    #[allow(dead_code)]
    pub fn with_over(mut self, over: Color) -> Self {
        self.over = over;
        self
    }

    #[allow(dead_code)]
    pub fn with_out(mut self, out: Color) -> Self {
        self.out = out;
        self
    }

    #[allow(dead_code)]
    pub fn with_pressed(mut self, pressed: Color) -> Self {
        self.pressed = pressed;
        self
    }
}

impl Default for ButtonBackground {
    fn default() -> Self {
        ButtonBackground::new(TEAL_500)
    }
}
