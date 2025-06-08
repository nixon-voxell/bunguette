use bevy::prelude::*;

use crate::asset_pipeline::{CurrentScene, PrefabAssets, PrefabName};
use crate::ui::Screen;

pub(super) struct EnemySpawnerPlugin;

impl Plugin for EnemySpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EnemySpawner>();

        app.add_sub_state::<SpawnWave>()
            .init_resource::<WaveCountdown>()
            .init_resource::<SpawnCount>()
            .init_resource::<SpawnTimer>()
            .add_systems(
                Update,
                (
                    (set_wave_countdown, set_spawn_count_and_timer)
                        .run_if(state_changed::<SpawnWave>),
                    ((wave_countdown, spawn_timer), spawn_enemy)
                        .chain(),
                )
                    .chain()
                    .run_if(in_state(Screen::EnterLevel)),
            )
            .add_observer(on_add_spawner);
    }
}

/// Enter [`SpawnWave::One`] on spawner added.
fn on_add_spawner(
    _: Trigger<OnAdd, EnemySpawner>,
    mut next_wave: ResMut<NextState<SpawnWave>>,
) {
    next_wave.set(SpawnWave::One);
}

fn spawn_enemy(
    mut commands: Commands,
    q_spawner: Query<&GlobalTransform, With<EnemySpawner>>,
    countdown: Res<WaveCountdown>,
    timer: Res<SpawnTimer>,
    mut spawn_count: ResMut<SpawnCount>,
    current_scene: Res<CurrentScene>,
    prefabs: Res<PrefabAssets>,
    gltfs: Res<Assets<Gltf>>,
    curr_wave: Res<State<SpawnWave>>,
    mut next_wave: ResMut<NextState<SpawnWave>>,
    mut next_screen: ResMut<NextState<Screen>>,
) -> Result {
    let Ok(transform) = q_spawner.single() else {
        return Ok(());
    };

    let Some(current_scene) = current_scene.get() else {
        return Ok(());
    };

    if countdown.finished() == false {
        return Ok(());
    }

    if timer.just_finished() == false {
        return Ok(());
    }

    if spawn_count.0 == 0 {
        match curr_wave.get() {
            SpawnWave::One => {
                next_wave.set(SpawnWave::Two);
                info!("Entering wave 2.")
            }
            SpawnWave::Two => {
                next_wave.set(SpawnWave::Three);
                info!("Entering wave 3.")
            }
            SpawnWave::Three => {
                next_wave.set(SpawnWave::None);
                next_screen.set(Screen::GameOver);
                info!("Game over!")
            }
            SpawnWave::None => {}
        }
        return Ok(());
    }

    spawn_count.0 -= 1;

    commands.spawn((
        SceneRoot(
            prefabs
                .get_gltf(PrefabName::FileName("mouse_a"), &gltfs)
                .ok_or("Can't find mouse prefab!")?
                .default_scene
                .clone()
                .ok_or("Tower prefab have a default scene.")?,
        ),
        transform.compute_transform(),
        ChildOf(current_scene),
    ));

    Ok(())
}

fn set_wave_countdown(
    current_wave: Res<State<SpawnWave>>,
    mut countdown: ResMut<WaveCountdown>,
    q_spawner: Query<&EnemySpawner>,
) {
    let Ok(spawner) = q_spawner.single() else {
        return;
    };

    let countdown_time = match current_wave.get() {
        SpawnWave::One => {
            info!("Setting wave 1 countdown.");
            spawner.wave_1.countdown
        }
        SpawnWave::Two => {
            info!("Setting wave 2 countdown.");
            spawner.wave_2.countdown
        }
        SpawnWave::Three => {
            info!("Setting wave 3 countdown.");
            spawner.wave_3.countdown
        }
        SpawnWave::None => {
            return;
        }
    };

    countdown.0 =
        Timer::from_seconds(countdown_time, TimerMode::Once);
}

fn set_spawn_count_and_timer(
    q_spawner: Query<&EnemySpawner>,
    current_wave: Res<State<SpawnWave>>,
    mut timer: ResMut<SpawnTimer>,
    mut spawn_count: ResMut<SpawnCount>,
) {
    let Ok(spawner) = q_spawner.single() else {
        return;
    };

    let (interval, count) = match current_wave.get() {
        SpawnWave::One => {
            info!("Setting wave 1 interval and count.");
            (
                spawner.wave_1.spawn_interval,
                spawner.wave_1.enemy_count,
            )
        }
        SpawnWave::Two => {
            info!("Setting wave 2 interval and count.");
            (
                spawner.wave_2.spawn_interval,
                spawner.wave_2.enemy_count,
            )
        }
        SpawnWave::Three => {
            info!("Setting wave 3 interval and count.");
            (
                spawner.wave_3.spawn_interval,
                spawner.wave_3.enemy_count,
            )
        }
        SpawnWave::None => {
            return;
        }
    };

    timer.0 = Timer::from_seconds(interval, TimerMode::Repeating);
    spawn_count.0 = count;
}

/// Tick every frame.
fn wave_countdown(
    mut countdown: ResMut<WaveCountdown>,
    time: Res<Time>,
) {
    if countdown.finished() == false {
        countdown.tick(time.delta());
    }
}

fn spawn_timer(
    countdown: Res<WaveCountdown>,
    mut timer: ResMut<SpawnTimer>,
    time: Res<Time>,
) {
    // Only tick after countdown is reached.
    if countdown.finished() {
        timer.tick(time.delta());
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct EnemySpawner {
    pub wave_1: WaveConfig,
    pub wave_2: WaveConfig,
    pub wave_3: WaveConfig,
}

#[derive(Reflect)]
pub struct WaveConfig {
    /// How long before the wave starts.
    pub countdown: f32,
    pub enemy_count: usize,
    pub spawn_interval: f32,
}

#[derive(
    SubStates, Default, Debug, Hash, Clone, Copy, Eq, PartialEq,
)]
#[source(Screen = Screen::EnterLevel)]
pub enum SpawnWave {
    #[default]
    None,
    One,
    Two,
    Three,
}

/// Countdown timer until enemies start to spawn.
#[derive(Resource, Deref, DerefMut, Default)]
pub struct WaveCountdown(Timer);

/// Number of enemies to spawn left.
#[derive(Resource, Deref, DerefMut, Default)]
pub struct SpawnCount(usize);

/// Time left before the next spawn.
#[derive(Resource, Deref, DerefMut, Default)]
pub struct SpawnTimer(Timer);
