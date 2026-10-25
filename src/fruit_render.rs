use std::cell::Cell;

use crate::{item::Item, perfect_fruit_policy::fruit_at_slot};

#[derive(Clone, Copy)]
struct Source {
    address: *const Item,
    fruit: Item,
    slots: u8,
}

thread_local! {
    static SOURCE: Cell<Option<Source>> = const { Cell::new(None) };
}

// Every fruit-info query invalidates the old record, including failed,
// forced and count-bearing queries. A stored pointer is only compared, never
// dereferenced: the native caller owns the temporary Item's lifetime.
pub(crate) fn clear() {
    SOURCE.with(|source| source.set(None));
}

pub(crate) fn remember(address: *const Item, fruit: Item, slots: u8) {
    SOURCE.with(|source| {
        source.set((slots != 0).then_some(Source {
            address,
            fruit,
            slots,
        }));
    });
}

#[must_use]
pub(crate) struct AttachmentScope(Option<Source>);

impl AttachmentScope {
    pub(crate) fn enter() -> Self {
        Self(SOURCE.with(|source| source.replace(None)))
    }

    pub(crate) fn choose(
        &mut self,
        address: *const Item,
        fruit: Item,
        slot: usize,
    ) -> Option<Item> {
        let source = self.0.take()?;
        if source.address != address || source.fruit != fruit || slot >= 3 {
            return None;
        }
        // Native regular-fruit attachment finishes at slot 2. Do not retain
        // the temporary address after its last use or after a mismatch.
        if slot < 2 {
            self.0 = Some(source);
        }
        Some(fruit_at_slot(fruit, source.slots, slot))
    }
}

impl Drop for AttachmentScope {
    fn drop(&mut self) {
        // Native attachment may nest another query/build. Restore the outer
        // record so its next slot cannot inherit the nested tree's selection.
        SOURCE.with(|source| source.set(self.0));
    }
}
