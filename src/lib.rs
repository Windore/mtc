//! # MMVK Lib
//!
//! This API provides the base functionality for mmvk. It can be also used for creating apps that
//! can sync with the mmvk CLI app or serve as an additional interface.

use chrono::prelude::*;
use sync::ItemState;

/// A General trait for sharing a implementation between TodoItems, Tasks and Events.
pub trait MmvkItem {
    /// Returns true if the item is for a given date.
    ///
    /// # Example
    /// ```
    /// use chrono::prelude::*;
    /// use mmvk::MmvkItem;
    ///
    /// struct WeekdayItem {
    ///     weekday: Weekday,
    /// }
    ///
    /// impl MmvkItem for WeekdayItem {
    ///     fn for_date(&self, date: NaiveDate) -> bool {
    ///         self.weekday == date.weekday()
    ///     }
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
    /// use mmvk::MmvkItem;
    ///
    /// struct WeekdayItem {
    ///     weekday: Weekday,
    /// }
    ///
    /// impl MmvkItem for WeekdayItem {
    ///     fn for_date(&self, date: NaiveDate) -> bool {
    ///         self.weekday == date.weekday()
    ///     }
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
    /// use mmvk::MmvkItem;
    ///
    /// struct WeekdayItem {
    ///     weekday: Weekday,
    /// }
    ///
    /// impl MmvkItem for WeekdayItem {
    ///     fn for_date(&self, date: NaiveDate) -> bool {
    ///         self.weekday == date.weekday()
    ///     }
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
    fn ignore_state_eq(&self, other: &Self) -> bool;
}

pub mod sync {
    use super::*;

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum ItemState {
        New,
        Removed,
        Neutral,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct MmvkList<T: MmvkItem + PartialEq + Clone> {
        items: Vec<T>,
        is_server: bool,
    }

    impl<T: MmvkItem + PartialEq + Clone> MmvkList<T> {
        pub fn new(is_server: bool) -> MmvkList<T> {
            MmvkList {
                items: Vec::new(),
                is_server,
            }
        }

        pub fn add(&mut self, mut item: T) {
            if self.is_server {
                item.set_state(ItemState::Neutral);
            } else {
                item.set_state(ItemState::New);
            }
            self.items.push(item);
        }

        pub fn mark_removed(&mut self, id: usize) -> Result<(), &str> {
            if let Some(item) = self.items.get_mut(id) {
                if !self.is_server {
                    item.set_state(ItemState::Removed);
                } else {
                    drop(item);
                    self.items.remove(id);
                }
                Ok(())
            } else {
                Err("No item with the given id found")
            }
        }

        pub fn items(&self) -> &Vec<T> {
            &self.items
        }

        pub fn items_for_date(&self, date: NaiveDate) -> Vec<&T> {
            let mut out = Vec::new();

            for item in self.items.iter() {
                if item.for_date(date) {
                    out.push(item);
                }
            }

            out
        }

        pub fn items_for_today(&self) -> Vec<&T> {
            let mut out = Vec::new();

            for item in self.items.iter() {
                if item.for_today() {
                    out.push(item);
                }
            }

            out
        }

        pub fn items_for_weekday(&self, weekday: Weekday) -> Vec<&T> {
            let mut out = Vec::new();

            for item in self.items.iter() {
                if item.for_weekday(weekday) {
                    out.push(item);
                }
            }

            out
        }

        pub fn sync_self(&mut self) {
            let mut removed = Vec::new();

            for (i, item) in self.items.iter_mut().enumerate() {
                if item.state() == ItemState::Removed {
                    removed.push(i);
                } else {
                    item.set_state(ItemState::Neutral);
                }
            }

            removed.sort();
            while let Some(r) = removed.pop() {
                self.items.remove(r);
            }
        }

        /// Synchronizes this MmvkList with the other MmvkList.
        /// Either one of these lists is expected to be a server and the other a client.
        /// Removes items that are marked for removal.
        /// # Panics
        /// If neither one of the lists is a server or if both are servers.
        pub fn sync(&mut self, other: &mut MmvkList<T>) {
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
                        for elem in server_list.items.iter_mut() {
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

            for item in server_list.items.iter() {
                if item.state() != ItemState::Removed
                    && client_list
                        .items
                        .iter()
                        .position(|elem| {
                            elem.ignore_state_eq(item) && elem.state() != ItemState::Removed
                        })
                        .is_none()
                {
                    client_list.add(item.clone());
                }
            }

            client_list.sync_self();
            server_list.sync_self();
        }
    }
}

/// A short term task that should be done on a optionally given weekday.
#[derive(Debug, PartialEq, Clone)]
pub struct TodoItem {
    weekday: Option<Weekday>,
    body: String,
    state: ItemState,
}

/// A repetitive task with a duration in minutes for a optionally given day.
#[derive(Debug, PartialEq, Clone)]
pub struct Task {
    weekday: Option<Weekday>,
    body: String,
    duration: u32,
    state: ItemState,
}

/// An event that will happen on a given date and optionally a time.
#[derive(Debug, PartialEq, Clone)]
pub struct Event {
    date: NaiveDate,
    time: Option<NaiveTime>,
    body: String,
    state: ItemState,
}

impl TodoItem {
    /// Creates a new `TodoItem` with a given body and optionally a weekday.
    pub fn new(body: String, weekday: Option<Weekday>) -> TodoItem {
        TodoItem {
            weekday,
            body,
            state: ItemState::Neutral,
        }
    }

    /// Returns a reference to the body of the `TodoItem`.
    pub fn body(&self) -> &String {
        &self.body
    }

    /// Returns a optionally specified weekday of the `TodoItem`.
    pub fn weekday(&self) -> Option<Weekday> {
        self.weekday
    }

    /// Sets the optional weekday of the `TodoItem`.
    pub fn set_weekday(&mut self, new_weekday: Option<Weekday>) {
        self.weekday = new_weekday;
    }
}

impl Task {
    /// Creates a new `Task` with a given body, duration in minutes and optionally a weekday.
    pub fn new(body: String, duration: u32, weekday: Option<Weekday>) -> Task {
        Task {
            weekday,
            body,
            duration,
            state: ItemState::Neutral,
        }
    }

    /// Returns a reference to the body of the `Task`.
    pub fn body(&self) -> &String {
        &self.body
    }

    /// Returns the optionally specified weekday of the `Task`.
    pub fn weekday(&self) -> Option<Weekday> {
        self.weekday
    }

    /// Returns the duration of the `Task`.
    pub fn duration(&self) -> u32 {
        self.duration
    }

    /// Sets the optional weekday of the `Task`.
    pub fn set_weekday(&mut self, new_weekday: Option<Weekday>) {
        self.weekday = new_weekday;
    }
}

impl Event {
    /// Creates a new `Event` with a given body, date and optionally a time.
    pub fn new(body: String, date: NaiveDate, time: Option<NaiveTime>) -> Event {
        Event {
            body,
            date,
            time,
            state: ItemState::Neutral,
        }
    }

    /// Returns a reference to the body of the `Event`.
    pub fn body(&self) -> &String {
        &self.body
    }

    /// Returns the date of the `Event`.
    pub fn date(&self) -> NaiveDate {
        self.date
    }

    /// Sets the date of the `Event`.
    pub fn set_date(&mut self, new_date: NaiveDate) {
        self.date = new_date;
    }

    /// Returns the optinal time of the `Event`.
    pub fn time(&self) -> Option<NaiveTime> {
        self.time
    }

    /// Sets the optional time of the `Event`.
    pub fn set_time(&mut self, new_time: Option<NaiveTime>) {
        self.time = new_time;
    }
}

impl MmvkItem for TodoItem {
    /// Returns true if the `TodoItem` is for a given date.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::prelude::*;
    /// use mmvk::{TodoItem, MmvkItem};
    ///
    /// let item = TodoItem::new("Do task A".to_string(), Some(Weekday::Mon));
    ///
    /// // 2021.12.6 was a monday.
    /// assert!(item.for_date(NaiveDate::from_ymd(2021, 12, 6)));
    /// assert!(!item.for_date(NaiveDate::from_ymd(2021, 12, 5)));
    /// ```
    fn for_date(&self, date: NaiveDate) -> bool {
        match self.weekday {
            Some(day) => day == date.weekday(),
            None => true,
        }
    }
    fn state(&self) -> sync::ItemState {
        self.state
    }
    fn set_state(&mut self, new_state: sync::ItemState) {
        self.state = new_state;
    }
    fn ignore_state_eq(&self, _: &TodoItem) -> bool {
        todo!()
    }
}

impl MmvkItem for Task {
    /// Returns true if the `Task` is for a given date.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::prelude::*;
    /// use mmvk::{Task, MmvkItem};
    ///
    /// let item = Task::new("Exercise".to_string(), 60, Some(Weekday::Mon));
    ///
    /// // 2021.12.6 was a monday.
    /// assert!(item.for_date(NaiveDate::from_ymd(2021, 12, 6)));
    /// assert!(!item.for_date(NaiveDate::from_ymd(2021, 12, 5)));
    /// ```
    fn for_date(&self, date: NaiveDate) -> bool {
        match self.weekday {
            Some(day) => day == date.weekday(),
            None => true,
        }
    }
    fn state(&self) -> sync::ItemState {
        self.state
    }
    fn set_state(&mut self, new_state: sync::ItemState) {
        self.state = new_state;
    }
    fn ignore_state_eq(&self, _: &Task) -> bool {
        todo!()
    }
}

impl MmvkItem for Event {
    /// Returns true if the `Event` is for a given date.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::prelude::*;
    /// use mmvk::{Event, MmvkItem};
    ///
    /// let item = Event::new("Do task A".to_string(), NaiveDate::from_ymd(2021, 11, 30), None);
    ///
    /// assert!(item.for_date(NaiveDate::from_ymd(2021, 11, 30)));
    /// assert!(!item.for_date(NaiveDate::from_ymd(2021, 12, 6)));
    /// ```
    fn for_date(&self, date: NaiveDate) -> bool {
        self.date == date
    }
    fn state(&self) -> sync::ItemState {
        self.state
    }
    fn set_state(&mut self, new_state: sync::ItemState) {
        self.state = new_state;
    }
    fn ignore_state_eq(&self, _: &Self) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::MmvkList;

    #[test]
    fn todo_item_for_date_returns_true() {
        let date = NaiveDate::from_ymd(2021, 12, 6);
        let ti = TodoItem::new("test".to_string(), Some(Weekday::Mon));

        assert!(ti.for_date(date));
    }

    #[test]
    fn todo_item_for_date_returns_false() {
        let date = NaiveDate::from_ymd(2021, 12, 6);
        let ti = TodoItem::new("test".to_string(), Some(Weekday::Tue));

        assert!(!ti.for_date(date));
    }

    #[test]
    fn task_for_date_returns_true() {
        let date = NaiveDate::from_ymd(2021, 12, 6);
        let ti = Task::new("test".to_string(), 50, None);

        assert!(ti.for_date(date));
    }

    #[test]
    fn task_for_date_returns_false() {
        let date = NaiveDate::from_ymd(2021, 12, 6);
        let ti = Task::new("test".to_string(), 50, Some(Weekday::Fri));

        assert!(!ti.for_date(date));
    }

    #[test]
    fn event_for_date_returns_true() {
        let date = NaiveDate::from_ymd(2021, 12, 6);
        let ti = Event::new("test".to_string(), NaiveDate::from_ymd(2021, 12, 6), None);

        assert!(ti.for_date(date));
    }

    #[test]
    fn event_for_date_returns_false() {
        let date = NaiveDate::from_ymd(2021, 12, 6);
        let ti = Event::new("test".to_string(), NaiveDate::from_ymd(2021, 12, 5), None);

        assert!(!ti.for_date(date));
    }

    #[test]
    fn mmvk_item_for_today_returns_true() {
        struct TestItem {}
        impl MmvkItem for TestItem {
            fn for_date(&self, date: NaiveDate) -> bool {
                date == Local::today().naive_local()
            }
            fn state(&self) -> sync::ItemState {
                todo!()
            }
            fn set_state(&mut self, _: sync::ItemState) {
                todo!()
            }
            fn ignore_state_eq(&self, _: &Self) -> bool {
                todo!()
            }
        }

        let item = TestItem {};

        assert!(item.for_today());
    }

    #[test]
    fn mmvk_item_for_today_returns_false() {
        struct TestItem {}
        impl MmvkItem for TestItem {
            fn for_date(&self, date: NaiveDate) -> bool {
                date != Local::today().naive_local()
            }
            fn state(&self) -> sync::ItemState {
                todo!()
            }
            fn set_state(&mut self, _: sync::ItemState) {
                todo!()
            }
            fn ignore_state_eq(&self, _: &Self) -> bool {
                todo!()
            }
        }

        let item = TestItem {};

        assert!(!item.for_today());
    }

    #[test]
    fn mmvk_item_for_weekday_returns_true() {
        struct TestItem {}
        impl MmvkItem for TestItem {
            fn for_date(&self, date: NaiveDate) -> bool {
                date.weekday() == Weekday::Mon
            }
            fn state(&self) -> sync::ItemState {
                todo!()
            }
            fn set_state(&mut self, _: sync::ItemState) {
                todo!()
            }
            fn ignore_state_eq(&self, _: &Self) -> bool {
                todo!()
            }
        }

        let item = TestItem {};

        assert!(item.for_weekday(Weekday::Mon));
    }

    #[test]
    fn mmvk_item_for_weekday_returns_false() {
        struct TestItem {}
        impl MmvkItem for TestItem {
            fn for_date(&self, date: NaiveDate) -> bool {
                date == NaiveDate::from_ymd(2021, 12, 6)
            }
            fn state(&self) -> sync::ItemState {
                todo!()
            }
            fn set_state(&mut self, _: sync::ItemState) {
                todo!()
            }
            fn ignore_state_eq(&self, _: &Self) -> bool {
                todo!()
            }
        }

        let item = TestItem {};

        assert!(!item.for_weekday(Weekday::Fri));
    }

    #[test]
    fn mmvk_items_date_returns_expected() {
        let mut list = MmvkList::new(true);
        list.add(TodoItem::new("test0".to_string(), None));
        list.add(TodoItem::new("test1".to_string(), Some(Weekday::Tue)));
        list.add(TodoItem::new("test2".to_string(), Some(Weekday::Wed)));
        list.add(TodoItem::new("test3".to_string(), None));
        list.add(TodoItem::new("test4".to_string(), Some(Weekday::Tue)));

        let expected = vec![
            TodoItem::new("test0".to_string(), None),
            TodoItem::new("test1".to_string(), Some(Weekday::Tue)),
            TodoItem::new("test3".to_string(), None),
            TodoItem::new("test4".to_string(), Some(Weekday::Tue)),
        ];

        let result: Vec<TodoItem> = list
            .items_for_date(NaiveDate::from_ymd(2021, 11, 30))
            .iter()
            .cloned()
            .cloned()
            .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn mmvk_items_today_returns_expected() {
        let today = Local::now().weekday();

        let mut items = MmvkList::new(true);
        items.add(Task::new("test0".to_string(), 40, Some(today)));
        items.add(Task::new("test1".to_string(), 30, Some(today)));
        items.add(Task::new("test2".to_string(), 10, Some(today.pred())));
        items.add(Task::new("test3".to_string(), 0, None));
        items.add(Task::new("test4".to_string(), 90, Some(today.succ().succ())));

        let expected = vec![
            Task::new("test0".to_string(), 40, Some(today)),
            Task::new("test1".to_string(), 30, Some(today)),
            Task::new("test3".to_string(), 0, None),
        ];

        let result: Vec<Task> = items.items_for_today().iter().cloned().cloned().collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn mmvk_items_weekday_returns_expected() {
        let mut items = MmvkList::new(true);
        items.add(Task::new("test0".to_string(), 40, Some(Weekday::Fri)));
        items.add(Task::new("test1".to_string(), 30, Some(Weekday::Fri)));
        items.add(Task::new("test2".to_string(), 10, Some(Weekday::Mon)));
        items.add(Task::new("test3".to_string(), 0, None));
        items.add(Task::new("test4".to_string(), 90, Some(Weekday::Wed)));

        let expected = vec![
            Task::new("test0".to_string(), 40, Some(Weekday::Fri)),
            Task::new("test1".to_string(), 30, Some(Weekday::Fri)),
            Task::new("test3".to_string(), 0, None),
        ];

        let result: Vec<Task> = items.items_for_weekday(Weekday::Fri)
            .iter()
            .cloned()
            .cloned()
            .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn mmvk_items_date_returns_expected_for_events() {
        let mut items = MmvkList::new(true);
        items.add(Event::new("test0".to_string(), NaiveDate::from_ymd(2021, 10, 5), None));
        items.add(Event::new("test1".to_string(), NaiveDate::from_ymd(2021, 11, 5), None));
        items.add(Event::new("test2".to_string(), NaiveDate::from_ymd(2021, 10, 6), None));
        items.add(Event::new("test3".to_string(), NaiveDate::from_ymd(2021, 10, 5), None));
        items.add(Event::new("test4".to_string(), NaiveDate::from_ymd(2021, 10, 5), None));
        items.add(Event::new("test5".to_string(), NaiveDate::from_ymd(2020, 10, 5), None));
        items.add(Event::new("test6".to_string(), NaiveDate::from_ymd(2020, 11, 30), None));

        let expected = vec![
            Event::new("test0".to_string(), NaiveDate::from_ymd(2021, 10, 5), None),
            Event::new("test3".to_string(), NaiveDate::from_ymd(2021, 10, 5), None),
            Event::new("test4".to_string(), NaiveDate::from_ymd(2021, 10, 5), None),
        ];

        let result: Vec<Event> = items.items_for_date(NaiveDate::from_ymd(2021, 10, 5))
            .iter()
            .cloned()
            .cloned()
            .collect();

        assert_eq!(result, expected);
    }

    mod mmvk_list_tests {
        use super::*;
        use crate::sync::*;

        #[derive(Debug, PartialEq, Clone)]
        struct TestMmvkItem {
            state: ItemState,
            body: String,
        }

        impl Ord for TestMmvkItem {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.body.cmp(&other.body)
            }
        }

        impl PartialOrd for TestMmvkItem {
            fn partial_cmp(&self, other: &Self) -> std::option::Option<std::cmp::Ordering> {
                Some(self.body.cmp(&other.body))
            }
        }

        impl Eq for TestMmvkItem {}

        impl TestMmvkItem {
            fn new(body: String) -> TestMmvkItem {
                TestMmvkItem {
                    state: ItemState::Neutral,
                    body,
                }
            }
        }

        impl MmvkItem for TestMmvkItem {
            fn for_date(&self, _: chrono::NaiveDate) -> bool {
                todo!()
            }
            fn state(&self) -> sync::ItemState {
                self.state
            }
            fn set_state(&mut self, state: sync::ItemState) {
                self.state = state;
            }
            fn ignore_state_eq(&self, other: &Self) -> bool {
                self.body == other.body
            }
        }

        #[test]
        fn mmvk_list_server_all_neutral() {
            let mut list = MmvkList::new(true);

            list.add(TestMmvkItem::new("Item 0".to_string()));
            list.add(TestMmvkItem::new("Item 1".to_string()));
            list.add(TestMmvkItem::new("Item 2".to_string()));

            let mut exp: Vec<TestMmvkItem> = vec![
                TestMmvkItem::new("Item 0".to_string()),
                TestMmvkItem::new("Item 1".to_string()),
                TestMmvkItem::new("Item 2".to_string()),
            ];
            exp.iter_mut().for_each(|x| {
                x.set_state(ItemState::Neutral);
            });

            let mut sorted = list.items().to_owned();
            sorted.sort();

            assert_eq!(sorted, exp);
        }

        #[test]
        fn mmvk_list_self_sync_removes_marked_and_sets_new_neutral() {
            let mut list = MmvkList::new(false);

            list.add(TestMmvkItem::new("Item 0".to_string()));
            list.add(TestMmvkItem::new("Item 1".to_string()));
            list.add(TestMmvkItem::new("Item 2".to_string()));
            list.add(TestMmvkItem::new("Item 3".to_string()));
            list.add(TestMmvkItem::new("Item 4".to_string()));

            list.mark_removed(1).unwrap();
            list.mark_removed(2).unwrap();

            list.sync_self();

            let mut exp: Vec<TestMmvkItem> = vec![
                TestMmvkItem::new("Item 0".to_string()),
                TestMmvkItem::new("Item 3".to_string()),
                TestMmvkItem::new("Item 4".to_string()),
            ];
            exp.iter_mut().for_each(|x| {
                x.set_state(ItemState::Neutral);
            });

            let mut sorted = list.items().to_owned();
            sorted.sort();

            assert_eq!(sorted, exp);
        }

        #[test]
        fn mmvk_list_sync_removes_marked_from_server() {
            let mut client_list = MmvkList::new(false);
            let mut server_list = MmvkList::new(true);

            client_list.add(TestMmvkItem::new("Item 0".to_string()));
            client_list.add(TestMmvkItem::new("Item 1".to_string()));
            client_list.add(TestMmvkItem::new("Item 2".to_string()));

            client_list.sync_self();

            client_list.mark_removed(1).unwrap();
            client_list.mark_removed(2).unwrap();

            server_list.add(TestMmvkItem::new("Item 0".to_string()));
            server_list.add(TestMmvkItem::new("Item 1".to_string()));

            client_list.sync(&mut server_list);

            let mut exp: Vec<TestMmvkItem> = vec![TestMmvkItem::new("Item 0".to_string())];
            exp.iter_mut().for_each(|x| {
                x.set_state(ItemState::Neutral);
            });

            let mut sorted = client_list.items().to_owned();
            sorted.sort();

            assert_eq!(sorted, exp);
            let mut sorted = server_list.items().to_owned();
            sorted.sort();

            assert_eq!(sorted, exp);
        }

        #[test]
        fn mmvk_list_sync_removes_marked_from_server_only_once() {
            let mut client_list = MmvkList::new(false);
            let mut server_list = MmvkList::new(true);

            client_list.add(TestMmvkItem::new("Item 0".to_string()));
            client_list.add(TestMmvkItem::new("Item 1".to_string()));

            client_list.sync_self();

            client_list.mark_removed(1).unwrap();

            server_list.add(TestMmvkItem::new("Item 0".to_string()));
            server_list.add(TestMmvkItem::new("Item 1".to_string()));
            server_list.add(TestMmvkItem::new("Item 1".to_string()));

            server_list.sync(&mut client_list);

            let mut exp: Vec<TestMmvkItem> = vec![
                TestMmvkItem::new("Item 0".to_string()),
                TestMmvkItem::new("Item 1".to_string()),
            ];
            exp.iter_mut().for_each(|x| {
                x.set_state(ItemState::Neutral);
            });

            let mut sorted = client_list.items().to_owned();
            sorted.sort();

            assert_eq!(sorted, exp);
            let mut sorted = server_list.items().to_owned();
            sorted.sort();

            assert_eq!(sorted, exp);
        }

        #[test]
        fn mmvk_list_sync_removes_marked_from_server_equal_times() {
            let mut client_list = MmvkList::new(false);
            let mut server_list = MmvkList::new(true);

            client_list.add(TestMmvkItem::new("Item 0".to_string()));

            client_list.sync_self();

            client_list.add(TestMmvkItem::new("Item 1".to_string()));
            client_list.add(TestMmvkItem::new("Item 1".to_string()));

            client_list.mark_removed(1).unwrap();
            client_list.mark_removed(2).unwrap();

            server_list.add(TestMmvkItem::new("Item 0".to_string()));
            server_list.add(TestMmvkItem::new("Item 1".to_string()));
            server_list.add(TestMmvkItem::new("Item 1".to_string()));

            client_list.sync(&mut server_list);

            let mut exp: Vec<TestMmvkItem> = vec![TestMmvkItem::new("Item 0".to_string())];
            exp.iter_mut().for_each(|x| {
                x.set_state(ItemState::Neutral);
            });

            let mut sorted = client_list.items().to_owned();
            sorted.sort();

            assert_eq!(sorted, exp);
            let mut sorted = server_list.items().to_owned();
            sorted.sort();

            assert_eq!(sorted, exp);
        }

        #[test]
        fn mmvk_list_sync_removes_nonnew_from_client() {
            let mut client_list = MmvkList::new(false);
            let mut server_list = MmvkList::new(true);

            client_list.add(TestMmvkItem::new("Item 0".to_string()));
            client_list.add(TestMmvkItem::new("Item 1".to_string()));
            client_list.add(TestMmvkItem::new("Item 2".to_string()));

            client_list.sync_self(); // Set items to neutral

            server_list.add(TestMmvkItem::new("Item 2".to_string()));

            server_list.sync(&mut client_list);

            let mut exp: Vec<TestMmvkItem> = vec![TestMmvkItem::new("Item 2".to_string())];
            exp.iter_mut().for_each(|x| {
                x.set_state(ItemState::Neutral);
            });

            let mut sorted = client_list.items().to_owned();
            sorted.sort();

            assert_eq!(sorted, exp);
            let mut sorted = server_list.items().to_owned();
            sorted.sort();

            assert_eq!(sorted, exp);
        }

        #[test]
        fn mmvk_list_sync_adds_new_from_client() {
            let mut client_list = MmvkList::new(false);
            let mut server_list = MmvkList::new(true);

            client_list.add(TestMmvkItem::new("Item 0".to_string()));

            client_list.sync_self();

            client_list.add(TestMmvkItem::new("Item 1".to_string()));
            client_list.add(TestMmvkItem::new("Item 2".to_string()));

            server_list.add(TestMmvkItem::new("Item 0".to_string()));

            client_list.sync(&mut server_list);

            let mut exp: Vec<TestMmvkItem> = vec![
                TestMmvkItem::new("Item 0".to_string()),
                TestMmvkItem::new("Item 1".to_string()),
                TestMmvkItem::new("Item 2".to_string()),
            ];
            exp.iter_mut().for_each(|x| {
                x.set_state(ItemState::Neutral);
            });

            let mut sorted = client_list.items().to_owned();
            sorted.sort();

            assert_eq!(sorted, exp);
            let mut sorted = server_list.items().to_owned();
            sorted.sort();

            assert_eq!(sorted, exp);
        }

        #[test]
        fn mmvk_list_sync_adds_new_from_server() {
            let mut client_list = MmvkList::new(false);
            let mut server_list = MmvkList::new(true);

            client_list.add(TestMmvkItem::new("Item 0".to_string()));

            client_list.sync_self();

            server_list.add(TestMmvkItem::new("Item 0".to_string()));
            server_list.add(TestMmvkItem::new("Item 1".to_string()));
            server_list.add(TestMmvkItem::new("Item 2".to_string()));

            client_list.sync(&mut server_list);

            let mut exp: Vec<TestMmvkItem> = vec![
                TestMmvkItem::new("Item 0".to_string()),
                TestMmvkItem::new("Item 1".to_string()),
                TestMmvkItem::new("Item 2".to_string()),
            ];
            exp.iter_mut().for_each(|x| {
                x.set_state(ItemState::Neutral);
            });

            let mut sorted = client_list.items().to_owned();
            sorted.sort();

            assert_eq!(sorted, exp);
            let mut sorted = server_list.items().to_owned();
            sorted.sort();

            assert_eq!(sorted, exp);
        }

        #[test]
        fn mmvk_list_sync_combines_correctly() {
            let mut client_list = MmvkList::new(false);
            let mut server_list = MmvkList::new(true);

            client_list.add(TestMmvkItem::new("Item 0".to_string()));
            client_list.add(TestMmvkItem::new("Item 1".to_string()));
            client_list.add(TestMmvkItem::new("Item 2".to_string()));

            client_list.sync_self();

            client_list.add(TestMmvkItem::new("Item 3".to_string()));

            client_list.mark_removed(1).unwrap();

            server_list.add(TestMmvkItem::new("Item 0".to_string()));
            server_list.add(TestMmvkItem::new("Item 1".to_string()));
            server_list.add(TestMmvkItem::new("Item 4".to_string()));

            client_list.sync(&mut server_list);

            let mut exp: Vec<TestMmvkItem> = vec![
                TestMmvkItem::new("Item 0".to_string()),
                TestMmvkItem::new("Item 3".to_string()),
                TestMmvkItem::new("Item 4".to_string()),
            ];
            exp.iter_mut().for_each(|x| {
                x.set_state(ItemState::Neutral);
            });

            let mut sorted = client_list.items().to_owned();
            sorted.sort();

            assert_eq!(sorted, exp);
            let mut sorted = server_list.items().to_owned();
            sorted.sort();

            assert_eq!(sorted, exp);
        }

        #[test]
        #[should_panic]
        fn mmvk_list_panics_if_both_servers() {
            let mut client: MmvkList<TestMmvkItem> = MmvkList::new(true);
            let mut client1: MmvkList<TestMmvkItem> = MmvkList::new(true);

            client.sync(&mut client1);
        }

        #[test]
        #[should_panic]
        fn mmvk_list_panics_if_neither_servers() {
            let mut client: MmvkList<TestMmvkItem> = MmvkList::new(false);
            let mut client1: MmvkList<TestMmvkItem> = MmvkList::new(false);

            client.sync(&mut client1);
        }
    }
}
