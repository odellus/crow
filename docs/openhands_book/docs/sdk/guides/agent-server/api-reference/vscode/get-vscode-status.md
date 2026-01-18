# Get Vscode Status

> Get the VSCode server status.

Returns:
    Dictionary with running status and enabled status



## OpenAPI

````yaml openapi/agent-sdk.json get /api/vscode/status
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/vscode/status:
    get:
      tags:
        - VSCode
      summary: Get Vscode Status
      description: |-
        Get the VSCode server status.

        Returns:
            Dictionary with running status and enabled status
      operationId: get_vscode_status_api_vscode_status_get
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                additionalProperties:
                  anyOf:
                    - type: boolean
                    - type: string
                type: object
                title: Response Get Vscode Status Api Vscode Status Get

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt