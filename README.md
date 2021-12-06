# MMVK

MMVK is a CLI time management app with the ability to synchronize todo-items, tasks and events via a server using a SSH connection. It also has a public API for writing additional interfaces.

`mmvk` is an yet another time-management app as enough of those don't exist yet. The main purpose of this app is to serve as simple rust practice for me while also allowing me to sync my todo-lists and events using my home server.

## Installation

This will be added soon.

## Usage

There are three types of items related to time management: todo-items, tasks, and events. None of these types handles clock times and they only deal with dates. Easy way to include times is just to specify them in the body of an any type.

### Todo-items

Todo-items are quite self-explanatory. They are used for simple one time tasks that don't necessarily have to be done at a specific time. You can however specify a weekday for a todo-item, but nothing more specific than that.

### Tasks

Tasks are something that one expects to do every week on a specific weekday. They have a duration in minutes and you can use `mmvk do` to have a timer for that duration. Tasks can also exist without a weekday specified.

### Events

Events are like todo-items, but they have a specific date when they occur. `mmvk` automatically removes events that are before the current date, when running **any** command.

### Commands

#### Show

Show all todo-items, tasks, and events:

```
mmvk show
```

Show only a specific type:

```
mmvk show <type>
```

Show everything for a weekday. (Note: If today is an tuesday and the specified weekday is monday this command will show next weeks monday)

```
mmvk show <weekday>
```

Show everything for today.

```
mmvk show today
```

Show everything for a week from this day.

```
mmvk show week
```

Show everything for a 30-day period from this day.

```
mmvk show month
```

#### Add

Add a new item of a given type.

```
mmvk add <type>
```

Add a todo-item or a task on a specific weekday. This doesn't work for events.

```
mmvk add <type> <weekday>
```

#### Remove

Remove an item. Each item of a type has an unique numerical id. You can get the id with the `show` command. 

```
mmvk remove <type> <id>
```

#### Move

Move an item. For event the command asks for a new date. For tasks and todo-items, the app asks for a weekday.

```
mmvk move <type> <id>
```

#### Do

Start a timer for a task. The timer can be paused and resumed with `p`

```
mmvk do <task id>
```

#### Sync

This will be added soon.

```
mmvk sync
```

#### Help

Show a help message.

```
mmvk help
```

## License
This will be added soon.
