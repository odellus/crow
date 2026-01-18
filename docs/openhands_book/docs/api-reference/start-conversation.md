# Start Conversation

> Start an agent loop for a conversation.

This endpoint calls the conversation_manager's maybe_start_agent_loop method
to start a conversation. If the conversation is already running, it will
return the existing agent loop info.



## OpenAPI

````yaml openapi/openapi.json post /api/conversations/{conversation_id}/start
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
  /api/conversations/{conversation_id}/start:
    post:
      summary: Start Conversation
      description: >-
        Start an agent loop for a conversation.


        This endpoint calls the conversation_manager's maybe_start_agent_loop
        method

        to start a conversation. If the conversation is already running, it will

        return the existing agent loop info.
      operationId: start_conversation_api_conversations__conversation_id__start_post
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
              $ref: '#/components/schemas/ProvidersSetModel'
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ConversationResponse'
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
    ProvidersSetModel:
      properties:
        providers_set:
          anyOf:
            - items:
                $ref: '#/components/schemas/ProviderType'
              type: array
            - type: 'null'
          title: Providers Set
      type: object
      title: ProvidersSetModel
    ConversationResponse:
      properties:
        status:
          type: string
          title: Status
        conversation_id:
          type: string
          title: Conversation Id
        message:
          anyOf:
            - type: string
            - type: 'null'
          title: Message
        conversation_status:
          anyOf:
            - $ref: '#/components/schemas/ConversationStatus'
            - type: 'null'
      type: object
      required:
        - status
        - conversation_id
      title: ConversationResponse
    HTTPValidationError:
      properties:
        detail:
          items:
            $ref: '#/components/schemas/ValidationError'
          type: array
          title: Detail
      type: object
      title: HTTPValidationError
    ProviderType:
      type: string
      enum:
        - github
        - gitlab
        - bitbucket
        - enterprise_sso
      title: ProviderType
    ConversationStatus:
      type: string
      enum:
        - STARTING
        - RUNNING
        - STOPPED
      title: ConversationStatus
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