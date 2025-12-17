use bevy::{ecs::entity::hash_set::EntityHashSet, prelude::*};

use crate::prelude::*;

/// Rules that determine which colliders are taken into account in [spatial queries](crate::spatial_query).
///
/// # Example
///
/// ```
#[cfg_attr(feature = "2d", doc = "use avian2d::prelude::*;")]
#[cfg_attr(feature = "3d", doc = "use avian3d::prelude::*;")]
/// use bevy::prelude::*;
///
/// fn setup(mut commands: Commands) {
#[cfg_attr(
    feature = "2d",
    doc = "    let object = commands.spawn(Collider::circle(0.5)).id();"
)]
#[cfg_attr(
    feature = "3d",
    doc = "    let object = commands.spawn(Collider::sphere(0.5)).id();"
)]
///
///     // A query filter that has three collision layers and excludes the `object` entity
///     let query_filter = SpatialQueryFilter::from_mask(0b1011).with_excluded_entities([object]);
///
///     // Spawn a ray caster with the query filter
///     commands.spawn(RayCaster::default().with_query_filter(query_filter));
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
// #[cfg_attr(feature = "serialize", reflect(Serialize, Deserialize))]
// #[reflect(Debug, PartialEq)]
pub struct SpatialQueryFilter<'a> {
    /// Specifies which [collision layers](CollisionLayers) will be included in the [spatial query](crate::spatial_query).
    pub mask: LayerMask,
    /// Predicate that can be used to exclude colliders based on custom logic.
    pub predicate: Option<&'a dyn Fn(Entity) -> bool>,
}

impl Default for SpatialQueryFilter<'_> {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl<'a> SpatialQueryFilter<'a> {
    /// The default [`SpatialQueryFilter`] configuration that includes all collision layers
    /// and has no excluded entities.
    pub const DEFAULT: Self = Self {
        mask: LayerMask::ALL,
        predicate: None,
    };

    /// Creates a new [`SpatialQueryFilter`] with the given [`LayerMask`] determining
    /// which [collision layers] will be included in the [spatial query].
    ///
    /// [collision layers]: CollisionLayers
    /// [spatial query]: crate::spatial_query
    pub fn from_mask(mask: impl Into<LayerMask>) -> Self {
        Self {
            mask: mask.into(),
            ..default()
        }
    }

    /// Creates a new [`SpatialQueryFilter`] with the given entities excluded from the [spatial query].
    ///
    /// [spatial query]: crate::spatial_query
    pub fn from_excluded_entities(entities: impl IntoIterator<Item = Entity>) -> Self {
        let f = |entity| !entities.into_iter().any(|e| e == entity);
        Self {
            predicate: Some(f),
            ..default()
        }
    }

    /// Creates a new [`SpatialQueryFilter`] with the given predicate ANDed to the existing predicate.
    pub fn and(mut self, predicate: &'a dyn Fn(Entity) -> bool) -> Self {
        self.predicate = Some(combine_predicates(self.predicate, predicate, |a, b| a && b));
        self
    }

    pub fn or(mut self, predicate: &'a dyn Fn(Entity) -> bool) -> Self {
        self.predicate = Some(combine_predicates(self.predicate, predicate, |a, b| a || b));
        self
    }

    /// Sets the [`LayerMask`] of the filter configuration. Only colliders with the corresponding
    /// [collision layer memberships] will be included in the [spatial query].
    ///
    /// [collision layer memberships]: CollisionLayers
    /// [spatial query]: crate::spatial_query
    pub fn with_mask(mut self, masks: impl Into<LayerMask>) -> Self {
        self.mask = masks.into();
        self
    }

    /// Excludes the given entities from the [spatial query](crate::spatial_query).
    pub fn with_excluded_entities(mut self, entities: impl IntoIterator<Item = Entity>) -> Self {
        let f = move |entity| !entities.into_iter().any(|e| e == entity);
        self.predicate = Some(combine_predicates(self.predicate, f, |a, b| a && b));
        self
    }

    /// Tests if an entity should be included in [spatial queries] based on the filter configuration.
    ///
    /// [spatial queries]: crate::spatial_query
    pub fn test(&self, entity: Entity, layers: CollisionLayers) -> bool {
        // !self.excluded_entities.contains(&entity)
        //     && CollisionLayers::new(LayerMask::ALL, self.mask)
        //         .interacts_with(CollisionLayers::new(layers.memberships, LayerMask::ALL))

        let layer_check = layers.interacts_with(CollisionLayers::new(LayerMask::ALL, self.mask));
        let exclusion_check = match &self.predicate {
            Some(pred) => pred(entity),
            None => true,
        };
        layer_check && exclusion_check
    }
}

fn combine_predicates<'b>(
    existing: Option<impl Fn(Entity) -> bool>,
    new: impl Fn(Entity) -> bool,
    combiner: fn(bool, bool) -> bool,
) -> impl Fn(Entity) -> bool {
    match existing {
        Some(pred) => {
            let f = move |entity| {
                let existing_result = pred(entity);
                let new_result = new(entity);
                combiner(existing_result, new_result)
            };
            &f
        }
        None => new,
    }
}
