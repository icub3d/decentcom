import { useEffect, useState } from "react";

import { ServerConnect } from "./components/connection/ServerConnect";
import { JoinByInvite } from "./components/invites/JoinByInvite";
import { AppShell } from "./components/layout/AppShell";
import { useInviteLink } from "./hooks/useInviteLink";
import { useIdentity } from "./hooks/useIdentity";
import { authenticateServer } from "./services/auth";
import { useAppStore } from "./stores/appStore";
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
  const { connect, status } = useServerStore();
  const joinInvite = useInvitesStore((state) => state.joinInvite);
  const { invite, clearInviteLink } = useInviteLink();
  const [connectLoading, setConnectLoading] = useState(false);
  const [connectError, setConnectError] = useState<string | null>(null);

  useEffect(() => {
    initTheme();
  }, [initTheme]);

  async function handleConnect(address: string) {
    setConnectLoading(true);
    setConnectError(null);
    try {
      await connect(address);
      addServer(address);
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
      const session = await authenticateServer(address);
      await joinInvite(address, session.token, inviteCode);
      await connect(address);
      addServer(address);
      clearInviteLink();
    } catch (err) {
      setConnectError(err instanceof Error ? err.message : String(err));
    } finally {
      setConnectLoading(false);
    }
  }

  if (loading && hasIdentity === null) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-ctp-base text-ctp-text">
        <div className="animate-pulse">Loading identity...</div>
      </div>
    );
  }

  if (!hasIdentity) {
    return (
      <main className="min-h-screen flex items-center justify-center bg-ctp-base p-4 text-ctp-text">
        <Setup
          onGenerate={generateIdentity}
          onImport={importIdentity}
          onComplete={refresh}
        />
      </main>
    );
  }

  if (!currentServerId) {
    if (invite) {
      return (
        <JoinByInvite
          serverAddress={invite.serverAddress}
          inviteCode={invite.inviteCode}
          loading={connectLoading || status === "connecting"}
          onJoin={handleJoinByInvite}
          onBack={clearInviteLink}
        />
      );
    }

    return (
      <ServerConnect
        loading={connectLoading || status === "connecting"}
        error={connectError}
        onConnect={handleConnect}
      />
    );
  }

  return (
    <>
      {error && (
        <div className="fixed right-4 top-4 z-20 rounded-lg border border-ctp-red bg-ctp-red/20 px-4 py-2 text-sm text-ctp-red">
          {error}
        </div>
      )}
      <AppShell />
    </>
  );
}

export default App;
