#![warn(missing_docs)]

use crate::{ItemState, MtcItem};
use chrono::prelude::*;

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
    pub fn new(body: String, date: NaiveDate) -> Event {
        Event {
            body,
            date,
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
}

impl MtcItem for TodoItem {
    /// Returns true if the `TodoItem` is for a given date.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::prelude::*;
    /// use mtc::{TodoItem, MtcItem};
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
    fn state(&self) -> ItemState {
        self.state
    }
    fn set_state(&mut self, new_state: ItemState) {
        self.state = new_state;
    }
    /// Returns true if self and other are equal except for the state which can differ.
    ///
    /// # Example
    ///
    /// ```
    ///
    /// use mtc::{TodoItem, ItemState, MtcItem};
    ///
    /// let mut item1 = TodoItem::new("Task 1".to_string(), None);
    /// item1.set_state(ItemState::New);
    ///
    /// let mut item2 = TodoItem::new("Task 1".to_string(), None);
    /// item2.set_state(ItemState::Neutral);
    ///
    /// assert!(item1.ignore_state_eq(&item2));
    /// ```
    fn ignore_state_eq(&self, other: &TodoItem) -> bool {
        self.body == other.body && self.weekday == other.weekday
    }
}

impl MtcItem for Task {
    /// Returns true if the `Task` is for a given date.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::prelude::*;
    /// use mtc::{Task, MtcItem};
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
    fn state(&self) -> ItemState {
        self.state
    }
    fn set_state(&mut self, new_state: ItemState) {
        self.state = new_state;
    }
    /// Returns true if self and other are equal except for the state which can differ.
    ///
    /// # Example
    ///
    /// ```
    ///
    /// use mtc::{Task, ItemState, MtcItem};
    ///
    /// let mut item1 = Task::new("Task 1".to_string(), 10, None);
    /// item1.set_state(ItemState::New);
    ///
    /// let mut item2 = Task::new("Task 1".to_string(), 10, None);
    /// item2.set_state(ItemState::Neutral);
    ///
    /// assert!(item1.ignore_state_eq(&item2));
    /// ```
    fn ignore_state_eq(&self, other: &Task) -> bool {
        self.body == other.body && self.weekday == other.weekday && self.duration == other.duration
    }
}

impl MtcItem for Event {
    /// Returns true if the `Event` is for a given date.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::prelude::*;
    /// use mtc::{Event, MtcItem};
    ///
    /// let item = Event::new("Do task A".to_string(), NaiveDate::from_ymd(2021, 11, 30));
    ///
    /// assert!(item.for_date(NaiveDate::from_ymd(2021, 11, 30)));
    /// assert!(!item.for_date(NaiveDate::from_ymd(2021, 12, 6)));
    /// ```
    fn for_date(&self, date: NaiveDate) -> bool {
        self.date == date
    }
    fn state(&self) -> ItemState {
        self.state
    }
    fn set_state(&mut self, new_state: ItemState) {
        self.state = new_state;
    }
    /// Returns true if self and other are equal except for the state which can differ.
    ///
    /// # Example
    ///
    /// ```
    ///
    /// use mtc::{Event, ItemState, MtcItem};
    /// use chrono::prelude::NaiveDate;
    ///
    /// let mut item1 = Event::new("Event 1".to_string(), NaiveDate::from_ymd(2022, 1, 1));
    /// item1.set_state(ItemState::New);
    ///
    /// let mut item2 = Event::new("Event 1".to_string(), NaiveDate::from_ymd(2022, 1, 1));
    /// item2.set_state(ItemState::Neutral);
    ///
    /// assert!(item1.ignore_state_eq(&item2));
    /// ```
    fn ignore_state_eq(&self, other: &Self) -> bool {
        self.body == other.body && self.date == other.date
    }
}

impl Ord for TodoItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.body.cmp(&other.body)
    }
}

impl PartialOrd for TodoItem {
    fn partial_cmp(&self, other: &Self) -> std::option::Option<std::cmp::Ordering> {
        Some(self.body.cmp(&other.body))
    }
}

impl Eq for TodoItem {}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.body.cmp(&other.body)
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> std::option::Option<std::cmp::Ordering> {
        Some(self.body.cmp(&other.body))
    }
}

impl Eq for Task {}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.body.cmp(&other.body)
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> std::option::Option<std::cmp::Ordering> {
        Some(self.body.cmp(&other.body))
    }
}

impl Eq for Event {}

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
        let ti = Event::new("test".to_string(), NaiveDate::from_ymd(2021, 12, 6));

        assert!(ti.for_date(date));
    }

    #[test]
    fn event_for_date_returns_false() {
        let date = NaiveDate::from_ymd(2021, 12, 6);
        let ti = Event::new("test".to_string(), NaiveDate::from_ymd(2021, 12, 5));

        assert!(!ti.for_date(date));
    }

    #[test]
    fn todo_item_ignore_state_eq_returns_true() {
        let mut item1 = TodoItem::new("Task 1".to_string(), None);
        item1.set_state(ItemState::New);

        let mut item2 = TodoItem::new("Task 1".to_string(), None);
        item2.set_state(ItemState::Neutral);

        assert!(item1.ignore_state_eq(&item2));
        assert!(item2.ignore_state_eq(&item1));
    }

    #[test]
    fn todo_item_ignore_state_eq_returns_false() {
        let mut item1 = TodoItem::new("Task 1".to_string(), Some(Weekday::Fri));
        item1.set_state(ItemState::New);

        let mut item2 = TodoItem::new("Task 1".to_string(), None);
        item2.set_state(ItemState::Neutral);

        assert!(!item1.ignore_state_eq(&item2));
        assert!(!item2.ignore_state_eq(&item1));
    }

    #[test]
    fn task_ignore_state_eq_returns_true() {
        let mut item1 = Task::new("Task 1".to_string(), 30, None);
        item1.set_state(ItemState::New);

        let mut item2 = Task::new("Task 1".to_string(), 30, None);
        item2.set_state(ItemState::Neutral);

        assert!(item1.ignore_state_eq(&item2));
        assert!(item2.ignore_state_eq(&item1));
    }

    #[test]
    fn task_ignore_state_eq_returns_false() {
        let mut item1 = Task::new("Task 1".to_string(), 31, None);
        item1.set_state(ItemState::New);

        let mut item2 = Task::new("Task 1".to_string(), 30, None);
        item2.set_state(ItemState::Neutral);

        assert!(!item1.ignore_state_eq(&item2));
        assert!(!item2.ignore_state_eq(&item1));
    }

    #[test]
    fn event_ignore_state_eq_returns_true() {
        let mut item1 = Event::new("Event 1".to_string(), NaiveDate::from_ymd(2022, 1, 1));
        item1.set_state(ItemState::New);

        let mut item2 = Event::new("Event 1".to_string(), NaiveDate::from_ymd(2022, 1, 1));
        item2.set_state(ItemState::Neutral);

        assert!(item1.ignore_state_eq(&item2));
        assert!(item2.ignore_state_eq(&item1));
    }

    #[test]
    fn event_ignore_state_eq_returns_false() {
        let mut item1 = Event::new("Event 1".to_string(), NaiveDate::from_ymd(2022, 1, 2));
        item1.set_state(ItemState::New);

        let mut item2 = Event::new("Event 1".to_string(), NaiveDate::from_ymd(2022, 1, 1));
        item2.set_state(ItemState::Neutral);

        assert!(!item1.ignore_state_eq(&item2));
        assert!(!item2.ignore_state_eq(&item1));
    }
}
