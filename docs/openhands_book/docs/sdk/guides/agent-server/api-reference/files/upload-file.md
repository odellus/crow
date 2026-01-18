# Upload File

> Upload a file to the workspace.



## OpenAPI

````yaml openapi/agent-sdk.json post /api/file/upload/{path}
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/file/upload/{path}:
    post:
      tags:
        - Files
      summary: Upload File
      description: Upload a file to the workspace.
      operationId: upload_file_api_file_upload__path__post
      parameters:
        - name: path
          in: path
          required: true
          schema:
            type: string
            description: Absolute file path.
            title: Path
          description: Absolute file path.
      requestBody:
        required: true
        content:
          multipart/form-data:
            schema:
              $ref: >-
                #/components/schemas/Body_upload_file_api_file_upload__path__post
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
    Body_upload_file_api_file_upload__path__post:
      properties:
        file:
          type: string
          format: binary
          title: File
      type: object
      required:
        - file
      title: Body_upload_file_api_file_upload__path__post
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