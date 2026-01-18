# Send Message

> Send a message to a conversation



## OpenAPI

````yaml openapi/agent-sdk.json post /api/conversations/{conversation_id}/events
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/conversations/{conversation_id}/events:
    post:
      tags:
        - Events
      summary: Send Message
      description: Send a message to a conversation
      operationId: send_message_api_conversations__conversation_id__events_post
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
              $ref: '#/components/schemas/SendMessageRequest'
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Success'
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
components:
  schemas:
    SendMessageRequest:
      properties:
        role:
          type: string
          enum:
            - user
            - system
            - assistant
            - tool
          title: Role
          default: user
        content:
          items:
            anyOf:
              - $ref: '#/components/schemas/TextContent'
              - $ref: '#/components/schemas/ImageContent'
          type: array
          title: Content
        run:
          type: boolean
          title: Run
          description: Whether the agent loop should automatically run if not running
          default: false
      type: object
      title: SendMessageRequest
      description: |-
        Payload to send a message to the agent.

        This is a simplified version of openhands.sdk.Message.
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
    TextContent:
      properties:
        cache_prompt:
          type: boolean
          title: Cache Prompt
          default: false
        type:
          type: string
          const: text
          title: Type
          default: text
        text:
          type: string
          title: Text
        enable_truncation:
          type: boolean
          title: Enable Truncation
          default: true
      additionalProperties: false
      type: object
      required:
        - text
      title: TextContent
    ImageContent:
      properties:
        cache_prompt:
          type: boolean
          title: Cache Prompt
          default: false
        type:
          type: string
          const: image
          title: Type
          default: image
        image_urls:
          items:
            type: string
          type: array
          title: Image Urls
      type: object
      required:
        - image_urls
      title: ImageContent
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