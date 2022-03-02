use alloc::vec::Vec;
use hashbrown::HashMap;

use core::{any::TypeId, cmp::Eq, hash::Hash, mem, slice};

pub struct TypeInfo {
	size_in_bytes: usize,
	drop_fn: unsafe fn(*mut u8),
}

impl TypeInfo {
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

	pub fn get_single<V: 'static>(&self, key: &K) -> Option<&V> {
		let entry = self.data_store.get(&TypeId::of::<V>())?;
		unsafe { entry.get(key) }
	}

	pub fn get_single_mut<V: 'static>(&mut self, key: &K) -> Option<&mut V> {
		let entry = self.data_store.get_mut(&TypeId::of::<V>())?;
		unsafe { entry.get_mut(key) }
	}

	pub fn insert_single<V: 'static>(&mut self, key: K, value: V) {
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

	pub fn remove<V: 'static>(&mut self, key: &K) {
		if let Some(entry) = self.data_store.get_mut(&TypeId::of::<V>()) {
			unsafe {
				entry.remove_and_drop(key);
			}
		}
	}

	pub fn remove_all(&mut self, key: &K) {
		for entry in self.data_store.values_mut() {
			unsafe {
				entry.remove_and_drop(key);
			}
		}
	}
}

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
		if mem::size_of::<V>() == 0 {
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
		if mem::size_of::<V>() == 0 {
			//╰( ⁰ ਊ ⁰ )━☆ﾟ.*･｡ﾟ Does it matter what gets returned here?
			Some(mem::transmute(core::ptr::NonNull::<V>::dangling()))
		} else {
			// (ﾉ◕ヮ◕)ﾉ*:･ﾟ✧ *sparkles*
			let pseudo_store = mem::transmute::<&mut Vec<u8>, &mut Vec<V>>(&mut self.data);
			// I'll admit, there are other ways..
			Some(&mut pseudo_store[index])
		}
	}

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

	fn range(&self, index: usize) -> core::ops::Range<usize> {
		index * self.type_info.size_in_bytes..(index + 1) * self.type_info.size_in_bytes
	}

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

	unsafe fn just_drop(&self, key: &K) {
		if let Some(&index) = self.indices.get(key) {
			let range = self.range(index);
			let data = self.data[range.start..].as_ptr() as *mut _;
			(self.type_info.drop_fn)(data);
		}
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
