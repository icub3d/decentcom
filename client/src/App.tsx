import { useEffect, useState } from "react";

import { ServerConnect } from "./components/connection/ServerConnect";
import { JoinByInvite } from "./components/invites/JoinByInvite";
import { AppShell } from "./components/layout/AppShell";
import { TitleBar } from "./components/layout/TitleBar";
import { useInviteLink } from "./hooks/useInviteLink";
import { useIdentity } from "./hooks/useIdentity";
import { authenticateServer } from "./services/auth";
import { getServerInfo } from "./api/server";
import { useAppStore, switchAppStoreAccount } from "./stores/appStore";
import { useIdentityStore } from "./stores/identityStore";
import { useInvitesStore } from "./stores/invites";
import { useServerStore } from "./stores/serverStore";
import { Setup } from "./pages/Setup";
import "./App.css";

function App() {
  const {
    hasIdentity,
    loading,
    error,
    generateIdentity,
    importIdentity,
    refresh,
  } = useIdentity();
  const { currentServerId, addServer, initTheme } = useAppStore();
  const { connect, disconnect, status } = useServerStore();
  const joinInvite = useInvitesStore((state) => state.joinInvite);
  const { invite, clearInviteLink } = useInviteLink();
  const [connectLoading, setConnectLoading] = useState(false);
  const [connectError, setConnectError] = useState<string | null>(null);
  const [addingAccount, setAddingAccount] = useState(false);

  const activeAccount = useIdentityStore((s) => s.activeAccount);
  const setActiveAccount = useIdentityStore((s) => s.setActiveAccount);

  useEffect(() => {
    initTheme();
  }, [initTheme]);

  // Clear connection errors once the gateway is connected
  useEffect(() => {
    if (status === "connected") {
      setConnectError(null);
    }
  }, [status]);

  useEffect(() => {
    if (
      hasIdentity &&
      !loading &&
      !addingAccount &&
      currentServerId &&
      status === "disconnected" &&
      !connectLoading &&
      useAppStore.persist.hasHydrated()
    ) {
      handleConnect(currentServerId);
    }
  }, [currentServerId, hasIdentity, loading, status, addingAccount]);

  async function handleConnect(address: string) {
    setConnectLoading(true);
    setConnectError(null);
    try {
      const info = await getServerInfo(address);
      await connect(address);
      addServer(address, info.name);
    } catch (err) {
      setConnectError(err instanceof Error ? err.message : String(err));
    } finally {
      setConnectLoading(false);
    }
  }

  async function handleJoinByInvite(address: string, inviteCode: string) {
    setConnectLoading(true);
    setConnectError(null);
    try {
      const info = await getServerInfo(address);
      const session = await authenticateServer(address);
      await joinInvite(address, session.token, inviteCode);
      await connect(address);
      addServer(address, info.name);
      clearInviteLink();
    } catch (err) {
      setConnectError(err instanceof Error ? err.message : String(err));
    } finally {
      setConnectLoading(false);
    }
  }

  async function handleSwitchAccount(pubkey: string) {
    // Disconnect current session, switch backend active key, reload store.
    disconnect();
    await setActiveAccount(pubkey);
    switchAppStoreAccount(pubkey);
    await refresh();
  }

  function handleAddAccount() {
    setAddingAccount(true);
  }

  async function handleAddAccountComplete() {
    setAddingAccount(false);
    await refresh();
    // The newly generated/imported account is now active.
    if (activeAccount) {
      switchAppStoreAccount(activeAccount);
    }
  }

  let content;
  if (loading && hasIdentity === null) {
    content = (
      <div className="flex-1 flex items-center justify-center bg-ctp-base text-ctp-text">
        <div className="animate-pulse">Loading identity...</div>
      </div>
    );
  } else if (!hasIdentity || addingAccount) {
    content = (
      <main className="flex-1 flex items-center justify-center bg-ctp-base p-4 text-ctp-text">
        <Setup
          onGenerate={generateIdentity}
          onImport={importIdentity}
          onComplete={() => void handleAddAccountComplete()}
          onCancel={addingAccount ? () => setAddingAccount(false) : undefined}
        />
      </main>
    );
  } else if (!currentServerId) {
    if (invite) {
      content = (
        <div className="flex-1">
          <JoinByInvite
            serverAddress={invite.serverAddress}
            inviteCode={invite.inviteCode}
            loading={connectLoading || status === "connecting"}
            onJoin={handleJoinByInvite}
            onBack={clearInviteLink}
          />
        </div>
      );
    } else {
      content = (
        <div className="flex-1">
          <ServerConnect
            loading={connectLoading || status === "connecting"}
            error={connectError}
            onConnect={handleConnect}
          />
        </div>
      );
    }
  } else {
    content = (
      <>
        {(error || connectError) && (
          <div className="fixed right-4 top-12 z-20 rounded-lg border border-ctp-red bg-ctp-red/20 px-4 py-2 text-sm text-ctp-red">
            {error || connectError}
          </div>
        )}
        <AppShell
          onSwitchAccount={handleSwitchAccount}
          onAddAccount={handleAddAccount}
        />
      </>
    );
  }

  return (
    <div className="h-screen flex flex-col overflow-hidden">
      <TitleBar />
      {content}
    </div>
  );
}

export default App;
