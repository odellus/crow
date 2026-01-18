# Update Conversation Secrets

> Update secrets for a conversation.



## OpenAPI

````yaml openapi/agent-sdk.json post /api/conversations/{conversation_id}/secrets
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/conversations/{conversation_id}/secrets:
    post:
      tags:
        - Conversations
      summary: Update Conversation Secrets
      description: Update secrets for a conversation.
      operationId: >-
        update_conversation_secrets_api_conversations__conversation_id__secrets_post
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
              $ref: '#/components/schemas/UpdateSecretsRequest'
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
    UpdateSecretsRequest:
      properties:
        secrets:
          additionalProperties:
            $ref: '#/components/schemas/SecretSource-Input'
          type: object
          title: Secrets
          description: Dictionary mapping secret keys to values
      type: object
      required:
        - secrets
      title: UpdateSecretsRequest
      description: Payload to update secrets in a conversation.
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
    SecretSource-Input:
      oneOf:
        - $ref: '#/components/schemas/LookupSecret-Input'
        - $ref: '#/components/schemas/StaticSecret-Input'
      discriminator:
        propertyName: kind
        mapping:
          openhands__sdk__secret__secrets__LookupSecret-Input__1: '#/components/schemas/LookupSecret-Input'
          openhands__sdk__secret__secrets__StaticSecret-Input__1: '#/components/schemas/StaticSecret-Input'
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
    LookupSecret-Input:
      properties:
        description:
          anyOf:
            - type: string
            - type: 'null'
          title: Description
          description: Optional description for this secret
        url:
          type: string
          title: Url
        headers:
          additionalProperties:
            type: string
          type: object
          title: Headers
        kind:
          type: string
          const: LookupSecret
          title: Kind
      type: object
      required:
        - url
      title: LookupSecret
      description: A secret looked up from some external url
    StaticSecret-Input:
      properties:
        description:
          anyOf:
            - type: string
            - type: 'null'
          title: Description
          description: Optional description for this secret
        value:
          anyOf:
            - type: string
              format: password
              writeOnly: true
            - type: 'null'
          title: Value
        kind:
          type: string
          const: StaticSecret
          title: Kind
      type: object
      title: StaticSecret
      description: A secret stored locally

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt