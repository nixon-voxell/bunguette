use bevy::prelude::*;
use bevy_seedling::prelude::*;
use bevy_seedling::sample::Sample;

use crate::machine::{Machine, OperationTimer};

pub(super) struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SeedlingPlugin::default())
            .init_resource::<GameAudio>()
            .add_observer(start_machine_audio)
            .add_observer(stop_machine_audio);
    }
}

/// Resource containing all game audio handles
#[derive(Resource)]
pub struct GameAudio {
    // Machine sounds
    pub rotisserie: Handle<Sample>,
    pub wok: Handle<Sample>,
}

impl FromWorld for GameAudio {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            rotisserie: asset_server
                .load("audios/machine/rotisserie.ogg"),
            wok: asset_server.load("audios/machine/wok.ogg"),
        }
    }
}

/// Start audio when machines begin operating
fn start_machine_audio(
    trigger: Trigger<OnAdd, OperationTimer>,
    mut commands: Commands,
    q_machines: Query<&Machine>,
    audio: Res<GameAudio>,
) {
    let machine_entity = trigger.target();
    let Ok(machine) = q_machines.get(machine_entity) else {
        return;
    };

    let sound_handle = match machine.recipe_id.as_str() {
        "rotisserie" => audio.rotisserie.clone(),
        "wok" => audio.wok.clone(),
        _ => return,
    };

    // Spawn sound and store its entity on the machine
    let sound_entity = commands
        .spawn(
            SamplePlayer::new(sound_handle)
                .looping()
                .with_volume(Volume::Linear(0.3)),
        )
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
