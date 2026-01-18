# Upload Files



## OpenAPI

````yaml openapi/openapi.json post /api/conversations/{conversation_id}/upload-files
openapi: 3.1.0
info:
  title: OpenHands
  description: 'OpenHands: Code Less, Make More'
  version: 0.53.0
servers:
  - url: https://app.all-hands.dev
    description: Production server
  - url: http://localhost:3000
    description: Local development server
security: []
paths:
  /api/conversations/{conversation_id}/upload-files:
    post:
      summary: Upload Files
      operationId: upload_files_api_conversations__conversation_id__upload_files_post
      parameters:
        - name: conversation_id
          in: path
          required: true
          schema:
            type: string
            title: Conversation Id
      requestBody:
        required: true
        content:
          multipart/form-data:
            schema:
              $ref: >-
                #/components/schemas/Body_upload_files_api_conversations__conversation_id__upload_files_post
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/POSTUploadFilesModel'
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
      security:
        - APIKeyHeader: []
components:
  schemas:
    Body_upload_files_api_conversations__conversation_id__upload_files_post:
      properties:
        files:
          items:
            type: string
            format: binary
          type: array
          title: Files
      type: object
      required:
        - files
      title: Body_upload_files_api_conversations__conversation_id__upload_files_post
    POSTUploadFilesModel:
      properties:
        file_urls:
          items:
            type: string
          type: array
          title: File Urls
        skipped_files:
          items:
            type: string
          type: array
          title: Skipped Files
      type: object
      required:
        - file_urls
        - skipped_files
      title: POSTUploadFilesModel
      description: Upload files response model
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
  securitySchemes:
    APIKeyHeader:
      type: apiKey
      in: header
      name: X-Session-API-Key

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt