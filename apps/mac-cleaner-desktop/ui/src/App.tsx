import { ActivitySection } from "./components/ActivitySection";
import { AppHeader } from "./components/AppHeader";
import { CleanupWorkspace } from "./components/CleanupWorkspace";
import { PlanReview } from "./components/PlanReview";
import { ProcessMonitor } from "./components/ProcessMonitor";
import { QuarantineSection } from "./components/QuarantineSection";
import { SystemOverview } from "./components/SystemOverview";
import { useMacCleaner } from "./hooks/useMacCleaner";
import { StorageSection } from "./StorageSection";

export function App() {
  const cleaner = useMacCleaner();

  return (
    <div className="app-shell">
      <AppHeader
        launchAtLogin={cleaner.launchAtLogin}
        loginItemAvailable={cleaner.loginItemAvailable}
        onLaunchAtLoginChange={cleaner.changeLaunchAtLogin}
      />

      <main>
        <SystemOverview snapshot={cleaner.snapshot} />

        <StorageSection
          snapshot={cleaner.snapshot}
          storage={cleaner.storage}
          analyzing={cleaner.storageAnalyzing}
          scanning={cleaner.phase === "scanning"}
          onAnalyze={cleaner.analyzeStorage}
          onReviewCleanup={cleaner.reviewCleanup}
          onOpenOwner={cleaner.openStorageOwner}
          onPruneUv={cleaner.pruneUvCache}
          onCleanSimulatorRecordings={cleaner.cleanSimulatorRecordings}
          toolOperation={cleaner.toolOperation}
          uvStatus={cleaner.uvStatus}
          simulatorStatus={cleaner.simulatorStatus}
        />

        <ProcessMonitor snapshot={cleaner.snapshot} />

        {cleaner.error && <div className="alert error" role="alert">{cleaner.error}</div>}
        <div className="sr-status" role="status" aria-live="polite">{cleaner.notice}</div>

        <CleanupWorkspace
          scan={cleaner.scan}
          selected={cleaner.selected}
          selectedBytes={cleaner.selectedBytes}
          phase={cleaner.phase}
          onScan={cleaner.runScan}
          onToggleCandidate={cleaner.toggleCandidate}
          onReviewPlan={cleaner.reviewPlan}
        />

        {cleaner.plan && <PlanReview plan={cleaner.plan} phase={cleaner.phase} onExecute={cleaner.executePlan} />}

        <QuarantineSection
          entries={cleaner.quarantine}
          phase={cleaner.phase}
          onAction={cleaner.runQuarantineAction}
        />

        <ActivitySection history={cleaner.history} toolHistory={cleaner.toolHistory} />
      </main>
    </div>
  );
}
