import { openUrl } from "@tauri-apps/plugin-opener";
import { Modal } from "../ui/Modal";

interface AboutProps {
  onClose: () => void;
}

const LINKS = {
  docs: "https://github.com/icub3d/decentcom/wiki",
  source: "https://github.com/icub3d/decentcom",
  download: "https://decentcom.app",
};

function ExternalLink({ href, children }: { href: string; children: React.ReactNode }) {
  const handleClick = (e: React.MouseEvent) => {
    e.preventDefault();
    void openUrl(href);
  };
  return (
    <a
      href={href}
      onClick={handleClick}
      className="text-ctp-blue hover:text-ctp-sapphire underline transition-colors cursor-pointer"
    >
      {children}
    </a>
  );
}

export function About({ onClose }: AboutProps) {
  return (
    <Modal onClose={onClose}>
      <div className="w-full max-w-lg p-8 space-y-6 text-ctp-text">
        <div className="flex items-center justify-between">
          <h2 className="text-2xl font-bold text-ctp-text">About decentcom</h2>
          <button
            onClick={onClose}
            className="text-ctp-subtext1 hover:text-ctp-text transition-colors text-lg leading-none"
            aria-label="Close"
          >
            ✕
          </button>
        </div>

        <p className="text-ctp-subtext1 leading-relaxed">
          decentcom is open-source community communication software you can self-host.
          Your identity is a cryptographic key pair — no password, no email, no central account.
        </p>

        <div>
          <h3 className="text-sm font-semibold text-ctp-subtext0 uppercase tracking-wider mb-3">
            Key Strengths
          </h3>
          <ul className="space-y-2 text-sm text-ctp-text">
            <li className="flex gap-2">
              <span className="text-ctp-green mt-0.5">✓</span>
              <span><strong className="text-ctp-green">Identity ownership</strong> — Your account is a key pair you generate. No email, no phone number, no password required.</span>
            </li>
            <li className="flex gap-2">
              <span className="text-ctp-green mt-0.5">✓</span>
              <span><strong className="text-ctp-green">No passwords</strong> — Authentication is cryptographic. There is nothing for a server to steal and nothing for a breach to expose.</span>
            </li>
            <li className="flex gap-2">
              <span className="text-ctp-green mt-0.5">✓</span>
              <span><strong className="text-ctp-green">Self-hostable</strong> — Any community can run its own server. No subscription, no third-party Terms of Service, no deplatforming risk.</span>
            </li>
            <li className="flex gap-2">
              <span className="text-ctp-green mt-0.5">✓</span>
              <span><strong className="text-ctp-green">Cryptographically secure</strong> — Every authentication action is signed. Servers verify identity without storing secrets.</span>
            </li>
            <li className="flex gap-2">
              <span className="text-ctp-green mt-0.5">✓</span>
              <span><strong className="text-ctp-green">Open source</strong> — MIT-licensed. Inspect it, fork it, self-host it. The core will always be free.</span>
            </li>
          </ul>
        </div>

        <div>
          <h3 className="text-sm font-semibold text-ctp-subtext0 uppercase tracking-wider mb-3">
            Why decentcom?
          </h3>
          <div className="text-sm text-ctp-subtext1 space-y-2 leading-relaxed">
            <p>
              Most platforms store a password or credential on their servers — which means
              every breach puts your account at risk. With decentcom, there is nothing to steal.
            </p>
            <p>
              With most services, your account exists at the platform's discretion.
              Centralized services accumulate metadata about who you talk to and when.
              On centralized platforms, servers — and your account — can be removed without recourse.
            </p>
            <p>
              decentcom puts control where it belongs: with you and your community.
              Self-hosting requires effort and being early-stage means fewer integrations,
              but the tradeoff is a platform that works for you, not for the platform.
            </p>
          </div>
        </div>

        <div className="pt-2 border-t border-ctp-overlay0">
          <h3 className="text-sm font-semibold text-ctp-subtext0 uppercase tracking-wider mb-3">
            Links
          </h3>
          <div className="flex flex-wrap gap-4 text-sm">
            <ExternalLink href={LINKS.docs}>Documentation</ExternalLink>
            <ExternalLink href={LINKS.source}>Source Repository</ExternalLink>
            <ExternalLink href={LINKS.download}>Download Page</ExternalLink>
          </div>
        </div>

        <div className="pt-2 flex justify-end">
          <button
            onClick={onClose}
            className="px-6 py-2 bg-ctp-surface0 hover:bg-ctp-surface1 text-ctp-text rounded-lg font-semibold transition-colors cursor-pointer"
          >
            Close
          </button>
        </div>
      </div>
    </Modal>
  );
}
