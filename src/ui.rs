use bevy::color::palettes::tailwind::*;
use bevy::ecs::spawn::SpawnWith;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use widgets::button::{ButtonBackground, LabelButton};

use crate::asset_pipeline::SceneAssetsLoader;

mod inventory_ui;
pub mod widgets;
pub mod world_space;

pub(super) struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            world_space::WorldSpaceUiPlugin,
            widgets::WidgetsPlugin,
            inventory_ui::InventoryUiPlugin,
        ));

        app.init_state::<Screen>()
            .add_systems(
                OnEnter(Screen::Menu),
                (
                    setup_menu,
                    set_cursor_grab_mode(CursorGrabMode::None),
                ),
            )
            .add_systems(
                OnEnter(Screen::EnterLevel),
                (
                    load_level1,
                    set_cursor_grab_mode(CursorGrabMode::Locked),
                ),
            );
    }
}

fn set_cursor_grab_mode(
    grab_mode: CursorGrabMode,
) -> impl Fn(Query<'_, '_, &mut Window, With<PrimaryWindow>>) -> Result
{
    move |mut q_windows: Query<
        &mut Window,
        With<PrimaryWindow>,
    >| -> Result {
        let mut primary_window = q_windows.single_mut()?;
        primary_window.cursor_options.grab_mode = grab_mode;
        primary_window.cursor_options.visible = match grab_mode {
            CursorGrabMode::None => true,
            CursorGrabMode::Confined => true,
            CursorGrabMode::Locked => false,
        };

        Ok(())
    }
}

fn load_level1(mut scenes: SceneAssetsLoader) -> Result {
    scenes.load_level1()
}

fn setup_menu(mut commands: Commands) {
    commands.spawn((
        StateScoped(Screen::Menu),
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
                padding: UiRect::all(Val::VMin(6.0)),
                justify_content: JustifyContent::SpaceAround,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.2)),
            BorderRadius::all(Val::VMin(4.0)),
            Children::spawn((
                Spawn((
                    Text::new("Recipe"),
                    TextFont::from_font_size(64.0),
                    TextColor(ORANGE_600.into()),
                    TextShadow::default(),
                )),
                SpawnWith(|parent: &mut ChildSpawner| {
                    parent
                        .spawn(
                            LabelButton::new("Play!")
                                .with_bacground(
                                    ButtonBackground::new(SKY_500),
                                )
                                .build(),
                        )
                        .observe(play_on_click);

                    // Only add exit button for non-web game.
                    #[cfg(not(target_arch = "wasm32"))]
                    parent
                        .spawn(
                            LabelButton::new("Exit..")
                                .with_bacground(
                                    ButtonBackground::new(RED_500),
                                )
                                .build(),
                        )
                        .observe(exit_on_click);
                }),
            )),
        ))),
    ));
}

fn play_on_click(
    _: Trigger<Pointer<Click>>,
    mut screen: ResMut<NextState<Screen>>,
) {
    // screen.set(Screen::LevelSelection);
    screen.set(Screen::EnterLevel);
}

#[cfg(not(target_arch = "wasm32"))]
fn exit_on_click(
    _: Trigger<Pointer<Click>>,
    mut exit: EventWriter<AppExit>,
) {
    exit.write(AppExit::Success);
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
#[states(scoped_entities)]
pub enum Screen {
    #[default]
    Menu,
    // LevelSelection,
    EnterLevel, // TODO: Create substates for levels (1, 2, 3, ...).
}
