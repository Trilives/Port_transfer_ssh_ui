import { type ReactNode } from "react";
import { Card, CardDescription, CardHeader, CardTitle } from "../components/ui/card";
import { Select } from "../components/ui/select";
import { t } from "../i18n";
import { languageLabels } from "../i18n";
import type { AppSettings, Language, LogLevel, ThemeName } from "../types";

function SettingSelect(props: { label: string; value: string; onChange: (value: string) => void; children: ReactNode }) {
  return (
    <label className="grid grid-cols-[160px_240px] items-center gap-4 text-sm font-medium text-slate-600 dark:text-slate-300">
      {props.label}
      <Select value={props.value} onChange={(event) => props.onChange(event.target.value)}>
        {props.children}
      </Select>
    </label>
  );
}

export function SettingsPage(props: { settings: AppSettings; setSettings: (settings: Partial<AppSettings>) => void }) {
  const lang = props.settings.language;
  return (
    <Card className="max-w-3xl">
      <CardHeader>
        <CardTitle>{t(lang, "settingsTitle")}</CardTitle>
        <CardDescription>{t(lang, "settingsDesc")}</CardDescription>
      </CardHeader>
      <div className="grid gap-4">
        <SettingSelect label={t(lang, "theme")} value={props.settings.theme} onChange={(v) => props.setSettings({ theme: v as ThemeName })}>
          <option value="light">{t(lang, "themeLight")}</option>
          <option value="dark">{t(lang, "themeDark")}</option>
        </SettingSelect>
        <SettingSelect label={t(lang, "language")} value={props.settings.language} onChange={(v) => props.setSettings({ language: v as Language })}>
          {Object.entries(languageLabels).map(([value, label]) => (
            <option key={value} value={value}>
              {label}
            </option>
          ))}
        </SettingSelect>
        <SettingSelect label={t(lang, "logLevel")} value={props.settings.logLevel} onChange={(v) => props.setSettings({ logLevel: v as LogLevel })}>
          <option value="debug">{t(lang, "logDebug")}</option>
          <option value="info">{t(lang, "logInfo")}</option>
          <option value="warning">{t(lang, "logWarning")}</option>
          <option value="error">{t(lang, "logError")}</option>
        </SettingSelect>
      </div>
    </Card>
  );
}
