use bevy::color::palettes::tailwind::*;
use bevy::prelude::*;

use crate::camera_controller::split_screen::{
    CameraType, QueryCameras,
};
use crate::enemy::Enemy;
use crate::tower::tower_attack::{Health, MaxHealth};
use crate::ui::world_space::WorldUi;

pub struct HealthBarPlugin;

impl Plugin for HealthBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_health_bar)
            .add_systems(Update, update_health_bars);
    }
}

fn spawn_health_bar(
    trigger: Trigger<OnAdd, Health>,
    mut commands: Commands,
    q_entity: Query<
        (&Health, &MaxHealth, Has<Enemy>),
        Without<HasHealthBar>,
    >,
    q_cameras: QueryCameras<Entity>,
) -> Result {
    let entity = trigger.target();

    let Ok((_health, _max_health, is_enemy)) = q_entity.get(entity)
    else {
        return Ok(());
    };

    let color = if is_enemy { RED_500 } else { GREEN_500 };

    let camera_a = q_cameras.get(CameraType::A)?;
    let camera_b = q_cameras.get(CameraType::B)?;

    let create_health_bar = |commands: &mut Commands,
                             camera_entity: Entity|
     -> Entity {
        let fill_bar = commands
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(color.into()),
                BorderRadius::all(Val::VMin(0.2)),
            ))
            .id();

        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::VMin(6.0),
                    height: Val::VMin(0.8),
                    ..default()
                },
                BackgroundColor(Color::BLACK.with_alpha(0.9)),
                BorderRadius::all(Val::VMin(0.2)),
                WorldUi::new(entity).with_world_offset(Vec3::Y * 1.0),
                UiTargetCamera(camera_entity),
            ))
            .add_child(fill_bar)
            .id()
    };

    // Create health bars for both cameras
    let health_bar_a = create_health_bar(&mut commands, camera_a);
    let health_bar_b = create_health_bar(&mut commands, camera_b);

    commands.entity(entity).insert(HasHealthBar {
        camera_a: health_bar_a,
        camera_b: health_bar_b,
    });

    Ok(())
}

fn update_health_bars(
    q_entities: Query<
        (&Health, &MaxHealth, &HasHealthBar),
        Changed<Health>,
    >,
    q_children: Query<&Children>,
    mut q_fill: Query<&mut Node>,
) {
    for (health, max_health, health_bars) in &q_entities {
        let percentage = health.0 / max_health.0;
        let width = Val::Percent(percentage * 100.0);

        for &health_bar_entity in
            &[health_bars.camera_a, health_bars.camera_b]
        {
            if let Ok(children) = q_children.get(health_bar_entity) {
                if let Some(&fill_entity) = children.first() {
                    if let Ok(mut fill_node) =
                        q_fill.get_mut(fill_entity)
                    {
                        fill_node.width = width;
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub struct HasHealthBar {
    pub camera_a: Entity,
    pub camera_b: Entity,
}
