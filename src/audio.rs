use bevy::prelude::*;
use bevy_seedling::prelude::*;
use bevy_seedling::sample::Sample;

use crate::machine::{Machine, OperationTimer};
use crate::ui::Screen;

pub(super) struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SeedlingPlugin::default())
            .init_resource::<GameAudio>()
            .add_systems(OnEnter(Screen::Menu), start_menu_music)
            .add_systems(
                OnEnter(Screen::EnterLevel),
                start_game_music,
            )
            .add_observer(start_machine_audio)
            .add_observer(stop_machine_audio);
    }
}

/// Start menu background music
fn start_menu_music(mut commands: Commands, audio: Res<GameAudio>) {
    commands.spawn((
        SamplePlayer::new(audio.menu_music.clone())
            .looping()
            .with_volume(Volume::Linear(0.3)),
        StateScoped(Screen::Menu),
    ));
}

/// Start in-game background music
fn start_game_music(mut commands: Commands, audio: Res<GameAudio>) {
    commands.spawn((
        SamplePlayer::new(audio.game_music.clone())
            .looping()
            .with_volume(Volume::Linear(0.3)),
        StateScoped(Screen::EnterLevel),
    ));
}

/// Start audio when machines start operating
fn start_machine_audio(
    trigger: Trigger<OnAdd, OperationTimer>,
    mut commands: Commands,
    q_machines: Query<(&Machine, &GlobalTransform)>,
    audio: Res<GameAudio>,
) {
    let machine_entity = trigger.target();
    let Ok((machine, machine_transform)) =
        q_machines.get(machine_entity)
    else {
        return;
    };

    let sound_handle = match machine.recipe_id.as_str() {
        "rotisserie" => audio.rotisserie.clone(),
        "wok" => audio.wok.clone(),
        _ => return,
    };

    // Spawn the sound player entity with spatial audio components
    let sound_entity = commands
        .spawn((
            SamplePlayer::new(sound_handle)
                .looping()
                .with_volume(Volume::Linear(0.25)),
            GlobalTransform::from_translation(
                machine_transform.translation(),
            ),
            SpatialBasicNode {
                panning_threshold: 0.2,
                ..Default::default()
            },
            SpatialScale(Vec3::splat(0.4)),
        ))
        .id();

    commands
        .entity(machine_entity)
        .insert(PlayingAudio(sound_entity));
}

/// Stop audio when machines finish operating
fn stop_machine_audio(
    trigger: Trigger<OnRemove, OperationTimer>,
    mut commands: Commands,
    q_playing_audio: Query<&PlayingAudio>,
) {
    let machine_entity = trigger.target();
    let Ok(playing_audio) = q_playing_audio.get(machine_entity)
    else {
        return;
    };

    commands.entity(playing_audio.0).despawn();
    commands.entity(machine_entity).remove::<PlayingAudio>();
}

/// Component that stores the entity ID of the playing audio
#[derive(Component)]
struct PlayingAudio(Entity);

/// Resource containing all game audio handles
#[derive(Resource)]
pub struct GameAudio {
    // Machine sounds
    pub rotisserie: Handle<Sample>,
    pub wok: Handle<Sample>,
    // Background music
    pub menu_music: Handle<Sample>,
    pub game_music: Handle<Sample>,
}

impl FromWorld for GameAudio {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            rotisserie: asset_server
                .load("audios/machine/rotisserie.ogg"),
            wok: asset_server.load("audios/machine/wok.ogg"),
            menu_music: asset_server
                .load("audios/music/menu_bgm.ogg"),
            game_music: asset_server
                .load("audios/music/game_bgm.ogg"),
        }
    }
}
