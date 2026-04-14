Make a UI/UX change to Meridian following the design system.

Arguments: $ARGUMENTS
Expected format: description of the change needed

## Design System — Apply These Without Exception

### Colors
- Accent: `indigo-500` ONLY for primary actions, active states, selected rings, CTAs
- All other interactive elements: `zinc` scale
- Hover backgrounds: `zinc-50` (light) / `zinc-800/60` (dark) — NOT transparent overlays like `zinc-50/60` (those look disabled)
- Borders at rest: `zinc-100` (light) / `zinc-800/50` (dark)
- Borders on hover: `zinc-200` (light) / `zinc-700` (dark)

### Typography Hierarchy
- Title / primary label: `text-[13.5px] font-semibold text-zinc-900 dark:text-zinc-100`
- Body / description: `text-[12px] text-zinc-500 dark:text-zinc-400 line-clamp-2`
- Metadata: `text-[11px] text-zinc-400 dark:text-zinc-500`
- Section labels: `text-[11px] font-semibold uppercase tracking-wider text-zinc-400`
- Letter spacing: `tracking-[-0.01em]` on Inter for crisp rendering

### Components

**Cards (TaskCard)**
- `border-l-[3px]` priority color + `border border-zinc-100 dark:border-zinc-800/50`
- Hover: `hover:bg-zinc-50 dark:hover:bg-zinc-800/60 hover:border-zinc-200`
- Selected: `bg-indigo-50/60 dark:bg-indigo-950/30 border-zinc-200 dark:border-indigo-900/60`
- Custom checkbox: `sr-only` native input + styled div with `Check` lucide icon

**Tabs (MainCanvas tab bar)**
- Underline style: `border-b-2 border-indigo-500` active, `border-transparent` inactive
- No pill/background fill on tabs — underline only

**Filter controls**
- Active filter: replace `<select>` with `<ActiveChip>` (colored pill + inline `×`)
- Priority colors: red for critical, orange for high, indigo default
- Popovers: `absolute top-full left-0 mt-1 z-50 shadow-xl animate-fade-in`

**Buttons**
- Primary CTA: `bg-indigo-500 hover:bg-indigo-600 active:bg-indigo-700 text-white`
- Secondary: `text-zinc-600 hover:bg-zinc-100 dark:hover:bg-zinc-800`
- Destructive: `hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20`

**Metadata strips** (TaskCard bottom row)
- Dot-separated: `<span className="mx-1.5 text-zinc-300 dark:text-zinc-700 text-[10px]">·</span>`
- Not flex gap — use the dot separator pattern

**Sidebar**
- Global nav items: `NavItem` component pattern (full-width button, active = `bg-zinc-100 dark:bg-zinc-800`)
- Utility strip: icon-only, `flex-1 justify-center`, 15px icons
- Project dots: 7px circle with `shadow-[0_0_0_1.5px_rgba(0,0,0,0.08)]`

### Animations
- `animate-fade-in`: dropdowns, popovers, expanded sections
- `animate-slide-down`: bulk actions bar entrance
- `transition-colors duration-120`: all interactive state changes
- No JS animation libraries needed — CSS only via globals.css

### Spacing
- Card padding: `px-3 py-2.5`
- Section padding: `px-4`
- Filter bar: `px-4 py-2`
- Sidebar sections: `px-2`

## Checklist Before Finishing

- [ ] Hover state uses solid `zinc-50 / zinc-800` NOT transparent values
- [ ] Indigo accent only appears on truly important/interactive elements
- [ ] Both light AND dark mode look correct
- [ ] Task title is clearly more prominent than description and metadata
- [ ] No layout reflow or overflow issues (check `min-w-0`, `truncate`, `flex-shrink-0`)
- [ ] `npx tsc --noEmit` passes
- [ ] No new console errors

## After the change

- Run `npm run test:e2e` to confirm no regressions
- If you changed a component pattern used in tests, update the relevant spec
- Update `CLAUDE.md` "Design System" section if you introduced a new reusable pattern
