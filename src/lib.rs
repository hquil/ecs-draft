#![no_std]
extern crate alloc;

/// Byte storage with associated type information
mod blobmap;

/// Implements logic to query [Component]s from the [World](world::World).
pub mod query;

/// Provides [World](world::World) to manage the ECS environment.
pub mod world;

/// Marker trait for a [Component] that can be managed by [World](world::World), [Fetch](query::Fetch) and [FetchMut](query::FetchMut).
pub trait Component: 'static + Sized {}

// Some markers for test purposes
impl Component for i32 {}
impl Component for alloc::string::String {}
impl Component for &'static str {}
impl Component for f32 {}

/// Repeats the given macro, dropping the first argument on each iteration.
///
/// Currently only useful for blocks, as there is no separating comma being generated.
#[macro_export]
macro_rules! repeat_with_first_arg_dropped {
	($marco:ident, $first:ident) => {
		$marco!($first);
	};
	($marco:ident, $first:ident, $($rest:ident),*) => {
		$marco!($first, $($rest),*);
		$crate::repeat_with_first_arg_dropped!($marco, $($rest),*);
	};
}
