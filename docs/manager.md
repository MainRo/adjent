# Adjent Manager

The manager is responsible for spawning an agent each time a new action is triggered.
The manager is started on a specific project. It then waits for actions via the HTTP server.

Each time a new action is received, the manager spawn the agent (for example claude code or gemini cli). The agent is provided as an argument of the "adjent manage" command. Once an agent is spawned, the manager is waiting until its completion. Then it waits again for another action to be available.

The manager must spawn the agent in the same environment that it is executed on. Typically, it must forward all the environment variables. The spawned agent will retrieve and create artifacts to process the action thanks to the resources exposed by MCP server. In order to do so, the manager injects an additional environment variable "ADJENT_MCP_ACTION_PATH". Its content is set with the full path of the round of the action. For example:

```
/projects/foo/tasks/bar/rounds/1
```
