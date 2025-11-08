# TailwindCSS v4 + shadcn-vue Setup

## Installation Summary

This project uses **TailwindCSS v4** with **shadcn-vue** components.

### Installed Packages

#### TailwindCSS v4
- `tailwindcss` - Core framework
- `@tailwindcss/postcss` - PostCSS plugin for v4
- `postcss` - CSS processor
- `autoprefixer` - Browser compatibility

#### shadcn-vue Dependencies
- `radix-vue` - Unstyled, accessible components
- `lucide-vue-next` - Icon library
- `class-variance-authority` - Component variant utilities
- `clsx` - Conditional className utility
- `tailwind-merge` - Merge Tailwind classes intelligently
- `tailwindcss-animate` - Animation utilities

### Configuration Files

#### postcss.config.js
```js
export default {
  plugins: {
    '@tailwindcss/postcss': {},
  },
}
```

#### src/assets/index.css
Uses TailwindCSS v4's new `@theme` directive for configuration:
- Color system using CSS variables
- Dark mode support via `@media (prefers-color-scheme: dark)`
- Custom theme tokens

#### tsconfig.json
Added path aliases:
```json
{
  "baseUrl": ".",
  "paths": {
    "@/*": ["./src/*"]
  }
}
```

#### vite.config.ts
Added resolver for path aliases:
```ts
resolve: {
  alias: {
    '@': path.resolve(__dirname, './src'),
  },
}
```

### Directory Structure

```
src/
├── assets/
│   └── index.css          # TailwindCSS v4 config with @theme
├── components/
│   └── ui/
│       └── Button.vue     # Example shadcn-style component
├── lib/
│   └── utils.ts           # cn() utility for className merging
├── App.vue
└── main.ts
```

### Key Differences from TailwindCSS v3

1. **No tailwind.config.js**: Configuration is now done in CSS using `@theme` directive
2. **@import instead of @tailwind**: Use `@import 'tailwindcss'` instead of `@tailwind` directives
3. **Color variables**: Use `--color-*` prefix for custom colors
4. **PostCSS plugin**: Uses `@tailwindcss/postcss` instead of `tailwindcss`

### Usage

#### Using the Button Component
```vue
<script setup>
import Button from '@/components/ui/Button.vue'
</script>

<template>
  <Button>Default</Button>
  <Button variant="secondary">Secondary</Button>
  <Button variant="outline">Outline</Button>
  <Button size="sm">Small</Button>
  <Button size="lg">Large</Button>
</template>
```

#### Using Custom Colors
TailwindCSS v4 automatically generates utility classes from your theme colors:

```html
<!-- Simple, clean class names -->
<div class="bg-primary text-primary-foreground">
  Primary background
</div>

<div class="bg-secondary text-secondary-foreground">
  Secondary background
</div>

<div class="border border-input">
  Input border
</div>
```

Available color utilities:
- `bg-background`, `text-foreground`
- `bg-primary`, `text-primary-foreground`
- `bg-secondary`, `text-secondary-foreground`
- `bg-destructive`, `text-destructive-foreground`
- `bg-muted`, `text-muted-foreground`
- `bg-accent`, `text-accent-foreground`
- `bg-card`, `text-card-foreground`
- `bg-popover`, `text-popover-foreground`
- `border-border`, `border-input`
- `ring-ring`

Define custom colors in your `@theme` block in `src/assets/index.css`.

### Development

```bash
# Run dev server
npm run dev

# Build for production
npm run build

# Run Tauri app
npm run tauri dev
```

### Adding More shadcn-vue Components

When adding new shadcn-style components:
1. Create component in `src/components/ui/`
2. Use `cn()` utility from `@/lib/utils` for className merging
3. Follow the pattern in `Button.vue` for variants and sizes
4. Use theme colors with `hsl(var(--color-*))` syntax

### Resources

- [TailwindCSS v4 Documentation](https://tailwindcss.com/docs)
- [Radix Vue](https://www.radix-vue.com/)
- [Lucide Icons](https://lucide.dev/)
- [shadcn/ui (original React version)](https://ui.shadcn.com/)
