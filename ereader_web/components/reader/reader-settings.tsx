"use client";

import { X, Minus, Plus, RotateCcw } from "lucide-react";
import { useReaderStore, themeStyles } from "@/lib/store/reader-store";
import { cn } from "@/lib/utils";

interface ReaderSettingsProps {
  onClose: () => void;
}

const fontFamilies = [
  { value: "Georgia, serif", label: "Georgia" },
  { value: "Times New Roman, serif", label: "Times New Roman" },
  { value: "Arial, sans-serif", label: "Arial" },
  { value: "Verdana, sans-serif", label: "Verdana" },
  { value: "system-ui, sans-serif", label: "System UI" },
];

export function ReaderSettings({ onClose }: ReaderSettingsProps) {
  const {
    fontSize,
    setFontSize,
    fontFamily,
    setFontFamily,
    theme,
    setTheme,
    lineHeight,
    setLineHeight,
    margins,
    setMargins,
    resetSettings,
  } = useReaderStore();

  return (
    <div className="absolute right-4 top-16 z-50 w-72 rounded-xl border border-foreground/10 bg-background shadow-lg">
      <div className="flex items-center justify-between border-b border-foreground/10 px-4 py-3">
        <h3 className="font-medium">Reading Settings</h3>
        <button onClick={onClose} className="rounded p-1 hover:bg-foreground/5">
          <X className="h-4 w-4" />
        </button>
      </div>

      <div className="p-4 space-y-5">
        <div>
          <label className="block text-sm text-foreground/70 mb-2">
            Font Size
          </label>
          <div className="flex items-center gap-3">
            <button
              onClick={() => setFontSize(Math.max(12, fontSize - 2))}
              className="rounded-lg border border-foreground/20 p-2 hover:bg-foreground/5"
            >
              <Minus className="h-4 w-4" />
            </button>
            <span className="flex-1 text-center font-medium">{fontSize}px</span>
            <button
              onClick={() => setFontSize(Math.min(32, fontSize + 2))}
              className="rounded-lg border border-foreground/20 p-2 hover:bg-foreground/5"
            >
              <Plus className="h-4 w-4" />
            </button>
          </div>
        </div>

        <div>
          <label className="block text-sm text-foreground/70 mb-2">
            Font Family
          </label>
          <select
            value={fontFamily}
            onChange={(e) => setFontFamily(e.target.value)}
            className="w-full rounded-lg border border-foreground/20 bg-transparent px-3 py-2 text-sm"
          >
            {fontFamilies.map((font) => (
              <option key={font.value} value={font.value}>
                {font.label}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="block text-sm text-foreground/70 mb-2">Theme</label>
          <div className="flex gap-2">
            {(Object.keys(themeStyles) as Array<keyof typeof themeStyles>).map(
              (t) => (
                <button
                  key={t}
                  onClick={() => setTheme(t)}
                  className={cn(
                    "flex-1 rounded-lg border-2 px-3 py-2 text-sm capitalize transition-colors",
                    theme === t
                      ? "border-foreground"
                      : "border-foreground/20 hover:border-foreground/40"
                  )}
                  style={{
                    backgroundColor: themeStyles[t].background,
                    color: themeStyles[t].text,
                  }}
                >
                  {t}
                </button>
              )
            )}
          </div>
        </div>

        <div>
          <label className="block text-sm text-foreground/70 mb-2">
            Line Height
          </label>
          <input
            type="range"
            min="1.2"
            max="2.2"
            step="0.1"
            value={lineHeight}
            onChange={(e) => setLineHeight(parseFloat(e.target.value))}
            className="w-full"
          />
          <div className="flex justify-between text-xs text-foreground/50 mt-1">
            <span>Compact</span>
            <span>Spacious</span>
          </div>
        </div>

        <div>
          <label className="block text-sm text-foreground/70 mb-2">
            Margins
          </label>
          <input
            type="range"
            min="10"
            max="80"
            step="10"
            value={margins}
            onChange={(e) => setMargins(parseInt(e.target.value))}
            className="w-full"
          />
          <div className="flex justify-between text-xs text-foreground/50 mt-1">
            <span>Narrow</span>
            <span>Wide</span>
          </div>
        </div>

        <button
          onClick={resetSettings}
          className="flex w-full items-center justify-center gap-2 rounded-lg border border-foreground/20 px-4 py-2 text-sm hover:bg-foreground/5"
        >
          <RotateCcw className="h-4 w-4" />
          Reset to Defaults
        </button>
      </div>
    </div>
  );
}
