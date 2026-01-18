# Search Bash Events

> Search / List bash event events



## OpenAPI

````yaml openapi/agent-sdk.json get /api/bash/bash_events/search
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/bash/bash_events/search:
    get:
      tags:
        - Bash
      summary: Search Bash Events
      description: Search / List bash event events
      operationId: search_bash_events_api_bash_bash_events_search_get
      parameters:
        - name: kind__eq
          in: query
          required: false
          schema:
            anyOf:
              - enum:
                  - BashCommand
                  - BashOutput
                type: string
              - type: 'null'
            title: Kind  Eq
        - name: command_id__eq
          in: query
          required: false
          schema:
            anyOf:
              - type: string
                format: uuid
              - type: 'null'
            title: Command Id  Eq
        - name: timestamp__gte
          in: query
          required: false
          schema:
            anyOf:
              - type: string
                format: date-time
              - type: 'null'
            title: Timestamp  Gte
        - name: timestamp__lt
          in: query
          required: false
          schema:
            anyOf:
              - type: string
                format: date-time
              - type: 'null'
            title: Timestamp  Lt
        - name: sort_order
          in: query
          required: false
          schema:
            $ref: '#/components/schemas/BashEventSortOrder'
            default: TIMESTAMP
        - name: page_id
          in: query
          required: false
          schema:
            anyOf:
              - type: string
              - type: 'null'
            title: Optional next_page_id from the previously returned page
        - name: limit
          in: query
          required: false
          schema:
            type: integer
            exclusiveMinimum: 0
            title: The max number of results in the page
            lte: 100
            default: 100
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/BashEventPage'
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
components:
  schemas:
    BashEventSortOrder:
      type: string
      enum:
        - TIMESTAMP
        - TIMESTAMP_DESC
      title: BashEventSortOrder
    BashEventPage:
      properties:
        items:
          items:
            $ref: '#/components/schemas/BashEventBase'
          type: array
          title: Items
        next_page_id:
          anyOf:
            - type: string
            - type: 'null'
          title: Next Page Id
      type: object
      required:
        - items
      title: BashEventPage
    HTTPValidationError:
      properties:
        detail:
          items:
            $ref: '#/components/schemas/ValidationError'
          type: array
          title: Detail
      type: object
      title: HTTPValidationError
    BashEventBase:
      oneOf:
        - $ref: '#/components/schemas/BashCommand'
        - $ref: '#/components/schemas/BashOutput'
      discriminator:
        propertyName: kind
        mapping:
          openhands__agent_server__models__BashCommand-Output__1: '#/components/schemas/BashCommand'
          openhands__agent_server__models__BashOutput-Output__1: '#/components/schemas/BashOutput'
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

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt