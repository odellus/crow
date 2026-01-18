# Git Changes



## OpenAPI

````yaml openapi/agent-sdk.json get /api/git/changes/{path}
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/git/changes/{path}:
    get:
      tags:
        - Git
      summary: Git Changes
      operationId: git_changes_api_git_changes__path__get
      parameters:
        - name: path
          in: path
          required: true
          schema:
            type: string
            format: path
            title: Path
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/GitChange'
                title: Response Git Changes Api Git Changes  Path  Get
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
components:
  schemas:
    GitChange:
      properties:
        status:
          $ref: '#/components/schemas/GitChangeStatus'
        path:
          type: string
          format: path
          title: Path
      type: object
      required:
        - status
        - path
      title: GitChange
    HTTPValidationError:
      properties:
        detail:
          items:
            $ref: '#/components/schemas/ValidationError'
          type: array
          title: Detail
      type: object
      title: HTTPValidationError
    GitChangeStatus:
      type: string
      enum:
        - MOVED
        - ADDED
        - DELETED
        - UPDATED
      title: GitChangeStatus
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