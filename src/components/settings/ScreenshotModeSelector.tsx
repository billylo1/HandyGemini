import React from "react";
import { useTranslation } from "react-i18next";
import { SettingContainer } from "../ui/SettingContainer";
import { Dropdown } from "../ui/Dropdown";
import { useSettings } from "../../hooks/useSettings";
import type { ScreenshotMode } from "@/bindings";

interface ScreenshotModeSelectorProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

const SCREENSHOT_MODES = [
  {
    value: "activewindow",
    label: "Active Window",
    description: "Capture only the active window",
  },
  {
    value: "fullscreen",
    label: "Full Screen",
    description: "Capture the entire screen",
  },
];

export const ScreenshotModeSelector: React.FC<ScreenshotModeSelectorProps> =
  React.memo(({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();

    const selectedMode = getSetting("screenshot_mode") || "activewindow";

    const options = SCREENSHOT_MODES.map((mode) => ({
      value: mode.value,
      label: mode.value === "activewindow" 
        ? t("settings.gemini.screenshotMode.options.active_window")
        : t("settings.gemini.screenshotMode.options.full_screen"),
    }));

    return (
      <SettingContainer
        title={t("settings.gemini.screenshotMode.label")}
        description={t("settings.gemini.screenshotMode.description")}
        descriptionMode={descriptionMode}
        layout="horizontal"
        grouped={grouped}
      >
        <div className="flex items-center gap-2">
          <Dropdown
            selectedValue={selectedMode}
            options={options}
            onSelect={(value) => {
              updateSetting("screenshot_mode", value as ScreenshotMode);
            }}
            placeholder={t("settings.gemini.screenshotMode.placeholder")}
            disabled={isUpdating("screenshot_mode")}
            className="min-w-[200px]"
          />
        </div>
      </SettingContainer>
    );
  });

ScreenshotModeSelector.displayName = "ScreenshotModeSelector";
