# Load Custom Secrets Names



## OpenAPI

````yaml openapi/openapi.json get /api/secrets
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
  /api/secrets:
    get:
      summary: Load Custom Secrets Names
      operationId: load_custom_secrets_names_api_secrets_get
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/GETCustomSecrets'
      security:
        - APIKeyHeader: []
components:
  schemas:
    GETCustomSecrets:
      properties:
        custom_secrets:
          anyOf:
            - items:
                $ref: '#/components/schemas/CustomSecretWithoutValueModel'
              type: array
            - type: 'null'
          title: Custom Secrets
      type: object
      title: GETCustomSecrets
      description: Custom secrets names
    CustomSecretWithoutValueModel:
      properties:
        name:
          type: string
          title: Name
        description:
          anyOf:
            - type: string
            - type: 'null'
          title: Description
      type: object
      required:
        - name
      title: CustomSecretWithoutValueModel
      description: Custom secret model without value
  securitySchemes:
    APIKeyHeader:
      type: apiKey
      in: header
      name: X-Session-API-Key

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt