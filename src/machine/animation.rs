use std::time::Duration;

use bevy::animation::AnimationTarget;
use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;

use crate::asset_pipeline::animation_pipeline::{
    AnimationGraphMap, NodeMap,
};
use crate::asset_pipeline::{AssetState, PrefabAssets};
use crate::interaction::MarkerPlayers;

use super::recipe::RecipeRegistry;
use super::{Machine, OperationTimer};

pub(super) struct MachineAnimationPlugin;

impl Plugin for MachineAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (setup_animation_graph, on_play)
                .run_if(in_state(AssetState::Loaded)),
        );
    }
}

fn on_play(
    q_machines: Query<
        (&NodeMap, &AnimationTarget, &Machine),
        With<OperationTimer>,
    >,
    mut q_animation_players: Query<(
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
) -> Result {
    for (node_map, animation_target, machine) in q_machines.iter() {
        let (mut anim_player, mut anim_transitions) =
            q_animation_players.get_mut(animation_target.player)?;

        if anim_player.all_finished() {
            let play_node =
                *node_map.get("OnPlay").ok_or(format!(
                    "No 'OnPlay' animation for {}!",
                    machine.recipe_id
                ))?;

            anim_transitions
                .play(
                    &mut anim_player,
                    play_node,
                    Duration::from_millis(0),
                )
                .repeat();
        }
    }

    Ok(())
}

fn on_trigger_animation<E: Event, B: Bundle, F: QueryFilter>(
    animation_name: &str,
) -> impl Fn(
    Trigger<'_, E, B>,
    Query<'_, '_, (&NodeMap, &AnimationTarget, &Machine), F>,
    Query<'_, '_, (&mut AnimationPlayer, &mut AnimationTransitions)>,
) -> Result {
    move |trigger: Trigger<E, B>,
          q_machines: Query<
        (&NodeMap, &AnimationTarget, &Machine),
        F,
    >,
          mut q_animation_players: Query<(
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>| {
        let entity = trigger.target();
        let Ok((node_map, animation_target, machine)) =
            q_machines.get(entity)
        else {
            return Ok(());
        };

        let (mut anim_player, mut anim_transitions) =
            q_animation_players.get_mut(animation_target.player)?;

        let node = *node_map.get(animation_name).ok_or(format!(
            "No '{animation_name}' animation for {}!",
            machine.recipe_id
        ))?;

        anim_transitions.play(
            &mut anim_player,
            node,
            Duration::from_millis(500),
        );

        Ok(())
    }
}

fn setup_animation_graph(
    mut commands: Commands,
    q_characters: Query<
        (&Machine, &AnimationTarget, Entity),
        Without<NodeMap>,
    >,
    prefabs: Res<PrefabAssets>,
    recipe_registry: RecipeRegistry,
) -> Result {
    for (machine, animation_target, entity) in q_characters.iter() {
        let recipe = machine.get_recipe(&recipe_registry).ok_or(
            format!("Unable to get recipe for machine {entity}"),
        )?;

        let AnimationGraphMap { graph, node_map } = prefabs
            .get_animation(recipe.prefab_name())
            .ok_or(format!(
                "Unable to get animation for {}!",
                machine.recipe_id
            ))?;

        commands
            .entity(entity)
            .insert(node_map.clone())
            .observe(on_trigger_animation::<
                OnAdd,
                MarkerPlayers,
                Without<OperationTimer>,
            >("OnEnter"))
            .observe(on_trigger_animation::<
                OnRemove,
                MarkerPlayers,
                Without<OperationTimer>,
            >("OnExit"))
            .observe(
                on_trigger_animation::<OnAdd, OperationTimer, ()>(
                    "OnStart",
                ),
            )
            .observe(on_trigger_animation::<
                OnRemove,
                OperationTimer,
                (),
            >("OnStop"));

        commands.entity(animation_target.player).insert((
            AnimationGraphHandle(graph.clone()),
            AnimationTransitions::new(),
        ));

        info!("Setup animation graph for {}.", machine.recipe_id);
    }

    Ok(())
}
