import { KeyExport } from "./KeyExport";
import { KeyImport } from "./KeyImport";

interface KeyBackupPanelProps {
  onImported?: (pubkey: string) => void;
}

export function KeyBackupPanel({ onImported }: KeyBackupPanelProps) {
  return (
    <div className="bg-ctp-mantle rounded-xl border border-ctp-overlay0 p-4 text-ctp-text space-y-4">
      <h3 className="text-sm font-semibold text-ctp-subtext1">
        🔐 Key Backup & Recovery
      </h3>
      <KeyExport />
      <hr className="border-ctp-surface1" />
      <KeyImport onImported={onImported} />
    </div>
  );
}
