//! A small FoundationDB-style ordered-key example using an in-memory `BTreeMap`.

use std::collections::BTreeMap;

use tuple_projections::LeftProjectionOf;

type EventKey = (u64, u64);
type TenantPrefix = (u64,);

/// Runtime prefix comparison for the key shapes used in this example.
trait PrefixMatches<K> {
    fn matches(&self, key: &K) -> bool;
}

impl PrefixMatches<EventKey> for () {
    fn matches(&self, _key: &EventKey) -> bool {
        true
    }
}

impl PrefixMatches<EventKey> for TenantPrefix {
    fn matches(&self, key: &EventKey) -> bool {
        self.0 == key.0
    }
}

impl PrefixMatches<EventKey> for EventKey {
    fn matches(&self, key: &EventKey) -> bool {
        self == key
    }
}

/// An in-memory map with tuple-prefix iteration.
struct MapIndex<K, V> {
    entries: BTreeMap<K, V>,
}

impl<K, V> Default for MapIndex<K, V> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}

impl<K: Ord, V> MapIndex<K, V> {
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.entries.insert(key, value)
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key)
    }

    fn contains_key(&self, key: &K) -> bool {
        self.entries.contains_key(key)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.entries.remove(key)
    }

    /// Iterates over entries matching a valid tuple prefix in `O(n)` time.
    fn iter<'a, P>(&'a self, prefix: P) -> impl Iterator<Item = (&'a K, &'a V)> + 'a
    where
        P: LeftProjectionOf<K> + PrefixMatches<K> + 'a,
    {
        self.entries
            .iter()
            .filter(move |(key, _)| prefix.matches(key))
    }

    fn clear(&mut self) {
        self.entries.clear();
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn main() {
    let mut events = MapIndex::<EventKey, _>::default();

    // A compound ordered key: (tenant_id, event_sequence).
    events.insert((7, 2), "second event");
    events.insert((3, 1), "another tenant's event");
    events.insert((7, 1), "first event");

    // Basic map CRUD.
    assert_eq!(events.get(&(7, 1)), Some(&"first event"));
    assert!(events.contains_key(&(7, 2)));
    assert_eq!(events.remove(&(3, 1)), Some("another tenant's event"));

    // The type system checks `(u64,)` as a left projection of `(u64, u64)`.
    // The example matcher then selects all entries for that tenant.
    let tenant_events = events.iter((7,)).collect::<Vec<_>>();
    assert_eq!(
        tenant_events,
        [(&(7, 1), &"first event"), (&(7, 2), &"second event")],
    );
    assert_eq!(events.iter((7, 1)).count(), 1);

    // An empty tuple is the prefix of every key.
    assert_eq!(events.iter(()).count(), 2);

    events.clear();
    assert!(events.is_empty());
}
