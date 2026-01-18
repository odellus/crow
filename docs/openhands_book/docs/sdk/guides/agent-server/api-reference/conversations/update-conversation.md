# Update Conversation

> Update conversation metadata.

This endpoint allows updating conversation details like title.



## OpenAPI

````yaml openapi/agent-sdk.json patch /api/conversations/{conversation_id}
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/conversations/{conversation_id}:
    patch:
      tags:
        - Conversations
      summary: Update Conversation
      description: |-
        Update conversation metadata.

        This endpoint allows updating conversation details like title.
      operationId: update_conversation_api_conversations__conversation_id__patch
      parameters:
        - name: conversation_id
          in: path
          required: true
          schema:
            type: string
            format: uuid
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
                $ref: '#/components/schemas/Success'
        '404':
          description: Item not found
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
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
      type: object
      required:
        - title
      title: UpdateConversationRequest
      description: Payload to update conversation metadata.
    Success:
      properties:
        success:
          type: boolean
          title: Success
          default: true
      type: object
      title: Success
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