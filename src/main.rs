use mtc::*;
use std::path::Path;
use std::io::{BufReader, BufWriter, Write};
use std::{fs, fs::File};
use serde::{Serialize, de::DeserializeOwned};

struct Items {
    todo_items: MtcList<TodoItem>,
    tasks: MtcList<Task>,
    events: MtcList<Event>,
}

fn handle_command(items: Items) -> Items {
    items
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
            Ok(i) => handle_command(i),
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

    Ok(
        Items {
            todo_items,
            tasks,
            events,
        }
    )
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

fn write_item<T: MtcItem + Clone + DeserializeOwned + Serialize>(item: MtcList<T>, path: &Path) -> Result<(), String> {
    let file = File::create(path).map_err(|e| e.to_string())?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &item).map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;
    Ok(())
}