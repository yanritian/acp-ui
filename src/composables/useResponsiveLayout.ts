import { ref, onMounted, onBeforeUnmount } from 'vue'

/**
 * Composable for responsive layout management.
 * Handles narrow screen detection, sidebar toggle, and backdrop click.
 */
export function useResponsiveLayout() {
  const showSidebar = ref(true)
  const isNarrowLayout = ref(false)
  
  let narrowMql: MediaQueryList | null = null
  
  function syncNarrowLayout() {
    if (narrowMql) isNarrowLayout.value = narrowMql.matches
  }
  
  function toggleSidebar() {
    showSidebar.value = !showSidebar.value
  }
  
  /** Close the drawer when the user taps the backdrop on a narrow viewport. */
  function handleBackdropClick() {
    if (isNarrowLayout.value) showSidebar.value = false
  }
  
  onMounted(() => {
    // Track viewport width so the sidebar can default-collapse into a drawer
    // on phones / narrow windows. We watch a MediaQueryList rather than
    // resize for correctness across orientation changes on iOS.
    if (typeof window !== 'undefined' && typeof window.matchMedia === 'function') {
      narrowMql = window.matchMedia('(max-width: 800px)')
      syncNarrowLayout()
      narrowMql.addEventListener('change', syncNarrowLayout)
      if (isNarrowLayout.value) showSidebar.value = false
    }
  })
  
  onBeforeUnmount(() => {
    if (narrowMql) {
      narrowMql.removeEventListener('change', syncNarrowLayout)
      narrowMql = null
    }
  })
  
  return {
    showSidebar,
    isNarrowLayout,
    toggleSidebar,
    handleBackdropClick,
  }
}
