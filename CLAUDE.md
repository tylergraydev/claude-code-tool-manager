## Design Context

### Users
Developers of all experience levels who manage Claude Code and other AI coding assistant configurations. They want a clear, approachable interface that makes complex configuration feel simple. The primary job: see everything at a glance, toggle tools on/off, and keep configs in sync — without hand-editing JSON files.

### Brand Personality
**Friendly, capable, modern.** Think Notion or Figma — approachable but clearly powerful. Never intimidating, never dumbed down.

### Aesthetic Direction
- **Visual tone:** Clean, information-rich, native-feeling. Inspired by VS Code's settings UI — dense but organized, functional but not ugly.
- **Theme:** Dark-first (class-based toggle), blue primary palette (Tailwind blue scale), gray neutrals, system-ui font stack.
- **Anti-references:** Must never feel like Electron bloat — no sluggish animations, no web-page-pretending-to-be-native energy. Should feel fast and lightweight like a native app.
- **Icons:** Lucide icon set throughout.
- **Layout:** Sidebar navigation + main content area. Card-based UI with rounded corners (`rounded-xl`), subtle shadows, custom scrollbars.

### Design Principles

1. **Clarity over cleverness** — Every element should be immediately understandable. Prefer familiar patterns (toggles, cards, lists) over novel interactions. Labels should be specific and jargon-free where possible.

2. **Density with breathing room** — Show lots of information without feeling cramped. Use consistent spacing, clear section boundaries, and visual hierarchy to keep dense layouts scannable.

3. **Native performance feel** — Interactions should feel instant. Avoid heavy animations. Transitions should be subtle and functional (hover states, focus rings), not decorative. The app should feel as fast as the terminal it configures.

4. **Dark mode first, light mode supported** — Design in dark mode, ensure light mode works well. The existing gray-900 background with gray-800 cards and blue accents is the foundation.

5. **Accessible by default** — Good contrast ratios, keyboard navigable, semantic HTML. Focus indicators on all interactive elements. No information conveyed by color alone.
