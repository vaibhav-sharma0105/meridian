import { useTranslation } from "react-i18next";

interface Props {
  onNext: () => void;
  onSkip: () => void;
}

export default function WelcomeStep({ onNext, onSkip }: Props) {
  const { t } = useTranslation();
  return (
    <div className="flex-1 flex flex-col items-center justify-center p-8 text-center">
      {/* Logo */}
      <svg width="64" height="64" viewBox="0 0 64 64" fill="none" className="mb-8">
        <rect width="64" height="64" rx="14" fill="#6366f1" />
        <path d="M16 48 L32 18 L48 48" stroke="white" strokeWidth="5" strokeLinecap="round" strokeLinejoin="round" fill="none" />
        <path d="M23 38 L41 38" stroke="white" strokeWidth="4" strokeLinecap="round" />
      </svg>

      <h1 className="text-3xl font-bold text-zinc-900 dark:text-zinc-50 mb-4 max-w-lg">
        {t("onboarding.welcome.headline")}
      </h1>
      <p className="text-zinc-500 dark:text-zinc-400 text-lg max-w-md mb-12 leading-relaxed">
        {t("onboarding.welcome.subline")}
      </p>

      <button
        onClick={onNext}
        className="px-8 py-3 bg-indigo-500 hover:bg-indigo-600 text-white rounded-lg font-semibold text-lg transition-colors"
      >
        {t("onboarding.welcome.cta")}
      </button>

      <button
        onClick={onSkip}
        className="mt-4 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-200 text-sm transition-colors"
      >
        {t("onboarding.welcome.skip")}
      </button>
    </div>
  );
}
