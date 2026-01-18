# Get File Content

> Return the content of the given file from the conversation workspace.



## OpenAPI

````yaml openapi/openapi.json get /api/conversations/{conversation_id}/select-file
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
  /api/conversations/{conversation_id}/select-file:
    get:
      summary: Get File Content
      description: Return the content of the given file from the conversation workspace.
      operationId: select_file_api_conversations__conversation_id__select_file_get
      parameters:
        - name: conversation_id
          in: path
          required: true
          schema:
            type: string
            title: Conversation Id
        - name: file
          in: query
          required: true
          schema:
            type: string
            title: File
      responses:
        '200':
          description: File content returned as JSON
          content:
            application/json:
              schema:
                type: object
                additionalProperties:
                  type: string
                title: >-
                  Response 200 Select File Api Conversations  Conversation Id 
                  Select File Get
        '415':
          description: Unsupported media type
          content:
            application/json:
              schema:
                type: object
                additionalProperties: true
                title: >-
                  Response 415 Select File Api Conversations  Conversation Id 
                  Select File Get
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
        '500':
          description: Error opening file
          content:
            application/json:
              schema:
                type: object
                additionalProperties: true
                title: >-
                  Response 500 Select File Api Conversations  Conversation Id 
                  Select File Get
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