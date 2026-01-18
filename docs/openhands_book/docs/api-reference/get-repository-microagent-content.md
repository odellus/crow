# Get Repository Microagent Content

> Fetch the content of a specific microagent file from a repository.



## OpenAPI

````yaml openapi/openapi.json get /api/user/repository/{repository_name}/microagents/content
openapi: 3.1.0
info:
  title: OpenHands
  description: 'OpenHands: Code Less, Make More'
  version: 0.53.0
servers:
  - url: https://app.all-hands.dev
    description: Production server
  - url: http://localhost:3000
    description: Local development server
security: []
paths:
  /api/user/repository/{repository_name}/microagents/content:
    get:
      summary: Get Repository Microagent Content
      description: Fetch the content of a specific microagent file from a repository.
      operationId: >-
        get_repository_microagent_content_api_user_repository__repository_name__microagents_content_get
      parameters:
        - name: repository_name
          in: path
          required: true
          schema:
            type: string
            title: Repository Name
        - name: file_path
          in: query
          required: true
          schema:
            type: string
            description: Path to the microagent file within the repository
            title: File Path
          description: Path to the microagent file within the repository
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/MicroagentContentResponse'
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
      security:
        - APIKeyHeader: []
components:
  schemas:
    MicroagentContentResponse:
      properties:
        content:
          type: string
          title: Content
        path:
          type: string
          title: Path
        triggers:
          items:
            type: string
          type: array
          title: Triggers
          default: []
        git_provider:
          anyOf:
            - type: string
            - type: 'null'
          title: Git Provider
      type: object
      required:
        - content
        - path
      title: MicroagentContentResponse
      description: Response model for individual microagent content endpoint.
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
  securitySchemes:
    APIKeyHeader:
      type: apiKey
      in: header
      name: X-Session-API-Key

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt