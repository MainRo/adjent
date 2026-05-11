# Todo list

- Add url with host in config for the CLI to remove hardcoded url
- move manager functions in cli.rs to manager.rs
- add an action create http API. Internally the round.do call should call it
- add an action delete http API
- add a command "adjent action prune" that cleans all actions in completed state
- add a command "adjent action retry" that change the status of a failed or assigned action to pending
