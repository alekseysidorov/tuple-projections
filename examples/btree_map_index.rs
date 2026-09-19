//! An in-memory, `BTreeMap`-backed sketch of an Exonum-style `MapIndex`.
//!
//! Unlike Exonum's persistent and optionally merkelized indexes, this example
//! only demonstrates ordered key/value operations and a statically checked key
//! prefix.

use std::{
    collections::{BTreeMap, btree_map::Range},
    marker::PhantomData,
    ops::RangeBounds,
};

use tuple_projections::{LeftProjectionOf, TupleProjection, TupleRepr};

/// A map that stores each key using its canonical tuple representation.
#[derive(Debug)]
struct MapIndex<K, V>
where
    K: TupleRepr,
    K::Tuple: Ord,
{
    entries: BTreeMap<K::Tuple, V>,
    key_type: PhantomData<fn() -> K>,
}

impl<K, V> Default for MapIndex<K, V>
where
    K: TupleRepr,
    K::Tuple: Ord,
{
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            key_type: PhantomData,
        }
    }
}

impl<K, V> MapIndex<K, V>
where
    K: TupleRepr,
    K::Tuple: Ord,
{
    fn put(&mut self, key: K, value: V) -> Option<V> {
        self.entries.insert(key.into_tuple(), value)
    }

    fn get(&self, key: &K::Tuple) -> Option<&V> {
        self.entries.get(key)
    }

    fn contains(&self, key: &K::Tuple) -> bool {
        self.entries.contains_key(key)
    }

    fn remove(&mut self, key: &K::Tuple) -> Option<V> {
        self.entries.remove(key)
    }

    fn iter(&self) -> impl Iterator<Item = (&K::Tuple, &V)> {
        self.entries.iter()
    }

    fn iter_from(&self, key: &K::Tuple) -> Range<'_, K::Tuple, V>
    where
        K::Tuple: Clone,
    {
        self.entries.range(key.clone()..)
    }

    fn range<R>(&self, range: R) -> Range<'_, K::Tuple, V>
    where
        R: RangeBounds<K::Tuple>,
    {
        self.entries.range(range)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn clear(&mut self) {
        self.entries.clear();
    }
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, TupleProjection)]
struct EventKey {
    tenant: u64,
    sequence: u64,
}

type TenantPrefix = (u64,);

/// Returns events for one tenant; the trait bound rejects non-prefix tuple shapes.
fn tenant_entries<V>(
    index: &MapIndex<EventKey, V>,
    (tenant,): TenantPrefix,
) -> impl Iterator<Item = (&(u64, u64), &V)> + '_
where
    TenantPrefix: LeftProjectionOf<EventKey, Remainder = (u64,)>,
{
    index.range((tenant, 0)..=(tenant, u64::MAX))
}

fn main() {
    let mut events = MapIndex::<EventKey, _>::default();
    events.put(
        EventKey {
            tenant: 7,
            sequence: 2,
        },
        "second",
    );
    events.put(
        EventKey {
            tenant: 3,
            sequence: 1,
        },
        "first",
    );
    events.put(
        EventKey {
            tenant: 7,
            sequence: 1,
        },
        "first for tenant 7",
    );

    assert_eq!(events.get(&(3, 1)), Some(&"first"));
    assert!(events.contains(&(7, 1)));
    assert_eq!(
        tenant_entries(&events, (7,)).collect::<Vec<_>>(),
        [(&(7, 1), &"first for tenant 7"), (&(7, 2), &"second")],
    );
    assert_eq!(events.iter_from(&(7, 1)).count(), 2);
    assert_eq!(events.iter().count(), events.len());
    assert_eq!(events.remove(&(3, 1)), Some("first"));

    events.clear();
    assert!(events.is_empty());
}
