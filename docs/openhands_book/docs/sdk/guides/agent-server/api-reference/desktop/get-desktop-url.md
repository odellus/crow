# Get Desktop Url

> Get the noVNC URL for desktop access.

Args:
    base_url: Base URL for the noVNC server (default: http://localhost:8002)

Returns:
    noVNC URL if available, None otherwise



## OpenAPI

````yaml openapi/agent-sdk.json get /api/desktop/url
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/desktop/url:
    get:
      tags:
        - Desktop
      summary: Get Desktop Url
      description: |-
        Get the noVNC URL for desktop access.

        Args:
            base_url: Base URL for the noVNC server (default: http://localhost:8002)

        Returns:
            noVNC URL if available, None otherwise
      operationId: get_desktop_url_api_desktop_url_get
      parameters:
        - name: base_url
          in: query
          required: false
          schema:
            type: string
            default: http://localhost:8002
            title: Base Url
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/DesktopUrlResponse'
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
components:
  schemas:
    DesktopUrlResponse:
      properties:
        url:
          anyOf:
            - type: string
            - type: 'null'
          title: Url
      type: object
      required:
        - url
      title: DesktopUrlResponse
      description: Response model for Desktop URL.
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