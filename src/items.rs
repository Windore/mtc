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
    fn ignore_state_eq(&self, _: &TodoItem) -> bool {
        todo!()
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
    fn ignore_state_eq(&self, _: &Task) -> bool {
        todo!()
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
    /// let item = Event::new("Do task A".to_string(), NaiveDate::from_ymd(2021, 11, 30), None);
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
    fn ignore_state_eq(&self, _: &Self) -> bool {
        todo!()
    }
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
}
