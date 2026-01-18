# Get Repository Branches

> Get branches for a repository.



## OpenAPI

````yaml openapi/openapi.json get /api/user/repository/branches
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
  /api/user/repository/branches:
    get:
      summary: Get Repository Branches
      description: Get branches for a repository.
      operationId: get_repository_branches_api_user_repository_branches_get
      parameters:
        - name: repository
          in: query
          required: true
          schema:
            type: string
            title: Repository
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/Branch'
                title: >-
                  Response Get Repository Branches Api User Repository Branches
                  Get
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
    Branch:
      properties:
        name:
          type: string
          title: Name
        commit_sha:
          type: string
          title: Commit Sha
        protected:
          type: boolean
          title: Protected
        last_push_date:
          anyOf:
            - type: string
            - type: 'null'
          title: Last Push Date
      type: object
      required:
        - name
        - commit_sha
        - protected
      title: Branch
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