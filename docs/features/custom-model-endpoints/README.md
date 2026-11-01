# Custom model endpoints

**Feature key:** \`customModelEndpoints\`  
**Deployments:** OpenWork Cloud and self-hosted  
**Default:** Off  
**Owner:** [MrPenguin-DEV](https://github.com/MrPenguin-DEV)

## What changes

AI Providers settings gains an **Add custom endpoint** dialog for:

- OpenAI-compatible APIs, using the provider's \`/v1\` base URL.
- Remote Ollama servers that expose Ollama's OpenAI-compatible API, including a public HTTPS tunnel to a notebook-hosted server such as Kaggle.

People can enter a model ID manually or ask the endpoint for its model list. Discovery is a browser request and can be blocked by CORS; manual entry remains available. An optional API key is stored through OpenCode's credential store and is not written into the workspace provider configuration.

The endpoint configuration is added to the selected workspace and the engine is reloaded through the existing local-provider configuration flow. The user can choose to make the model the workspace default.

## How to try it

1. Enable \`customModelEndpoints\` for a test organization or the deployment in Den's Features controls.
2. Open **Settings → AI Providers → Add custom endpoint**.
3. Select **OpenAI-compatible** or **Ollama**, give the provider a name, and enter its base URL.
4. For a Kaggle Ollama session, paste the HTTPS URL supplied by the tunnel and keep the Kaggle notebook and tunnel alive while using the model. Use the model ID reported by \`ollama list\`, or load the list in the dialog if the tunnel allows browser CORS.
5. Save the endpoint, then select it in the model picker.

The endpoint must be reachable from the machine running the OpenWork workspace server/engine. A notebook runtime or tunnel that sleeps, expires, or changes its URL will make the configured provider unreachable.

## Turning it off

The deployment or organization feature flag hides the ability to add new custom endpoints. Existing workspace provider configurations are not deleted, so turning the flag off does not rewrite user configuration or revoke credentials. A normal configuration cleanup can remove an endpoint later.

## What to watch

- Model-list discovery failures caused by CORS, authentication, or an unavailable tunnel.
- Provider-auth failures when an API key is required.
- Workspace engine reload failures after the provider configuration is written.

## UI evidence

A real-size application screenshot is still required before this feature is ready for review. It was not possible to launch the repository's desktop/web runtime from the GitHub connector in this session, so no screenshot is claimed here.
