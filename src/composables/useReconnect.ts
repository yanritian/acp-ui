import { onMounted, onBeforeUnmount } from 'vue'
import { useSessionStore } from '../stores/session'

/**
 * Composable for foreground-reconnect functionality.
 * Handles visibility change, online events, and manual reconnect.
 * 
 * Mobile OSes freeze the WebView when the app is backgrounded and routers
 * may drop the idle TCP connection while we're away. When the user returns
 * we ask the session store to silently reattach to the last session.
 */
export function useReconnect() {
  const sessionStore = useSessionStore()
  
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null
  
  function scheduleReconnect() {
    // Coalesce rapid visibility/online flips into a single attempt.
    if (reconnectTimer) return
    reconnectTimer = setTimeout(() => {
      reconnectTimer = null
      // Skip if the browser still thinks we're offline; we'll be re-triggered
      // by the `online` event when connectivity returns.
      if (typeof navigator !== 'undefined' && navigator.onLine === false) return
      void sessionStore.tryReconnect()
    }, 250)
  }
  
  function handleVisibilityChange() {
    if (typeof document !== 'undefined' && !document.hidden) scheduleReconnect()
  }
  
  function handleOnline() {
    scheduleReconnect()
  }
  
  async function handleManualReconnect() {
    await sessionStore.tryReconnect()
  }
  
  onMounted(() => {
    // Hook foreground-reconnect listeners. `pageshow` fires both on initial
    // navigation and when iOS restores a frozen WebView from the back/forward
    // cache, so it complements `visibilitychange` on Safari/iOS.
    if (typeof document !== 'undefined') {
      document.addEventListener('visibilitychange', handleVisibilityChange)
    }
    if (typeof window !== 'undefined') {
      window.addEventListener('pageshow', scheduleReconnect)
      window.addEventListener('online', handleOnline)
    }
  })
  
  onBeforeUnmount(() => {
    if (typeof document !== 'undefined') {
      document.removeEventListener('visibilitychange', handleVisibilityChange)
    }
    if (typeof window !== 'undefined') {
      window.removeEventListener('pageshow', scheduleReconnect)
      window.removeEventListener('online', handleOnline)
    }
    if (reconnectTimer) {
      clearTimeout(reconnectTimer)
      reconnectTimer = null
    }
  })
  
  return {
    handleManualReconnect,
  }
}
