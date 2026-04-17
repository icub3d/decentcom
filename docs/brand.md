# decentcom Brand Voice

This document is the source of truth for user-facing copy: the in-app About component, the public landing site, release notes, and any other surface a non-developer reads. Internal developer docs (CLAUDE.md, design docs, code comments) are a different audience and are intentionally out of scope.

---

## Tagline candidates

Pick one when the landing site ships. Until then, use whichever fits the context.

1. **"Your identity. Your server. Your community."** — short, possessive, reinforces decentralization.
2. **"Communication that works for you, not for the platform."** — contrast-forward without naming anyone.
3. **"Decent communication, decentralized."** — plays on the name; good for audiences already familiar with decentralization.

---

## Value props (one-liners)

Use these as the basis for feature cards, bullet lists, and elevator-pitch copy.

| Prop | One-liner |
|---|---|
| **Identity ownership** | Your account is a key pair you generate. No email, no phone number, no password required. |
| **No passwords** | Authentication is cryptographic. There is nothing for a server to steal and nothing for a breach to expose. |
| **Self-hostable** | Any community can run its own server. No subscription, no third-party Terms of Service, no deplatforming risk. |
| **Cryptographically secure** | Every authentication action is signed. Servers verify identity without storing secrets. |
| **Server operator control** | Each server sets its own policies — open, invite-only, or allowlist — without asking permission from anyone. |
| **Open source** | MIT-licensed. Inspect it, fork it, self-host it. The core will always be free. |

---

## Tone of voice

- **Plain.** Say what you mean in the fewest words. No jargon, no buzzwords.
- **Confident.** State facts. Don't hedge with "may", "might", "could potentially".
- **Not aggressive.** Don't mock other products. Let the design speak for itself.
- **Technical but accessible.** Our audience includes developers and community managers. Prefer concrete nouns ("your public key") over vague abstractions ("your digital presence").

---

## Comparative copy rules

decentcom competes on design, not on attacks. When contrasting with other products:

1. **Lead with our strengths.** Don't open a sentence with a competitor's weakness.
2. **Use generic terms.** Acceptable: "other platforms", "centralized services", "most chat services", "traditional platforms". Never: Discord, Slack, Microsoft Teams, Matrix, IRC in user-facing copy.
3. **Frame around user benefit.** "With decentcom, your account cannot be deleted by a third party." Not: "Unlike [platform], we don't delete accounts."
4. **Acknowledge the tradeoffs honestly.** Self-hosting requires effort. Being early-stage means fewer integrations. Don't oversell.

### Approved contrast phrases

| Situation | Use |
|---|---|
| Mentioning that most services store passwords | "Most platforms store a password or credential on their servers." |
| Mentioning that most services can delete accounts | "With most services, your account exists at the platform's discretion." |
| Mentioning centralized data collection | "Centralized services accumulate metadata about who you talk to and when." |
| Mentioning deplatforming | "On centralized platforms, servers — and your account — can be removed without recourse." |

---

## Approved short descriptions

Use these verbatim where a short description is required (meta tags, app store, social previews).

- **One sentence:** "Open-source, self-hostable community communication where your identity is a key pair you own."
- **Two sentences:** "decentcom is open-source community communication software you can self-host. Your identity is a cryptographic key pair — no password, no email, no central account."
- **Tweet-length:** "Self-hosted community chat. Your identity is yours. No passwords, no central control, no platform risk."
