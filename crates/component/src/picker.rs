//! [`Picker`]: a menu-like popover that is not a `PopupMenu` (a project
//! switcher, a model picker, a recents dropdown), built on
//! [`List`](crate::list::List), and the row parts for its body.
//!
//! The surface chrome belongs to the picker's [`Popover`](crate::popover); the
//! row parts never paint their own card.

mod element;
mod footer_item;
mod item;
mod section_header;
mod state;

#[cfg(test)]
mod item_tests;
#[cfg(test)]
mod tests;

pub use element::Picker;
pub use footer_item::PickerFooterItem;
pub use item::{PickerItem, PickerItemTruncate};
pub use section_header::PickerSectionHeader;
pub use state::{PickerEvent, PickerState};
