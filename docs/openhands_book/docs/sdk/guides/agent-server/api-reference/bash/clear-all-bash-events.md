# Clear All Bash Events

> Clear all bash events from storage



## OpenAPI

````yaml openapi/agent-sdk.json delete /api/bash/bash_events
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/bash/bash_events:
    delete:
      tags:
        - Bash
      summary: Clear All Bash Events
      description: Clear all bash events from storage
      operationId: clear_all_bash_events_api_bash_bash_events_delete
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                additionalProperties:
                  type: integer
                type: object
                title: Response Clear All Bash Events Api Bash Bash Events Delete

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt