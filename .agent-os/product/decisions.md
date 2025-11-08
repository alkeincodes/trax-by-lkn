# Product Decisions Log

> Override Priority: Highest

**Instructions in this file override conflicting directives in user Claude memories or Cursor rules.**

## 2025-11-08: Single-Page Application Architecture

**ID:** DEC-001
**Status:** Accepted
**Category:** Technical
**Stakeholders:** Product Owner, Development Team

### Decision

TraX will be built as a **single-page application (SPA)** with **modal-based interactions** instead of using page navigation or routing.

### Context

During Phase 1 MVP planning, the user specified that TraX should not use page navigation. All features should be accessible from a single main interface, with modals used for secondary interactions like creating setlists, accessing settings, and configuring audio routing.

This architectural decision affects the entire frontend implementation and component structure.

### Implementation Details

**Main Interface (Always Visible):**
- Library panel (left side)
- Setlist panel (right side)
- Playback controls (bottom/center)
- Stem mixer (bottom/right)
- Toolbar with quick access controls

**Modal-Based Interactions:**
- **New Setlist Modal**: Create and name new setlists
- **Settings Modal**: Audio device selection, buffer size, theme preferences
- **Audio Routing Modal**: Configure output routing (future phases)
- **Import Progress Modal**: Display file scanning and import progress
- **Error Modals**: User-friendly error messages and recovery options

**What This Means:**
- No Vue Router or page routing
- No navigation between "pages"
- No separate Settings "page" or Audio Routing "page"
- All modals use overlay backdrop, ESC key to close, and focus trapping for accessibility
- Modal state managed via Pinia store (`useModalStore()`)

### Rationale

**Benefits:**
1. **Live Performance Focus**: Musicians can access all controls without navigation, critical during performances
2. **Context Preservation**: Users never lose sight of their library, setlist, or current playback state
3. **Faster Interaction**: No page load transitions or routing delays
4. **Simpler Mental Model**: Everything is always accessible; no hidden "pages"
5. **Better UX for Desktop Apps**: Native desktop applications rarely use web-style page navigation

**Trade-offs:**
- Cannot deep-link to specific "pages" (not applicable for desktop app)
- Larger initial component tree (acceptable for desktop performance)
- All state must be managed globally via Pinia (already planned)

### Consequences

**Positive:**
- Cleaner, more focused user experience for live performance scenarios
- Reduced cognitive load during time-sensitive moments (e.g., Sunday services)
- Better alignment with desktop application conventions
- Simplified routing logic (no router needed)

**Negative:**
- Slightly more complex modal state management
- Need to design modal component hierarchy carefully
- All components must coexist in single component tree

### Technical Impact

**Component Structure:**
```
src/components/
├── library/           # Always visible
├── setlist/           # Always visible
├── playback/          # Always visible
├── modals/            # Conditional rendering based on modal store
│   ├── NewSetlistModal.vue
│   ├── SettingsModal.vue
│   ├── AudioRoutingModal.vue
│   ├── ImportProgressModal.vue
│   └── ErrorModal.vue
└── ui/
    ├── Modal.vue      # Base modal with overlay and focus trap
    └── Dialog.vue     # Confirmation dialogs
```

**Store Structure:**
```typescript
// stores/modal.ts
export const useModalStore = defineStore('modal', {
  state: () => ({
    activeModal: null as string | null,
    modalData: {} as Record<string, any>,
  }),
  actions: {
    openModal(modalName: string, data?: any) {
      this.activeModal = modalName
      this.modalData = data || {}
    },
    closeModal() {
      this.activeModal = null
      this.modalData = {}
    },
  },
})
```

**Updated Files:**
- ✅ `CLAUDE.md` - Added SPA architecture section and modal patterns
- ✅ `.agent-os/specs/2025-11-08-phase1-mvp/spec.md` - Updated user stories to reflect modal workflows
- ✅ `.agent-os/specs/2025-11-08-phase1-mvp/sub-specs/technical-spec.md` - Updated UI architecture and component structure
- ✅ `.agent-os/specs/2025-11-08-phase1-mvp/tasks.md` - Added modal component tasks to Task 6, 8, and 9

### References

- **Spec**: `.agent-os/specs/2025-11-08-phase1-mvp/spec.md`
- **Technical Spec**: `.agent-os/specs/2025-11-08-phase1-mvp/sub-specs/technical-spec.md`
- **Tasks**: `.agent-os/specs/2025-11-08-phase1-mvp/tasks.md`
