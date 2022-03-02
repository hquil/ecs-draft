use alloc::vec::Vec;
use core::iter::Iterator;

use crate::{
	blobmap::BlobMap,
	query::{Query, QueryIter, QueryMut, QueryMutIter},
	Component,
};

/// An [Entity] that is represented by a single [u32] id.
///
/// While it is not linked to a specific [World], passing an [Entity] to
/// a different [World] than it was created in, will refer to different data entirely.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Entity(u32);

/// The [EntityBuilder] allows for components to be attached in a functional chain.
/// See [`World::spawn`].
///
/// This might change to be lazy-building in the future, when the need arises.
pub struct EntityBuilder<'a> {
	pub world: &'a mut World,
	pub entity: Entity,
}

impl<'a> EntityBuilder<'a> {
	/// Adds or replaces a [Component] for the current [Entity] being built.
	/// See [`World::add_component`]
	pub fn with<T: Component>(self, component: T) -> Self {
		self.world.add_component(&self.entity, component);
		self
	}
	/// Consumes `self` and returns the [Entity] for further use.
	pub fn build(self) -> Entity {
		self.entity
	}
}

/// Provides a singular path to interact and manage [`Entity`] and [`Component`].
///
/// Any world is able to manage [`u32::MAX`] number of entities at most,
/// overstepping this Boundary will overflow the entity counter.
pub struct World {
	components: BlobMap<Entity>,
	entities: Vec<Entity>,
	entity_count: u32,
}

impl World {
	/// Creates an empty World.
	pub fn new() -> Self {
		Self {
			components: BlobMap::new(),
			entities: Vec::new(),
			entity_count: 0,
		}
	}

	/// Returns an iterator over every [Entity] spawned in this world.
	pub fn entities(&self) -> core::slice::Iter<Entity> {
		self.entities.iter()
	}

	/// Returns a reference to a single generic [Component] belonging to the [Entity].
	pub fn entity_component<V: 'static>(&self, entity: &Entity) -> Option<&V> {
		self.components.get(entity)
	}

	/// Returns a mutable reference to a single generic [Component] belonging to the [Entity].
	pub fn entity_component_mut<T: 'static>(&mut self, entity: &Entity) -> Option<&mut T> {
		self.components.get_mut(entity)
	}

	/// Spawns a new [Entity], and returns an [EntityBuilder] to keep attaching Components to it.
	/// ```
	/// let world = World::new();
	/// let entity = world
	/// 	.spawn()
	/// 	.with("Hello, World!")
	///  	.with(3.14159274 as f32)
	/// 	.build(); // returns the Entity "id" for further use
	/// ```
	pub fn spawn<'a>(&'a mut self) -> EntityBuilder<'a> {
		let entity = Entity(self.entity_count);
		self.entities.push(entity.clone());
		self.entity_count += 1;
		EntityBuilder {
			world: self,
			entity: entity.clone(),
		}
	}

	/// Deletes all [Component]s belonging to an [Entity], and removes it from this [World].
	pub fn despawn(&mut self, entity: &Entity) {
		self.components.remove_key(entity);
		if let Some(index) = self.entities.iter().position(|x| x == entity) {
			self.entities.remove(index);
		}
	}

	/// Deletes a single generic [Component] belonging to the [Entity].
	pub fn remove_component<T: Component>(&mut self, entity: &Entity) {
		self.components.remove::<T>(entity);
	}

	/// Adds or replaces a single [Component] to the [Entity].
	///
	/// An entity can only have one [Component] type (like an [f32]) at a time.
	/// An already existing component type will be overwritten, and dropped in place.
	pub fn add_component<T: Component>(&mut self, entity: &Entity, component: T) {
		self.components.insert_or_replace(entity.clone(), component)
	}

	/// Returns an immutable iterator for the specified [Query].
	pub fn query<'a, Q: Query<'a>>(&'a self) -> QueryIter<'a, Q::Tuple> {
		Q::query(self)
	}

	/// Returns a mutable iterator for the specified [Query].
	pub fn query_mut<'a, Q: QueryMut<'a>>(&'a mut self) -> QueryMutIter<'a, Q::Tuple> {
		Q::query_mut(self)
	}
}
