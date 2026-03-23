import { useEffect, useState } from "react";
import { BrowserRouter, Route, Routes } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Toaster } from "react-hot-toast";
import { useTranslation } from "react-i18next";
import AppShell from "@/components/layout/AppShell";
import OnboardingWizard from "@/components/onboarding/OnboardingWizard";
import { getAppSettings } from "@/lib/tauri";
import { useUIStore } from "@/stores/uiStore";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 30,
      retry: 1,
    },
  },
});

function AppContent() {
  const [onboardingComplete, setOnboardingComplete] = useState<boolean | null>(null);
  const { theme, setTheme, setLanguage } = useUIStore();
  const { i18n } = useTranslation();

  useEffect(() => {
    // Load app settings
    getAppSettings().then((settings) => {
      const complete = settings["onboarding_complete"] === "true";
      setOnboardingComplete(complete);

      const savedTheme = settings["theme"] as "light" | "dark" | "system";
      if (savedTheme) setTheme(savedTheme);

      const lang = settings["language"] || "en";
      setLanguage(lang);
      i18n.changeLanguage(lang);
    }).catch(() => {
      setOnboardingComplete(false);
    });
  }, []);

  // Apply theme
  useEffect(() => {
    const root = document.documentElement;
    if (theme === "dark") {
      root.classList.add("dark");
    } else if (theme === "light") {
      root.classList.remove("dark");
    } else {
      // System
      const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      root.classList.toggle("dark", prefersDark);
    }
  }, [theme]);

  if (onboardingComplete === null) {
    return (
      <div className="h-full flex items-center justify-center bg-zinc-50 dark:bg-zinc-950">
        <div className="w-8 h-8 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  return (
    <BrowserRouter>
      <Routes>
        <Route
          path="/*"
          element={
            onboardingComplete ? (
              <AppShell />
            ) : (
              <OnboardingWizard onComplete={() => setOnboardingComplete(true)} />
            )
          }
        />
      </Routes>
    </BrowserRouter>
  );
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AppContent />
      <Toaster
        position="bottom-left"
        toastOptions={{
          className: "dark:bg-zinc-800 dark:text-zinc-50",
          duration: 3000,
        }}
      />
    </QueryClientProvider>
  );
}
