/// 88 bytes are leaked from HashMap, so we're fine in the sandbox so far.
///
/// ```[bash]
/// cargo build --examples leak_test
/// valgrind --leak-check=full --show-leak-kinds=all target/debug/examples/leak_test
/// ```
fn main() {
	let map = hashbrown::HashMap::<i32, i32>::new();
}
