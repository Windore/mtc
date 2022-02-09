# MTC - My Time Contract

A CLI time management app with the ability to synchronize todos, tasks and events via a server using an SSH connection.

MTC is a yet another time-management app as enough of those don't exist yet. The main purpose of this app is to serve as
simple rust practice for me while also allowing me to sync my todo-lists and events using my home server. There is also
an [android app](https://github.com/Windore/mtc-android) for MTC.

MTC also has a public API for writing additional interfaces. Clone the project and run `cargo doc --open` to access the
documentation of the API. Note that the documentation doesn't have that many examples. Instead, you can check out
the [mtc-android project](https://github.com/Windore/mtc-android) for an example project using this API. However, code
there is probably not the most well written since I didn't bother to do anything else but to make it work.

## Installation

You can install MTC using the following command. The same command is used for updating MTC as well.

```
cargo install --git https://github.com/Windore/mtc.git
```

## Usage

There are three types of items related to time management: todos, tasks, and events. None of these types handles clock
times, and they only deal with dates. Easy way to include times is just to specify them in the body of an any type.
Items are sorted by an item's body so 24-hour clock times at the start of an item work well. If you need to, you can
start the time with "AM" or "PM" and the order will be correct as well.

### Todos

Todos are quite self-explanatory. They are used for simple one time tasks that don't necessarily have to be done at a
specific time. You can however specify a weekday for a todo, but nothing more specific than that.

### Tasks

Tasks are something that you expect to do every week on a specific weekday. They have a duration in minutes, and you can
use `mtc do` to have a timer for that duration. Tasks can also exist without a weekday specified. Then the task is for
every day.

### Events

Events are like todos, but they have a specific date when they occur. Events that are more than three days old are
removed automatically during sync.

### Commands

Weekdays can be shortened to three initial letters. If a weekday is wanted to be left unspecified, don't supply a
weekday argument.

#### Show

Show all todos, tasks, and events.

```
mtc show
```

Show only a specific type. This is the only way to show tasks.

```
mtc show <type>
```

Show todos and events for a weekday. (Note: If today is a tuesday and the specified weekday is monday this command will show
next weeks monday)

```
mtc show <weekday>
```

Show todos and events for today.

```
mtc show today
```

Show todos and events for tomorrow.

```
mtc show tomorrow
```

Show a quick overview of today and next three days. 'overview' can be shortened to 'ov'.

```
mtc show overview
```

Show todos and events for a week from this day.

```
mtc show week
```

Show todos and events for a 30-day period from this day.

```
mtc show month
```

#### Add

Add a new todo with a given body. A weekday may also be supplied.

```
mtc add todo <body> [weekday]
```

Add a new task with a given body and a duration. A weekday may also be supplied.

```
mtc add task <body> <duration> [weekday]
```

Add a new event with a body and a date. Date is given in `year-month-day` format.

```
mtc add event <body> <date>
```

#### Remove

Remove an item. The app will ask for an id. Each item of a type has a unique numerical id. You can get the id with
the `show` command. Note that the id may change for an item when syncing.

```
mtc remove <type> <id> 
```

#### Set

Set a property of an item.

```
mtc set <type> <id> <property> <value>
```

Possible properties for...

- Todos: body, weekday
- Tasks: body, duration, weekday
- Events: body, date

To set a weekday to all weekdays use:

```
mtc set <type> <id> weekday
```

#### Do

Start a timer for a task. The app will ask for an id of a task.

```
mtc do <task id>
```

#### Sync

Using sync requires a bit setting up to do. The app expects a config file located in the mtc directory in the user's
config directory. For example for linux this config file is `~/.config/mtc/sync.json`. The app will not create a config
file on its own. Example of a config file:

`sync.json`:

```
{
  "username": "user",
  "address": "127.0.0.1:22",
  "server_path": "/home/user/mtc/"
}
```

Note that the server path needs to exist as the app doesn't create it automatically. Also sync currently only supports
password based authentication (as that is currently what I need).

First time syncing with a server requires using the following command.

```
mtc sync overwrite
```

This will overwrite the saved items on the server. Note: that if you have synced to a same server from any client,
overwrite is not needed to sync with a new client. Only new servers require using overwrite. Overwrite is used because
it automatically creates the initial server files instead of trying to look for them and failing.

After the setup sync happens with the following command.

```
mtc sync
```

If for some reason the app is used only locally, the following command needs to be run occasionally:

```
mtc sync self
```

This is because internally the app doesn't actually remove items with the remove command. Instead, it only marks them as
removed and then sync finally removes them.

#### Help

Show a help message.

```
mtc help
```

## License

This project is licensed under the [MIT license](LICENSE.md).
