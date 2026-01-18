# List Available Tools

> List all available tools.



## OpenAPI

````yaml openapi/agent-sdk.json get /api/tools/
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/tools/:
    get:
      tags:
        - Tools
      summary: List Available Tools
      description: List all available tools.
      operationId: list_available_tools_api_tools__get
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                items:
                  type: string
                type: array
                title: Response List Available Tools Api Tools  Get

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt