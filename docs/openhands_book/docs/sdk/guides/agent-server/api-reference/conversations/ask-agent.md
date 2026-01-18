# Ask Agent

> Ask the agent a simple question without affecting conversation state.



## OpenAPI

````yaml openapi/agent-sdk.json post /api/conversations/{conversation_id}/ask_agent
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/conversations/{conversation_id}/ask_agent:
    post:
      tags:
        - Conversations
      summary: Ask Agent
      description: Ask the agent a simple question without affecting conversation state.
      operationId: ask_agent_api_conversations__conversation_id__ask_agent_post
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
              $ref: '#/components/schemas/AskAgentRequest'
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AskAgentResponse'
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
    AskAgentRequest:
      properties:
        question:
          type: string
          title: Question
          description: The question to ask the agent
      type: object
      required:
        - question
      title: AskAgentRequest
      description: Payload to ask the agent a simple question.
    AskAgentResponse:
      properties:
        response:
          type: string
          title: Response
          description: The agent's response to the question
      type: object
      required:
        - response
      title: AskAgentResponse
      description: Response containing the agent's answer.
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