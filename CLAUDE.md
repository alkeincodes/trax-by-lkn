# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**TraX by LKN** is a professional backing track management and playback application built with Tauri v2, Vue 3, and TypeScript. The application targets worship teams, live musicians, and performers who need reliable multi-track audio playback, practice tools, and setlist management.

### Key Product Goals
- Multi-track audio playback with individual stem control
- Professional setlist management for live performances
- Integrated practice tools (looping, speed/pitch control)
- Team collaboration features
- Local-first architecture (all audio files stored on user's device)

### Application Architecture
- **Single-Page Application**: No page navigation; all features accessible from main interface
- **Modal-Based Interactions**: Settings, new setlist creation, audio routing, and dialogs use modals
- **Always-Visible Panels**: Library, setlist, playback controls, and stem mixer remain visible
- **No Router**: Application uses a single-page layout without Vue Router or page transitions

## Tech Stack

### Frontend
- **Framework**: Vue 3.5+ with Composition API (`<script setup>`)
- **Language**: TypeScript 5.6+
- **Build Tool**: Vite 6.0+
- **UI Framework**: TailwindCSS v4 (using `@theme` directive in CSS)
- **Component Library**: shadcn-vue pattern with Radix Vue primitives
- **Icons**: Lucide Vue Next

### Desktop Runtime
- **Framework**: Tauri v2
- **Backend Language**: Rust (edition 2021)
- **IPC**: Tauri commands for Rust ↔ TypeScript communication

### Key Dependencies
- `radix-vue`: Unstyled, accessible component primitives
- `class-variance-authority`: Component variant management
- `tailwind-merge` + `clsx`: Intelligent className merging via `cn()` utility

## Development Commands

### Frontend Development
```bash
# Start Vite dev server only (frontend hot reload)
npm run dev

# Type check without building
npm run build  # Runs vue-tsc --noEmit && vite build

# Preview production build
npm run preview
```

### Tauri (Desktop App) Development
```bash
# Start Tauri in dev mode (opens desktop window)
npm run tauri dev

# Build production desktop app
npm run tauri build
```

**Important**: Vite dev server runs on port 1420 (strictPort: true). If unavailable, dev server will fail rather than use alternate port.

## Architecture

### Frontend Structure
```
src/
├── assets/
│   └── index.css          # TailwindCSS v4 theme configuration
├── components/
│   ├── library/           # Library view components
│   ├── setlist/           # Setlist builder components
│   ├── playback/          # Playback controls and stem mixer
│   ├── modals/            # Modal dialogs (New Setlist, Settings, Errors, etc.)
│   └── ui/                # shadcn-style UI primitives (Button, Modal, Dialog)
├── stores/                # Pinia stores (library, playback, setlist, modal)
├── lib/
│   └── utils.ts           # Shared utilities (cn() function)
├── App.vue                # Root single-page component
└── main.ts                # Vue app entry point
```

### Tauri (Rust) Structure
```
src-tauri/
├── src/
│   ├── lib.rs             # Tauri commands and app logic
│   └── main.rs            # Entry point (calls lib.rs::run())
├── Cargo.toml             # Rust dependencies
└── tauri.conf.json        # Tauri app configuration
```

### Frontend-Backend Communication
- **Pattern**: Use `invoke()` from `@tauri-apps/api/core` to call Rust commands
- **Example**:
  ```typescript
  // TypeScript (frontend)
  import { invoke } from '@tauri-apps/api/core'
  const result = await invoke('greet', { name: 'World' })
  ```
  ```rust
  // Rust (backend)
  #[tauri::command]
  fn greet(name: &str) -> String {
      format!("Hello, {}!", name)
  }
  ```
- **Registration**: Commands must be registered in `src-tauri/src/lib.rs` via `.invoke_handler(tauri::generate_handler![command_name])`

## TailwindCSS v4 Specifics

### Critical Differences from v3
1. **No `tailwind.config.js`**: Configuration moved to CSS using `@theme` directive in `src/assets/index.css`
2. **Import syntax**: Use `@import 'tailwindcss'` instead of `@tailwind base/components/utilities`
3. **Color system**: Define colors as CSS variables with `--color-*` prefix
4. **PostCSS plugin**: Uses `@tailwindcss/postcss` (configured in `postcss.config.js`)

### Theme Colors
Available utility classes (defined in `src/assets/index.css`):
- `bg-background` / `text-foreground`
- `bg-primary` / `text-primary-foreground`
- `bg-secondary` / `text-secondary-foreground`
- `bg-destructive` / `text-destructive-foreground`
- `bg-muted` / `text-muted-foreground`
- `bg-accent` / `text-accent-foreground`
- `bg-card` / `text-card-foreground`
- `border-border` / `border-input` / `ring-ring`

Dark mode uses `@media (prefers-color-scheme: dark)` in the `@theme` block.

## Component Development

### shadcn-vue Pattern
Components follow the shadcn pattern (see `src/components/ui/Button.vue`):

1. **Use `cn()` utility** for className merging:
   ```typescript
   import { cn } from '@/lib/utils'
   const classes = cn('base-class', props.class)
   ```

2. **Variants with CVA**:
   ```typescript
   import { cva, type VariantProps } from 'class-variance-authority'

   const buttonVariants = cva('base-classes', {
     variants: {
       variant: {
         default: 'variant-classes',
         secondary: 'variant-classes',
       },
       size: {
         sm: 'size-classes',
         lg: 'size-classes',
       },
     },
   })
   ```

3. **Path aliases**: Use `@/` for imports (resolves to `./src`)

### Adding New Components
- **UI Primitives**: Place in `src/components/ui/` (Button, Modal, Dialog, etc.)
- **Feature Components**: Organize by feature in `src/components/library/`, `src/components/setlist/`, etc.
- **Modal Components**: Place in `src/components/modals/`
- Use Radix Vue primitives for accessibility
- Follow Button.vue pattern for variants and props
- Use theme colors via `hsl(var(--color-*))`

### Modal Pattern
All modals follow this pattern:
1. **Base Component**: Use `src/components/ui/Modal.vue` for overlay, backdrop, and focus trap
2. **Modal Store**: Use `useModalStore()` to manage active modal state
3. **Opening**: `modalStore.openModal('modal-name', { data })` from any component
4. **Closing**: ESC key, backdrop click, or `modalStore.closeModal()`
5. **Examples**: NewSetlistModal, SettingsModal, ErrorModal, ImportProgressModal

**Important**: Never use page navigation or Vue Router. All interactions stay within the single-page layout.

## Rust Development

### Command Pattern
```rust
#[tauri::command]
fn command_name(param: &str) -> Result<String, String> {
    // Logic here
    Ok(result)
}

// Register in lib.rs:
.invoke_handler(tauri::generate_handler![command_name])
```

### Plugin System
- Plugins initialized in `lib.rs` builder chain
- Example: `tauri_plugin_opener` for opening URLs/files

## Important Constraints

1. **Audio Files**: Must remain local (never uploaded/synced to cloud)
2. **Performance**: Target <10ms audio latency for live performance use
3. **Reliability**: Zero crashes during performances (critical requirement)
4. **Platform Support**: macOS 11+, Windows 10/11 64-bit (mobile planned for later phases)
5. **Multi-Track Capacity**: Dynamic stem allocation with configurable presets
   - Standard: 16 stems (suitable for most backing tracks)
   - Extended: 32 stems (for complex arrangements)
   - Professional: 64 stems (for orchestral/large productions)
   - Custom: Up to 256 stems maximum (user-configurable)

## Product Development Phases

Refer to `PRD.md` for complete roadmap. Current focus areas:

**Phase 1 (MVP)**:
- Core audio engine
- Multi-track playback
- Basic setlist management
- File import/organization
- Click track generation

Future phases include practice tools, team collaboration, and advanced routing.

## Configuration Files

- **`vite.config.ts`**: Vite configuration with `@/` alias resolver
- **`tsconfig.json`**: TypeScript config with path mappings
- **`postcss.config.js`**: PostCSS with TailwindCSS v4 plugin
- **`src-tauri/tauri.conf.json`**: Tauri app configuration
- **`src-tauri/Cargo.toml`**: Rust dependencies

## Testing Strategy

(To be implemented in Phase 1)
- Focus on audio playback reliability
- Test multi-track synchronization
- Verify file import/organization
- Performance testing under live-use scenarios
