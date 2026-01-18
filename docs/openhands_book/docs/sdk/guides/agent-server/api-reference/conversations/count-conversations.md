# Count Conversations

> Count conversations matching the given filters



## OpenAPI

````yaml openapi/agent-sdk.json get /api/conversations/count
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/conversations/count:
    get:
      tags:
        - Conversations
      summary: Count Conversations
      description: Count conversations matching the given filters
      operationId: count_conversations_api_conversations_count_get
      parameters:
        - name: status
          in: query
          required: false
          schema:
            anyOf:
              - $ref: '#/components/schemas/ConversationExecutionStatus'
              - type: 'null'
            title: Optional filter by conversation execution status
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                type: integer
                title: Response Count Conversations Api Conversations Count Get
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
components:
  schemas:
    ConversationExecutionStatus:
      type: string
      enum:
        - idle
        - running
        - paused
        - waiting_for_confirmation
        - finished
        - error
        - stuck
        - deleting
      title: ConversationExecutionStatus
      description: Enum representing the current execution state of the conversation.
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