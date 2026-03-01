import { useState, useEffect, useCallback } from "react";
import {
  Loader2,
  CheckCircle2,
  AlertCircle,
  Eye,
  EyeOff,
  Trash2,
  Wifi,
  WifiOff,
  Key,
} from "lucide-react";
import { cn } from "@/lib/utils";
import * as settingsService from "@/services/settingsService";
import * as kqlService from "@/services/kqlService";
import type { AppSettings, SettingsUpdate } from "@/types";

interface LLMConfigProps {
  settings: AppSettings;
  onSettingsUpdated: (updated: AppSettings) => void;
}

export function LLMConfig({ settings, onSettingsUpdated }: LLMConfigProps) {
  // Form state
  const [endpoint, setEndpoint] = useState(settings.llm_endpoint);
  const [model, setModel] = useState(settings.llm_model);
  const [apiKey, setApiKey] = useState("");
  const [showApiKey, setShowApiKey] = useState(false);

  // UI state
  const [saving, setSaving] = useState(false);
  const [savingKey, setSavingKey] = useState(false);
  const [clearingKey, setClearingKey] = useState(false);
  const [testing, setTesting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<{
    reachable: boolean;
    message: string;
  } | null>(null);

  // Sync when settings change externally
  useEffect(() => {
    setEndpoint(settings.llm_endpoint);
    setModel(settings.llm_model);
  }, [settings.llm_endpoint, settings.llm_model]);

  const handleSaveEndpoint = useCallback(async () => {
    setSaving(true);
    setError(null);
    setSuccess(null);
    setTestResult(null);

    try {
      const update: SettingsUpdate = {
        llm_endpoint: endpoint,
        llm_model: model,
      };
      const updated = await settingsService.updateSettings(update);
      onSettingsUpdated(updated);
      setSuccess("LLM settings saved");
      setTimeout(() => setSuccess(null), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  }, [endpoint, model, onSettingsUpdated]);

  const handleSetApiKey = useCallback(async () => {
    if (!apiKey.trim()) {
      setError("API key cannot be empty");
      return;
    }

    setSavingKey(true);
    setError(null);
    setSuccess(null);

    try {
      await settingsService.setApiKey(apiKey);
      setApiKey("");
      setShowApiKey(false);
      // Refresh settings to update has_api_key
      const updated = await settingsService.getSettings();
      onSettingsUpdated(updated);
      setSuccess("API key saved to keychain");
      setTimeout(() => setSuccess(null), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSavingKey(false);
    }
  }, [apiKey, onSettingsUpdated]);

  const handleClearApiKey = useCallback(async () => {
    setClearingKey(true);
    setError(null);
    setSuccess(null);

    try {
      await settingsService.clearApiKey();
      const updated = await settingsService.getSettings();
      onSettingsUpdated(updated);
      setSuccess("API key removed from keychain");
      setTimeout(() => setSuccess(null), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setClearingKey(false);
    }
  }, [onSettingsUpdated]);

  const handleTestConnection = useCallback(async () => {
    setTesting(true);
    setError(null);
    setTestResult(null);

    try {
      const result = await kqlService.checkConfig();
      if (result.is_reachable) {
        setTestResult({
          reachable: true,
          message: `Connected to ${result.endpoint}`,
        });
      } else {
        setTestResult({
          reachable: false,
          message: result.error ?? "Connection failed",
        });
      }
    } catch (err) {
      setTestResult({
        reachable: false,
        message: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setTesting(false);
    }
  }, []);

  return (
    <div className="flex flex-col gap-6">
      <div className="flex items-center gap-2">
        <Key className="h-4 w-4 text-muted-foreground" />
        <h3 className="text-sm font-semibold text-foreground">
          LLM Configuration
        </h3>
      </div>

      <p className="text-xs text-muted-foreground -mt-4">
        Configure an OpenAI-compatible API endpoint for KQL query generation.
        Supports Azure OpenAI, OpenAI, Ollama, LM Studio, and other compatible
        providers.
      </p>

      {/* Endpoint URL */}
      <div className="flex flex-col gap-1.5">
        <label
          htmlFor="llm-endpoint"
          className="text-sm font-medium text-foreground"
        >
          Endpoint URL
        </label>
        <input
          id="llm-endpoint"
          type="url"
          value={endpoint}
          onChange={(e) => setEndpoint(e.target.value)}
          placeholder="https://api.openai.com/v1"
          className="rounded-md border border-input bg-background px-3 py-2 text-sm focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
        />
        <p className="text-xs text-muted-foreground">
          The base URL of the OpenAI-compatible API (e.g.,
          https://api.openai.com/v1)
        </p>
      </div>

      {/* Model Name */}
      <div className="flex flex-col gap-1.5">
        <label
          htmlFor="llm-model"
          className="text-sm font-medium text-foreground"
        >
          Model Name (optional)
        </label>
        <input
          id="llm-model"
          type="text"
          value={model}
          onChange={(e) => setModel(e.target.value)}
          placeholder="gpt-4o"
          className="rounded-md border border-input bg-background px-3 py-2 text-sm focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
        />
        <p className="text-xs text-muted-foreground">
          Specify the model to use (leave empty for provider default)
        </p>
      </div>

      {/* Save endpoint + model button */}
      <button
        type="button"
        onClick={handleSaveEndpoint}
        disabled={saving}
        className={cn(
          "self-start inline-flex items-center gap-2 rounded-md px-4 py-2 text-sm font-medium",
          "bg-primary text-primary-foreground hover:bg-primary/90",
          "disabled:pointer-events-none disabled:opacity-50"
        )}
      >
        {saving && <Loader2 className="h-4 w-4 animate-spin" />}
        {saving ? "Saving..." : "Save Endpoint Settings"}
      </button>

      {/* Divider */}
      <hr className="border-border" />

      {/* API Key */}
      <div className="flex flex-col gap-1.5">
        <label
          htmlFor="llm-api-key"
          className="text-sm font-medium text-foreground"
        >
          API Key
        </label>

        {settings.has_api_key ? (
          <div className="flex items-center gap-2">
            <div className="flex-1 flex items-center gap-2 rounded-md border border-input bg-muted/30 px-3 py-2 text-sm">
              <CheckCircle2 className="h-4 w-4 text-emerald-500" />
              <span className="text-muted-foreground">
                API key stored in keychain
              </span>
            </div>
            <button
              type="button"
              onClick={handleClearApiKey}
              disabled={clearingKey}
              className="inline-flex items-center gap-1.5 rounded-md border border-destructive/30 px-3 py-2 text-sm text-destructive hover:bg-destructive/5"
            >
              {clearingKey ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <Trash2 className="h-4 w-4" />
              )}
              Remove
            </button>
          </div>
        ) : (
          <div className="flex gap-2">
            <div className="relative flex-1">
              <input
                id="llm-api-key"
                type={showApiKey ? "text" : "password"}
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="sk-..."
                className="w-full rounded-md border border-input bg-background px-3 py-2 pr-10 text-sm focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
              />
              <button
                type="button"
                onClick={() => setShowApiKey(!showApiKey)}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                title={showApiKey ? "Hide" : "Show"}
              >
                {showApiKey ? (
                  <EyeOff className="h-4 w-4" />
                ) : (
                  <Eye className="h-4 w-4" />
                )}
              </button>
            </div>
            <button
              type="button"
              onClick={handleSetApiKey}
              disabled={savingKey || !apiKey.trim()}
              className={cn(
                "inline-flex items-center gap-2 rounded-md px-4 py-2 text-sm font-medium",
                "bg-primary text-primary-foreground hover:bg-primary/90",
                "disabled:pointer-events-none disabled:opacity-50"
              )}
            >
              {savingKey && <Loader2 className="h-4 w-4 animate-spin" />}
              Save Key
            </button>
          </div>
        )}
        <p className="text-xs text-muted-foreground">
          Stored securely in your operating system&apos;s keychain
        </p>
      </div>

      {/* Divider */}
      <hr className="border-border" />

      {/* Connection Test */}
      <div className="flex flex-col gap-3">
        <button
          type="button"
          onClick={handleTestConnection}
          disabled={testing || !endpoint.trim()}
          className={cn(
            "self-start inline-flex items-center gap-2 rounded-md border border-border px-4 py-2 text-sm font-medium",
            "text-foreground hover:bg-accent",
            "disabled:pointer-events-none disabled:opacity-50"
          )}
        >
          {testing ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <Wifi className="h-4 w-4" />
          )}
          {testing ? "Testing..." : "Test Connection"}
        </button>

        {testResult && (
          <div
            className={cn(
              "flex items-center gap-2 rounded-md border px-3 py-2 text-sm",
              testResult.reachable
                ? "border-emerald-500/20 bg-emerald-500/5 text-emerald-600 dark:text-emerald-400"
                : "border-destructive/20 bg-destructive/5 text-destructive"
            )}
          >
            {testResult.reachable ? (
              <Wifi className="h-4 w-4" />
            ) : (
              <WifiOff className="h-4 w-4" />
            )}
            {testResult.message}
          </div>
        )}
      </div>

      {/* Status messages */}
      {error && (
        <div className="flex items-center gap-2 rounded-md border border-destructive/20 bg-destructive/5 px-3 py-2 text-sm text-destructive">
          <AlertCircle className="h-4 w-4" />
          {error}
        </div>
      )}
      {success && (
        <div className="flex items-center gap-2 rounded-md border border-emerald-500/20 bg-emerald-500/5 px-3 py-2 text-sm text-emerald-600 dark:text-emerald-400">
          <CheckCircle2 className="h-4 w-4" />
          {success}
        </div>
      )}
    </div>
  );
}
