# Murmur — Design Language (contract)

The product must read as if built by a top-tier product design team — think the
craft bar of **Linear, Things 3, Arc, Stripe, iA, Signal Desktop done right** —
**not** a generated template. Dark theme is permanent. This doc is binding for
every client (web, desktop, mobile); UI PRs are reviewed against it.

---

## 0. The anti-"AI-generated" rules (hard NOs)
These are the tells that scream template/generated. Banned:
- ❌ The default **indigo→violet gradient hero**. No blue-purple SaaS gradient.
- ❌ **Everything rounded** (pill buttons + 16px cards + round avatars all at once).
  Pick a deliberate radius system and hold it.
- ❌ **Glassmorphism** blur-card clichés, neon glows, drop-shadow soup.
- ❌ **Emoji as UI** (🚀 in headings, ✅ in feature lists).
- ❌ **Center-everything** landing layouts with three equal feature cards.
- ❌ Generic **Inter-for-everything** with no typographic intent.
- ❌ Stocky gradients on icons; default Heroicons used without a consistent stroke.
- ❌ Fake density — huge padding everywhere so the app feels empty and "marketing."
- ❌ Lorem-ipsum-shaped microcopy. Copy is specific, terse, human.

## 1. Design principles
1. **Quiet confidence.** It's a security product. Restraint signals trust. One
   accent, used sparingly. Color earns its place; most of the UI is neutral.
2. **Density with air.** Power-user chat = information-dense but never cramped.
   Tight, consistent rhythm beats generous-everywhere padding.
3. **Optical precision.** Align to the optical edge, not the bounding box. Icons
   and text baselines line up. Nothing is "close enough."
4. **Motion is feedback, not decoration.** Fast, physical, purposeful. No
   gratuitous fades or parallax.
5. **Earned hierarchy.** Type scale + weight + spacing do the work; borders and
   boxes are a last resort.

## 2. Color — dark theme tokens (the one source of truth)
Near-black, slightly **cool neutral** (not pure #000 — pure black crushes depth).
Elevation = small lightness steps, never heavy shadows. WCAG AA minimum.

```
/* Surfaces (low → high elevation) */
--bg-base:      #0B0C0E;  /* app background */
--bg-surface:   #131519;  /* panels, sidebar */
--bg-elevated:  #1A1D22;  /* cards, menus, modals */
--bg-overlay:   #20242B;  /* hover/active fills */

/* Borders / hairlines (use sparingly) */
--border-subtle: #23272E;
--border-strong: #313742;

/* Text */
--text-primary:   #E8EAED;  /* not pure white — reduces glare */
--text-secondary: #A1A8B3;
--text-tertiary:  #6B7280;
--text-disabled:  #464C56;

/* Accent — ONE confident, non-cliché hue: cold signal-teal */
--accent:        #3DD4B8;  /* primary action, focus ring, active nav */
--accent-hover:  #5BE3CC;
--accent-press:  #2BB89E;
--accent-quiet:  rgba(61,212,184,0.12); /* tinted selection/active bg */

/* Semantic */
--success: #46C77E;
--warning: #E0A33E;
--danger:  #E5604D;   /* destructive / security alerts */
--info:    #5AA9E6;
```
Rules: accent appears on **primary action, focus, and active state only**.
Encryption/verified states use a calm success-teal family, never alarm-red unless
something is actually wrong (key change, unverified device).

## 3. Typography
- **UI / body:** a precise grotesque — **Söhne** or **Inter Display** (tuned), not
  default Inter. Tight tracking on headings, normal on body.
- **Monospace:** **Berkeley Mono** / **Commit Mono** for key fingerprints, IDs,
  code — fingerprints MUST be mono and grouped (e.g. `A1B2 C3D4 …`).
- **Scale (1.25 minor-third, optical):** 12 / 13 / 14(base) / 16 / 20 / 26 / 33.
- Body line-height 1.5; dense lists 1.4; headings 1.15–1.2.
- Weights: 400 body, 500 UI labels, 600 headings. Avoid 700+ except rare display.

## 4. Space, grid, radius
- **4px base unit.** Spacing scale: 2,4,8,12,16,24,32,48,64.
- **Radius system (deliberate, not uniform):** 6px controls (buttons/inputs),
  10px cards/menus, 4px tags/badges, full only for avatars/presence dots.
- App shell: fixed left rail (channels) + content + optional right context panel.
  Real product chrome, not a centered marketing column.

## 5. Components — the bar
- **Buttons:** primary = accent fill, text near-bg for contrast; secondary =
  surface + subtle border; ghost = text only. 32–36px height. Crisp 1px focus ring
  in accent, offset 2px.
- **Inputs:** surface fill, subtle border, accent focus ring. No heavy inner shadow.
- **Message rows:** compact, hover reveals actions, sender grouping, mono for
  code/fingerprints. Timestamps in `--text-tertiary`.
- **Security surfaces:** device list, key-transparency self-check, "verified"
  badges — calm, legible, mono fingerprints, explicit human copy.
- **Empty states:** specific and useful, never a big emoji + one sentence.

## 6. Iconography & motion
- One icon set, consistent **1.5px stroke**, optically sized to text. (Lucide,
  restyled — not default Heroicons.)
- Motion: 120–200ms, ease-out for enters, ease-in for exits; spring only for
  direct-manipulation (drag, reorder). No looping ambient animation.

## 7. Accessibility (non-negotiable)
- Text contrast ≥ 4.5:1 (AA); large text ≥ 3:1. Verified against tokens above.
- Full keyboard nav; visible focus everywhere; respects `prefers-reduced-motion`.
- Don't encode meaning in color alone (verified = icon + label, not just teal).

## 8. Implementation
- Tokens live as CSS variables (web/desktop) + a shared token JSON exported to
  React Native (mobile) so all clients share ONE palette/scale.
- Build phase uses the **`ui-ux-pro-max`** skill to implement components against
  this contract; every UI PR is checked back against §0 and §2.
- Reference the named apps in §intro as the quality bar during review.

> Status: contract locked 2026-05-29. Implemented in Phase 4 (web + mobile clients).
> Accent hue (`#3DD4B8` cold teal) is a proposal — confirm or swap before Phase 4;
> everything else (anti-slop rules, scale, spacing, a11y) is binding.
