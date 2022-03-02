use alloc::vec::Vec;
use hashbrown::HashMap;

use core::{any::TypeId, cmp::Eq, hash::Hash, mem, slice};

/// Manages a separate Map for each TypeId given, bridging the unsafe layer.
pub(crate) struct BlobMap<K>
where
	K: Eq + Hash,
{
	data_store: HashMap<TypeId, BlobStorage<K>>,
}

impl<K> BlobMap<K>
where
	K: Eq + Hash,
{
	pub(crate) fn new() -> Self {
		Self {
			data_store: HashMap::new(),
		}
	}

	/// Inserts a single element associated with the key and type.
	/// If the given type was not registered yet, a new type-specific storage is created.
	///
	/// Note that keys can be registered for multiple types, referring to different data (N:N).
	/// If a value for both the key and type already exists, it will be replaced.
	pub fn insert_or_replace<V: 'static>(&mut self, key: K, value: V) {
		let entry = self
			.data_store
			.entry(TypeId::of::<V>())
			.or_insert(BlobStorage {
				type_info: TypeInfo::of::<V>(),
				data: alloc::vec![],
				indices: HashMap::new(),
			});
		unsafe {
			entry.insert_or_replace(key, value);
		}
	}

	/// Returns a reference to the type containing the key.
	pub fn get<V: 'static>(&self, key: &K) -> Option<&V> {
		let entry = self.data_store.get(&TypeId::of::<V>())?;
		unsafe { entry.get(key) }
	}

	/// Returns a mutable reference to the type containing the key.
	pub fn get_mut<V: 'static>(&mut self, key: &K) -> Option<&mut V> {
		let entry = self.data_store.get_mut(&TypeId::of::<V>())?;
		unsafe { entry.get_mut(key) }
	}

	/// Removes and drops the data associated with the key and type
	pub fn remove<V: 'static>(&mut self, key: &K) {
		if let Some(entry) = self.data_store.get_mut(&TypeId::of::<V>()) {
			unsafe {
				entry.remove_and_drop(key);
			}
		}
	}

	/// Removes and drops all typed data associated with the key
	pub fn remove_key(&mut self, key: &K) {
		for entry in self.data_store.values_mut() {
			unsafe {
				entry.remove_and_drop(key);
			}
		}
	}

	/// Removes and drops all data associated with the type
	///
	/// Todo: Test Functionality
	#[allow(dead_code)]
	pub fn remove_type<V: 'static>(&mut self) {
		self.data_store.remove_entry(&TypeId::of::<V>());
	}
}

/// Additional Dynamic Type information.
/// Used to handle operations when the call is just "assumed" to be correct.
pub struct TypeInfo {
	size_in_bytes: usize,
	drop_fn: unsafe fn(*mut u8),
}

impl TypeInfo {
	/// Returns the [TypeInfo] for the given generic type.
	pub fn of<T>() -> Self {
		unsafe fn drop_fn<V>(data: *mut u8) {
			data.cast::<V>().drop_in_place();
		}
		Self {
			size_in_bytes: mem::size_of::<T>(),
			drop_fn: drop_fn::<T>,
		}
	}
}

/// A byte storage for a single type, with generics being applied on a function-level.
///
/// The provided functionality is mostly unsafe, due to the assumption that the correct
/// type was supplied on each call.
///
/// Todo: Consider Storing [TypeId] with the [TypeInfo], and assert that the calls are correct.
/// This was not done due to premature optimization.
struct BlobStorage<K>
where
	K: Eq + Hash,
{
	type_info: TypeInfo,
	data: Vec<u8>,
	indices: HashMap<K, usize>,
}

