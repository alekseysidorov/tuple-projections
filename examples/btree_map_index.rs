//! An in-memory `MapIndex` using `FoundationDB`'s ordered tuple encoding.

use std::{collections::BTreeMap, marker::PhantomData};

use foundationdb_tuple::{Subspace, TuplePack};
use tuple_projections::LeftProjectionOf;

type EventKey = (i64, i64);
type TenantPrefix = (i64,);

/// A small in-memory map whose keys are encoded as `FoundationDB` tuples.
struct MapIndex<K, V> {
    subspace: Subspace,
    entries: BTreeMap<Vec<u8>, V>,
    key_type: PhantomData<fn() -> K>,
}

impl<K, V> MapIndex<K, V> {
    fn new(subspace: Subspace) -> Self {
        Self {
            subspace,
            entries: BTreeMap::new(),
            key_type: PhantomData,
        }
    }

    fn clear(&mut self) {
        self.entries.clear();
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<K: TuplePack, V> MapIndex<K, V> {
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.entries.insert(self.subspace.pack(&key), value)
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(&self.subspace.pack(key))
    }

    fn contains_key(&self, key: &K) -> bool {
        self.entries.contains_key(&self.subspace.pack(key))
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.entries.remove(&self.subspace.pack(key))
    }

    /// Iterates keys beginning with `prefix` in `FoundationDB` tuple order.
    ///
    /// The projection bound checks at compile time that `prefix` is a leading
    /// part of the index's full key type. The range lookup is `O(log n + m)`
    /// for `m` matching keys. A full-key prefix is included as an exact match;
    /// `Subspace::range` itself covers strict descendants only.
    #[allow(clippy::needless_pass_by_value)]
    fn iter<P>(&self, prefix: P) -> impl Iterator<Item = (&[u8], &V)>
    where
        P: TuplePack + LeftProjectionOf<K>,
    {
        let prefix_subspace = self.subspace.subspace(&prefix);
        let (begin, end) = prefix_subspace.range();
        let exact_key = self.subspace.pack(&prefix);
        let exact = self
            .entries
            .get_key_value(&exact_key)
            .map(|(key, value)| (key.as_slice(), value));
        let descendants = self
            .entries
            .range(begin..end)
            .map(|(key, value)| (key.as_slice(), value));

        exact.into_iter().chain(descendants)
    }
}

fn main() {
    let event_subspace = Subspace::all().subspace(&("events",));
    let mut events = MapIndex::<EventKey, _>::new(event_subspace);

    assert_eq!(events.insert((7, 1), "first event"), None);
    assert_eq!(events.insert((7, 2), "second event"), None);
    assert_eq!(events.insert((8, 1), "another tenant"), None);

    assert_eq!(events.get(&(7, 1)), Some(&"first event"));
    assert!(events.contains_key(&(7, 2)));

    // A one-element projection selects every event for tenant 7.
    let tenant_events: Vec<_> = events
        .iter((7_i64,))
        .map(|(key, value)| (events.subspace.unpack::<EventKey>(key).unwrap(), *value))
        .collect();
    assert_eq!(
        tenant_events,
        [((7, 1), "first event"), ((7, 2), "second event")]
    );

    // The full key is also a valid projection and selects that exact map row.
    let exact: Vec<_> = events.iter((7_i64, 2_i64)).collect();
    assert_eq!(exact.len(), 1);
    assert_eq!(
        events.subspace.unpack::<EventKey>(exact[0].0).unwrap(),
        (7, 2)
    );

    // Empty projection selects the full index.
    assert_eq!(events.iter(()).count(), 3);

    // A namespace can be independently unpacked as a tuple prefix.
    let tenant_subspace = events.subspace.subspace(&(7_i64,));
    let decoded_prefix: TenantPrefix = events.subspace.unpack(tenant_subspace.bytes()).unwrap();
    assert_eq!(decoded_prefix, (7,));

    assert_eq!(events.remove(&(7, 1)), Some("first event"));
    assert!(!events.contains_key(&(7, 1)));
    events.clear();
    assert!(events.is_empty());
}
