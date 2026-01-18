# Set Conversation Security Analyzer

> Set the security analyzer for a conversation.



## OpenAPI

````yaml openapi/agent-sdk.json post /api/conversations/{conversation_id}/security_analyzer
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/conversations/{conversation_id}/security_analyzer:
    post:
      tags:
        - Conversations
      summary: Set Conversation Security Analyzer
      description: Set the security analyzer for a conversation.
      operationId: >-
        set_conversation_security_analyzer_api_conversations__conversation_id__security_analyzer_post
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
              $ref: '#/components/schemas/SetSecurityAnalyzerRequest'
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
    SetSecurityAnalyzerRequest:
      properties:
        security_analyzer:
          anyOf:
            - $ref: '#/components/schemas/SecurityAnalyzerBase-Input'
            - type: 'null'
          description: The security analyzer to set
      type: object
      required:
        - security_analyzer
      title: SetSecurityAnalyzerRequest
      description: Payload to set security analyzer for a conversation
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
    SecurityAnalyzerBase-Input:
      $ref: '#/components/schemas/LLMSecurityAnalyzer-Input'
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
    LLMSecurityAnalyzer-Input:
      properties:
        kind:
          type: string
          const: LLMSecurityAnalyzer
          title: Kind
      type: object
      title: LLMSecurityAnalyzer
      description: >-
        LLM-based security analyzer.


        This analyzer respects the security_risk attribute that can be set by
        the LLM

        when generating actions, similar to OpenHands' LLMRiskAnalyzer.


        It provides a lightweight security analysis approach that leverages the
        LLM's

        understanding of action context and potential risks.

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt