//! # MMVK Lib
//!
//! This API provides the base functionality for mmvk. It can be also used for creating apps that
//! can sync with the mmvk CLI app or serve as an additional interface.

use chrono::prelude::*;

/// A short term task that should be done on a optionally given weekday.
#[derive(Debug, PartialEq, Clone)]
pub struct TodoItem {
    weekday: Option<Weekday>,
    body: String,
}

/// A repetitive task with a duration in minutes for a optionally given day.
#[derive(Debug, PartialEq, Clone)]
pub struct Task {
    weekday: Option<Weekday>,
    body: String,
    duration: u32,
}

/// An event that will happen on a given date and optionally a time.
#[derive(Debug, PartialEq, Clone)]
pub struct Event {
    date: NaiveDate,
    time: Option<NaiveTime>,
    body: String,
}

impl TodoItem {
    /// Creates a new `TodoItem` with a given body and optionally a weekday.
    pub fn new(body: String, weekday: Option<Weekday>) -> TodoItem {
        TodoItem { weekday, body }
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
        Event { body, date, time }
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
}

/// Returns a `Vec<&T>` containing references to all items that are for a given date.
///
/// # Example
///
/// ```
/// use chrono::prelude::*;
/// use mmvk::{TodoItem, mmvk_items_date};
///
/// let items = vec![
///     TodoItem::new("Do task A".to_string(), None),
///     TodoItem::new("To dask B".to_string(), Some(Weekday::Tue)),
///     TodoItem::new("Do task C".to_string(), Some(Weekday::Wed)),
///     TodoItem::new("D".to_string(), Some(Weekday::Fri)),
///     TodoItem::new("F".to_string(), Some(Weekday::Mon)),
/// ];
///
/// let expected = vec![
///     TodoItem::new("Do task A".to_string(), None),
///     TodoItem::new("To dask B".to_string(), Some(Weekday::Tue)),
/// ];
///
/// // 2021.11.30 was a Tuesday.
///
/// // The result is cloned to allow for comparison.
/// let result: Vec<TodoItem> = mmvk_items_date(&items, NaiveDate::from_ymd(2021, 11, 30))
///     .iter()
///     .cloned()
///     .cloned()
///     .collect();
///
/// assert_eq!(result, expected);
/// ```
pub fn mmvk_items_date<'a, T: MmvkItem>(vec: &'a Vec<T>, date: NaiveDate) -> Vec<&'a T> {
    let mut out = Vec::new();

    for item in vec.iter() {
        if item.for_date(date) {
            out.push(item);
        }
    }

    out
}

/// Returns a `Vec<&T>` containing references to all items that are for today.
///
/// # Example
///
/// ```
/// use chrono::prelude::*;
/// use mmvk::{TodoItem, mmvk_items_today};
///
/// let items = vec![
///     TodoItem::new("Do task A".to_string(), None),
///     TodoItem::new("To dask B".to_string(), Some(Local::today().weekday())),
///     TodoItem::new("Do task C".to_string(), Some(Local::today().weekday().pred())),
///     TodoItem::new("D".to_string(), Some(Local::today().weekday().pred().pred())),
///     TodoItem::new("F".to_string(), Some(Local::today().weekday().succ())),
/// ];
///
/// let expected = vec![
///     TodoItem::new("Do task A".to_string(), None),
///     TodoItem::new("To dask B".to_string(), Some(Local::today().weekday())),
/// ];
///
/// // The result is cloned to allow for comparison.
/// let result: Vec<TodoItem> = mmvk_items_today(&items)
///     .iter()
///     .cloned()
///     .cloned()
///     .collect();
///
/// assert_eq!(result, expected);
/// ```
pub fn mmvk_items_today<'a, T: MmvkItem>(vec: &'a Vec<T>) -> Vec<&'a T> {
    let mut out = Vec::new();

    for item in vec.iter() {
        if item.for_today() {
            out.push(item);
        }
    }

    out
}

/// Returns a `Vec<&T>` containing references to all items that are for a given weekday.
///
/// # Example
///
/// ```
/// use chrono::prelude::*;
/// use mmvk::{TodoItem, mmvk_items_weekday};
///
/// let items = vec![
///     TodoItem::new("Do task A".to_string(), None),
///     TodoItem::new("To dask B".to_string(), Some(Weekday::Tue)),
///     TodoItem::new("Do task C".to_string(), Some(Weekday::Wed)),
///     TodoItem::new("D".to_string(), Some(Weekday::Fri)),
///     TodoItem::new("F".to_string(), Some(Weekday::Mon)),
/// ];
///
/// let expected = vec![
///     TodoItem::new("Do task A".to_string(), None),
///     TodoItem::new("To dask B".to_string(), Some(Weekday::Tue)),
/// ];
///
/// // The result is cloned to allow for comparison.
/// let result: Vec<TodoItem> = mmvk_items_weekday(&items, Weekday::Tue)
///     .iter()
///     .cloned()
///     .cloned()
///     .collect();
///
/// assert_eq!(result, expected);
/// ```
pub fn mmvk_items_weekday<'a, T: MmvkItem>(vec: &'a Vec<T>, weekday: Weekday) -> Vec<&'a T> {
    let mut out = Vec::new();

    for item in vec.iter() {
        if item.for_weekday(weekday) {
            out.push(item);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

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
        }

        let item = TestItem {};

        assert!(!item.for_today());
    }

    #[test]
    fn mmvk_item_for_weekday_returns_true() {
        struct TestItem {}
        impl MmvkItem for TestItem {
            fn for_date(&self, date: NaiveDate) -> bool {
                date == NaiveDate::from_ymd(2021, 12, 6)
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
        }

        let item = TestItem {};

        assert!(!item.for_weekday(Weekday::Fri));
    }

    #[test]
    fn mmvk_items_date_returns_expected() {
        let items = vec![
            TodoItem::new("test0".to_string(), None),
            TodoItem::new("test1".to_string(), Some(Weekday::Tue)),
            TodoItem::new("test2".to_string(), Some(Weekday::Wed)),
            TodoItem::new("test3".to_string(), None),
            TodoItem::new("test4".to_string(), Some(Weekday::Tue)),
        ];

        let expected = vec![
            TodoItem::new("test0".to_string(), None),
            TodoItem::new("test1".to_string(), Some(Weekday::Tue)),
            TodoItem::new("test3".to_string(), None),
            TodoItem::new("test4".to_string(), Some(Weekday::Tue)),
        ];

        let result: Vec<TodoItem> = mmvk_items_date(&items, NaiveDate::from_ymd(2021, 11, 30))
            .iter()
            .cloned()
            .cloned()
            .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn mmvk_items_today_returns_expected() {
        let today = Local::now().weekday();

        let items = vec![
            Task::new("test0".to_string(), 40, Some(today)),
            Task::new("test1".to_string(), 30, Some(today)),
            Task::new("test2".to_string(), 10, Some(today.pred())),
            Task::new("test3".to_string(), 0, None),
            Task::new("test4".to_string(), 90, Some(today.succ().succ())),
        ];

        let expected = vec![
            Task::new("test0".to_string(), 40, Some(today)),
            Task::new("test1".to_string(), 30, Some(today)),
            Task::new("test3".to_string(), 0, None),
        ];

        let result: Vec<Task> = mmvk_items_today(&items).iter().cloned().cloned().collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn mmvk_items_weekday_returns_expected() {
        let items = vec![
            Task::new("test0".to_string(), 40, Some(Weekday::Fri)),
            Task::new("test1".to_string(), 30, Some(Weekday::Fri)),
            Task::new("test2".to_string(), 10, Some(Weekday::Mon)),
            Task::new("test3".to_string(), 0, None),
            Task::new("test4".to_string(), 90, Some(Weekday::Wed)),
        ];

        let expected = vec![
            Task::new("test0".to_string(), 40, Some(Weekday::Fri)),
            Task::new("test1".to_string(), 30, Some(Weekday::Fri)),
            Task::new("test3".to_string(), 0, None),
        ];

        let result: Vec<Task> = mmvk_items_weekday(&items, Weekday::Fri)
            .iter()
            .cloned()
            .cloned()
            .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn mmvk_items_date_returns_expected_for_events() {
        let items = vec![
            Event::new("test0".to_string(), NaiveDate::from_ymd(2021, 10, 5), None),
            Event::new("test1".to_string(), NaiveDate::from_ymd(2021, 11, 5), None),
            Event::new("test2".to_string(), NaiveDate::from_ymd(2021, 10, 6), None),
            Event::new("test3".to_string(), NaiveDate::from_ymd(2021, 10, 5), None),
            Event::new("test4".to_string(), NaiveDate::from_ymd(2021, 10, 5), None),
            Event::new("test5".to_string(), NaiveDate::from_ymd(2020, 10, 5), None),
            Event::new("test6".to_string(), NaiveDate::from_ymd(2020, 11, 30), None),
        ];

        let expected = vec![
            Event::new("test0".to_string(), NaiveDate::from_ymd(2021, 10, 5), None),
            Event::new("test3".to_string(), NaiveDate::from_ymd(2021, 10, 5), None),
            Event::new("test4".to_string(), NaiveDate::from_ymd(2021, 10, 5), None),
        ];

        let result: Vec<Event> = mmvk_items_date(&items, NaiveDate::from_ymd(2021, 10, 5))
            .iter()
            .cloned()
            .cloned()
            .collect();

        assert_eq!(result, expected);
    }
}
