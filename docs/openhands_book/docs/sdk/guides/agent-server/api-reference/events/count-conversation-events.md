# Count Conversation Events

> Count local events matching the given filters



## OpenAPI

````yaml openapi/agent-sdk.json get /api/conversations/{conversation_id}/events/count
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/conversations/{conversation_id}/events/count:
    get:
      tags:
        - Events
      summary: Count Conversation Events
      description: Count local events matching the given filters
      operationId: >-
        count_conversation_events_api_conversations__conversation_id__events_count_get
      parameters:
        - name: conversation_id
          in: path
          required: true
          schema:
            type: string
            format: uuid
            title: Conversation Id
        - name: kind
          in: query
          required: false
          schema:
            anyOf:
              - type: string
              - type: 'null'
            title: >-
              Optional filter by event kind/type (e.g., ActionEvent,
              MessageEvent)
        - name: source
          in: query
          required: false
          schema:
            anyOf:
              - type: string
              - type: 'null'
            title: Optional filter by event source (e.g., agent, user, environment)
        - name: body
          in: query
          required: false
          schema:
            anyOf:
              - type: string
              - type: 'null'
            title: Optional filter by message content (case-insensitive)
        - name: timestamp__gte
          in: query
          required: false
          schema:
            anyOf:
              - type: string
                format: date-time
              - type: 'null'
            title: 'Filter: event timestamp >= this datetime'
        - name: timestamp__lt
          in: query
          required: false
          schema:
            anyOf:
              - type: string
                format: date-time
              - type: 'null'
            title: 'Filter: event timestamp < this datetime'
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                type: integer
                title: >-
                  Response Count Conversation Events Api Conversations 
                  Conversation Id  Events Count Get
        '404':
          description: Conversation not found
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
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

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt