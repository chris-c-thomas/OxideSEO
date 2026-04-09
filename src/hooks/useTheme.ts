/**
 * Theme management hook.
 *
 * Detects system preference via `prefers-color-scheme`, allows manual
 * override (light / dark / system), and persists choice to localStorage.
 * Applies `data-theme` attribute to the document root and toggles the
 * `dark` class for Tailwind `dark:` variant compatibility.
 */

import { useCallback, useEffect, useState } from "react";

export type Theme = "light" | "dark" | "system";
type ResolvedTheme = "light" | "dark";

const STORAGE_KEY = "oxide-seo-theme";

function getSystemTheme(): ResolvedTheme {
  if (typeof window === "undefined") return "light";
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function getStoredTheme(): Theme | null {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === "light" || stored === "dark" || stored === "system") return stored;
  } catch {
    // localStorage unavailable -- fall through.
  }
  return null;
}

function applyTheme(resolved: ResolvedTheme) {
  const root = document.documentElement;
  root.setAttribute("data-theme", resolved);
  // Toggle .dark class for Tailwind dark: variant compatibility
  // (shadcn components use dark: utilities for fine-tuning).
  root.classList.toggle("dark", resolved === "dark");
}

export function useTheme() {
  const [theme, setThemeState] = useState<Theme>(() => {
    return getStoredTheme() ?? "light";
  });

  // Track system theme separately so changes trigger re-renders.
  const [systemTheme, setSystemTheme] = useState<ResolvedTheme>(getSystemTheme);

  const resolved = theme === "system" ? systemTheme : theme;

  useEffect(() => {
    applyTheme(resolved);
  }, [resolved]);

  // Listen for OS theme changes.
  useEffect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (e: MediaQueryListEvent) => {
      setSystemTheme(e.matches ? "dark" : "light");
    };
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, []);

  const setTheme = useCallback((newTheme: Theme) => {
    setThemeState(newTheme);
    try {
      localStorage.setItem(STORAGE_KEY, newTheme);
    } catch {
      // Ignore.
    }
  }, []);

  return { theme, resolved, setTheme };
}
