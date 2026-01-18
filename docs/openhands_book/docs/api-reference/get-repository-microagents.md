# Get Repository Microagents

> Scan the microagents directory of a repository and return the list of microagents.

The microagents directory location depends on the git provider and actual repository name:
- If git provider is not GitLab and actual repository name is ".openhands": scans "microagents" folder
- If git provider is GitLab and actual repository name is "openhands-config": scans "microagents" folder
- Otherwise: scans ".openhands/microagents" folder



## OpenAPI

````yaml openapi/openapi.json get /api/user/repository/{repository_name}/microagents
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
  /api/user/repository/{repository_name}/microagents:
    get:
      summary: Get Repository Microagents
      description: >-
        Scan the microagents directory of a repository and return the list of
        microagents.


        The microagents directory location depends on the git provider and
        actual repository name:

        - If git provider is not GitLab and actual repository name is
        ".openhands": scans "microagents" folder

        - If git provider is GitLab and actual repository name is
        "openhands-config": scans "microagents" folder

        - Otherwise: scans ".openhands/microagents" folder
      operationId: >-
        get_repository_microagents_api_user_repository__repository_name__microagents_get
      parameters:
        - name: repository_name
          in: path
          required: true
          schema:
            type: string
            title: Repository Name
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/MicroagentResponse'
                title: >-
                  Response Get Repository Microagents Api User Repository 
                  Repository Name  Microagents Get
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
    MicroagentResponse:
      properties:
        name:
          type: string
          title: Name
        path:
          type: string
          title: Path
        created_at:
          type: string
          format: date-time
          title: Created At
      type: object
      required:
        - name
        - path
        - created_at
      title: MicroagentResponse
      description: |-
        Response model for microagents endpoint.

        Note: This model only includes basic metadata that can be determined
        without parsing microagent content. Use the separate content API
        to get detailed microagent information.
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