use mtc::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::env;
use std::fmt::Display;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::Path;
use std::str::FromStr;
use std::thread;
use std::time::{Duration, Instant};
use std::{fs, fs::File};

pub struct Items {
    pub todos: MtcList<Todo>,
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
            Some("do") => do_task(&items, args),
            Some("sync") => sync::sync(&mut items, args),
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
        println!("MTC - My Time Contract - a CLI time management app.");
        println!("usage: mtc <command> [<args>]");
        println!("Read the README.md for more information");
        println!("");
        println!("Commands:");
        println!("\tshow [<type> | weekday | today | month]");
        println!("\tShows saved items.\n");
        println!("\tadd <type>");
        println!("\tAdds a item of a given type.\n");
        println!("\tremove <type> <id>");
        println!("\tRemoves a item of a given type.\n");
        println!("\tdo <task id>");
        println!("\tShows a timer for a task.\n");
        println!("\tsync [self | overwrite]");
        println!("\tSyncs all items with a server specified by a config. Using 'self' or 'overwrite' isn't usually necessary nor recommended.\n");
        println!("\thelp");
        println!("\tShows this help output.");
    }

    fn tip() {
        println!("Use: 'mtc help' for help.");
    }

    fn do_task<'a, T>(items: &Items, args: T)
    where
        T: Iterator<Item = &'a str>,
    {
        let id = read_id(args);
        if id.is_err() {
            eprintln!("{}", id.err().unwrap());
            return;
        }
        let id = id.unwrap();

        if let Some(task) = items.tasks.items().iter().find(|item| item.id() == id) {
            let mut millis_left = task.duration() as u128 * 60_000;
            loop {
                let now = Instant::now();
                // "Clear" the line.
                print!("\r                                 ");
                let seconds_left = millis_left / 1000;
                let hours = seconds_left / 3600;
                let minutes = (seconds_left - hours * 3600) / 60;
                let seconds = seconds_left - hours * 3600 - minutes * 60;
                print!("\rTime left: {} h {} min {} s", hours, minutes, seconds);
                io::stdout().flush().expect("Failed to flush stdout.");
                thread::sleep(Duration::from_millis(500));
                if let Some(n) = millis_left.checked_sub(now.elapsed().as_millis()) {
                    millis_left = n;
                } else {
                    // Print here this one last time since the timer could otherwise stop at 0 h 0 min 1 s for example
                    // which is quite annoying.
                    println!("\rTime left: 0 h 0 min 0 s");
                    return;
                }
            }
        } else {
            eprintln!("No task with the given ID found.");
        }
    }

    fn remove<'a, T>(items: &mut Items, mut args: T)
    where
        T: Iterator<Item = &'a str>,
    {
        match args.next() {
            Some("todo") => {
                let id = read_id(args);
                if id.is_err() {
                    eprintln!("{}", id.err().unwrap());
                    return;
                }
                let id = id.unwrap();
                if let Err(e) = items.todos.mark_removed(id) {
                    eprintln!("{}", e);
                }
            },
            Some("task") => {
                let id = read_id(args);
                if id.is_err() {
                    eprintln!("{}", id.err().unwrap());
                    return;
                }
                let id = id.unwrap();
                if let Err(e) = items.tasks.mark_removed(id) {
                    eprintln!("{}", e);
                }
            },
            Some("event") => {
                let id = read_id(args);
                if id.is_err() {
                    eprintln!("{}", id.err().unwrap());
                    return;
                }
                let id = id.unwrap();
                if let Err(e) = items.events.mark_removed(id) {
                    eprintln!("{}", e);
                }
            },
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

    fn read_id<'a, T>(mut args: T) -> Result<usize, &'static str>
    where
        T: Iterator<Item = &'a str>,
    {
        if let Some(s) = args.next() {
            return usize::from_str(s).map_err(|_| "Invalid input. Input a valid ID.");
        }
        Err("No ID specified.")
    }

    mod add_cmd {
        use super::*;

        pub fn add<'a, T>(items: &mut Items, mut args: T)
        where
            T: Iterator<Item = &'a str>,
        {
            match args.next() {
                Some("todo") => add_todo(items),
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

        fn add_todo(items: &mut Items) {
            println!("New todo: ");
            let body = read_body();
            let weekday = read_weekday();
            items.todos.add(Todo::new(body, weekday))
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
                Some("todos") => show_all_todos(items),
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
                println!("\tTodos: ");
                show_list_weekday(&items.todos, *wd);
                println!("\tTasks: ");
                show_list_weekday(&items.tasks, *wd);
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

            while {
                show_all_date(items, day);
                day = day.succ();

                day.weekday() != orig_day
            } {}
        }

        fn show_month(items: &Items) {
            let mut day = Local::today().naive_local();

            for _ in 0..30 {
                show_all_date(items, day);
                day = day.succ();
            }
        }

        fn show_all_todos(items: &Items) {
            for wd in WEEKDAYS.iter() {
                println!("{}", wd);
                println!("\tTodos: ");
                show_list_weekday(&items.todos, *wd);
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
            println!("\tTodos: ");
            show_list_date(&items.todos, date);

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

        fn show_list<T: Display + MtcItem>(list: &[&T]) {
            for i in list.iter() {
                println!("\t\t{}", i);
            }
        }
    }

    mod sync {
        use super::*;
        use ssh2::Session;
        use std::io::Error;
        use std::net::TcpStream;

        #[derive(Serialize, Deserialize)]
        struct Config {
            username: String,
            address: String,
            server_path: String,
        }

        pub fn sync<'a, T>(items: &mut Items, mut args: T)
        where
            T: Iterator<Item = &'a str>,
        {
            let mut overwrite = false;
            match args.next() {
                None => {}
                Some("overwrite") => overwrite = true,
                Some("self") => {
                    items.todos.sync_self();
                    items.tasks.sync_self();
                    items.events.sync_self();
                    return;
                }
                _ => {
                    eprintln!("Unknown command.");
                    tip();
                    return;
                }
            }

            match read_config() {
                Ok(conf) => {
                    if let Err(e) = connect(items, &conf, overwrite) {
                        eprintln!("Failed to sync: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Reading sync config failed.");
                    eprintln!("{}", e);
                }
            }
        }

        fn connect(items: &mut Items, conf: &Config, overwrite: bool) -> Result<(), Error> {
            let tcp = TcpStream::connect(&conf.address)?;
            let mut sess = Session::new()?;
            sess.set_tcp_stream(tcp);
            sess.handshake()?;

            let pass = rpassword::prompt_password_stdout(&format!(
                "{}@{}'s password: ",
                conf.username, conf.address
            ))?;
            sess.userauth_password(&conf.username, &pass)?;

            sync_remote(
                &sess,
                &mut items.todos,
                &Path::new(&conf.server_path).join(Path::new("todos.json")),
                overwrite,
            )?;
            sync_remote(
                &sess,
                &mut items.tasks,
                &Path::new(&conf.server_path).join(Path::new("tasks.json")),
                overwrite,
            )?;
            sync_remote(
                &sess,
                &mut items.events,
                &Path::new(&conf.server_path).join(Path::new("events.json")),
                overwrite,
            )?;

            Ok(())
        }

        fn read_config() -> Result<Config, String> {
            // From main we know that this should exist so unwrap is ok here.
            // There may be some cases where this doesn't work but those are likely very rare...?
            let path = dirs::data_dir().unwrap().join("mtc/sync-conf.json");

            let file = File::open(path).map_err(|e| e.to_string())?;
            let reader = BufReader::new(file);

            serde_json::from_reader(reader).map_err(|e| e.to_string())
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
    let todo_file = dir.join(Path::new("todos.json"));
    let task_file = dir.join(Path::new("tasks.json"));
    let event_file = dir.join(Path::new("events.json"));

    let todos = read_item(&todo_file)?;
    let tasks = read_item(&task_file)?;
    let events = read_item(&event_file)?;

    Ok(Items {
        todos,
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
    let todo_file = dir.join(Path::new("todos.json"));
    let task_file = dir.join(Path::new("tasks.json"));
    let event_file = dir.join(Path::new("events.json"));

    write_item(items.todos, &todo_file)?;
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