impl<K> BlobStorage<K>
where
	K: Eq + Hash,
{
	unsafe fn get<V>(&self, key: &K) -> Option<&V> {
		let index = *self.indices.get(key)?;
		if self.type_info.size_in_bytes == 0 {
			// Return something made-up for ZST
			// ╰(•̀ 3 •́)━☆ﾟ.*･｡ﾟ Does it even matter what gets returned here?
			Some(mem::transmute(core::ptr::NonNull::<V>::dangling()))
		} else {
			// ╰( ͡° ͜ʖ ͡° )つ──☆*:・ﾟ It's magic
			let pseudo_store = mem::transmute::<&Vec<u8>, &Vec<V>>(&self.data);
			// Even though the pseudo store is now typed correctly,
			// its length and capacity are still internally measured in bytes.
			// Simply indexing into it should work just fine though.
			Some(&pseudo_store[index])
		}
	}

	unsafe fn get_mut<V>(&mut self, key: &K) -> Option<&mut V> {
		let index = *self.indices.get(key)?;
		if self.type_info.size_in_bytes == 0 {
			// Return something made-up for ZST
			//╰( ⁰ ਊ ⁰ )━☆ﾟ.*･｡ﾟ Does it matter what gets returned here?
			Some(mem::transmute(core::ptr::NonNull::<V>::dangling()))
		} else {
			// (ﾉ◕ヮ◕)ﾉ*:･ﾟ✧ *sparkles*
			let pseudo_store = mem::transmute::<&mut Vec<u8>, &mut Vec<V>>(&mut self.data);
			// I'll admit, there are other ways..
			Some(&mut pseudo_store[index])
		}
	}

	/// Claims ownership of the data, by copying it to the buffer and disabling its destructor.
	/// The destructor is run once the data gets actually deleted from the buffer.
	///
	/// When replacing an existing key, the previous data gets [Drop]ped, and overwritten.
	unsafe fn insert_or_replace<V>(&mut self, key: K, value: V) {
		let size_in_bytes = self.type_info.size_in_bytes;
		let raw_data = slice::from_raw_parts(&value as *const _ as *const u8, size_in_bytes);
		if let Some(&index) = self.indices.get(&key) {
			self.just_drop(&key);
			let range = self.range(index);
			let existing_data = &mut self.data[range];
			existing_data.copy_from_slice(raw_data);
		} else {
			if size_in_bytes == 0 {
				// Tag the key as existing, just make sure to not fetch any data for ZST
				self.indices.insert(key, 0);
			} else {
				self.indices.insert(key, self.data.len() / size_in_bytes);
				self.data.extend_from_slice(raw_data);
			}
		}
		mem::forget(value);
	}

	/// [Drop]s the element of the given key, and removes the data from storage.
	///
	/// Note that no type is generically provided, since this method needs to be called
	/// without type information when cleaning up.
	unsafe fn remove_and_drop(&mut self, key: &K) {
		self.just_drop(key);
		if self.type_info.size_in_bytes != 0 {
			if let Some(&index) = self.indices.get(key) {
				let range = self.range(index);
				self.data.drain(range);
				for v in self.indices.values_mut().filter(|&&mut v| v > index) {
					*v -= 1;
				}
			}
		}
		self.indices.remove(key);
	}

	/// [Drop]s the data using the initially defined drop function of this storages [TypeId].
	///
	/// Note that no type is generically provided, since this method needs to be called
	/// without type information when cleaning up.
	unsafe fn just_drop(&self, key: &K) {
		if let Some(&index) = self.indices.get(key) {
			let range = self.range(index);
			let data = self.data[range.start..].as_ptr() as *mut _;
			(self.type_info.drop_fn)(data);
		}
	}

	/// Returns the Range of bytes matching the underlying type, based on the index.
	fn range(&self, index: usize) -> core::ops::Range<usize> {
		index * self.type_info.size_in_bytes..(index + 1) * self.type_info.size_in_bytes
	}
}

impl<K> Drop for BlobStorage<K>
where
	K: Eq + Hash,
{
	fn drop(&mut self) {
		for key in self.indices.keys() {
			unsafe {
				self.just_drop(key);
			}
		}
	}
}
