# Modal Runtime

Our partners at [Modal](https://modal.com/) have provided a runtime for OpenHands.
To use the Modal Runtime, create an account, and then [create an API key.](https://modal.com/settings)

You'll then need to set the following environment variables when starting OpenHands:

```bash  theme={null}
docker run # ...
    -e RUNTIME=modal \
    -e MODAL_API_TOKEN_ID="your-id" \
    -e MODAL_API_TOKEN_SECRET="modal-api-key" \
```


---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt