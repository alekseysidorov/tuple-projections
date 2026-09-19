//! A small FoundationDB-style ordered-key example using an in-memory `BTreeMap`.

use std::collections::BTreeMap;

use tuple_projections::LeftProjectionOf;

type EventKey = (u64, u64);
type TenantPrefix = (u64,);

/// Selects the range of keys whose first tuple element is the given tenant.
fn events_for_tenant<V>(
    events: &BTreeMap<EventKey, V>,
    (tenant,): TenantPrefix,
) -> impl Iterator<Item = (&EventKey, &V)> + '_
where
    TenantPrefix: LeftProjectionOf<EventKey, Remainder = (u64,)>,
{
    events.range((tenant, 0)..=(tenant, u64::MAX))
}

fn main() {
    let mut events = BTreeMap::new();

    // A compound ordered key: (tenant_id, event_sequence).
    events.insert((7, 2), "second event");
    events.insert((3, 1), "another tenant's event");
    events.insert((7, 1), "first event");

    // Basic map CRUD.
    assert_eq!(events.get(&(7, 1)), Some(&"first event"));
    assert!(events.contains_key(&(7, 2)));
    assert_eq!(events.remove(&(3, 1)), Some("another tenant's event"));

    // `(u64,)` is checked as a valid left projection of the full key `(u64, u64)`.
    let tenant_events = events_for_tenant(&events, (7,)).collect::<Vec<_>>();
    assert_eq!(
        tenant_events,
        [(&(7, 1), &"first event"), (&(7, 2), &"second event")],
    );

    events.clear();
    assert!(events.is_empty());
}
