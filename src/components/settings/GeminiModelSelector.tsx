import React from "react";
import { useTranslation } from "react-i18next";
import { SettingContainer } from "../ui/SettingContainer";
import { Dropdown } from "../ui/Dropdown";
import { useSettings } from "../../hooks/useSettings";

interface GeminiModelSelectorProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

const GEMINI_MODELS = [
  {
    value: "gemini-3-pro",
    label: "Gemini 3 Pro",
    description: "Advanced reasoning capabilities, best for complex tasks",
  },
  {
    value: "gemini-3-flash",
    label: "Gemini 3 Flash",
    description: "Fast and cost-efficient, optimized for speed",
  },
];

export const GeminiModelSelector: React.FC<GeminiModelSelectorProps> =
  React.memo(({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();

    const selectedModel = getSetting("gemini_model") || "gemini-3-pro";

    const options = GEMINI_MODELS.map((model) => ({
      value: model.value,
      label: model.label,
    }));

    return (
      <SettingContainer
        title={t("settings.gemini.model.label")}
        description={t("settings.gemini.model.description")}
        descriptionMode={descriptionMode}
        layout="horizontal"
        grouped={grouped}
      >
        <div className="flex items-center gap-2">
          <Dropdown
            selectedValue={selectedModel}
            options={options}
            onSelect={(value) => {
              updateSetting("gemini_model", value);
            }}
            placeholder={t("settings.gemini.model.placeholder")}
            disabled={isUpdating("gemini_model")}
            className="min-w-[200px]"
          />
        </div>
      </SettingContainer>
    );
  });

GeminiModelSelector.displayName = "GeminiModelSelector";
