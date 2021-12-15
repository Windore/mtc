//! # MTC Lib
//!
//! This API provides the base functionality for mtc. It can be also used for creating apps that
//! can sync with the mtc CLI app or serve as an additional interface.

#![warn(missing_docs)]

use chrono::prelude::*;
use serde::{Deserialize, Serialize};

mod items;
pub use crate::items::*;

mod remote;
pub use crate::remote::*;

/// An Item for MtcList. A struct implementing MtcItem is usually defined for a specific time and it has a state.
pub trait MtcItem {
    /// Returns true if the item is for a given date.
    ///
    /// # Example
    /// ```
    /// use chrono::prelude::*;
    /// use mtc::{MtcItem,ItemState};
    ///
    /// struct WeekdayItem {
    ///     weekday: Weekday,
    /// }
    ///
    /// impl MtcItem for WeekdayItem {
    ///     fn for_date(&self, date: NaiveDate) -> bool {
    ///         self.weekday == date.weekday()
    ///     }
    ///     fn state(&self) -> ItemState { todo!() }
    ///     fn set_state(&mut self, state: ItemState) { todo!() }
    ///     fn ignore_state_eq(&self, other: &Self) -> bool { todo!() }
    ///     fn id(&self) -> usize { 0 }
    ///     fn set_id(&mut self, id: usize) {}
    /// }
    ///
    /// assert!(WeekdayItem { weekday: Weekday::Mon }.for_date(NaiveDate::from_ymd(2021, 12, 6)));
    /// assert!(!WeekdayItem { weekday: Weekday::Mon }.for_date(NaiveDate::from_ymd(2021, 12, 5)));
    /// ```
    fn for_date(&self, date: NaiveDate) -> bool;
    /// Returns true if the item is for today.
    ///
    /// # Example
    /// ```
    /// use chrono::prelude::*;
    /// use mtc::{MtcItem,ItemState};
    ///
    /// struct WeekdayItem {
    ///     weekday: Weekday,
    /// }
    ///
    /// impl MtcItem for WeekdayItem {
    ///     fn for_date(&self, date: NaiveDate) -> bool {
    ///         self.weekday == date.weekday()
    ///     }
    ///     fn state(&self) -> ItemState { todo!() }
    ///     fn set_state(&mut self, state: ItemState) { todo!() }
    ///     fn ignore_state_eq(&self, other: &Self) -> bool { todo!() }
    ///     fn id(&self) -> usize { 0 }
    ///     fn set_id(&mut self, id: usize) {}
    /// }
    ///
    /// assert!(WeekdayItem { weekday: Local::today().weekday() }.for_today());
    /// ```
    fn for_today(&self) -> bool {
        self.for_date(Local::today().naive_local())
    }
    /// Returns true if the item is for a given weekday.
    ///
    /// # Example
    /// ```
    /// use chrono::prelude::*;
    /// use mtc::{MtcItem,ItemState};
    ///
    /// struct WeekdayItem {
    ///     weekday: Weekday,
    /// }
    ///
    /// impl MtcItem for WeekdayItem {
    ///     fn for_date(&self, date: NaiveDate) -> bool {
    ///         self.weekday == date.weekday()
    ///     }
    ///     fn state(&self) -> ItemState { todo!() }
    ///     fn set_state(&mut self, state: ItemState) { todo!() }
    ///     fn ignore_state_eq(&self, other: &Self) -> bool { todo!() }
    ///     fn id(&self) -> usize { 0 }
    ///     fn set_id(&mut self, id: usize) {}
    /// }
    ///
    /// assert!(WeekdayItem { weekday: Weekday::Fri }.for_weekday(Weekday::Fri));
    /// assert!(!WeekdayItem { weekday: Weekday::Fri }.for_weekday(Weekday::Mon));
    /// ```
    fn for_weekday(&self, weekday: Weekday) -> bool {
        let mut weekday_date = Local::today().naive_local();
        while weekday_date.weekday() != weekday {
            weekday_date = weekday_date.succ();
        }
        self.for_date(weekday_date)
    }
    /// Returns the `ItemState` of the item.
    fn state(&self) -> ItemState;
    /// Sets the `ItemState` of the item.
    fn set_state(&mut self, state: ItemState);
    /// Compares the MtcItems while ignoring the state.
    ///
    /// # Example
    /// ```
    /// use chrono::prelude::{Weekday, NaiveDate};
    /// use mtc::{MtcItem, ItemState};
    ///
    /// struct TodoItem {
    ///     weekday: Option<Weekday>,
    ///     body: String,
    ///     state: ItemState,
    /// }
    ///
    /// impl MtcItem for TodoItem {
    ///     fn ignore_state_eq(&self, other: &TodoItem) -> bool {
    ///         self.body == other.body && self.weekday == other.weekday
    ///     }
    ///     fn for_date(&self, date: NaiveDate) -> bool { todo!() }
    ///     fn state(&self) -> ItemState { todo!() }
    ///     fn set_state(&mut self, state: ItemState) { todo!() }
    ///     fn id(&self) -> usize { 0 }
    ///     fn set_id(&mut self, id: usize) {}
    /// }
    ///
    /// let item1 = TodoItem {
    ///     weekday: Some(Weekday::Mon),
    ///     body: "Item".to_string(),
    ///     state: ItemState::New,
    /// };
    ///
    /// let item2 = TodoItem {
    ///     weekday: Some(Weekday::Mon),
    ///     body: "Item".to_string(),
    ///     state: ItemState::Neutral,
    /// };
    ///
    /// assert!(item1.ignore_state_eq(&item2));
    /// ```
    fn ignore_state_eq(&self, other: &Self) -> bool;
    /// Gets the id of the item. Returns -1 if the item is not in a `MtcList`
    fn id(&self) -> usize;
    /// Sets the id of the items. `MtcList` usually handles setting the id so in most cases calling this manually is not needed.
    fn set_id(&mut self, new_id: usize);
}

/// A state of an MtcItem used for synchronising MtcLists correctly
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum ItemState {
    /// MtcItem is a new item and the server list does not contain it
    New,
    /// MtcItem is removed and if it exists on the server it should be removed from there too.
    Removed,
    /// MtcItem is not new nor should it be removed. If it doesn't exist on the server it will be removed.
    Neutral,
}

/// A wrapper for a Vec containing MtcItems. The wrapper helps to manage the state of the items and sync them correctly.
/// A MtcList can be either a client or a server list which affect the functionality of the list. Server lists don't track
/// the state since multiple clients could be interacting with the same server.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MtcList<T: MtcItem + Clone> {
    items: Vec<T>,
    is_server: bool,
}

impl<T: MtcItem + Clone> MtcList<T> {
    /// Creates a new MtcList which will be either a server or a client.
    pub fn new(is_server: bool) -> MtcList<T> {
        MtcList {
            items: Vec::new(),
            is_server,
        }
    }

    /// Appends a new MtcItem to the list setting the items state to new.
    pub fn add(&mut self, mut item: T) {
        if self.is_server {
            item.set_state(ItemState::Neutral);
        } else {
            item.set_state(ItemState::New);
        }
        item.set_id(self.items.len());
        self.items.push(item);
    }

    /// Marks a MtcItem of a given id to be removed. The id is the same as the index in the inner `Vec`. Returns `Err(&str)` if index is out of bounds.
    pub fn mark_removed(&mut self, id: usize) -> Result<(), &str> {
        if let Some(item) = self.items.get_mut(id) {
            if self.is_server {
                drop(item);
                self.items.remove(id);
                self.map_indices_to_ids();
            } else {
                if item.state() == ItemState::Removed {
                    // Return err even when such item exists because it is in a way removed.
                    return Err("No item with the given id found.");
                }
                item.set_state(ItemState::Removed);
            }
            Ok(())
        } else {
            Err("No item with the given id found.")
        }
    }

    /// Returns a new vector containing references to all items within this list in the same order.
    pub fn items(&self) -> Vec<&T> {
        let mut new = Vec::new();

        for item in &self.items {
            if item.state() != ItemState::Removed {
                new.push(item);
            }
        }

        new
    }

    /// Returns a new vector containing references to all items that are for a given date.
    pub fn items_for_date(&self, date: NaiveDate) -> Vec<&T> {
        self.items()
            .into_iter()
            .filter(|item| item.for_date(date))
            .collect()
    }

    /// Return a new vector containing references to all items that are for today.
    pub fn items_for_today(&self) -> Vec<&T> {
        self.items()
            .into_iter()
            .filter(|item| item.for_today())
            .collect()
    }

    /// Return a vector containing references to all items that are for a given weekday.
    pub fn items_for_weekday(&self, weekday: Weekday) -> Vec<&T> {
        self.items()
            .into_iter()
            .filter(|item| item.for_weekday(weekday))
            .collect()
    }

    /// Returns a clone of self but as a server
    pub fn clone_to_server(&self) -> MtcList<T> {
        let mut clone = self.clone();
        clone.is_server = true;
        clone.sync_self();

        clone
    }

    /// Synchronizes the list with itself by removing all items with the Removed state and setting the state of the rest to Neutral.
    pub fn sync_self(&mut self) {
        self.items.retain(|item| item.state() != ItemState::Removed);
        for (i, item) in self.items.iter_mut().enumerate() {
            item.set_state(ItemState::Neutral);
            item.set_id(i);
        }
    }

    /// Synchronizes this MtcList with the other MtcList.
    /// Either one of these lists is expected to be a server and the other a client.
    /// Removes items that are marked for removal.
    ///
    /// # Panics
    ///
    /// If neither one of the lists is a server or if both are servers.
    ///
    /// # Examples
    ///
    /// ```
    /// use mtc::{MtcList, TodoItem};
    /// use chrono::prelude::Weekday;
    ///
    /// let mut client_list = MtcList::new(false);
    ///
    /// client_list.add(TodoItem::new("Task 1".to_string(), None));
    /// client_list.add(TodoItem::new("Task 2".to_string(), Some(Weekday::Mon)));
    /// client_list.add(TodoItem::new("Task 3".to_string(), Some(Weekday::Fri)));
    ///
    /// // Set the state of all items in the client list from New to Neutral by using sync_self
    ///
    /// client_list.sync_self();
    ///
    /// // This one will have the state New
    /// client_list.add(TodoItem::new("Task 4".to_string(), Some(Weekday::Sat)));
    ///
    /// // Set the Task 2 element to Removed
    /// client_list.mark_removed(1); // It will have the index of 1
    ///
    /// let mut server_list = MtcList::new(true);
    /// server_list.add(TodoItem::new("Task 1".to_string(), None));
    /// server_list.add(TodoItem::new("Task 2".to_string(), Some(Weekday::Mon)));
    /// server_list.add(TodoItem::new("Task 5".to_string(), Some(Weekday::Mon)));
    ///
    /// client_list.sync(&mut server_list);
    /// // server_list.sync(&mut client_list); may be used as well. There is no difference
    ///
    /// // The operation should result in the following list. Both server and client will have same items but the order may be different.
    /// let mut resultting_client_list = MtcList::new(false);
    /// resultting_client_list.add(TodoItem::new("Task 1".to_string(), None));
    /// resultting_client_list.add(TodoItem::new("Task 4".to_string(), Some(Weekday::Sat)));
    /// resultting_client_list.add(TodoItem::new("Task 5".to_string(), Some(Weekday::Mon)));
    ///
    /// // The resultting list needs to be self synced since the items otherwise would have the state New
    /// resultting_client_list.sync_self();
    ///
    /// // The only difference between the lists is the is_server field of both lists which is not a result of the sync function.
    /// let mut resultting_server_list = MtcList::new(true);
    /// resultting_server_list.add(TodoItem::new("Task 1".to_string(), None));
    /// resultting_server_list.add(TodoItem::new("Task 5".to_string(), Some(Weekday::Mon)));
    /// resultting_server_list.add(TodoItem::new("Task 4".to_string(), Some(Weekday::Sat)));
    ///
    /// assert_eq!(client_list, resultting_client_list);
    /// assert_eq!(server_list, resultting_server_list);
    /// ```
    pub fn sync(&mut self, other: &mut MtcList<T>) {
        if self.is_server && other.is_server {
            panic!("Both self and other are servers.");
        } else if !self.is_server && !other.is_server {
            panic!("Neither self or other is a server.");
        }

        let server_list;
        let client_list;
        if self.is_server {
            server_list = self;
            client_list = other
        } else {
            server_list = other;
            client_list = self;
        }

        for item in client_list.items.iter_mut() {
            match item.state() {
                ItemState::Removed => {
                    // Remove same item from server list if it exists
                    for elem in server_list.items.iter_mut() {
                        // All servers have only neutral items so if an item is removed that indicates that there are duplicate items.
                        // This prevents duplicate items blocking others removal.
                        if elem.ignore_state_eq(item) && elem.state() != ItemState::Removed {
                            elem.set_state(ItemState::Removed);
                            break;
                        }
                    }
                }
                ItemState::New => {
                    server_list.add(item.clone());
                }
                ItemState::Neutral => {
                    // Check if server list contains the item. If not it should be removed.
                    if server_list
                        .items
                        .iter()
                        .position(|elem| elem.ignore_state_eq(item))
                        .is_none()
                    {
                        item.set_state(ItemState::Removed);
                    }
                }
            };
        }

        // Add items from server that don't yet exist on the client.

        for item in server_list.items.iter() {
            // Go through every non removed item in the server list.
            if item.state() != ItemState::Removed {
                let mut should_add = true;
                // Check for a similar item in the client list.
                for elem in client_list.items.iter() {
                    // Don't add the item if a non removed similar item already exists in the client list.
                    if elem.ignore_state_eq(item) && elem.state() != ItemState::Removed {
                        should_add = false;
                        break;
                    }
                }
                if should_add {
                    client_list.add(item.clone());
                }
            }
        }

        client_list.sync_self();
        server_list.sync_self();
    }

    fn map_indices_to_ids(&mut self) {
        for (i, item) in self.items.iter_mut().enumerate() {
            item.set_id(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mtc_item_for_today_returns_true() {
        struct TestItem {}
        impl MtcItem for TestItem {
            fn for_date(&self, date: NaiveDate) -> bool {
                date == Local::today().naive_local()
            }
            fn state(&self) -> ItemState {
                todo!()
            }
            fn set_state(&mut self, _: ItemState) {
                todo!()
            }
            fn ignore_state_eq(&self, _: &Self) -> bool {
                todo!()
            }
            fn id(&self) -> usize {
                todo!()
            }
            fn set_id(&mut self, _: usize) {
                todo!()
            }
        }

        let item = TestItem {};

        assert!(item.for_today());
    }

    #[test]
    fn mtc_item_for_today_returns_false() {
        struct TestItem {}
        impl MtcItem for TestItem {
            fn for_date(&self, date: NaiveDate) -> bool {
                date != Local::today().naive_local()
            }
            fn state(&self) -> ItemState {
                todo!()
            }
            fn set_state(&mut self, _: ItemState) {
                todo!()
            }
            fn ignore_state_eq(&self, _: &Self) -> bool {
                todo!()
            }
            fn id(&self) -> usize {
                todo!()
            }
            fn set_id(&mut self, _: usize) {
                todo!()
            }
        }

        let item = TestItem {};

        assert!(!item.for_today());
    }

    #[test]
    fn mtc_item_for_weekday_returns_true() {
        struct TestItem {}
        impl MtcItem for TestItem {
            fn for_date(&self, date: NaiveDate) -> bool {
                date.weekday() == Weekday::Mon
            }
            fn state(&self) -> ItemState {
                todo!()
            }
            fn set_state(&mut self, _: ItemState) {
                todo!()
            }
            fn ignore_state_eq(&self, _: &Self) -> bool {
                todo!()
            }
            fn id(&self) -> usize {
                todo!()
            }
            fn set_id(&mut self, _: usize) {
                todo!()
            }
        }

        let item = TestItem {};

        assert!(item.for_weekday(Weekday::Mon));
    }

    #[test]
    fn mtc_item_for_weekday_returns_false() {
        struct TestItem {}
        impl MtcItem for TestItem {
            fn for_date(&self, date: NaiveDate) -> bool {
                date == NaiveDate::from_ymd(2021, 12, 6)
            }
            fn state(&self) -> ItemState {
                todo!()
            }
            fn set_state(&mut self, _: ItemState) {
                todo!()
            }
            fn ignore_state_eq(&self, _: &Self) -> bool {
                todo!()
            }
            fn id(&self) -> usize {
                0
            }
            fn set_id(&mut self, _: usize) {}
        }

        let item = TestItem {};

        assert!(!item.for_weekday(Weekday::Fri));
    }

    #[test]
    fn mtc_list_for_date_returns_expected() {
        let mut list = MtcList::new(true);
        list.add(TodoItem::new("test0".to_string(), None));
        list.add(TodoItem::new("test1".to_string(), Some(Weekday::Tue)));
        list.add(TodoItem::new("test2".to_string(), Some(Weekday::Wed)));
        list.add(TodoItem::new("test3".to_string(), None));
        list.add(TodoItem::new("test4".to_string(), Some(Weekday::Tue)));

        let mut expected = vec![
            TodoItem::new("test0".to_string(), None),
            TodoItem::new("test1".to_string(), Some(Weekday::Tue)),
            TodoItem::new("test3".to_string(), None),
            TodoItem::new("test4".to_string(), Some(Weekday::Tue)),
        ];

        expected[0].set_id(0);
        expected[1].set_id(1);
        expected[2].set_id(3);
        expected[3].set_id(4);

        let result: Vec<TodoItem> = list
            .items_for_date(NaiveDate::from_ymd(2021, 11, 30))
            .iter()
            .cloned()
            .cloned()
            .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn mtc_list_for_today_returns_expected() {
        let today = Local::now().weekday();

        let mut items = MtcList::new(true);
        items.add(Task::new("test0".to_string(), 40, Some(today)));
        items.add(Task::new("test1".to_string(), 30, Some(today)));
        items.add(Task::new("test2".to_string(), 10, Some(today.pred())));
        items.add(Task::new("test3".to_string(), 0, None));
        items.add(Task::new(
            "test4".to_string(),
            90,
            Some(today.succ().succ()),
        ));

        let mut expected = vec![
            Task::new("test0".to_string(), 40, Some(today)),
            Task::new("test1".to_string(), 30, Some(today)),
            Task::new("test3".to_string(), 0, None),
        ];

        expected[0].set_id(0);
        expected[1].set_id(1);
        expected[2].set_id(3);

        let result: Vec<Task> = items.items_for_today().iter().cloned().cloned().collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn mtc_list_for_weekday_returns_expected() {
        let mut items = MtcList::new(true);
        items.add(Task::new("test0".to_string(), 40, Some(Weekday::Fri)));
        items.add(Task::new("test1".to_string(), 30, Some(Weekday::Fri)));
        items.add(Task::new("test2".to_string(), 10, Some(Weekday::Mon)));
        items.add(Task::new("test3".to_string(), 0, None));
        items.add(Task::new("test4".to_string(), 90, Some(Weekday::Wed)));

        let mut expected = vec![
            Task::new("test0".to_string(), 40, Some(Weekday::Fri)),
            Task::new("test1".to_string(), 30, Some(Weekday::Fri)),
            Task::new("test3".to_string(), 0, None),
        ];

        expected[0].set_id(0);
        expected[1].set_id(1);
        expected[2].set_id(3);

        let result: Vec<Task> = items
            .items_for_weekday(Weekday::Fri)
            .iter()
            .cloned()
            .cloned()
            .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn mtc_list_for_date_returns_expected_for_events() {
        let mut items = MtcList::new(true);
        items.add(Event::new(
            "test0".to_string(),
            NaiveDate::from_ymd(2021, 10, 5),
        ));
        items.add(Event::new(
            "test1".to_string(),
            NaiveDate::from_ymd(2021, 11, 5),
        ));
        items.add(Event::new(
            "test2".to_string(),
            NaiveDate::from_ymd(2021, 10, 6),
        ));
        items.add(Event::new(
            "test3".to_string(),
            NaiveDate::from_ymd(2021, 10, 5),
        ));
        items.add(Event::new(
            "test4".to_string(),
            NaiveDate::from_ymd(2021, 10, 5),
        ));
        items.add(Event::new(
            "test5".to_string(),
            NaiveDate::from_ymd(2020, 10, 5),
        ));
        items.add(Event::new(
            "test6".to_string(),
            NaiveDate::from_ymd(2020, 11, 30),
        ));

        let mut expected = vec![
            Event::new("test0".to_string(), NaiveDate::from_ymd(2021, 10, 5)),
            Event::new("test3".to_string(), NaiveDate::from_ymd(2021, 10, 5)),
            Event::new("test4".to_string(), NaiveDate::from_ymd(2021, 10, 5)),
        ];

        expected[0].set_id(0);
        expected[1].set_id(3);
        expected[2].set_id(4);

        let result: Vec<Event> = items
            .items_for_date(NaiveDate::from_ymd(2021, 10, 5))
            .iter()
            .cloned()
            .cloned()
            .collect();

        assert_eq!(result, expected);
    }

    #[derive(Debug, PartialEq, Clone)]
    struct TestMtcItem {
        state: ItemState,
        body: String,
    }

    impl Ord for TestMtcItem {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.body.cmp(&other.body)
        }
    }

    impl PartialOrd for TestMtcItem {
        fn partial_cmp(&self, other: &Self) -> std::option::Option<std::cmp::Ordering> {
            Some(self.body.cmp(&other.body))
        }
    }

    impl Eq for TestMtcItem {}

    impl TestMtcItem {
        fn new(body: String) -> TestMtcItem {
            TestMtcItem {
                state: ItemState::Neutral,
                body,
            }
        }
    }

    impl MtcItem for TestMtcItem {
        fn for_date(&self, _: chrono::NaiveDate) -> bool {
            todo!()
        }
        fn state(&self) -> ItemState {
            self.state
        }
        fn set_state(&mut self, state: ItemState) {
            self.state = state;
        }
        fn ignore_state_eq(&self, other: &Self) -> bool {
            self.body == other.body
        }
        fn id(&self) -> usize {
            0
        }
        fn set_id(&mut self, _: usize) {}
    }

    #[test]
    fn mtc_list_server_all_neutral() {
        let mut list = MtcList::new(true);

        list.add(TestMtcItem::new("Item 0".to_string()));
        list.add(TestMtcItem::new("Item 1".to_string()));
        list.add(TestMtcItem::new("Item 2".to_string()));

        let mut exp: Vec<TestMtcItem> = vec![
            TestMtcItem::new("Item 0".to_string()),
            TestMtcItem::new("Item 1".to_string()),
            TestMtcItem::new("Item 2".to_string()),
        ];
        exp.iter_mut().for_each(|x| {
            x.set_state(ItemState::Neutral);
        });

        let mut sorted: Vec<TestMtcItem> = list.items().iter().cloned().cloned().collect();
        sorted.sort();

        assert_eq!(sorted, exp);
    }

    #[test]
    fn mtc_list_self_sync_removes_marked_and_sets_new_neutral() {
        let mut list = MtcList::new(false);

        list.add(TestMtcItem::new("Item 0".to_string()));
        list.add(TestMtcItem::new("Item 1".to_string()));
        list.add(TestMtcItem::new("Item 2".to_string()));
        list.add(TestMtcItem::new("Item 3".to_string()));
        list.add(TestMtcItem::new("Item 4".to_string()));

        list.mark_removed(1).unwrap();
        list.mark_removed(2).unwrap();

        list.sync_self();

        let mut exp: Vec<TestMtcItem> = vec![
            TestMtcItem::new("Item 0".to_string()),
            TestMtcItem::new("Item 3".to_string()),
            TestMtcItem::new("Item 4".to_string()),
        ];
        exp.iter_mut().for_each(|x| {
            x.set_state(ItemState::Neutral);
        });

        let mut sorted: Vec<TestMtcItem> = list.items().iter().cloned().cloned().collect();
        sorted.sort();

        assert_eq!(sorted, exp);
    }

    #[test]
    fn mtc_list_sync_removes_marked_from_server() {
        let mut client_list = MtcList::new(false);
        let mut server_list = MtcList::new(true);

        client_list.add(TestMtcItem::new("Item 0".to_string()));
        client_list.add(TestMtcItem::new("Item 1".to_string()));
        client_list.add(TestMtcItem::new("Item 2".to_string()));

        client_list.sync_self();

        client_list.mark_removed(1).unwrap();
        client_list.mark_removed(2).unwrap();

        server_list.add(TestMtcItem::new("Item 0".to_string()));
        server_list.add(TestMtcItem::new("Item 1".to_string()));

        client_list.sync(&mut server_list);

        let mut exp: Vec<TestMtcItem> = vec![TestMtcItem::new("Item 0".to_string())];
        exp.iter_mut().for_each(|x| {
            x.set_state(ItemState::Neutral);
        });

        let mut sorted: Vec<TestMtcItem> = client_list.items().iter().cloned().cloned().collect();
        sorted.sort();

        assert_eq!(sorted, exp);
        let mut sorted: Vec<TestMtcItem> = server_list.items().iter().cloned().cloned().collect();
        sorted.sort();

        assert_eq!(sorted, exp);
    }

    #[test]
    fn mtc_list_sync_removes_marked_from_server_only_once() {
        let mut client_list = MtcList::new(false);
        let mut server_list = MtcList::new(true);

        client_list.add(TestMtcItem::new("Item 0".to_string()));
        client_list.add(TestMtcItem::new("Item 1".to_string()));

        client_list.sync_self();

        client_list.mark_removed(1).unwrap();

        server_list.add(TestMtcItem::new("Item 0".to_string()));
        server_list.add(TestMtcItem::new("Item 1".to_string()));
        server_list.add(TestMtcItem::new("Item 1".to_string()));

        server_list.sync(&mut client_list);

        let mut exp: Vec<TestMtcItem> = vec![
            TestMtcItem::new("Item 0".to_string()),
            TestMtcItem::new("Item 1".to_string()),
        ];
        exp.iter_mut().for_each(|x| {
            x.set_state(ItemState::Neutral);
        });

        let mut sorted: Vec<TestMtcItem> = client_list.items().iter().cloned().cloned().collect();
        sorted.sort();

        assert_eq!(sorted, exp);
        let mut sorted: Vec<TestMtcItem> = server_list.items().iter().cloned().cloned().collect();
        sorted.sort();

        assert_eq!(sorted, exp);
    }

    #[test]
    fn mtc_list_sync_removes_marked_from_server_equal_times() {
        let mut client_list = MtcList::new(false);
        let mut server_list = MtcList::new(true);

        client_list.add(TestMtcItem::new("Item 0".to_string()));

        client_list.sync_self();

        client_list.add(TestMtcItem::new("Item 1".to_string()));
        client_list.add(TestMtcItem::new("Item 1".to_string()));

        client_list.mark_removed(1).unwrap();
        client_list.mark_removed(2).unwrap();

        server_list.add(TestMtcItem::new("Item 0".to_string()));
        server_list.add(TestMtcItem::new("Item 1".to_string()));
        server_list.add(TestMtcItem::new("Item 1".to_string()));

        client_list.sync(&mut server_list);

        let mut exp: Vec<TestMtcItem> = vec![TestMtcItem::new("Item 0".to_string())];
        exp.iter_mut().for_each(|x| {
            x.set_state(ItemState::Neutral);
        });

        let mut sorted: Vec<TestMtcItem> = client_list.items().iter().cloned().cloned().collect();
        sorted.sort();

        assert_eq!(sorted, exp);
        let mut sorted: Vec<TestMtcItem> = server_list.items().iter().cloned().cloned().collect();
        sorted.sort();

        assert_eq!(sorted, exp);
    }

    #[test]
    fn mtc_list_sync_removes_nonnew_from_client() {
        let mut client_list = MtcList::new(false);
        let mut server_list = MtcList::new(true);

        client_list.add(TestMtcItem::new("Item 0".to_string()));
        client_list.add(TestMtcItem::new("Item 1".to_string()));
        client_list.add(TestMtcItem::new("Item 2".to_string()));

        client_list.sync_self(); // Set items to neutral

        server_list.add(TestMtcItem::new("Item 2".to_string()));

        server_list.sync(&mut client_list);

        let mut exp: Vec<TestMtcItem> = vec![TestMtcItem::new("Item 2".to_string())];
        exp.iter_mut().for_each(|x| {
            x.set_state(ItemState::Neutral);
        });

        let mut sorted: Vec<TestMtcItem> = client_list.items().iter().cloned().cloned().collect();
        sorted.sort();

        assert_eq!(sorted, exp);
        let mut sorted: Vec<TestMtcItem> = server_list.items().iter().cloned().cloned().collect();
        sorted.sort();

        assert_eq!(sorted, exp);
    }

    #[test]
    fn mtc_list_sync_adds_new_from_client() {
        let mut client_list = MtcList::new(false);
        let mut server_list = MtcList::new(true);

        client_list.add(TestMtcItem::new("Item 0".to_string()));

        client_list.sync_self();

        client_list.add(TestMtcItem::new("Item 1".to_string()));
        client_list.add(TestMtcItem::new("Item 2".to_string()));

        server_list.add(TestMtcItem::new("Item 0".to_string()));

        client_list.sync(&mut server_list);

        let mut exp: Vec<TestMtcItem> = vec![
            TestMtcItem::new("Item 0".to_string()),
            TestMtcItem::new("Item 1".to_string()),
            TestMtcItem::new("Item 2".to_string()),
        ];
        exp.iter_mut().for_each(|x| {
            x.set_state(ItemState::Neutral);
        });

        let mut sorted: Vec<TestMtcItem> = client_list.items().iter().cloned().cloned().collect();
        sorted.sort();

        assert_eq!(sorted, exp);
        let mut sorted: Vec<TestMtcItem> = server_list.items().iter().cloned().cloned().collect();
        sorted.sort();

        assert_eq!(sorted, exp);
    }

    #[test]
    fn mtc_list_sync_adds_new_from_server() {
        let mut client_list = MtcList::new(false);
        let mut server_list = MtcList::new(true);

        client_list.add(TestMtcItem::new("Item 0".to_string()));

        client_list.sync_self();

        server_list.add(TestMtcItem::new("Item 0".to_string()));
        server_list.add(TestMtcItem::new("Item 1".to_string()));
        server_list.add(TestMtcItem::new("Item 2".to_string()));

        client_list.sync(&mut server_list);

        let mut exp: Vec<TestMtcItem> = vec![
            TestMtcItem::new("Item 0".to_string()),
            TestMtcItem::new("Item 1".to_string()),
            TestMtcItem::new("Item 2".to_string()),
        ];
        exp.iter_mut().for_each(|x| {
            x.set_state(ItemState::Neutral);
        });

        let mut sorted: Vec<TestMtcItem> = client_list.items().iter().cloned().cloned().collect();
        sorted.sort();

        assert_eq!(sorted, exp);
        let mut sorted: Vec<TestMtcItem> = server_list.items().iter().cloned().cloned().collect();
        sorted.sort();

        assert_eq!(sorted, exp);
    }

    #[test]
    fn mtc_list_sync_combines_correctly() {
        let mut client_list = MtcList::new(false);
        let mut server_list = MtcList::new(true);

        client_list.add(TestMtcItem::new("Item 0".to_string()));
        client_list.add(TestMtcItem::new("Item 1".to_string()));
        client_list.add(TestMtcItem::new("Item 2".to_string()));

        client_list.sync_self();

        client_list.add(TestMtcItem::new("Item 3".to_string()));

        client_list.mark_removed(1).unwrap();

        server_list.add(TestMtcItem::new("Item 0".to_string()));
        server_list.add(TestMtcItem::new("Item 1".to_string()));
        server_list.add(TestMtcItem::new("Item 4".to_string()));

        client_list.sync(&mut server_list);

        let mut exp: Vec<TestMtcItem> = vec![
            TestMtcItem::new("Item 0".to_string()),
            TestMtcItem::new("Item 3".to_string()),
            TestMtcItem::new("Item 4".to_string()),
        ];
        exp.iter_mut().for_each(|x| {
            x.set_state(ItemState::Neutral);
        });

        let mut sorted: Vec<TestMtcItem> = client_list.items().iter().cloned().cloned().collect();
        sorted.sort();

        assert_eq!(sorted, exp);
        let mut sorted: Vec<TestMtcItem> = server_list.items().iter().cloned().cloned().collect();
        sorted.sort();

        assert_eq!(sorted, exp);
    }

    #[test]
    #[should_panic]
    fn mtc_list_panics_if_both_servers() {
        let mut client: MtcList<TestMtcItem> = MtcList::new(true);
        let mut client1: MtcList<TestMtcItem> = MtcList::new(true);

        client.sync(&mut client1);
    }

    #[test]
    #[should_panic]
    fn mtc_list_panics_if_neither_servers() {
        let mut client: MtcList<TestMtcItem> = MtcList::new(false);
        let mut client1: MtcList<TestMtcItem> = MtcList::new(false);

        client.sync(&mut client1);
    }

    #[test]
    fn mtc_list_clone_server_clones_valid_server() {
        let mut client = MtcList::new(false);
        client.add(TestMtcItem::new("Item 0".to_string()));
        client.add(TestMtcItem::new("Item 1".to_string()));
        client.add(TestMtcItem::new("Item 2".to_string()));

        client.mark_removed(1).unwrap();

        let mut expected = MtcList::new(true);
        expected.add(TestMtcItem::new("Item 0".to_string()));
        expected.add(TestMtcItem::new("Item 2".to_string()));

        assert_eq!(client.clone_to_server(), expected);
    }

    #[test]
    fn mtc_list_id_matches() {
        let mut client = MtcList::new(false);

        client.add(TodoItem::new("Item 0".to_string(), Some(Weekday::Mon)));
        client.add(TodoItem::new("Item 1".to_string(), Some(Weekday::Tue)));
        client.add(TodoItem::new("Item 2".to_string(), Some(Weekday::Wed)));

        client.mark_removed(0).unwrap();

        client.sync_self();

        assert_eq!(client.items_for_weekday(Weekday::Wed)[0].id(), 1);
        let mut item = TodoItem::new("Item 2".to_string(), Some(Weekday::Wed));
        item.set_id(1);
        assert_eq!(*client.items()[1], item);
    }

    #[test]
    fn mtc_list_doesnt_return_marked_as_removed() {
        let mut client = MtcList::new(false);

        client.add(TodoItem::new("Item 0".to_string(), None));
        client.add(TodoItem::new("Item 1".to_string(), None));
        client.add(TodoItem::new("Item 2".to_string(), None));

        client.mark_removed(1).unwrap();

        for item in client.items() {
            assert!(!item.ignore_state_eq(&TodoItem::new("Item 1".to_string(), None)));
        }
    }
}
