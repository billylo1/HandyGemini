import React from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";

interface GeminiToggleProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const GeminiToggle: React.FC<GeminiToggleProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();

    const enabled = getSetting("gemini_enabled") || false;

    return (
      <ToggleSwitch
        checked={enabled}
        onChange={(enabled) => updateSetting("gemini_enabled", enabled)}
        isUpdating={isUpdating("gemini_enabled")}
        label={t("settings.gemini.enabled.label")}
        description={t("settings.gemini.enabled.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
      />
    );
  },
);

GeminiToggle.displayName = "GeminiToggle";
