//! An in-memory map index demonstrating `FoundationDB` tuple prefix scans.

use std::{collections::BTreeMap, marker::PhantomData};

use foundationdb_tuple::{Subspace, TuplePack};
use tuple_projections::{LeftProjectionOf, TupleProjection, TupleRepr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, TupleProjection)]
struct EventKey {
    tenant_id: i64,
    sequence_number: i64,
}

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

impl<K, V> MapIndex<K, V>
where
    K: TupleRepr + Clone,
    K::Tuple: TuplePack,
{
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.entries
            .insert(self.subspace.pack(&key.into_tuple()), value)
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.entries
            .get(&self.subspace.pack(&key.clone().into_tuple()))
    }

    fn contains_key(&self, key: &K) -> bool {
        self.entries
            .contains_key(&self.subspace.pack(&key.clone().into_tuple()))
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.entries
            .remove(&self.subspace.pack(&key.clone().into_tuple()))
    }

    /// Iterates keys beginning with `prefix` in `FoundationDB` tuple order.
    ///
    /// The projection bound checks at compile time that `prefix` is a leading
    /// part of the index's full key type. The range lookup is `O(log n + m)`
    /// for `m` matching keys. A full-key prefix is included as an exact match;
    /// `Subspace::range` itself covers strict descendants only.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "accept tuple literals by value for a concise prefix-query API"
    )]
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

    let first = EventKey {
        tenant_id: 7,
        sequence_number: 1,
    };
    let second = EventKey {
        tenant_id: 7,
        sequence_number: 2,
    };
    let other_tenant = EventKey {
        tenant_id: 8,
        sequence_number: 1,
    };

    assert_eq!(events.insert(first, "first event"), None);
    assert_eq!(events.insert(second, "second event"), None);
    assert_eq!(events.insert(other_tenant, "another tenant"), None);

    assert_eq!(events.get(&first), Some(&"first event"));
    assert!(events.contains_key(&second));

    // A one-element projection selects every event for tenant 7.
    let tenant_events: Vec<_> = events
        .iter((7_i64,))
        .map(|(key, value)| {
            let tuple = events
                .subspace
                .unpack::<<EventKey as TupleRepr>::Tuple>(key)
                .unwrap();
            (EventKey::from_tuple(tuple), *value)
        })
        .collect();
    assert_eq!(
        tenant_events,
        [(first, "first event"), (second, "second event")]
    );

    // The full key is also a valid projection and selects that exact map row.
    let exact: Vec<_> = events.iter((7_i64, 2_i64)).collect();
    assert_eq!(exact.len(), 1);
    let tuple = events
        .subspace
        .unpack::<<EventKey as TupleRepr>::Tuple>(exact[0].0)
        .unwrap();
    assert_eq!(EventKey::from_tuple(tuple), second);

    // Empty projection selects the full index.
    assert_eq!(events.iter(()).count(), 3);

    assert_eq!(events.remove(&first), Some("first event"));
    assert!(!events.contains_key(&first));
    events.clear();
    assert!(events.is_empty());
}
