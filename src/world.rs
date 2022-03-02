use alloc::vec::Vec;
use core::iter::Iterator;

use crate::{
	blobmap::BlobMap,
	query::{Query, QueryIter, QueryMut, QueryMutIter},
	Component,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Entity(u32);

// Should probably be lazy-building, TBD
pub struct EntityBuilder<'a> {
	pub world: &'a mut World,
	pub entity: Entity,
}

impl<'a> EntityBuilder<'a> {
	pub fn with<T: Component>(self, component: T) -> Self {
		self.world
			.components
			.insert_single(self.entity.clone(), component);
		self
	}
	pub fn build(self) -> Entity {
		self.entity
	}
}

pub struct World {
	components: BlobMap<Entity>,
	// component: DynTyped<
	entities: Vec<Entity>,
	entity_count: u32,
}

impl World {
	pub fn new() -> Self {
		Self {
			components: BlobMap::new(),
			entities: Vec::new(),
			entity_count: 0,
		}
	}

	pub fn entities(&self) -> core::slice::Iter<Entity> {
		self.entities.iter()
	}

	pub fn entity_component<V: 'static>(&self, entity_id: &Entity) -> Option<&V> {
		self.components.get_single(entity_id)
	}

	pub fn entity_component_mut<T: 'static>(&mut self, entity_id: &Entity) -> Option<&mut T> {
		self.components.get_single_mut(entity_id)
	}

	pub fn query_entity_mut<'a, Q: Query<'a>>(
		&'a mut self,
		entity_id: &'a Entity,
	) -> Option<Q::Tuple> {
		Q::query_entity(self, entity_id)
	}

	pub fn spawn<'a>(&'a mut self) -> EntityBuilder<'a> {
		let entity = Entity(self.entity_count);
		self.entities.push(entity.clone());
		self.entity_count += 1;
		EntityBuilder {
			world: self,
			entity: entity.clone(),
		}
	}

	pub fn despawn(&mut self, entity: &Entity) {
		self.components.remove_all(entity);
		if let Some(index) = self.entities.iter().position(|x| x == entity) {
			self.entities.remove(index);
		}
	}

	pub fn remove_component<T: Component>(&mut self, entity: &Entity) {
		self.components.remove::<T>(entity);
	}

	pub fn add_component<T: Component>(&mut self, entity: &Entity, component: T) {
		self.components.insert_single(entity.clone(), component)
	}
}

// Query wrappers, just so that the user doesn't need to go through the trait.
impl World {
	pub fn query<'a, Q: Query<'a>>(&'a self) -> QueryIter<'a, Q::Tuple> {
		Q::query(self)
	}
	pub fn query_mut<'a, Q: QueryMut<'a>>(&'a mut self) -> QueryMutIter<'a, Q::Tuple> {
		Q::query_mut(self)
	}
}
