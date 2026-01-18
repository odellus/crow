# Start Bash Command

> Execute a bash command in the background



## OpenAPI

````yaml openapi/agent-sdk.json post /api/bash/start_bash_command
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/bash/start_bash_command:
    post:
      tags:
        - Bash
      summary: Start Bash Command
      description: Execute a bash command in the background
      operationId: start_bash_command_api_bash_start_bash_command_post
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/ExecuteBashRequest'
        required: true
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/BashCommand'
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
components:
  schemas:
    ExecuteBashRequest:
      properties:
        command:
          type: string
          title: Command
          description: The bash command to execute
        cwd:
          anyOf:
            - type: string
            - type: 'null'
          title: Cwd
          description: The current working directory
        timeout:
          type: integer
          title: Timeout
          description: The max number of seconds a command may be permitted to run.
          default: 300
      type: object
      required:
        - command
      title: ExecuteBashRequest
    BashCommand:
      properties:
        command:
          type: string
          title: Command
          description: The bash command to execute
        cwd:
          anyOf:
            - type: string
            - type: 'null'
          title: Cwd
          description: The current working directory
        timeout:
          type: integer
          title: Timeout
          description: The max number of seconds a command may be permitted to run.
          default: 300
        id:
          type: string
          title: Id
        timestamp:
          type: string
          format: date-time
          title: Timestamp
        kind:
          type: string
          const: BashCommand
          title: Kind
      type: object
      required:
        - command
        - kind
      title: BashCommand
    HTTPValidationError:
      properties:
        detail:
          items:
            $ref: '#/components/schemas/ValidationError'
          type: array
          title: Detail
      type: object
      title: HTTPValidationError
    ValidationError:
      properties:
        loc:
          items:
            anyOf:
              - type: string
              - type: integer
          type: array
          title: Location
        msg:
          type: string
          title: Message
        type:
          type: string
          title: Error Type
      type: object
      required:
        - loc
        - msg
        - type
      title: ValidationError

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt