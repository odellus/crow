# Set Conversation Confirmation Policy

> Set the confirmation policy for a conversation.



## OpenAPI

````yaml openapi/agent-sdk.json post /api/conversations/{conversation_id}/confirmation_policy
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/conversations/{conversation_id}/confirmation_policy:
    post:
      tags:
        - Conversations
      summary: Set Conversation Confirmation Policy
      description: Set the confirmation policy for a conversation.
      operationId: >-
        set_conversation_confirmation_policy_api_conversations__conversation_id__confirmation_policy_post
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
              $ref: '#/components/schemas/SetConfirmationPolicyRequest'
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
    SetConfirmationPolicyRequest:
      properties:
        policy:
          $ref: '#/components/schemas/ConfirmationPolicyBase-Input'
          description: The confirmation policy to set
      type: object
      required:
        - policy
      title: SetConfirmationPolicyRequest
      description: Payload to set confirmation policy for a conversation.
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
    ConfirmationPolicyBase-Input:
      oneOf:
        - $ref: '#/components/schemas/AlwaysConfirm-Input'
        - $ref: '#/components/schemas/ConfirmRisky-Input'
        - $ref: '#/components/schemas/NeverConfirm-Input'
      discriminator:
        propertyName: kind
        mapping:
          openhands__sdk__security__confirmation_policy__AlwaysConfirm-Input__1: '#/components/schemas/AlwaysConfirm-Input'
          openhands__sdk__security__confirmation_policy__ConfirmRisky-Input__1: '#/components/schemas/ConfirmRisky-Input'
          openhands__sdk__security__confirmation_policy__NeverConfirm-Input__1: '#/components/schemas/NeverConfirm-Input'
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
    AlwaysConfirm-Input:
      properties:
        kind:
          type: string
          const: AlwaysConfirm
          title: Kind
      type: object
      title: AlwaysConfirm
    ConfirmRisky-Input:
      properties:
        threshold:
          $ref: '#/components/schemas/SecurityRisk'
          default: HIGH
        confirm_unknown:
          type: boolean
          title: Confirm Unknown
          default: true
        kind:
          type: string
          const: ConfirmRisky
          title: Kind
      type: object
      title: ConfirmRisky
    NeverConfirm-Input:
      properties:
        kind:
          type: string
          const: NeverConfirm
          title: Kind
      type: object
      title: NeverConfirm
    SecurityRisk:
      type: string
      enum:
        - UNKNOWN
        - LOW
        - MEDIUM
        - HIGH
      title: SecurityRisk
      description: |-
        Security risk levels for actions.

        Based on OpenHands security risk levels but adapted for agent-sdk.
        Integer values allow for easy comparison and ordering.

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt