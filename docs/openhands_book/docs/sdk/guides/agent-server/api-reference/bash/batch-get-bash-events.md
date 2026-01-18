# Batch Get Bash Events

> Get a batch of bash event events given their ids, returning null for any
missing item.



## OpenAPI

````yaml openapi/agent-sdk.json get /api/bash/bash_events/
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/bash/bash_events/:
    get:
      tags:
        - Bash
      summary: Batch Get Bash Events
      description: |-
        Get a batch of bash event events given their ids, returning null for any
        missing item.
      operationId: batch_get_bash_events_api_bash_bash_events__get
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: array
              items:
                type: string
              title: Event Ids
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                items:
                  anyOf:
                    - $ref: '#/components/schemas/BashEventBase'
                    - type: 'null'
                type: array
                title: Response Batch Get Bash Events Api Bash Bash Events  Get
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