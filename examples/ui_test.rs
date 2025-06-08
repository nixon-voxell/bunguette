use bevy::asset::load_internal_binary_asset;
use bevy::ecs::spawn::SpawnWith;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use recipe_game::ui::widgets::{
    WidgetsPlugin,
    button::{ButtonBackground, LabelButton},
};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(WidgetsPlugin)
        .add_systems(Startup, setup_menu);

    load_internal_binary_asset!(
        app,
        TextFont::default().font,
        "../assets/fonts/Cherry_Bomb_One/CherryBombOne-Regular.ttf",
        |bytes: &[u8], _path: String| {
            Font::try_from_bytes(bytes.to_vec()).unwrap()
        }
    );

    app.run();
}

fn setup_menu(mut commands: Commands) {
    commands.spawn(Camera2d);

    let bg_color = Srgba::hex("BFB190").unwrap();
    let font_color = Srgba::hex("342C24").unwrap();
    let play_color = Srgba::hex("DAC682").unwrap();
    let exit_color = Srgba::hex("A39175").unwrap();

    let font_size = 30.0;

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(40.0)),
            justify_content: JustifyContent::End,
            align_items: AlignItems::End,
            ..default()
        },
        FocusPolicy::Pass,
        Pickable::IGNORE,
        Children::spawn(Spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_grow: 0.0,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(bg_color.with_alpha(0.4).into()),
            BorderRadius::all(Val::Px(40.0)),
            Children::spawn((
                Spawn((
                    Text::new("Bunguette"),
                    TextFont::from_font_size(font_size * 1.5),
                    TextColor(font_color.into()),
                )),
                SpawnWith(move |parent: &mut ChildSpawner| {
                    parent.spawn(
                        LabelButton::new("Play")
                            .with_background(ButtonBackground::new(
                                play_color,
                            ))
                            .with_text_color(font_color)
                            .with_font_size(font_size)
                            .build(),
                    );

                    // Only add exit button for non-web game.
                    #[cfg(not(target_arch = "wasm32"))]
                    parent.spawn(
                        LabelButton::new("Exit")
                            .with_background(ButtonBackground::new(
                                exit_color,
                            ))
                            .with_text_color(font_color)
                            .with_font_size(font_size)
                            .build(),
                    );
                }),
            )),
        ))),
    ));
}
