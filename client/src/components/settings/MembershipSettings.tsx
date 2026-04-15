interface MembershipSettingsProps {
  mode: string;
}

export function MembershipSettings({ mode }: MembershipSettingsProps) {
  return (
    <section className="rounded-xl border border-ctp-overlay0 bg-ctp-mantle/90 p-3">
      <h3 className="mb-2 text-sm font-bold text-ctp-text">Membership mode</h3>
      <p className="text-xs text-ctp-subtext0">
        Current mode: <span className="font-semibold text-ctp-text">{mode}</span>
      </p>
      <p className="mt-2 text-xs text-ctp-subtext0">
        Runtime mode editing is not yet available in the API. Change this in server config and restart.
      </p>
    </section>
  );
}
