use ecs_draft::{world::World, Component};

fn main() {
	let mut world = World::new();

	let hello = world
		.spawn()
		.with("Hello")
		.with(String::from("World"))
		.with(Glyph('?')) // overwrite and Drop '?'
		.with(Glyph('!'))
		.build();

	for (entity, s) in world.query::<(&&str, &String, &Glyph)>() {
		println!("{:?} says {}, {}{}", entity, s.0, s.1, s.2);
	}

	let player = world
		.spawn()
		.with(Position(2, 3))
		.with(Glyph('A'))
		.with(Player)
		.build();

	world.add_component(&player, Glyph('P'));
	world.add_component(&player, Player); // overwrite and drop Player

	world.spawn().with(Position(9, 1)).with(Glyph('N'));

	// Figured out the `query` / `query_mut` approach. `query` won't work here.
	for (entity, (glyph, pos)) in world.query_mut::<(&Glyph, &mut Position)>() {
		let name = if entity == &player { "Player" } else { "NPC" };

		println!("{} has position {:?}, glyph: {:?}. ", name, pos, glyph);
		pos.0 *= pos.0;
	}

	world.despawn(&hello);
	world.remove_component::<Player>(&player);

	// trailing comma for a single-element tuple. TBD.
	for (_, (pos,)) in world.query_mut::<(&mut Position,)>() {
		print!("Moving... ");
		*pos = Position(0, 0);
		println!("Travelling without moving...");
		pos.0 = 69;
		pos.1 = 420;
	}

	println!("Cleanup...");
}

#[derive(Debug)]
pub struct Position(i32, i32);

#[derive(Debug)]
pub struct Glyph(char);

#[derive(Debug)]
pub struct Player;

impl core::fmt::Display for Glyph {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use std::fmt::Write;
		f.write_char(self.0)
	}
}

impl_component!(Player);
impl_component!(Glyph);
impl_component!(Position);

#[macro_export]
macro_rules! impl_component {
	($name:ident) => {
		impl Component for $name {}

		impl Drop for $name {
			fn drop(&mut self) {
				println!("Dropped {:?}", self);
			}
		}
	};
}
