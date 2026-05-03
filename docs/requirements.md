
# Projects

## Project id format

The project id can be composed only of ascii alphanumerc characters. The first
character must be a letter.

When a project is created, the user provides this id.


# Tasks

## Task id format

The format of the id of a task is in the form:

```
[prefix]-[task-name]
```

Where prefix is a date in the form "yyyymmdd". 
and where task-name has been provided by the user.

When a task is created, the user provides a name. A date prefix is added to this
name so that tasks can be sorted or archived by date.

For example if the task "foo" is created on March the 2nd of 2026, then the 
corresponding task id is: 

```
20260302-foo
```

The task name can be composed only of ascii alphanumerc characters. The first
character must be a letter.