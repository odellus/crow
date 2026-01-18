# Git Diff



## OpenAPI

````yaml openapi/agent-sdk.json get /api/git/diff/{path}
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/git/diff/{path}:
    get:
      tags:
        - Git
      summary: Git Diff
      operationId: git_diff_api_git_diff__path__get
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
                $ref: '#/components/schemas/GitDiff'
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
components:
  schemas:
    GitDiff:
      properties:
        modified:
          anyOf:
            - type: string
            - type: 'null'
          title: Modified
        original:
          anyOf:
            - type: string
            - type: 'null'
          title: Original
      type: object
      required:
        - modified
        - original
      title: GitDiff
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