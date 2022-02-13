use crate::{ItemState, MtcItem};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// A short-term task that should be done on a optionally given weekday.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Todo {
    weekday: Option<Weekday>,
    body: String,
    state: ItemState,
    id: usize,
}

/// A repeating task with a duration in minutes for a optionally given weekday.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Task {
    weekdays: [bool; 7],
    body: String,
    duration: u32,
    state: ItemState,
    id: usize,
}

/// An event that will happen on a given date.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Event {
    date: NaiveDate,
    body: String,
    state: ItemState,
    id: usize,
}

impl Todo {
    /// Creates a new `Todo` with a given body and optionally a weekday.
    pub fn new(body: String, weekday: Option<Weekday>) -> Todo {
        Todo {
            weekday,
            body,
            state: ItemState::Neutral,
            id: 0,
        }
    }

    /// Returns a reference to the body of the `Todo`.
    pub fn body(&self) -> &String {
        &self.body
    }

    /// Returns the optionally specified weekday of the `Todo`.
    pub fn weekday(&self) -> Option<Weekday> {
        self.weekday
    }

    /// Sets the optional weekday of the `Todo`.
    pub fn set_weekday(&mut self, new_weekday: Option<Weekday>) {
        self.weekday = new_weekday;
    }
}

impl Task {
    /// Creates a new `Task` with a given body, duration in minutes and optionally a weekday.
    pub fn new(body: String, duration: u32, weekday: Option<Weekday>) -> Task {
        let mut weekdays: [bool; 7] = [false, false, false, false, false, false, false];

        if let Some(day) = weekday {
            weekdays[(day.number_from_monday() - 1) as usize] = true;
        }

        Task {
            weekdays,
            body,
            duration,
            state: ItemState::Neutral,
            id: 0,
        }
    }

    /// Returns a reference to the body of the `Task`.
    pub fn body(&self) -> &String {
        &self.body
    }

    /// Returns the duration of the `Task`.
    pub fn duration(&self) -> u32 {
        self.duration
    }

    /// Returns a array defining all the weekdays this task is for. 0th element indicates monday.
    /// A value of `true` indicates that a task is for the day.
    pub fn weekdays(&self) -> [bool; 7] {
        self.weekdays.clone()
    }

    /// Sets the array defining all the weekdays this task is for. 0th element indicates monday.
    /// A value of `true` indicates that a task is for the day.
    pub fn set_weekdays(&mut self, weekdays: [bool; 7]) {
        self.weekdays = weekdays;
    }

    /// Returns true if the `Task` is for a weekday.
    pub fn is_for_weekday(&self, weekday: Weekday) -> bool {
        let mut all_false = true;
        for b in self.weekdays {
            if b {
                all_false = false;
            }
        }

        // If all days are false then every day is for the task.
        self.weekdays[(weekday.number_from_monday() - 1) as usize] || all_false
    }

    /// Set if a weekday is for the `Task`. `true` indicates that the task is for the weekday.
    pub fn set_for_weekday(&mut self, weekday: Weekday, is_for: bool) {
        self.weekdays[(weekday.number_from_monday() - 1) as usize] = is_for;
    }
}

impl Event {
    /// Creates a new `Event` with a given body, date and optionally a time.
    pub fn new(body: String, date: NaiveDate) -> Event {
        Event {
            body,
            date,
            state: ItemState::Neutral,
            id: 0,
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

impl MtcItem for Todo {
    /// Returns true if the `Todo` is for a given date.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::prelude::*;
    /// use mtc::{Todo, MtcItem};
    ///
    /// let item = Todo::new("Do task A".to_string(), Some(Weekday::Mon));
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
    /// Returns true if self and other are equal excluding the `ItemState` of both self and other.
    ///
    /// # Example
    ///
    /// ```
    ///
    /// use mtc::{Todo, ItemState, MtcItem};
    ///
    /// let mut item1 = Todo::new("Task 1".to_string(), None);
    /// item1.set_state(ItemState::New);
    ///
    /// let mut item2 = Todo::new("Task 1".to_string(), None);
    /// item2.set_state(ItemState::Neutral);
    ///
    /// assert!(item1.ignore_state_eq(&item2));
    /// ```
    fn ignore_state_eq(&self, other: &Todo) -> bool {
        self.body == other.body && self.weekday == other.weekday
    }
    fn id(&self) -> usize {
        self.id
    }
    fn set_id(&mut self, new_id: usize) {
        self.id = new_id;
    }
    fn expired(&self) -> bool {
        false
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
        self.is_for_weekday(date.weekday())
    }
    fn state(&self) -> ItemState {
        self.state
    }
    fn set_state(&mut self, new_state: ItemState) {
        self.state = new_state;
    }
    /// Returns true if self and other are equal excluding the `ItemState` of both self and other.
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
        self.body == other.body && self.weekdays == other.weekdays && self.duration == other.duration
    }
    fn id(&self) -> usize {
        self.id
    }
    fn set_id(&mut self, new_id: usize) {
        self.id = new_id;
    }
    fn expired(&self) -> bool {
        false
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
    /// Returns true if self and other are equal excluding the `ItemState` of both self and other.
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
    fn id(&self) -> usize {
        self.id
    }
    fn set_id(&mut self, new_id: usize) {
        self.id = new_id;
    }
    fn expired(&self) -> bool {
        let today = Local::today().naive_local();
        return self.date.signed_duration_since(today).num_days() < -3;
    }
}

impl Ord for Todo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.body.cmp(&other.body)
    }
}

impl PartialOrd for Todo {
    fn partial_cmp(&self, other: &Self) -> std::option::Option<std::cmp::Ordering> {
        Some(self.body.cmp(&other.body))
    }
}

impl Eq for Todo {}

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
        let order = self.date.cmp(&other.date);
        if order == std::cmp::Ordering::Equal {
            self.body.cmp(&other.body)
        } else {
            order
        }
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> std::option::Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Eq for Event {}

impl Display for Todo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{} (ID: {})", self.body, self.id)
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "{}: {} minutes (ID: {})",
            self.body, self.duration, self.id
        )
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}: {} (ID: {})", self.date, self.body, self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo_item_for_date_returns_true() {
        let date = NaiveDate::from_ymd(2021, 12, 6);
        let ti = Todo::new("test".to_string(), Some(Weekday::Mon));

        assert!(ti.for_date(date));
    }

    #[test]
    fn todo_item_for_date_returns_false() {
        let date = NaiveDate::from_ymd(2021, 12, 6);
        let ti = Todo::new("test".to_string(), Some(Weekday::Tue));

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
        let mut item1 = Todo::new("Task 1".to_string(), None);
        item1.set_state(ItemState::New);

        let mut item2 = Todo::new("Task 1".to_string(), None);
        item2.set_state(ItemState::Neutral);

        assert!(item1.ignore_state_eq(&item2));
        assert!(item2.ignore_state_eq(&item1));
    }

    #[test]
    fn todo_item_ignore_state_eq_returns_false() {
        let mut item1 = Todo::new("Task 1".to_string(), Some(Weekday::Fri));
        item1.set_state(ItemState::New);

        let mut item2 = Todo::new("Task 1".to_string(), None);
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

    #[test]
    fn event_order_works() {
        let mut events = vec![
            Event::new("2 Event".to_string(), NaiveDate::from_ymd(2022, 1, 1)),
            Event::new("1 Event".to_string(), NaiveDate::from_ymd(2022, 1, 1)),
            Event::new("0 Event".to_string(), NaiveDate::from_ymd(2022, 1, 2)),
            Event::new("9 Event".to_string(), NaiveDate::from_ymd(2021, 1, 1)),
            Event::new("1 Event".to_string(), NaiveDate::from_ymd(2022, 1, 2)),
        ];

        let expected = vec![
            Event::new("9 Event".to_string(), NaiveDate::from_ymd(2021, 1, 1)),
            Event::new("1 Event".to_string(), NaiveDate::from_ymd(2022, 1, 1)),
            Event::new("2 Event".to_string(), NaiveDate::from_ymd(2022, 1, 1)),
            Event::new("0 Event".to_string(), NaiveDate::from_ymd(2022, 1, 2)),
            Event::new("1 Event".to_string(), NaiveDate::from_ymd(2022, 1, 2)),
        ];

        events.sort();

        assert_eq!(events, expected);
    }

    #[test]
    fn todo_item_display_works() {
        let todo_item = Todo::new("Do Task 1".to_string(), Some(Weekday::Mon));
        assert_eq!(format!("{}", todo_item), "Do Task 1 (ID: 0)");
    }

    #[test]
    fn task_display_works() {
        let task = Task::new("Do Task 1".to_string(), 10, Some(Weekday::Mon));
        assert_eq!(format!("{}", task), "Do Task 1: 10 minutes (ID: 0)");
    }

    #[test]
    fn event_display_works() {
        let event = Event::new("Event 1".to_string(), NaiveDate::from_ymd(2021, 1, 5));
        assert_eq!(format!("{}", event), "2021-01-05: Event 1 (ID: 0)");
    }

    #[test]
    fn event_is_expired_works() {
        let today = Local::today().naive_local();
        let event = Event::new("Event 1".to_string(), today);
        assert!(!event.expired());

        let event = Event::new("Event 1".to_string(), today.pred());
        assert!(!event.expired());

        let event = Event::new("Event 1".to_string(), today.succ());
        assert!(!event.expired());

        let event = Event::new("Event 1".to_string(), today.pred().pred().pred());
        assert!(!event.expired());

        let event = Event::new("Event 1".to_string(), today.pred().pred().pred().pred());
        assert!(event.expired());
    }
}
