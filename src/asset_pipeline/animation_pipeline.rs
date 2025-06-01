use std::sync::Arc;

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;

use super::{AssetState, PrefabAssets, PrefabName};

pub(super) struct AnimationPipelinePlugin;

impl Plugin for AnimationPipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AssetState::LoadingAnimation),
            setup_prefab_animation_graphs,
        )
        .add_observer(setup_animation_player_target);

        #[cfg(feature = "dev")]
        app.register_type::<AnimationPlayerTargets>();
    }
}

fn setup_prefab_animation_graphs(
    mut prefabs: ResMut<PrefabAssets>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut state: ResMut<NextState<AssetState>>,
) -> Result {
    let mut named_graphs = Vec::new();

    for name in prefabs.named_prefabs.keys() {
        let gltf = prefabs
            .get_gltf(PrefabName::Absolute(name), &gltfs)
            .ok_or("Prefab should have been loaded.")?;

        let mut graph = AnimationGraph::new();
        let mut node_map = HashMap::new();

        for (name, clip) in gltf.named_animations.iter() {
            let Some(node_name) =
                name.split('.').nth(1).map(|name| name.to_string())
            else {
                warn!(
                    "Animation should have exactly 1 '.', got '{name}' instead."
                );
                continue;
            };

            node_map.insert(
                node_name,
                graph.add_clip(clip.clone(), 1.0, graph.root),
            );
        }

        let graph_handle = graphs.add(graph);
        named_graphs.push((
            name.clone(),
            AnimationGraphMap {
                graph: graph_handle,
                node_map: NodeMap(Arc::new(node_map)),
            },
        ));
    }

    for (name, graph) in named_graphs {
        prefabs.named_animations.insert(name, graph);
    }

    info!(
        "Loading state '{:?}' is done",
        AssetState::LoadingAnimation
    );
    state.set(AssetState::Loaded);

    Ok(())
}

fn setup_animation_player_target(
    trigger: Trigger<SceneInstanceReady, ()>,
    mut commands: Commands,
    q_is_animatable: Query<(), With<IsAnimatable>>,
    q_children: Query<&Children>,
    q_animation_player: Query<&Name, With<AnimationPlayer>>,
) {
    let scene_entity = trigger.target();
    // Only setup scenes with IsAnimatable tag.
    if q_is_animatable.contains(scene_entity) == false {
        return;
    };

    let mut targets = AnimationPlayerTargets::default();

    for child in q_children.iter_descendants(scene_entity) {
        if let Ok(name) = q_animation_player.get(child) {
            targets.0.insert(name.to_string(), child);
        }
    }
    commands.entity(scene_entity).insert(targets);
}

#[derive(Debug)]
#[cfg_attr(feature = "dev", derive(Reflect))]
pub struct AnimationGraphMap {
    pub graph: Handle<AnimationGraph>,
    pub node_map: NodeMap,
}

#[derive(Component, Deref, Debug, Clone)]
#[cfg_attr(feature = "dev", derive(Reflect))]
pub struct NodeMap(Arc<HashMap<String, AnimationNodeIndex>>);

/// Map [`Name`] to their respective [`Entity`].
#[derive(Component, Deref, Default, Debug)]
#[cfg_attr(feature = "dev", derive(Reflect))]
#[cfg_attr(feature = "dev", reflect(Component))]
pub struct AnimationPlayerTargets(HashMap<String, Entity>);

#[derive(Component)]
pub struct IsAnimatable;
