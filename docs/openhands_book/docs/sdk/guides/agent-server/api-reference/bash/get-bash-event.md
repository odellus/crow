# Get Bash Event

> Get a bash event event given an id



## OpenAPI

````yaml openapi/agent-sdk.json get /api/bash/bash_events/{event_id}
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/bash/bash_events/{event_id}:
    get:
      tags:
        - Bash
      summary: Get Bash Event
      description: Get a bash event event given an id
      operationId: get_bash_event_api_bash_bash_events__event_id__get
      parameters:
        - name: event_id
          in: path
          required: true
          schema:
            type: string
            title: Event Id
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/BashEventBase'
        '404':
          description: Item not found
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
components:
  schemas:
    BashEventBase:
      oneOf:
        - $ref: '#/components/schemas/BashCommand'
        - $ref: '#/components/schemas/BashOutput'
      discriminator:
        propertyName: kind
        mapping:
          openhands__agent_server__models__BashCommand-Output__1: '#/components/schemas/BashCommand'
          openhands__agent_server__models__BashOutput-Output__1: '#/components/schemas/BashOutput'
    HTTPValidationError:
      properties:
        detail:
          items:
            $ref: '#/components/schemas/ValidationError'
          type: array
          title: Detail
      type: object
      title: HTTPValidationError
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
    BashOutput:
      properties:
        id:
          type: string
          title: Id
        timestamp:
          type: string
          format: date-time
          title: Timestamp
        command_id:
          type: string
          title: Command Id
        order:
          type: integer
          title: Order
          description: The order for this output, sequentially starting with 0
          default: 0
        exit_code:
          anyOf:
            - type: integer
            - type: 'null'
          title: Exit Code
          description: Exit code None implies the command is still running.
        stdout:
          anyOf:
            - type: string
            - type: 'null'
          title: Stdout
          description: The standard output from the command
        stderr:
          anyOf:
            - type: string
            - type: 'null'
          title: Stderr
          description: The error output from the command
        kind:
          type: string
          const: BashOutput
          title: Kind
      type: object
      required:
        - command_id
        - kind
      title: BashOutput
      description: >-
        Output of a bash command. A single command may have multiple pieces of
        output

        depending on how large the output is.
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