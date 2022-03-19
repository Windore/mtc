//! This crate provides the base functionality for mtc. It can be also used for creating apps that
//! can sync with the mtc CLI app or serve as an additional interface.
//!
//! # Examples
//!
//! Creating a new `MtcList`, adding `Event`s and accessing those events.
//! ```
//! use chrono::NaiveDate;
//!
//! use mtc::*;
//!
//! // Create a new local MtcList of Events
//! let mut events = MtcList::new(false);
//!
//! // Add some events to it.
//! events.add(Event::new("An important event.".to_string(), NaiveDate::from_ymd(2022, 4, 6)));
//! events.add(Event::new(
//!     "11:00-12:00 A second important event with a time as well.".to_string(),
//!     NaiveDate::from_ymd(2022, 5, 17)
//! ));
//! events.add(Event::new("A Generic event 1".to_string(), NaiveDate::from_ymd(2022, 5, 17)));
//! events.add(Event::new("A Generic event 2".to_string(), NaiveDate::from_ymd(2022, 6, 6)));
//! events.add(Event::new("A Generic event 3".to_string(), NaiveDate::from_ymd(2022, 6, 7)));
//!
//! // Loop through every event on the list and print it.
//! for event in events.items() {
//!     println!("{}", event);
//! }
//!
//! // Get all events that are for 2022-5-17
//! let events_for_date = events.items_for_date(NaiveDate::from_ymd(2022, 5, 17));
//!  
//! assert_eq!(events_for_date.len(), 2);
//! 
//! // ignore_state_eq can be used to compare two MtcItems (like Events) 
//! // without considering their ids or states.
//! assert!(events_for_date[0].ignore_state_eq(
//!     &Event::new(
//!         "11:00-12:00 A second important event with a time as well.".to_string(),
//!          NaiveDate::from_ymd(2022, 5, 17)
//!     )
//! ));
//! assert!(events_for_date[1].ignore_state_eq(
//!     &Event::new("A Generic event 1".to_string(), NaiveDate::from_ymd(2022, 5, 17))
//! ));
//! 
//! // Get the event with the id 1
//! assert!(events.get_by_id(1).unwrap().ignore_state_eq(
//!     &Event::new(
//!         "11:00-12:00 A second important event with a time as well.".to_string(),
//!          NaiveDate::from_ymd(2022, 5, 17)
//!     )
//! ));
//! 
//! assert_eq!(events.items().len(), 5);
//! 
//! // Remove the event with the id 1
//! events.mark_removed(1).unwrap();
//! 
//! assert_eq!(events.items().len(), 4);
//! 
//! // Trying to get the removed event fails
//! assert!(events.get_by_id(1).is_none());
//! 
//! // Syncing will change event ids so if a sync were to happen now, the event ids would shift.
//! ```
//!
//! Syncing with a remote server:
//! ```no_run
//! use std::path::Path;
//! use std::net::TcpStream;
//!
//! use chrono::Weekday;
//! use ssh2::Session;
//!
//! use mtc::*;
//!
//! // Create a new local MtcList of Todos
//! let mut local_list = MtcList::new(false);
//!
//! // Add new two new todos to it.
//! local_list.add(Todo::new("My todo for the next monday".to_string(), Some(Weekday::Mon)));
//! local_list.add(Todo::new("My todo for the next friday".to_string(), Some(Weekday::Fri)));
//!
//! // Create a ssh connection to a server using the ssh2 crate.
//! let tcp = TcpStream::connect("127.0.0.1").unwrap();
//! let mut sess = Session::new().unwrap();
//! sess.set_tcp_stream(tcp);
//! sess.handshake().unwrap();
//! sess.userauth_password("mtc", "hunter2").unwrap();
//!
//! // Sync the local list with the remote list on the server
//!
//! // Since this is not the first time syncing set overwrite to false
//! let overwrite = false;
//! // For syncing the first time the directory path should exist and overwrite should be true.
//!
//! sync_remote(
//!     &sess,
//!     &mut local_list,
//!     &Path::new("/this/path/is/on/the/server/and/contains/the/servers/list/of/todos.json"),
//!     overwrite,
//! ).unwrap();
//! ```

#![warn(missing_docs)]

use chrono::{Datelike, Local, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

mod items;

pub use crate::items::*;

mod remote;

pub use crate::remote::*;

/// An Item for `MtcList`. A struct implementing `MtcItem` is usually defined for a specific time and has a `ItemState`.
pub trait MtcItem {
    /// Returns true if the item is for a given date.
    fn for_date(&self, date: NaiveDate) -> bool;
    /// Returns true if the item is for today.
    fn for_today(&self) -> bool {
        self.for_date(Local::today().naive_local())
    }
    /// Returns true if the item is for a given weekday.
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
    /// Compares the MtcItems while ignoring the state and id of both items.
    fn ignore_state_eq(&self, other: &Self) -> bool;
    /// Gets the id of the item.
    fn id(&self) -> usize;
    /// Sets the id of the items. `MtcList` usually handles setting the id so in most cases calling this manually is not needed nor recommended.
    fn set_id(&mut self, new_id: usize);
    /// Returns true if the item is expired. If an item is expired it should be removed when syncing.
    fn expired(&self) -> bool;
}

/// A state of a `MtcItem` used for synchronising `MtcList`s correctly
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum ItemState {
    /// `MtcItem` is a new item and the server list does not contain it
    New,
    /// `MtcItem` is removed and if it exists on the server it should be removed from there too.
    Removed,
    /// `MtcItem` is not new nor should it be removed. If it doesn't exist on the server it will be removed.
    Neutral,
}

/// A wrapper for a `Vec` containing `MtcItem`s. The wrapper helps to manage the state of the items and sync them correctly.
/// A `MtcList` can be either a client or a server list which affect the functionality of the list. Server lists don't track
/// the state since multiple clients could be interacting with the same server.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MtcList<T: MtcItem + Clone> {
    items: Vec<T>,
    is_server: bool,
}

impl<T: MtcItem + Clone> MtcList<T> {
    /// Creates a new `MtcList` which will be either a server or a client.
    pub fn new(is_server: bool) -> MtcList<T> {
        MtcList {
            items: Vec::new(),
            is_server,
        }
    }

    /// Appends a new `MtcItem` to the list setting the item's state to new. Returns the id of the item.
    pub fn add(&mut self, mut item: T) -> usize {
        if self.is_server {
            item.set_state(ItemState::Neutral);
        } else {
            item.set_state(ItemState::New);
        }
        item.set_id(self.items.len());
        let id = item.id();
        self.items.push(item);

        id
    }

    /// Marks a `MtcItem` of a given id to be removed. The id is the same as the index in the inner `Vec`.
    /// Returns `Err(&str)` if index is out of bounds. The string can be shown to the user.
    pub fn mark_removed(&mut self, id: usize) -> Result<(), &str> {
        if let Some(item) = self.items.get_mut(id) {
            if self.is_server {
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

    /// Returns a reference to the item with the id if it exists.
    pub fn get_by_id(&self, id: usize) -> Option<&T> {
        let item = self.items.get(id);
        item.and_then(|i| {
            if i.state() != ItemState::Removed {
                Some(i)
            } else {
                None
            }
        })
    }

    /// Returns a new `Vec` containing references to all items within this list in the same order.
    /// Note that this filters all items that are marked as removed.
    pub fn items(&self) -> Vec<&T> {
        let mut new = Vec::new();

        for item in &self.items {
            if item.state() != ItemState::Removed {
                new.push(item);
            }
        }

        new
    }

    /// Returns a new `Vec` containing references to all items that are for a given date.
    pub fn items_for_date(&self, date: NaiveDate) -> Vec<&T> {
        self.items()
            .into_iter()
            .filter(|item| item.for_date(date))
            .collect()
    }

    /// Return a new `Vec` containing references to all items that are for today.
    pub fn items_for_today(&self) -> Vec<&T> {
        self.items()
            .into_iter()
            .filter(|item| item.for_today())
            .collect()
    }

    /// Return a `Vec` containing references to all items that are for a given weekday.
    pub fn items_for_weekday(&self, weekday: Weekday) -> Vec<&T> {
        self.items()
            .into_iter()
            .filter(|item| item.for_weekday(weekday))
            .collect()
    }

    /// Returns a clone of this list but as a server
    pub fn clone_to_server(&self) -> MtcList<T> {
        let mut clone = self.clone();
        clone.is_server = true;
        clone.sync_self();

        clone
    }

    /// Synchronizes the list with itself by removing all items with the `Removed` state and setting the state of the rest to `Neutral`.
    /// Item ids could change during sync.
    pub fn sync_self(&mut self) {
        self.items.retain(|item| item.state() != ItemState::Removed);
        for (i, item) in self.items.iter_mut().enumerate() {
            item.set_state(ItemState::Neutral);
            item.set_id(i);
        }
    }

    /// Synchronizes this `MtcList` with the other `MtcList`.
    /// Either one of these lists is expected to be a server and the other a client.
    /// Removes items that are marked for removal.
    /// Item ids could change during sync.
    ///
    /// # Panics
    ///
    /// If neither one of the lists is a server or if both are servers.
    ///
    /// # Example
    ///
    /// ```
    /// use mtc::{MtcList, Todo};
    /// use chrono::prelude::Weekday;
    ///
    /// let mut client_list = MtcList::new(false);
    ///
    /// client_list.add(Todo::new("Task 1".to_string(), None));
    /// client_list.add(Todo::new("Task 2".to_string(), Some(Weekday::Mon)));
    /// client_list.add(Todo::new("Task 3".to_string(), Some(Weekday::Fri)));
    ///
    /// // Set the state of all items in the client list from New to Neutral by using sync_self
    ///
    /// client_list.sync_self();
    ///
    /// // This one will have the state New
    /// client_list.add(Todo::new("Task 4".to_string(), Some(Weekday::Sat)));
    ///
    /// // Set the Task 2 element to Removed
    /// client_list.mark_removed(1); // It will have the index of 1
    ///
    /// let mut server_list = MtcList::new(true);
    /// server_list.add(Todo::new("Task 1".to_string(), None));
    /// server_list.add(Todo::new("Task 2".to_string(), Some(Weekday::Mon)));
    /// server_list.add(Todo::new("Task 5".to_string(), Some(Weekday::Mon)));
    ///
    /// client_list.sync(&mut server_list);
    /// // server_list.sync(&mut client_list); may be used as well. There is no difference
    ///
    /// // The operation should result in the following list.
    /// // Both server and client will have same items but the order may be different.
    /// let mut resulting_client_list = MtcList::new(false);
    /// resulting_client_list.add(Todo::new("Task 1".to_string(), None));
    /// resulting_client_list.add(Todo::new("Task 4".to_string(), Some(Weekday::Sat)));
    /// resulting_client_list.add(Todo::new("Task 5".to_string(), Some(Weekday::Mon)));
    ///
    /// // The resulting list needs to be self synced since the items otherwise would 
    /// // have a New state
    /// resulting_client_list.sync_self();
    ///
    /// // The only difference between the lists is the is_server field of both lists
    /// // which is not a result of the sync function.
    /// let mut resulting_server_list = MtcList::new(true);
    /// resulting_server_list.add(Todo::new("Task 1".to_string(), None));
    /// resulting_server_list.add(Todo::new("Task 5".to_string(), Some(Weekday::Mon)));
    /// resulting_server_list.add(Todo::new("Task 4".to_string(), Some(Weekday::Sat)));
    ///
    /// assert_eq!(client_list, resulting_client_list);
    /// assert_eq!(server_list, resulting_server_list);
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
                    if !server_list
                        .items
                        .iter()
                        .any(|elem| elem.ignore_state_eq(item))
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

    /// Removes all `MtcItem`s that are expired.
    pub fn remove_expired(&mut self) {
        for item in self.items.iter_mut() {
            if item.expired() {
                item.set_state(ItemState::Removed);
            }
        }
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

    #[derive(Debug, PartialEq, Clone)]
    struct TestMtcItem {
        date: NaiveDate,
        state: ItemState,
        body: String,
        id: usize,
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
            TestMtcItem::new_dated(body, Local::today().naive_local())
        }

        fn new_dated(body: String, date: NaiveDate) -> TestMtcItem {
            TestMtcItem {
                date,
                state: ItemState::Neutral,
                body,
                id: 0,
            }
        }
    }

    impl MtcItem for TestMtcItem {
        fn for_date(&self, date: chrono::NaiveDate) -> bool {
            date == self.date
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
            self.id
        }
        fn set_id(&mut self, id: usize) {
            self.id = id;
        }
        fn expired(&self) -> bool {
            let today = Local::today().naive_local();
            self.date.signed_duration_since(today).num_days() < -3
        }
    }

    #[test]
    fn mtc_item_for_today_returns_true() {
        let item = TestMtcItem::new_dated(String::new(), Local::today().naive_local());
        assert!(item.for_today());
    }

    #[test]
    fn mtc_item_for_today_returns_false() {
        let item = TestMtcItem::new_dated(String::new(), Local::today().naive_local().succ());
        assert!(!item.for_today());
    }

    #[test]
    fn mtc_item_for_weekday_returns_true() {
        let item =
            TestMtcItem::new_dated(String::new(), Local::today().naive_local().succ().succ());
        assert!(item.for_weekday(Local::today().weekday().succ().succ()));
    }

    #[test]
    fn mtc_item_for_weekday_returns_false() {
        let item = TestMtcItem::new_dated(String::new(), NaiveDate::from_ymd(2022, 1, 6));
        assert!(!item.for_weekday(Weekday::Fri));
    }

    #[test]
    fn mtc_list_for_date_returns_expected() {
        let mut list = MtcList::new(true);
        list.add(TestMtcItem::new_dated(
            "test0".to_string(),
            NaiveDate::from_ymd(2022, 4, 6),
        ));
        list.add(TestMtcItem::new_dated(
            "test1".to_string(),
            NaiveDate::from_ymd(2022, 1, 6),
        ));
        list.add(TestMtcItem::new_dated(
            "test2".to_string(),
            NaiveDate::from_ymd(2022, 1, 5),
        ));
        list.add(TestMtcItem::new_dated(
            "test3".to_string(),
            NaiveDate::from_ymd(2022, 1, 7),
        ));
        list.add(TestMtcItem::new_dated(
            "test4".to_string(),
            NaiveDate::from_ymd(2022, 1, 6),
        ));

        let mut expected = vec![
            TestMtcItem::new_dated("test1".to_string(), NaiveDate::from_ymd(2022, 1, 6)),
            TestMtcItem::new_dated("test4".to_string(), NaiveDate::from_ymd(2022, 1, 6)),
        ];

        expected[0].set_id(1);
        expected[1].set_id(4);

        let result: Vec<TestMtcItem> = list
            .items_for_date(NaiveDate::from_ymd(2022, 1, 6))
            .iter()
            .cloned()
            .cloned()
            .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn mtc_list_for_today_returns_expected() {
        let today = Local::today().naive_local();

        let mut items = MtcList::new(true);
        items.add(TestMtcItem::new_dated("test0".to_string(), today));
        items.add(TestMtcItem::new_dated("test1".to_string(), today));
        items.add(TestMtcItem::new_dated("test2".to_string(), today.pred()));
        items.add(TestMtcItem::new_dated("test3".to_string(), today.succ()));

        let mut expected = vec![
            TestMtcItem::new_dated("test0".to_string(), today),
            TestMtcItem::new_dated("test1".to_string(), today),
        ];

        expected[0].set_id(0);
        expected[1].set_id(1);

        let result: Vec<TestMtcItem> = items.items_for_today().iter().cloned().cloned().collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn mtc_list_for_weekday_returns_expected() {
        let today = Local::today().naive_local();

        let mut items = MtcList::new(true);
        items.add(TestMtcItem::new_dated("test0".to_string(), today));
        items.add(TestMtcItem::new_dated(
            "test1".to_string(),
            today.succ().succ(),
        ));
        items.add(TestMtcItem::new_dated("test2".to_string(), today.pred()));
        items.add(TestMtcItem::new_dated("test3".to_string(), today.succ()));

        let mut expected = vec![TestMtcItem::new_dated("test3".to_string(), today.succ())];

        expected[0].set_id(3);

        let result: Vec<TestMtcItem> = items
            .items_for_weekday(today.weekday().succ())
            .iter()
            .cloned()
            .cloned()
            .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn mtc_list_get_by_id_returns_some_and_none() {
        let todo = TestMtcItem::new("Item".to_string());
        let mut list = MtcList::new(false);
        let id = list.add(todo.clone());

        assert!(todo.ignore_state_eq(list.get_by_id(id).unwrap()));
        assert_eq!(None, list.get_by_id(66));

        list.mark_removed(id).unwrap();

        assert_eq!(None, list.get_by_id(id));
        assert_eq!(None, list.get_by_id(66));
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

        exp[0].set_id(0);
        exp[1].set_id(1);
        exp[2].set_id(2);

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

        exp[0].set_id(0);
        exp[1].set_id(1);
        exp[2].set_id(2);

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

        exp[0].set_id(0);
        exp[1].set_id(1);

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
    fn mtc_list_sync_removes_non_new_from_client() {
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
        exp[0].set_id(0);
        exp[1].set_id(1);
        exp[2].set_id(2);

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

        exp[0].set_id(0);
        exp[1].set_id(1);
        exp[2].set_id(2);

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

        // Client and server list have a different order of ids
        exp[0].set_id(0);
        exp[1].set_id(1);
        exp[2].set_id(2);

        let mut sorted: Vec<TestMtcItem> = client_list.items().iter().cloned().cloned().collect();
        sorted.sort();

        assert_eq!(sorted, exp);

        // Client and server list have a different order of ids
        exp[0].set_id(0);
        exp[1].set_id(2);
        exp[2].set_id(1);

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
    fn mtc_list_doesnt_return_marked_as_removed() {
        let mut client = MtcList::new(false);

        client.add(Todo::new("Item 0".to_string(), None));
        client.add(Todo::new("Item 1".to_string(), None));
        client.add(Todo::new("Item 2".to_string(), None));

        client.mark_removed(1).unwrap();

        for item in client.items() {
            assert!(!item.ignore_state_eq(&Todo::new("Item 1".to_string(), None)));
        }
    }

    #[test]
    fn mtc_remove_expired_removes_correct() {
        let mut client = MtcList::new(false);

        let today = Local::today().naive_local();

        client.add(TestMtcItem::new_dated("TestMtcItem 1".to_string(), today));
        client.add(TestMtcItem::new_dated(
            "TestMtcItem 2".to_string(),
            today.succ(),
        ));
        client.add(TestMtcItem::new_dated(
            "TestMtcItem 3".to_string(),
            today.pred(),
        ));
        client.add(TestMtcItem::new_dated(
            "TestMtcItem 4".to_string(),
            today.pred().pred().pred().pred(),
        ));
        client.add(TestMtcItem::new_dated(
            "TestMtcItem 5".to_string(),
            today.pred().pred().pred().pred().pred().pred(),
        ));

        let mut exp: Vec<TestMtcItem> = vec![
            TestMtcItem::new_dated("TestMtcItem 1".to_string(), today),
            TestMtcItem::new_dated("TestMtcItem 2".to_string(), today.succ()),
            TestMtcItem::new_dated("TestMtcItem 3".to_string(), today.pred()),
        ];
        let mut id_counter = 0;
        exp.iter_mut().for_each(|x| {
            x.set_state(ItemState::Neutral);
            x.set_id(id_counter);
            id_counter += 1;
        });

        client.remove_expired();
        client.sync_self();

        let result: Vec<TestMtcItem> = client.items().iter().cloned().cloned().collect();

        assert_eq!(exp, result);
    }
}
