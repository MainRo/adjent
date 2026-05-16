# Model Context Protocol (MCP)

The MCP server is implemented with the [rmcp](https://crates.io/crates/rmcp) crate.
It is exposed on the `/mcp` route of the server (started alongside the main HTTP server).

## Context & Scoping

The MCP server uses HTTP headers to scope all tool operations to a specific project, task, and round.
The client (agent) is configured by the manager to provide these headers in every request:

- `X-Adjent-ProjectId`: The ID of the project.
- `X-Adjent-ActionId`: The ID of the current action (which resolves to a task and round).

## Tools

Instead of static resources, the server provides a set of tools for dynamic artifact interaction:

### `list_artifacts`
Lists files in a specific artifact directory.
- **Parameters**: `type` ("inputs", "outputs", or "logs")
- **Returns**: A newline-separated list of filenames.

### `read_artifact`
Reads the content of a specific artifact file.
- **Parameters**: `type`, `filename`
- **Returns**: The raw text content of the file.

### `write_artifact`
Writes or updates an artifact file.
- **Parameters**: `type`, `filename`, `content`
- **Returns**: A success message.

## Transport

The server uses the standard MCP SSE transport:
- **SSE Endpoint**: `GET /mcp`
- **Message Endpoint**: `POST /mcp`

The session is established upon the first `initialize` request.
