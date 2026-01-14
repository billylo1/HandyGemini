import React from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";

interface GeminiSendAudioProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const GeminiSendAudio: React.FC<GeminiSendAudioProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();

    const sendAudio = getSetting("gemini_send_audio") || false;

    return (
      <ToggleSwitch
        checked={sendAudio}
        onChange={(sendAudio) => updateSetting("gemini_send_audio", sendAudio)}
        isUpdating={isUpdating("gemini_send_audio")}
        label={t("settings.gemini.sendAudio.label")}
        description={t("settings.gemini.sendAudio.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
      />
    );
  },
);

GeminiSendAudio.displayName = "GeminiSendAudio";
