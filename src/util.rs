use bevy::prelude::*;

pub trait PropagateComponentAppExt {
    fn propagate_component<C, R>(&mut self) -> &mut Self
    where
        C: Component + Clone,
        R: RelationshipTarget;
}

impl PropagateComponentAppExt for App {
    fn propagate_component<C, R>(&mut self) -> &mut Self
    where
        C: Component + Clone,
        R: RelationshipTarget,
    {
        self.add_systems(
            PostUpdate,
            propagate_component::<C, R>.in_set(PropagateComponentSet),
        )
    }
}

/// Propagate component to the relationship hierarchy.
pub fn propagate_component<C, R>(
    mut commands: Commands,
    q_relations: Query<
        (&C, &R),
        // Just added or the relationship changes.
        Or<(Added<C>, Changed<R>)>,
    >,
) where
    C: Component + Clone,
    R: RelationshipTarget,
{
    for (component, targets) in q_relations.iter() {
        for entity in targets.iter() {
            if let Ok(mut cmd) = commands.get_entity(entity) {
                cmd.insert(component.clone());
            }
        }
    }
}

#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct PropagateComponentSet;
