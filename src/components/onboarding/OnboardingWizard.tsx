import { useState } from "react";
import WelcomeStep from "./steps/WelcomeStep";
import AISetupStep from "./steps/AISetupStep";
import FirstProjectStep from "./steps/FirstProjectStep";
import FirstTranscriptStep from "./steps/FirstTranscriptStep";
import { setAppSetting } from "@/lib/tauri";

interface Props {
  onComplete: () => void;
}

export default function OnboardingWizard({ onComplete }: Props) {
  const [step, setStep] = useState(0);
  const [createdProjectId, setCreatedProjectId] = useState<string | null>(null);

  const complete = async () => {
    await setAppSetting("onboarding_complete", "true");
    onComplete();
  };

  const skip = async () => {
    await setAppSetting("onboarding_complete", "true");
    onComplete();
  };

  return (
    <div className="h-full flex flex-col bg-zinc-50 dark:bg-zinc-950">
      {step === 0 && (
        <WelcomeStep onNext={() => setStep(1)} onSkip={skip} />
      )}
      {step === 1 && (
        <AISetupStep onNext={() => setStep(2)} onSkip={() => setStep(2)} />
      )}
      {step === 2 && (
        <FirstProjectStep
          onNext={(projectId) => {
            setCreatedProjectId(projectId);
            setStep(3);
          }}
          onSkip={() => setStep(3)}
        />
      )}
      {step === 3 && (
        <FirstTranscriptStep
          projectId={createdProjectId}
          onFinish={complete}
          onSkip={complete}
        />
      )}
    </div>
  );
}
