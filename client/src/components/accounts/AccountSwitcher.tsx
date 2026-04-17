import { useState } from "react";
import { useIdentityStore, type AccountInfo } from "../../stores/identityStore";
import { useIdentity } from "../../hooks/useIdentity";
import { useAccountManager } from "../../hooks/useAccountManager";
import { Modal } from "../ui/Modal";
import { Setup } from "../../pages/Setup";
import { KeyExport } from "../backup/KeyExport";

interface AccountSwitcherProps {
  onSwitchAccount: (pubkey: string) => void | Promise<void>;
}

function shortPubkey(pubkey: string): string {
  if (pubkey.length <= 12) return pubkey;
  return `${pubkey.slice(0, 6)}…${pubkey.slice(-4)}`;
}

function displayName(account: AccountInfo): string {
  if (account.label) {
    return `${account.label} (${shortPubkey(account.pubkey)})`;
  }
  return shortPubkey(account.pubkey);
}

export function AccountSwitcher({ onSwitchAccount }: AccountSwitcherProps) {
  const { accounts, activeAccount, deleteAccount, renameAccount } = useIdentityStore();
  const { generateIdentity, importIdentity, refresh } = useIdentity();
  const { handleSetupComplete } = useAccountManager();
  const [showSetup, setShowSetup] = useState(false);
  const [preSetupAccount, setPreSetupAccount] = useState<string | null>(null);
  const [showBackup, setShowBackup] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null);
  const [editing, setEditing] = useState<string | null>(null);
  const [editLabel, setEditLabel] = useState("");

  if (accounts.length === 0) return null;

  const handleDelete = async (pubkey: string) => {
    await deleteAccount(pubkey);
    setConfirmDelete(null);
  };

  const startRename = (account: AccountInfo) => {
    setEditing(account.pubkey);
    setEditLabel(account.label ?? "");
  };

  const commitRename = async () => {
    if (editing) {
      await renameAccount(editing, editLabel);
      setEditing(null);
    }
  };

  const onSetupComplete = async () => {
    setShowSetup(false);
    await handleSetupComplete(refresh, onSwitchAccount, preSetupAccount);
  };

  return (
    <>
      {showSetup && (
        <Modal onClose={() => setShowSetup(false)}>
          <Setup
            onGenerate={generateIdentity}
            onImport={importIdentity}
            onComplete={() => void onSetupComplete()}
            onCancel={() => setShowSetup(false)}
          />
        </Modal>
      )}
      {showBackup && (
        <Modal onClose={() => setShowBackup(false)}>
          <div className="p-4 w-80">
            <KeyExport />
          </div>
        </Modal>
      )}
      <div className="bg-ctp-mantle rounded-xl border border-ctp-overlay0 p-4 text-ctp-text">
      <h3 className="text-sm font-semibold text-ctp-subtext1 mb-2">Accounts</h3>
      <div className="space-y-1">
        {accounts.map((account: AccountInfo) => (
          <div
            key={account.pubkey}
            className={`flex items-center gap-2 rounded-lg px-3 py-2 text-sm transition ${
              account.pubkey === activeAccount
                ? "bg-ctp-blue/20 text-ctp-blue"
                : "hover:bg-ctp-surface0 text-ctp-subtext1 cursor-pointer"
            }`}
          >
            {editing === account.pubkey ? (
              <form
                className="flex-1 flex gap-1"
                onSubmit={(e) => { e.preventDefault(); void commitRename(); }}
              >
                <input
                  autoFocus
                  value={editLabel}
                  onChange={(e) => setEditLabel(e.target.value)}
                  onBlur={() => void commitRename()}
                  placeholder="Account name"
                  className="flex-1 bg-ctp-surface0 text-ctp-text rounded px-2 py-0.5 text-xs outline-none"
                />
              </form>
            ) : (
              <button
                onClick={() => {
                  if (account.pubkey !== activeAccount) {
                    onSwitchAccount(account.pubkey);
                  }
                }}
                onDoubleClick={() => startRename(account)}
                className="flex-1 text-left text-xs truncate"
                title={`${account.pubkey}\nDouble-click to rename`}
              >
                {account.pubkey === activeAccount && (
                  <span className="mr-1 text-ctp-green">●</span>
                )}
                {displayName(account)}
              </button>
            )}
            {confirmDelete === account.pubkey ? (
              <div className="flex gap-1">
                <button
                  onClick={() => void handleDelete(account.pubkey)}
                  className="text-[10px] px-1.5 py-0.5 bg-ctp-red text-ctp-crust rounded font-bold"
                >
                  Yes
                </button>
                <button
                  onClick={() => setConfirmDelete(null)}
                  className="text-[10px] px-1.5 py-0.5 bg-ctp-surface1 rounded"
                >
                  No
                </button>
              </div>
            ) : (
              <div className="flex items-center gap-1 shrink-0">
                {account.pubkey === activeAccount && (
                  <button
                    onClick={() => setShowBackup(true)}
                    className="text-ctp-overlay1 hover:text-ctp-yellow text-xs"
                    title="Export key backup"
                  >
                    💾
                  </button>
                )}
                <button
                  onClick={() => setConfirmDelete(account.pubkey)}
                  className="text-ctp-overlay1 hover:text-ctp-red text-xs"
                  title="Delete account"
                >
                  🗑
                </button>
              </div>
            )}
          </div>
        ))}
      </div>
      <button
        onClick={() => { setPreSetupAccount(activeAccount); setShowSetup(true); }}
        className="mt-2 w-full py-2 text-sm text-ctp-subtext1 hover:text-ctp-blue hover:bg-ctp-surface0 rounded-lg transition border border-dashed border-ctp-overlay0"
      >
        + Add Account
      </button>
    </div>
    </>
  );
}
