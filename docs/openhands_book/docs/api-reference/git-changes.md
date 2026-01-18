# Git Changes



## OpenAPI

````yaml openapi/openapi.json get /api/conversations/{conversation_id}/git/changes
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
  /api/conversations/{conversation_id}/git/changes:
    get:
      summary: Git Changes
      operationId: git_changes_api_conversations__conversation_id__git_changes_get
      parameters:
        - name: conversation_id
          in: path
          required: true
          schema:
            type: string
            title: Conversation Id
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                type: array
                items:
                  additionalProperties:
                    type: string
                  type: object
                title: >-
                  Response Git Changes Api Conversations  Conversation Id  Git
                  Changes Get
        '404':
          description: Not a git repository
          content:
            application/json:
              schema:
                type: object
                additionalProperties: true
                title: >-
                  Response 404 Git Changes Api Conversations  Conversation Id 
                  Git Changes Get
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
        '500':
          description: Error getting changes
          content:
            application/json:
              schema:
                type: object
                additionalProperties: true
                title: >-
                  Response 500 Git Changes Api Conversations  Conversation Id 
                  Git Changes Get
      security:
        - APIKeyHeader: []
components:
  schemas:
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