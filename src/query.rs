use crate::{
	world::{Entity, World},
	Component,
};

use core::marker::PhantomData;

pub trait Query<'a> {
	type Tuple;
	fn query_entity(world: &'a World, entity: &'a Entity) -> Option<Self::Tuple>;

	fn query(world: &'a World) -> QueryIter<'a, Self::Tuple>;
}

pub trait QueryMut<'a> {
	type Tuple;
	fn query_entity_mut(world: &'a mut World, entity: &'a Entity) -> Option<Self::Tuple>;
	fn query_mut(world: &'a mut World) -> QueryMutIter<'a, Self::Tuple>;
}

pub trait Fetch<'a>: Sized {
	fn fetch(world: &'a World, entity: &'a Entity) -> Option<Self>;
}

impl<'a, C: 'static + Component> Fetch<'a> for &'a C {
	fn fetch(world: &'a World, entity: &'a Entity) -> Option<Self> {
		world.entity_component::<C>(entity)
	}
}

pub trait FetchMut<'a>: Sized {
	fn fetch_mut(world: &'a mut World, entity: &'a Entity) -> Option<Self>;
}

impl<'a, C: 'static + Component> FetchMut<'a> for &'a C {
	fn fetch_mut(world: &'a mut World, entity: &'a Entity) -> Option<Self> {
		world.entity_component::<C>(entity)
	}
}

impl<'a, C: 'static + Component> FetchMut<'a> for &'a mut C {
	fn fetch_mut(world: &'a mut World, entity: &'a Entity) -> Option<Self> {
		world.entity_component_mut::<C>(entity)
	}
}

pub struct QueryMutIter<'a, Q> {
	world: &'a mut World,
	marker: PhantomData<Q>,
	entity_index: usize,
}

impl<'a, Q: QueryMut<'a> + 'a> Iterator for QueryMutIter<'a, Q> {
	type Item = (&'a Entity, Q::Tuple);
	fn next(&mut self) -> Option<Self::Item> {
		loop {
			// error[E0495]: cannot infer an appropriate lifetime due to conflicting requirements
			// = note: expected `QueryMut<'_>`
			//            found `QueryMut<'a>`
			//
			// ╰( ⁰ ਊ ⁰ )━☆ﾟ.*･｡ﾟ raise undead
			let this: &'a mut Self = unsafe { core::mem::transmute(&mut *self) };

			let entity = this.world.entities().nth(this.entity_index)?;

			// error[E0502]: cannot borrow `*this.world` as mutable because it is also borrowed as immutable
			//
			// ༼∩ •́ ヮ •̀ ༽⊃━☆ﾟ. * ･ ｡ﾟ Midnight lazyness Hack
			let entity = unsafe { core::mem::transmute(entity) };
			// I'm considering to use entity as value-type, so this will go away soon

			this.entity_index += 1;
			if let Some(result) = Q::query_entity_mut(this.world, entity) {
				return Some((entity, result));
			}
		}
	}
}

pub struct QueryIter<'a, Q> {
	world: &'a World,
	marker: PhantomData<Q>,
	entity_index: usize,
}

impl<'a, Q: Query<'a> + 'a> Iterator for QueryIter<'a, Q> {
	type Item = (&'a Entity, Q::Tuple);
	fn next(&mut self) -> Option<Self::Item> {
		loop {
			let entity_id = self.world.entities().nth(self.entity_index)?;
			self.entity_index += 1;
			if let Some(result) = Q::query_entity(self.world, entity_id) {
				return Some((entity_id, result));
			}
		}
	}
}

macro_rules! impl_queries {
	($($generic:ident),* ) => {
		impl<'a, $($generic),*> Query<'a> for ( $($generic),* ,)
		where
			$($generic: Fetch<'a>),*
		{
			// The whole tuple. Thankfully what comes in, goes out!
			type Tuple = ( $( $generic),* ,);

			fn query_entity(world: &'a World, entity: &'a Entity) -> Option<Self::Tuple> {
				Some((
					$( <$generic>::fetch(world, entity)? ),* ,
				))
			}

			fn query(world: &'a World) -> QueryIter<'a, Self::Tuple>  {
				QueryIter {
					world,
					marker: PhantomData::default(),
					entity_index: 0,
				}
			}
		}

		impl<'a, $($generic),*> QueryMut<'a> for ( $($generic),* ,)
		where
			$($generic: FetchMut<'a>),*
		{
			type Tuple = ( $( $generic),* ,);

			fn query_entity_mut(world: &'a mut World, entity: &'a Entity) -> Option<Self::Tuple> {
				Some((
					$( {
						// error[E0499]: cannot borrow `*world` as mutable more than once at a time
						// guess we'll just...
						let world = unsafe { core::mem::transmute(&mut *world) }; // *zap* . * ･ ｡ﾟ☆━੧༼ •́ ヮ •̀ ༽୨

						<$generic>::fetch_mut(world, entity)?
					} ),* ,
				))
			}

			fn query_mut(world: &'a mut World) -> QueryMutIter<'a, Self::Tuple>  {
				QueryMutIter {
					world,
					marker: PhantomData::default(),
					entity_index: 0,
				}
			}
		}
	};
}

crate::repeat_with_first_arg_dropped! {
	impl_queries,
	N, Y, M, P, H, S, B, E, G, F, O, R, Q, U, I, C, K, W, A, L, T, Z
}
