#![no_std]
extern crate alloc;

mod blobmap;
pub mod query;
pub mod world;

pub trait Component: 'static + Sized {}

// Some markers for test purposes
impl Component for i32 {}
impl Component for alloc::string::String {}
impl Component for &'static str {}
impl Component for f32 {}

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
