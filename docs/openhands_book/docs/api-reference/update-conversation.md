# Update Conversation

> Update conversation metadata.

This endpoint allows updating conversation details like title.
Only the conversation owner can update the conversation.



## OpenAPI

````yaml openapi/openapi.json patch /api/conversations/{conversation_id}
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
  /api/conversations/{conversation_id}:
    patch:
      summary: Update Conversation
      description: |-
        Update conversation metadata.

        This endpoint allows updating conversation details like title.
        Only the conversation owner can update the conversation.
      operationId: update_conversation_api_conversations__conversation_id__patch
      parameters:
        - name: conversation_id
          in: path
          required: true
          schema:
            type: string
            title: Conversation Id
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/UpdateConversationRequest'
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                type: boolean
                title: >-
                  Response Update Conversation Api Conversations  Conversation
                  Id  Patch
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
    UpdateConversationRequest:
      properties:
        title:
          type: string
          maxLength: 200
          minLength: 1
          title: Title
          description: New conversation title
      additionalProperties: false
      type: object
      required:
        - title
      title: UpdateConversationRequest
      description: Request model for updating conversation metadata.
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