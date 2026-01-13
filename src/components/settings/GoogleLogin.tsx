import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../ui/Button";
import { SettingContainer } from "../ui/SettingContainer";
import { commands } from "@/bindings";
import { listen } from "@tauri-apps/api/event";

interface GoogleAuthStatus {
  is_authenticated: boolean;
  email: string | null;
  name: string | null;
}

export const GoogleLogin: React.FC<{
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}> = ({ descriptionMode = "tooltip", grouped = false }) => {
  const { t } = useTranslation();
  const [authStatus, setAuthStatus] = useState<GoogleAuthStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [authenticating, setAuthenticating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadAuthStatus = async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await commands.getGoogleAuthStatus();
      if (result.status === "ok") {
        setAuthStatus(result.data);
      } else {
        setError(result.error);
      }
    } catch (err) {
      console.error("Failed to load auth status:", err);
      setError(err instanceof Error ? err.message : "Failed to load auth status");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadAuthStatus();

    // Listen for auth success/error events
    const setupListeners = async () => {
      const unlistenSuccess = await listen("google-auth-success", () => {
        console.log("Google auth success event received");
        loadAuthStatus();
        setAuthenticating(false);
      });

      const unlistenError = await listen("google-auth-error", (event) => {
        console.error("Google auth error:", event.payload);
        setError(event.payload as string);
        setAuthenticating(false);
      });

      return () => {
        unlistenSuccess();
        unlistenError();
      };
    };

    let cleanupPromise = setupListeners();

    return () => {
      cleanupPromise.then((cleanup) => {
        if (cleanup) cleanup();
      });
    };
  }, []);

  const handleLogin = async () => {
    try {
      setAuthenticating(true);
      setError(null);
      const result = await commands.startGoogleOauth();
      if (result.status === "error") {
        setError(result.error);
        setAuthenticating(false);
      }
      // If successful, the event listener will handle updating the status
    } catch (err) {
      console.error("Failed to start OAuth:", err);
      setError(err instanceof Error ? err.message : "Failed to start authentication");
      setAuthenticating(false);
    }
  };

  const handleLogout = async () => {
    try {
      setError(null);
      const result = await commands.logoutGoogle();
      if (result.status === "ok") {
        setAuthStatus({
          is_authenticated: false,
          email: null,
          name: null,
        });
      } else {
        setError(result.error);
      }
    } catch (err) {
      console.error("Failed to logout:", err);
      setError(err instanceof Error ? err.message : "Failed to logout");
    }
  };

  if (loading) {
    return (
      <SettingContainer
        title={t("settings.gemini.googleLogin.title")}
        description={t("settings.gemini.googleLogin.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
      >
        <div className="animate-pulse">
          <div className="h-10 bg-gray-200 rounded w-32"></div>
        </div>
      </SettingContainer>
    );
  }

  return (
    <SettingContainer
      title={t("settings.gemini.googleLogin.title")}
      description={t("settings.gemini.googleLogin.description")}
      descriptionMode={descriptionMode}
      grouped={grouped}
    >
      <div className="flex flex-col gap-3">
        {error && (
          <div className="p-3 bg-red-50 border border-red-200 rounded-lg">
            <p className="text-red-600 text-sm">{error}</p>
          </div>
        )}

        {authStatus?.is_authenticated ? (
          <div className="flex flex-col gap-2">
            <div className="flex items-center gap-2">
              <div className="flex-1">
                <p className="text-sm font-medium">
                  {authStatus.name || authStatus.email || t("settings.gemini.googleLogin.authenticated")}
                </p>
                {authStatus.email && (
                  <p className="text-xs text-gray-500">{authStatus.email}</p>
                )}
              </div>
              <Button
                variant="secondary"
                size="sm"
                onClick={handleLogout}
                disabled={loading}
              >
                {t("settings.gemini.googleLogin.logout")}
              </Button>
            </div>
          </div>
        ) : (
          <Button
            variant="primary"
            onClick={handleLogin}
            disabled={authenticating || loading}
          >
            {authenticating
              ? t("settings.gemini.googleLogin.authenticating")
              : t("settings.gemini.googleLogin.login")}
          </Button>
        )}
      </div>
    </SettingContainer>
  );
};
