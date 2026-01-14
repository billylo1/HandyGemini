import React from "react";
import { useTranslation } from "react-i18next";
import { SettingContainer } from "../ui/SettingContainer";
import { Input } from "../ui/Input";
import { useSettings } from "../../hooks/useSettings";

interface GeminiApiKeyProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const GeminiApiKey: React.FC<GeminiApiKeyProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();

    const apiKey = getSetting("gemini_api_key") || "";

    return (
      <SettingContainer
        title={t("settings.gemini.apiKey.label")}
        description={t("settings.gemini.apiKey.description")}
        descriptionMode={descriptionMode}
        layout="horizontal"
        grouped={grouped}
      >
        <div className="flex items-center gap-2">
          <Input
            type="password"
            value={apiKey}
            onChange={(e) => updateSetting("gemini_api_key", e.target.value)}
            onBlur={(e) => updateSetting("gemini_api_key", e.target.value)}
            placeholder={t("settings.gemini.apiKey.placeholder")}
            disabled={isUpdating("gemini_api_key")}
            className="min-w-[320px]"
            variant="compact"
          />
        </div>
      </SettingContainer>
    );
  },
);

GeminiApiKey.displayName = "GeminiApiKey";
