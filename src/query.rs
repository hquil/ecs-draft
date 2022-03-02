use crate::{
	world::{Entity, World},
	Component,
};

use core::marker::PhantomData;

/// Provides the ability to immutably fetch a single query element.
///
/// A query element can be a reference type, like `&Position`
pub trait Fetch<'a>: Sized {
	/// Returns a single query element from the [Entity] in the [World]
	fn fetch(world: &'a World, entity: &'a Entity) -> Option<Self>;
}

impl<'a, C: 'static + Component> Fetch<'a> for &'a C {
	fn fetch(world: &'a World, entity: &'a Entity) -> Option<Self> {
		world.entity_component::<C>(entity)
	}
}

/// Provides the ability to mutably fetch a single query element.
///
/// A query element can be a reference type, like `&mut Position`
pub trait FetchMut<'a>: Sized {
	/// Returns a single query element from the [Entity] in the mutable [World]
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

/// Represents any type that can be fetched immutably from the world.
///
/// This is implemented for any generic tuple (&A, &B, ...) for which the generics implement [Fetch]
pub trait Query<'a> {
	/// The entire [Query]able representation, ex. `(&Position, &&str)`.
	///
	/// This is useful, since the types that come in can be used directly as the output type.
	/// Note that this is not restricted to Tuples, however, for now the naming seems clearer this way.
	type Tuple;

	/// Returns a single set of query elements specified in [`Self::Tuple`].
	fn query_entity(world: &'a World, entity: &'a Entity) -> Option<Self::Tuple>;

	/// Returns a [QueryIter] for all query elements specified in [`Self::Tuple`]
	fn query(world: &'a World) -> QueryIter<'a, Self::Tuple>;
}

/// Represents any type that can be fetched mutably from the world.
///
/// This is implemented for any generic tuple (&A, &mut B, ...) for which the generics implement [FetchMut]
pub trait QueryMut<'a> {
	/// The entire [Query]able representation, ex. `(&mut Position, &&str)`.
	type Tuple;

	/// Returns a single set of query elements specified in [`Self::Tuple`].
	fn query_entity_mut(world: &'a mut World, entity: &'a Entity) -> Option<Self::Tuple>;

	/// Returns a [QueryMutIter] for all query elements specified in [`Self::Tuple`].
	fn query_mut(world: &'a mut World) -> QueryMutIter<'a, Self::Tuple>;
}

/// A [Query] iterator for an immutable [World]
pub struct QueryIter<'a, Q> {
	world: &'a World,
	entity_index: usize,
	marker: PhantomData<Q>,
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

/// A [QueryMut] iterator for a mutable [World]
pub struct QueryMutIter<'a, Q> {
	world: &'a mut World,
	entity_index: usize,
	marker: PhantomData<Q>,
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

macro_rules! impl_queries {
	($($generic:ident),* ) => {
		impl<'a, $($generic),*> Query<'a> for ( $($generic),* ,)
		where
			$($generic: Fetch<'a>),*
		{
			type Tuple = ( $( $generic),* ,);

			fn query_entity(world: &'a World, entity: &'a Entity) -> Option<Self::Tuple> {
				Some((
					$( <$generic>::fetch(world, entity)? ),* ,
				))
			}

			fn query(world: &'a World) -> QueryIter<'a, Self::Tuple>  {
				QueryIter {
					world,
					entity_index: 0,
					marker: PhantomData::default(),
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
					entity_index: 0,
					marker: PhantomData::default(),
				}
			}
		}
	};
}

crate::repeat_with_first_arg_dropped! {
	impl_queries,
	N, Y, M, P, H, S, B, E, G, F, O, R, Q, U, I, C, K, W, A, L, T, Z
}
