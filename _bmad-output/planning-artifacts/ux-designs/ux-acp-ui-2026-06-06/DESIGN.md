---
title: "ACP-UI Design System"
status: "final"
created: "2026-06-06"
updated: "2026-06-06"
form_factor: "web-desktop"
ui_system: "custom-css-variables"
---

# DESIGN.md — ACP-UI Visual Identity

## Brand & Style

ACP-UI uses a modern, developer-friendly aesthetic with indigo as the primary brand color. The design system is built on CSS custom properties for seamless dark mode support and consistent theming across all surfaces.

## Colors

### Primary
- **--primary**: `#6366f1` (Indigo 500) — main actions, primary buttons, active states
- **--primary-hover**: `#4f46e5` (Indigo 600) — hover states
- **--primary-light**: `#818cf8` (Indigo 400) — light backgrounds, badges
- **--primary-dark**: `#3730a3` (Indigo 800) — dark mode primary

### Accent
- **--accent**: `#22d3ee` (Cyan 400) — secondary actions, highlights
- **--accent-hover**: `#06b6d4` (Cyan 500)

### Semantic
- **--success**: `#10b981` — running states, completion, positive feedback
- **--error**: `#ef4444` — errors, disconnected states, destructive actions
- **--warning**: `#f59e0b` — warnings, pending states, caution indicators

### Surface
- **--bg-base**: `#f8fafc` / `#0f172a` (dark)
- **--bg-surface**: `#ffffff` / `#1e293b` (dark)
- **--bg-hover**: `#f1f5f9` / `#334155` (dark)
- **--border-color**: `#e2e8f0` / `#334155` (dark)

### Text
- **--text-primary**: `#0f172a` / `#f1f5f9` (dark)
- **--text-secondary**: `#475569` / `#e2e8f0` (dark)
- **--text-muted**: `#64748b` / `#cbd5e1` (dark)

## Typography

- **Font family**: `'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif`
- **Monospace**: `'JetBrains Mono', 'Fira Code', monospace`
- **Scale**: 12px → 14px → 16px → 18px → 20px → 24px → 30px

## Rounded

- **sm**: 4px (badges, small buttons)
- **md**: 8px (inputs, buttons, small cards)
- **lg**: 12px (main cards, modals)
- **xl**: 16px (large panels)
- **full**: 9999px (pills, avatars, toggles)

## Spacing

- **space-1**: 4px
- **space-2**: 8px
- **space-3**: 12px
- **space-4**: 16px
- **space-5**: 20px
- **space-6**: 24px
- **space-8**: 32px

## Elevation & Depth

- **shadow-sm**: `0 1px 2px 0 rgba(0, 0, 0, 0.05)`
- **shadow-md**: `0 4px 6px -1px rgba(0, 0, 0, 0.1)`
- **shadow-lg**: `0 10px 15px -3px rgba(0, 0, 0, 0.1)`
- **shadow-xl**: `0 20px 25px -5px rgba(0, 0, 0, 0.1)`

## Shapes

Cards use rounded-lg (12px) with subtle borders and shadow-sm. Buttons use rounded-md (8px). Pills and badges use rounded-full.

## Components

### Buttons
- Primary: `--primary` bg, `--text-inverse` text
- Secondary: `--bg-subtle` bg, `--text-secondary` text
- Success: `--success` bg (connect agents, save)
- Warning: `--warning` bg (disconnect)
- Danger: `--error` bg (delete)
- Active state: `transform: scale(0.98)` for tactile feedback

### Cards
- Hover: `translateY(-2px)` with `shadow-md`
- Border: `1px solid --border-color`
- Padding: `20px` / `space-5`

### Inputs
- Focus ring: `0 0 0 3px rgba(99, 102, 241, 0.1)`
- Border transitions to `--primary` on focus

### Status Badges
- Running: green bg + green text
- Stopped: gray bg + gray text
- Pulse animation for running state

### Skeleton Loaders
- Shimmer animation (1.5s infinite loop)
- Variants: text, title, avatar, card
- Background gradient between `--bg-subtle` and `--bg-muted`

### Toast Notifications
- Positioned: fixed top-right, z-index 9999
- Slide-in animation from right
- Color-coded left border (success/error/warning/info)

## Do's and Don'ts

### Do
- Use CSS variables for all colors — enables dark mode
- Use animation utility classes for consistent motion
- Use semantic color tokens (success/error/warning)
- Keep button text white on colored backgrounds

### Don't
- Hardcode hex colors — breaks theming
- Use emoji as UI icons — looks unprofessional
- Skip loading states — causes jarring blank flashes
- Ignore mobile breakpoints — 30% of users on tablets
