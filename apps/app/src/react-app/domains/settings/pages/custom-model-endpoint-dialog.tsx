/** @jsxImportSource react */
import { useState } from "react";
import { Loader2, Plus } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";

import type { CustomEndpointInstallInput } from "../openai-image-extension";

type EndpointKind = "openai-compatible" | "ollama";

type CustomModelEndpointDialogProps = {
  disabled?: boolean;
  busy: boolean;
  error?: string | null;
  status?: string | null;
  onAdd?: (input: CustomEndpointInstallInput) => Promise<void>;
};

function normalizeBaseURL(value: string): string {
  let url: URL;
  try {
    url = new URL(value.trim());
  } catch {
    throw new Error("Enter a complete endpoint URL, including https:// or http://.");
  }

  if (url.protocol !== "https:" && url.protocol !== "http:") {
    throw new Error("Endpoint URLs must use HTTPS or HTTP.");
  }
  if (url.username || url.password || url.search || url.hash) {
    throw new Error("Remove credentials, query parameters, and fragments from the endpoint URL.");
  }

  const path = url.pathname.replace(/\/+$/, "");
  url.pathname = /\/v1$/i.test(path) ? path : \`\${path}/v1\`;
  return url.toString().replace(/\/$/, "");
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function parseModelIds(payload: unknown, kind: EndpointKind): string[] {
  if (!isRecord(payload)) return [];
  const entries = payload[kind === "ollama" ? "models" : "data"];
  if (!Array.isArray(entries)) return [];

  const ids = entries.flatMap((entry) => {
    if (!isRecord(entry)) return [];
    const id = entry[kind === "ollama" ? "name" : "id"];
    return typeof id === "string" && id.trim() ? [id.trim()] : [];
  });
  return [...new Set(ids)];
}

async function discoverModels(kind: EndpointKind, baseURL: string, apiKey: string): Promise<string[]> {
  const endpoint = kind === "ollama"
    ? \`\${baseURL.replace(/\/v1\/?$/i, "")}/api/tags\`
    : \`\${baseURL}/models\`;
  const headers = apiKey.trim() ? { Authorization: \`Bearer \${apiKey.trim()}\` } : {};
  const response = await fetch(endpoint, {
    method: "GET",
    headers,
    credentials: "omit",
    signal: AbortSignal.timeout(8000),
  });

  if (!response.ok) {
    throw new Error(\`The server returned HTTP \${response.status} while listing models.\`);
  }

  const models = parseModelIds(await response.json(), kind);
  if (!models.length) {
    throw new Error("The endpoint responded but returned no model IDs. Enter a model ID manually.");
  }
  return models;
}

function providerIdFromName(name: string): string {
  const slug = name.toLowerCase().replace(/[^a-z0-9]+/g, "_").replace(/^_+|_+$/g, "").slice(0, 48);
  if (!slug) throw new Error("Provider name must contain at least one letter or number.");
  return \`custom_\${slug}\`;
}

export function CustomModelEndpointDialog(props: CustomModelEndpointDialogProps) {
  const [open, setOpen] = useState(false);
  const [kind, setKind] = useState<EndpointKind>("openai-compatible");
  const [name, setName] = useState("My custom provider");
  const [baseURL, setBaseURL] = useState("");
  const [modelId, setModelId] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [setDefault, setSetDefault] = useState(true);
  const [models, setModels] = useState<string[]>([]);
  const [loadingModels, setLoadingModels] = useState(false);
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [modelHint, setModelHint] = useState<string | null>(null);

  const loadModels = async () => {
    setFormError(null);
    setModelHint(null);
    setLoadingModels(true);
    try {
      const normalized = normalizeBaseURL(baseURL);
      const found = await discoverModels(kind, normalized, apiKey);
      setModels(found);
      setModelId((current) => current.trim() || found[0] || "");
      setModelHint(\`Found \${found.length} model\${found.length === 1 ? "" : "s"}.\`);
    } catch (error) {
      setModels([]);
      setFormError(error instanceof Error
        ? \`\${error.message} You can still enter a model ID manually; some tunnels require CORS to be enabled for model discovery.\`
        : "Couldn't read models from this endpoint. Enter a model ID manually.");
    } finally {
      setLoadingModels(false);
    }
  };

  const addEndpoint = async () => {
    setFormError(null);
    try {
      const trimmedName = name.trim();
      const trimmedModelId = modelId.trim();
      if (!trimmedName) throw new Error("Enter a provider name.");
      if (!trimmedModelId) throw new Error("Enter a model ID or load one from the endpoint.");
      const normalized = normalizeBaseURL(baseURL);
      const providerId = providerIdFromName(trimmedName);

      setSaving(true);
      await props.onAdd?.({
        providerId,
        name: trimmedName,
        baseURL: normalized,
        modelId: trimmedModelId,
        modelName: trimmedModelId,
        setDefault,
        apiKey,
      });
      setOpen(false);
      setApiKey("");
      setFormError(null);
      setModelHint(null);
    } catch (error) {
      setFormError(error instanceof Error ? error.message : "Couldn't add this endpoint.");
    } finally {
      setSaving(false);
    }
  };

  const submitting = saving || props.busy;

  return (
    <>
      <Button
        variant="outline"
        disabled={props.disabled || !props.onAdd}
        onClick={() => setOpen(true)}
      >
        <Plus className="size-4" />
        Add custom endpoint
      </Button>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent className="max-h-[85vh] overflow-y-auto sm:max-w-lg">
          <DialogHeader>
            <DialogTitle>Add a custom model endpoint</DialogTitle>
          </DialogHeader>

          <FieldGroup className="gap-4">
            <Field>
              <FieldLabel>Provider type</FieldLabel>
              <RadioGroup
                value={kind}
                onValueChange={(value) => {
                  if (value === "ollama" || value === "openai-compatible") {
                    setKind(value);
                    setModels([]);
                    setModelHint(null);
                    setFormError(null);
                  }
                }}
                className="grid gap-2 sm:grid-cols-2"
              >
                <Field orientation="horizontal" className="rounded-lg border border-border p-3">
                  <RadioGroupItem id="custom-endpoint-openai" value="openai-compatible" />
                  <FieldLabel htmlFor="custom-endpoint-openai">OpenAI-compatible</FieldLabel>
                </Field>
                <Field orientation="horizontal" className="rounded-lg border border-border p-3">
                  <RadioGroupItem id="custom-endpoint-ollama" value="ollama" />
                  <FieldLabel htmlFor="custom-endpoint-ollama">Ollama</FieldLabel>
                </Field>
              </RadioGroup>
            </Field>

            <Field>
              <FieldLabel htmlFor="custom-endpoint-name">Provider name</FieldLabel>
              <Input
                id="custom-endpoint-name"
                value={name}
                onChange={(event) => setName(event.currentTarget.value)}
                placeholder="Kaggle Ollama"
                autoComplete="off"
              />
              <FieldDescription>A local provider ID is generated from this name.</FieldDescription>
            </Field>

            <Field>
              <FieldLabel htmlFor="custom-endpoint-url">Base URL</FieldLabel>
              <Input
                id="custom-endpoint-url"
                value={baseURL}
                onChange={(event) => {
                  setBaseURL(event.currentTarget.value);
                  setModels([]);
                  setModelHint(null);
                }}
                placeholder={kind === "ollama" ? "https://your-kaggle-tunnel.example/v1" : "https://api.example.com/v1"}
                autoComplete="url"
                spellCheck={false}
              />
              <FieldDescription>
                {kind === "ollama"
                  ? "For Kaggle, paste the HTTPS tunnel URL for your running Ollama server. Keep the notebook session and tunnel running."
                  : "Enter the OpenAI-compatible API base URL. /v1 is added automatically when omitted."}
              </FieldDescription>
            </Field>

            <Field>
              <FieldLabel htmlFor="custom-endpoint-model">Model ID</FieldLabel>
              <div className="flex items-center gap-2">
                <Input
                  id="custom-endpoint-model"
                  value={modelId}
                  onChange={(event) => setModelId(event.currentTarget.value)}
                  placeholder={kind === "ollama" ? "qwen2.5-coder:7b" : "model-name"}
                  list="custom-endpoint-model-options"
                  autoComplete="off"
                  spellCheck={false}
                />
                <Button type="button" variant="outline" onClick={() => void loadModels()} disabled={loadingModels || !baseURL.trim()}>
                  {loadingModels ? <Loader2 className="size-4 animate-spin" /> : null}
                  Load models
                </Button>
              </div>
              <datalist id="custom-endpoint-model-options">
                {models.map((model) => <option key={model} value={model} />)}
              </datalist>
              <FieldDescription>{modelHint ?? "Enter a model ID manually or load the endpoint's model list."}</FieldDescription>
            </Field>

            <Field>
              <FieldLabel htmlFor="custom-endpoint-api-key">API key (optional)</FieldLabel>
              <Input
                id="custom-endpoint-api-key"
                type="password"
                value={apiKey}
                onChange={(event) => setApiKey(event.currentTarget.value)}
                placeholder="Paste an API key when the endpoint requires one"
                autoComplete="new-password"
              />
              <FieldDescription>The key is saved in OpenCode's credential store, not in the workspace config.</FieldDescription>
            </Field>

            <Field orientation="horizontal">
              <Checkbox
                id="custom-endpoint-default"
                checked={setDefault}
                onCheckedChange={(checked) => setSetDefault(checked === true)}
                nativeButton
                render={<button type="button" />}
              />
              <FieldLabel htmlFor="custom-endpoint-default">Use this model as the workspace default</FieldLabel>
            </Field>
          </FieldGroup>

          <p className="text-xs text-muted-foreground">
            Model discovery runs in this window. If a Kaggle tunnel blocks browser CORS, enter the model ID manually; model requests still run through your workspace server.
          </p>
          {formError || props.error ? (
            <p role="alert" className="text-sm text-red-11">{formError ?? props.error}</p>
          ) : null}

          <DialogFooter>
            <Button variant="outline" disabled={submitting} onClick={() => setOpen(false)}>Cancel</Button>
            <Button disabled={submitting || !props.onAdd || !name.trim() || !baseURL.trim() || !modelId.trim()} onClick={() => void addEndpoint()}>
              {submitting ? <Loader2 className="size-4 animate-spin" /> : null}
              Add endpoint
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      {props.status ? <p className="text-xs text-muted-foreground" role="status">{props.status}</p> : null}
    </>
  );
}
