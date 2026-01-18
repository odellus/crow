# Get Vscode Url

> Get the VSCode URL with authentication token.

Args:
    base_url: Base URL for the VSCode server (default: http://localhost:8001)
    workspace_dir: Path to workspace directory

Returns:
    VSCode URL with token if available, None otherwise



## OpenAPI

````yaml openapi/agent-sdk.json get /api/vscode/url
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/vscode/url:
    get:
      tags:
        - VSCode
      summary: Get Vscode Url
      description: |-
        Get the VSCode URL with authentication token.

        Args:
            base_url: Base URL for the VSCode server (default: http://localhost:8001)
            workspace_dir: Path to workspace directory

        Returns:
            VSCode URL with token if available, None otherwise
      operationId: get_vscode_url_api_vscode_url_get
      parameters:
        - name: base_url
          in: query
          required: false
          schema:
            type: string
            default: http://localhost:8001
            title: Base Url
        - name: workspace_dir
          in: query
          required: false
          schema:
            type: string
            default: workspace
            title: Workspace Dir
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/VSCodeUrlResponse'
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
components:
  schemas:
    VSCodeUrlResponse:
      properties:
        url:
          anyOf:
            - type: string
            - type: 'null'
          title: Url
      type: object
      required:
        - url
      title: VSCodeUrlResponse
      description: Response model for VSCode URL.
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