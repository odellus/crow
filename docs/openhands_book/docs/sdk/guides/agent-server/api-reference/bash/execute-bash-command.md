# Execute Bash Command

> Execute a bash command and wait for a result



## OpenAPI

````yaml openapi/agent-sdk.json post /api/bash/execute_bash_command
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/bash/execute_bash_command:
    post:
      tags:
        - Bash
      summary: Execute Bash Command
      description: Execute a bash command and wait for a result
      operationId: execute_bash_command_api_bash_execute_bash_command_post
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
                $ref: '#/components/schemas/BashOutput'
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