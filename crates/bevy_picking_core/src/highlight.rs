use super::selection::*;
use crate::PausedForBlockers;
use bevy::{asset::Asset, prelude::*, render::color::Color};

/// Marker component to flag an entity as highlightable
#[derive(Component, Clone, Debug, Default)]
pub struct Highlight;

/// Component used to track the initial asset of a highlightable object, as well as for overriding
/// the default highlight materials.
#[derive(Component, Clone, Debug)]
pub struct Highlighting<T: Asset> {
    pub initial: Handle<T>,
    pub hovered: Option<Handle<T>>,
    pub pressed: Option<Handle<T>>,
    pub selected: Option<Handle<T>>,
}

/// Resource that defines the default highlighting assets to use. This can be overridden per-entity
/// with the [`Highlighting`] component.
pub struct DefaultHighlighting<T: Highlightable + ?Sized> {
    pub hovered: Handle<T>,
    pub pressed: Handle<T>,
    pub selected: Handle<T>,
}

/// This trait makes it possible for highlighting to be generic over any type of asset.
pub trait Highlightable: Default + Asset {
    /// The asset used to highlight the picked object. For a 3D mesh, this might be [`StandardMaterial`].
    fn highlight_defaults(materials: Mut<Assets<Self>>) -> DefaultHighlighting<Self>;
    fn materials(world: &mut World) -> Mut<Assets<Self>> {
        world
            .get_resource_mut::<Assets<Self>>()
            .expect("Failed to get resource")
    }
}

impl Highlightable for StandardMaterial {
    fn highlight_defaults(mut materials: Mut<Assets<Self>>) -> DefaultHighlighting<Self> {
        DefaultHighlighting {
            hovered: materials.add(Color::rgb(0.35, 0.35, 0.35).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
            selected: materials.add(Color::rgb(0.35, 0.35, 0.75).into()),
        }
    }
}

impl Highlightable for ColorMaterial {
    fn highlight_defaults(mut materials: Mut<Assets<Self>>) -> DefaultHighlighting<Self> {
        DefaultHighlighting {
            hovered: materials.add(Color::rgb(0.35, 0.35, 0.35).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
            selected: materials.add(Color::rgb(0.35, 0.35, 0.75).into()),
        }
    }
}

impl<T: Highlightable> FromWorld for DefaultHighlighting<T> {
    fn from_world(world: &mut World) -> Self {
        T::highlight_defaults(T::materials(world))
    }
}

#[allow(clippy::type_complexity)]
pub fn get_initial_highlight_asset<T: Asset>(
    mut commands: Commands,
    entity_asset_query: Query<(Entity, &Handle<T>), Added<Highlight>>,
    mut highlighting_query: Query<Option<&mut Highlighting<T>>>,
) {
    for (entity, material) in entity_asset_query.iter() {
        match highlighting_query.get_mut(entity) {
            Ok(Some(mut highlighting)) => highlighting.initial = material.to_owned(),
            _ => {
                let init_component = Highlighting {
                    initial: material.to_owned(),
                    hovered: None,
                    pressed: None,
                    selected: None,
                };
                commands.entity(entity).insert(init_component);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn highlight_assets<T: 'static + Highlightable + Send + Sync>(
    paused: Option<Res<PausedForBlockers>>,
    global_default_highlight: Res<DefaultHighlighting<T>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut Handle<T>,
            Option<&Selection>,
            &Highlighting<T>,
        ),
        Or<(Changed<Interaction>, Changed<Selection>)>,
    >,
) {
    // Set non-hovered material when picking is paused (e.g. while hovering a picking blocker).
    if let Some(paused) = paused {
        if paused.is_paused() {
            for (_, mut material, selection, highlight) in interaction_query.iter_mut() {
                *material = if selection.filter(|s| s.selected()).is_some() {
                    if let Some(highlight_asset) = &highlight.selected {
                        highlight_asset
                    } else {
                        &global_default_highlight.selected
                    }
                } else {
                    &highlight.initial
                }
                .to_owned();
            }
            return;
        }
    }
    for (interaction, mut active_asset, selection, initial_asset) in interaction_query.iter_mut() {
        *active_asset = match *interaction {
            Interaction::Clicked => {
                if let Some(highlight_asset) = &initial_asset.pressed {
                    highlight_asset
                } else {
                    &global_default_highlight.pressed
                }
            }
            Interaction::Hovered => {
                if let Some(highlight_asset) = &initial_asset.hovered {
                    highlight_asset
                } else {
                    &global_default_highlight.hovered
                }
            }
            Interaction::None => {
                if selection.filter(|s| s.selected()).is_some() {
                    if let Some(highlight_asset) = &initial_asset.selected {
                        highlight_asset
                    } else {
                        &global_default_highlight.selected
                    }
                } else {
                    &initial_asset.initial
                }
            }
        }
        .to_owned();
    }
}