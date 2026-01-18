# Download Trajectory

> Download a file from the workspace.



## OpenAPI

````yaml openapi/agent-sdk.json get /api/file/download-trajectory/{conversation_id}
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/file/download-trajectory/{conversation_id}:
    get:
      tags:
        - Files
      summary: Download Trajectory
      description: Download a file from the workspace.
      operationId: download_trajectory_api_file_download_trajectory__conversation_id__get
      parameters:
        - name: conversation_id
          in: path
          required: true
          schema:
            type: string
            format: uuid
            title: Conversation Id
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