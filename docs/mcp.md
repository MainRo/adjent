
The MCP server is implemented with the [rmcp](https://crates.io/crates/rmcp) crate.

It is exposed on the /mcp route of the server. So it is started at the same time than the HTTP server.
The MCP server expose resources for the current round of the current task.

When started, agent determines the project, and task with these environment variables:

- ADJENT_PROJECT_ID
- ADJENT_ACTION_ID

These are provided by the manager to the agent for each task execution.
The agent is configured to provide these two values as HTTP headers in the requests:

- X-Adjent-ProjectId
- X-Adjent-ActionId

This allows the MCP server to expose only the resources of the round:

- inputs resources, are read-only resources
- logs and outputs and read-write resources. The agent can create new resources here.
