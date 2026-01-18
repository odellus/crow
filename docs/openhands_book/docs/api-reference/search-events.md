# Search Events

> Search through the event stream with filtering and pagination.



## OpenAPI

````yaml openapi/openapi.json get /api/conversations/{conversation_id}/events
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
  /api/conversations/{conversation_id}/events:
    get:
      summary: Search Events
      description: Search through the event stream with filtering and pagination.
      operationId: search_events_api_conversations__conversation_id__events_get
      parameters:
        - name: conversation_id
          in: path
          required: true
          schema:
            type: string
            title: Conversation Id
        - name: start_id
          in: query
          required: false
          schema:
            type: integer
            default: 0
            title: Start Id
        - name: end_id
          in: query
          required: false
          schema:
            anyOf:
              - type: integer
              - type: 'null'
            title: End Id
        - name: reverse
          in: query
          required: false
          schema:
            type: boolean
            default: false
            title: Reverse
        - name: limit
          in: query
          required: false
          schema:
            type: integer
            default: 20
            title: Limit
      requestBody:
        content:
          application/json:
            schema:
              anyOf:
                - $ref: '#/components/schemas/EventFilter'
                - type: 'null'
              title: Filter
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema: {}
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
    EventFilter:
      properties:
        exclude_hidden:
          type: boolean
          title: Exclude Hidden
          default: false
        query:
          anyOf:
            - type: string
            - type: 'null'
          title: Query
        include_types:
          anyOf:
            - items: {}
              type: array
            - type: 'null'
          title: Include Types
        exclude_types:
          anyOf:
            - items: {}
              type: array
            - type: 'null'
          title: Exclude Types
        source:
          anyOf:
            - type: string
            - type: 'null'
          title: Source
        start_date:
          anyOf:
            - type: string
            - type: 'null'
          title: Start Date
        end_date:
          anyOf:
            - type: string
            - type: 'null'
          title: End Date
      type: object
      title: EventFilter
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