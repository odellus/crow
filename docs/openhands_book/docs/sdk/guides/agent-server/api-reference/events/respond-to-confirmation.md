# Respond To Confirmation

> Accept or reject a pending action in confirmation mode.



## OpenAPI

````yaml openapi/agent-sdk.json post /api/conversations/{conversation_id}/events/respond_to_confirmation
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/conversations/{conversation_id}/events/respond_to_confirmation:
    post:
      tags:
        - Events
      summary: Respond To Confirmation
      description: Accept or reject a pending action in confirmation mode.
      operationId: >-
        respond_to_confirmation_api_conversations__conversation_id__events_respond_to_confirmation_post
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
              $ref: '#/components/schemas/ConfirmationResponseRequest'
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
    ConfirmationResponseRequest:
      properties:
        accept:
          type: boolean
          title: Accept
        reason:
          type: string
          title: Reason
          default: User rejected the action.
      type: object
      required:
        - accept
      title: ConfirmationResponseRequest
      description: Payload to accept or reject a pending action.
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