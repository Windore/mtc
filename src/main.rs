use mtc::*;
use serde::{de::DeserializeOwned, Serialize};
use std::env;
use std::fmt::Display;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::Path;
use std::str::FromStr;
use std::{fs, fs::File};

pub struct Items {
    pub todo_items: MtcList<TodoItem>,
    pub tasks: MtcList<Task>,
    pub events: MtcList<Event>,
}

mod commands {
    use super::*;
    use chrono::prelude::*;

    pub fn handle_command(mut items: Items) -> Items {
        // There probably is a better way to do this. However the overhead is insignificant so it doesn't matter that much.
        let args: Vec<String> = env::args().collect();
        let mut args = args.iter().map(|s| s.as_str());

        args.next();

        match args.next() {
            Some("show") => show_cmd::show(&items, args),
            Some("help") => help(),
            Some("add") => add_cmd::add(&mut items, args),
            Some("remove") => remove(&mut items, args),
            None => {
                eprintln!("Not enough arguments.");
                tip();
            }
            _ => {
                eprintln!("Unknown command.");
                tip();
            }
        }

        items
    }

    fn help() {
        println!("TODO: Add help");
    }

    fn tip() {
        println!("Use: 'mtc help' for help.");
    }

    fn remove<'a, T>(items: &mut Items, mut args: T)
    where
        T: Iterator<Item = &'a str>,
    {
        match args.next() {
            Some("todo-item") => {
                loop {
                    let id = read_id();
                    if let Err(e) = items.todo_items.mark_removed(id) {
                        eprintln!("{}", e);
                    } else {
                        break;
                    }
                }
            },
            Some("task") => {
                loop {
                    let id = read_id();
                    if let Err(e) = items.tasks.mark_removed(id) {
                        eprintln!("{}", e);
                    } else {
                        break;
                    }
                }
            },
            Some("event") => {
                loop {
                    let id = read_id();
                    if let Err(e) = items.events.mark_removed(id) {
                        eprintln!("{}", e);
                    } else {
                        break;
                    }
                }
            },
            None => {
                eprintln!("Not enough arguments.");
                tip();
            },
            _ => {
                eprintln!("No type specified.");
                tip();
            },
        }
    }

    fn read_id() -> usize {
        loop {
            print!("Input an ID: ");
            io::stdout().flush().expect("Failed to flush stdout.");
            let mut inp = String::new();
            io::stdin()
                .read_line(&mut inp)
                .expect("Failed to read stdin.");
            inp = inp.replace('\n', "");

            match usize::from_str(&inp) {
                Ok(n) => return n,
                Err(_) => {
                    eprintln!("Cannot parse '{}' to a number.", &inp);
                }
            }
        }
    }

    mod add_cmd {
        use super::*;

        pub fn add<'a, T>(items: &mut Items, mut args: T)
        where
            T: Iterator<Item = &'a str>,
        {
            match args.next() {
                Some("todo-item") => add_todo_item(items),
                Some("task") => add_task(items),
                Some("event") => add_event(items),
                Some(typ) => {
                    eprintln!("Unknown type: '{}'", typ);
                    tip();
                }
                None => {
                    eprintln!("No type specified.");
                    tip();
                }
            }
        }

        fn add_todo_item(items: &mut Items) {
            println!("New todo-item: ");
            let body = read_body();
            let weekday = read_weekday();
            items.todo_items.add(TodoItem::new(body, weekday))
        }

        fn add_task(items: &mut Items) {
            println!("New task: ");
            let body = read_body();
            let duration = read_duration();
            let weekday = read_weekday();
            items.tasks.add(Task::new(body, duration, weekday))
        }

        fn add_event(items: &mut Items) {
            println!("New event: ");
            let body = read_body();
            let date = read_date();
            items.events.add(Event::new(body, date))
        }

        fn read_weekday() -> Option<Weekday> {
            loop {
                print!("Input a weekday (empty for none): ");
                io::stdout().flush().expect("Failed to flush stdout.");
                let mut inp = String::new();
                io::stdin()
                    .read_line(&mut inp)
                    .expect("Failed to read stdin.");
                inp = inp.replace('\n', "");

                if inp.trim().len() == 0 {
                    return None;
                }

                match Weekday::from_str(&inp) {
                    Ok(day) => return Some(day),
                    Err(_) => {
                        eprintln!("Cannot parse '{}' to a weekday.", &inp);
                    }
                }
            }
        }

        fn read_duration() -> u32 {
            loop {
                print!("Input a duration in minutes: ");
                io::stdout().flush().expect("Failed to flush stdout.");
                let mut inp = String::new();
                io::stdin()
                    .read_line(&mut inp)
                    .expect("Failed to read stdin.");
                inp = inp.replace('\n', "");

                match u32::from_str(&inp) {
                    Ok(n) => return n,
                    Err(_) => {
                        eprintln!("Cannot parse '{}' to a number.", &inp);
                    }
                }
            }
        }

        fn read_body() -> String {
            loop {
                print!("Input a body: ");
                io::stdout().flush().expect("Failed to flush stdout.");
                let mut inp = String::new();
                io::stdin()
                    .read_line(&mut inp)
                    .expect("Failed to read stdin.");
                inp = inp.replace('\n', "");

                return inp;
            }
        }

        fn read_date() -> NaiveDate {
            loop {
                print!("Input a date (yyyy-mm-dd): ");
                io::stdout().flush().expect("Failed to flush stdout.");
                let mut inp = String::new();
                io::stdin()
                    .read_line(&mut inp)
                    .expect("Failed to read stdin.");
                inp = inp.replace('\n', "");

                match NaiveDate::from_str(&inp) {
                    Ok(date) => return date,
                    Err(_) => {
                        eprintln!("Cannot parse '{}' to a date.", &inp);
                    }
                }
            }
        }
    }

    mod show_cmd {
        use super::*;

        const WEEKDAYS: &[Weekday] = &[
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ];

        pub fn show<'a, T>(items: &Items, mut args: T)
        where
            T: Iterator<Item = &'a str>,
        {
            match args.next() {
                Some("todo-items") => show_all_todo_items(items),
                Some("tasks") => show_all_tasks(items),
                Some("events") => show_all_events(items),
                Some("today") => show_today(items),
                Some("week") => show_week(items),
                Some("month") => show_month(items),
                Some(weekday) => {
                    if let Ok(wd) = weekday.parse::<Weekday>() {
                        show_weekday(items, wd);
                    } else {
                        eprintln!("Cannot parse '{}' to a weekday.", weekday);
                        tip();
                    }
                }
                None => show_all(items),
            }
        }

        fn show_all(items: &Items) {
            for wd in WEEKDAYS.iter() {
                println!("{}", wd);
                println!("\tTodo-Items: ");
                show_list_weekday(&items.todo_items, *wd);
                println!("");
                println!("\tTasks: ");
                show_list_weekday(&items.tasks, *wd);
                println!("");
            }
            show_all_events(&items);
        }

        fn show_weekday(items: &Items, weekday: Weekday) {
            let mut date = Local::today();
            while date.weekday() != weekday {
                date = date.succ();
            }
            show_all_date(&items, date.naive_local());
        }

        fn show_today(items: &Items) {
            let day = Local::today();
            show_all_date(items, day.naive_local());
        }

        fn show_week(items: &Items) {
            let mut day = Local::today().naive_local();
            let orig_day = day.weekday();

            while day.succ().weekday() != orig_day {
                show_all_date(items, day);
                day = day.succ();
            }
        }

        fn show_month(items: &Items) {
            let mut day = Local::today().naive_local();

            for _ in 0..30 {
                show_all_date(items, day);
                day = day.succ();
            }
        }

        fn show_all_todo_items(items: &Items) {
            for wd in WEEKDAYS.iter() {
                println!("{}", wd);
                println!("\tTodo-Items: ");
                show_list_weekday(&items.todo_items, *wd);
            }
        }

        fn show_all_tasks(items: &Items) {
            for wd in WEEKDAYS.iter() {
                println!("{}", wd);
                println!("\tTasks: ");
                show_list_weekday(&items.tasks, *wd);
            }
        }

        fn show_all_events(items: &Items) {
            println!("Events: ");
            let mut events_vec = items.events.items();
            events_vec.sort();
            for i in events_vec.iter() {
                println!("\t{}", i);
            }
        }

        fn show_all_date(items: &Items, date: NaiveDate) {
            println!("{} {}:", date.weekday(), date);
            println!("\tTodo-Items: ");
            show_list_date(&items.todo_items, date);

            println!("\tTasks: ");
            show_list_date(&items.tasks, date);

            println!("\tEvents: ");
            show_list_date(&items.events, date);
        }

        fn show_list_date<T: MtcItem + Clone + Ord + Display>(list: &MtcList<T>, date: NaiveDate) {
            let mut items_vec = list.items_for_date(date);
            items_vec.sort();
            show_list(&items_vec);
        }

        fn show_list_weekday<T: MtcItem + Clone + Ord + Display>(
            list: &MtcList<T>,
            weekday: Weekday,
        ) {
            let mut items_vec = list.items_for_weekday(weekday);
            items_vec.sort();
            show_list(&items_vec);
        }

        fn show_list<T: Display>(list: &[&T]) {
            for i in list.iter() {
                println!("\t\t{}", i);
            }
        }
    }
}

fn main() {
    if let Some(dir) = dirs::data_dir() {
        let dir = dir.join(Path::new("mtc/"));
        if let Err(msg) = fs::create_dir_all(&dir) {
            eprintln!("Failed to create missing directories.");
            eprintln!("{}", msg);
            return;
        }

        let modified_items = match read_items(&dir) {
            Ok(i) => commands::handle_command(i),
            Err(msg) => {
                eprintln!("Reading saved items failed.");
                eprintln!("{}", msg);
                return;
            }
        };

        if let Err(msg) = write_items(&dir, modified_items) {
            eprintln!("Writing items failed.");
            eprintln!("{}", msg);
        }
    } else {
        eprintln!("Non supported OS");
        return;
    }
}

fn read_items(dir: &Path) -> Result<Items, String> {
    let todo_item_file = dir.join(Path::new("todo-items.json"));
    let task_file = dir.join(Path::new("tasks.json"));
    let event_file = dir.join(Path::new("events.json"));

    let todo_items = read_item(&todo_item_file)?;
    let tasks = read_item(&task_file)?;
    let events = read_item(&event_file)?;

    Ok(Items {
        todo_items,
        tasks,
        events,
    })
}

fn read_item<T: MtcItem + Clone + DeserializeOwned>(path: &Path) -> Result<MtcList<T>, String> {
    if path.exists() {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).map_err(|e| e.to_string())
    } else {
        Ok(MtcList::new(false))
    }
}

fn write_items(dir: &Path, items: Items) -> Result<(), String> {
    let todo_item_file = dir.join(Path::new("todo-items.json"));
    let task_file = dir.join(Path::new("tasks.json"));
    let event_file = dir.join(Path::new("events.json"));

    write_item(items.todo_items, &todo_item_file)?;
    write_item(items.tasks, &task_file)?;
    write_item(items.events, &event_file)?;

    Ok(())
}

fn write_item<T: MtcItem + Clone + DeserializeOwned + Serialize>(
    item: MtcList<T>,
    path: &Path,
) -> Result<(), String> {
    let file = File::create(path).map_err(|e| e.to_string())?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &item).map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;
    Ok(())
}
