import { create } from "zustand";
import { persist } from "zustand/middleware";

type ReaderTheme = "light" | "dark" | "sepia";

interface ReaderSettings {
  fontSize: number;
  fontFamily: string;
  theme: ReaderTheme;
  lineHeight: number;
  margins: number;
}

interface ReaderState extends ReaderSettings {
  showToc: boolean;
  showSettings: boolean;

  setFontSize: (size: number) => void;
  setFontFamily: (family: string) => void;
  setTheme: (theme: ReaderTheme) => void;
  setLineHeight: (height: number) => void;
  setMargins: (margins: number) => void;
  setShowToc: (show: boolean) => void;
  setShowSettings: (show: boolean) => void;
  resetSettings: () => void;
}

const defaultSettings: ReaderSettings = {
  fontSize: 18,
  fontFamily: "Georgia, serif",
  theme: "light",
  lineHeight: 1.6,
  margins: 40,
};

export const useReaderStore = create<ReaderState>()(
  persist(
    (set) => ({
      ...defaultSettings,
      showToc: false,
      showSettings: false,

      setFontSize: (fontSize) => set({ fontSize }),
      setFontFamily: (fontFamily) => set({ fontFamily }),
      setTheme: (theme) => set({ theme }),
      setLineHeight: (lineHeight) => set({ lineHeight }),
      setMargins: (margins) => set({ margins }),
      setShowToc: (showToc) => set({ showToc }),
      setShowSettings: (showSettings) => set({ showSettings }),
      resetSettings: () => set(defaultSettings),
    }),
    {
      name: "ereader-reader-settings",
      partialize: (state) => ({
        fontSize: state.fontSize,
        fontFamily: state.fontFamily,
        theme: state.theme,
        lineHeight: state.lineHeight,
        margins: state.margins,
      }),
    }
  )
);

export const themeStyles = {
  light: {
    background: "#ffffff",
    text: "#1a1a1a",
  },
  dark: {
    background: "#1a1a1a",
    text: "#e0e0e0",
  },
  sepia: {
    background: "#f4ecd8",
    text: "#5b4636",
  },
};
