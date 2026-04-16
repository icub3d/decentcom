import { useCallback, useEffect, useState } from "react";

import { ServerConnect } from "./components/connection/ServerConnect";
import { JoinByInvite } from "./components/invites/JoinByInvite";
import { AppShell } from "./components/layout/AppShell";
import { TitleBar } from "./components/layout/TitleBar";
import { useInviteLink } from "./hooks/useInviteLink";
import { useIdentity } from "./hooks/useIdentity";
import { authenticateServer } from "./services/auth";
import { getServerInfo } from "./api/server";
import { useAppStore, switchAppStoreAccount, initAppStoreForAccount } from "./stores/appStore";
import { useIdentityStore } from "./stores/identityStore";
import { useInvitesStore } from "./stores/invites";
import { useServerStore } from "./stores/serverStore";
import { Setup } from "./pages/Setup";
import "./App.css";

function App() {
  const {
    hasIdentity,
    publicKey,
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

  const activeAccount = useIdentityStore((s) => s.activeAccount);
  const setActiveAccount = useIdentityStore((s) => s.setActiveAccount);

  useEffect(() => {
    initTheme();
  }, [initTheme]);

  // Once identity is resolved, make sure the app store is namespaced
  // to the active account so persist reads/writes go to the right key.
  useEffect(() => {
    if (hasIdentity && publicKey && !loading) {
      initAppStoreForAccount(publicKey);
    }
  }, [hasIdentity, publicKey, loading]);

  // Clear connection errors once the gateway is connected
  useEffect(() => {
    if (status === "connected") {
      setConnectError(null);
    }
  }, [status]);

  const handleConnect = useCallback(async (address: string) => {
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
  }, [connect, addServer]);

  useEffect(() => {
    if (
      hasIdentity &&
      !loading &&
      currentServerId &&
      status === "disconnected" &&
      !connectLoading &&
      useAppStore.persist.hasHydrated()
    ) {
      handleConnect(currentServerId);
    }
  }, [currentServerId, hasIdentity, loading, status, connectLoading, handleConnect]);

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
    const oldPubkey = activeAccount;
    await setActiveAccount(pubkey);
    switchAppStoreAccount(oldPubkey, pubkey);
    await refresh();
  }

  async function handleFirstRunComplete() {
    await refresh();
    const newActive = useIdentityStore.getState().activeAccount;
    if (newActive) {
      switchAppStoreAccount(activeAccount, newActive);
    }
  }

  let content;
  if (loading && hasIdentity === null) {
    content = (
      <div className="flex-1 flex items-center justify-center bg-ctp-base text-ctp-text">
        <div className="animate-pulse">Loading identity...</div>
      </div>
    );
  } else if (!hasIdentity) {
    content = (
      <main className="flex-1 flex items-center justify-center bg-ctp-base p-4 text-ctp-text">
        <Setup
          onGenerate={generateIdentity}
          onImport={importIdentity}
          onComplete={() => void handleFirstRunComplete()}
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
