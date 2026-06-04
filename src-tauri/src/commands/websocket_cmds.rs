use crate::AppState;
use crate::websocket;
use tauri::{AppHandle, State};
use uuid::Uuid;

#[tauri::command]
pub fn generate_app_qrcode(state: State<AppState>) -> Result<String, String> {
    let token = Uuid::new_v4().to_string();

    // Sync token to ws_server so the client can actually authenticate
    {
        let ws = state.ws_server.lock().unwrap();
        if let Some(server) = ws.as_ref() {
            server.set_auth_token(Some(token.clone()));
        }
    }

    // Check if tunnel is running and has a public URL
    let tunnel_manager = state.tunnel_manager.lock().unwrap();
    let tunnel_url = tunnel_manager.as_ref().and_then(|tm| tm.get_public_url());

    // Use tunnel URL if available, otherwise use local IP
    let ws_url = if let Some(public_url) = tunnel_url {
        // Convert https URL to wss WebSocket URL
        let wss_url = public_url.replace("https://", "wss://").replace("http://", "ws://");
        format!("{}?token={}", wss_url, token)
    } else {
        let local_ip = get_local_ip().unwrap_or_else(|| "localhost".to_string());
        let ws = state.ws_server.lock().unwrap();
        let port = ws.as_ref().map(|s| s.port).unwrap_or(1420);
        format!("ws://{}:{}?token={}", local_ip, port, token)
    };

    println!("Generated QR code URL: {}", ws_url);
    Ok(ws_url)
}

#[tauri::command]
pub async fn start_ws_server(
    port: u16,
    bind_external: bool,
    state: State<'_, AppState>,
    app_handle: AppHandle
) -> Result<String, String> {
    // Create server
    let server = websocket::WebSocketServer::new(port);

    // Start the server (bind_external determines if we bind to 0.0.0.0 or 127.0.0.1)
    let url = server.start(app_handle, bind_external).await?;

    // Store in state after async operation completes
    {
        let mut ws = state.ws_server.lock().unwrap();
        *ws = Some(server);
    }

    Ok(url)
}

#[tauri::command]
pub fn stop_ws_server(state: State<AppState>) -> Result<(), String> {
    let ws = state.ws_server.lock().unwrap();
    if let Some(server) = ws.as_ref() {
        server.stop();
    }
    Ok(())
}

#[tauri::command]
pub fn get_connected_clients(state: State<AppState>) -> Result<Vec<websocket::RemoteClient>, String> {
    let ws = state.ws_server.lock().unwrap();
    if let Some(server) = ws.as_ref() {
        Ok(server.get_clients())
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
pub fn ws_server_status(state: State<AppState>) -> Result<bool, String> {
    let ws = state.ws_server.lock().unwrap();
    if let Some(server) = ws.as_ref() {
        Ok(server.is_running())
    } else {
        Ok(false)
    }
}

/// Get local IP address
pub fn get_local_ip() -> Option<String> {
    use std::net::UdpSocket;
    // Try to connect to a public address to find local IP
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}
