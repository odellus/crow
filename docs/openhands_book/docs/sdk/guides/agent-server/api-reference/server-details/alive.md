# Alive



## OpenAPI

````yaml openapi/agent-sdk.json get /alive
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /alive:
    get:
      tags:
        - Server Details
      summary: Alive
      operationId: alive_alive_get
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema: {}

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt