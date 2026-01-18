# Get Server Info



## OpenAPI

````yaml openapi/agent-sdk.json get /
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /:
    get:
      summary: Get Server Info
      operationId: get_server_info__get
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ServerInfo'
components:
  schemas:
    ServerInfo:
      properties:
        uptime:
          type: number
          title: Uptime
        idle_time:
          type: number
          title: Idle Time
        title:
          type: string
          title: Title
          default: OpenHands Agent Server
        version:
          type: string
          title: Version
          default: 1.8.2
        docs:
          type: string
          title: Docs
          default: /docs
        redoc:
          type: string
          title: Redoc
          default: /redoc
      type: object
      required:
        - uptime
        - idle_time
      title: ServerInfo

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt