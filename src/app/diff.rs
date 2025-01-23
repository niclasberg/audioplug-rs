pub enum VecDiff<T> {
	Removed { index: usize, len: usize},
	Changed { index: usize, new_value: T },
	Inserted { index: usize, value: T }
}

/*fn diff_vecs<K: Hash + PartialEq>(a: &HashMap<K, usize>, b: &HashMap<K, usize>) {
	if a.is_empty() && b.is_empty() {

	}
}*/