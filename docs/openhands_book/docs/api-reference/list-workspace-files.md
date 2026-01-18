# List Workspace Files

> List workspace files visible to the conversation runtime. Applies .gitignore and internal ignore rules.



## OpenAPI

````yaml openapi/openapi.json get /api/conversations/{conversation_id}/list-files
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
  /api/conversations/{conversation_id}/list-files:
    get:
      summary: List Workspace Files
      description: >-
        List workspace files visible to the conversation runtime. Applies
        .gitignore and internal ignore rules.
      operationId: list_files_api_conversations__conversation_id__list_files_get
      parameters:
        - name: conversation_id
          in: path
          required: true
          schema:
            type: string
            title: Conversation Id
        - name: path
          in: query
          required: false
          schema:
            anyOf:
              - type: string
              - type: 'null'
            title: Path
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                type: array
                items:
                  type: string
                title: >-
                  Response List Files Api Conversations  Conversation Id  List
                  Files Get
        '404':
          description: Runtime not initialized
          content:
            application/json:
              schema:
                type: object
                additionalProperties: true
                title: >-
                  Response 404 List Files Api Conversations  Conversation Id 
                  List Files Get
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
        '500':
          description: Error listing or filtering files
          content:
            application/json:
              schema:
                type: object
                additionalProperties: true
                title: >-
                  Response 500 List Files Api Conversations  Conversation Id 
                  List Files Get
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