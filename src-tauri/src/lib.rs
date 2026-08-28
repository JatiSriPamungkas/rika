use tauri::{Emitter, Manager, WebviewWindow};
use serde::Serialize;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};

#[derive(Clone, Serialize)]
struct TrackState {
    player_running: bool,
    title: String,
    artist: String,
    duration: f64,
    position: f64,
    status: String,
    synced_lyrics: Option<String>,
    plain_lyrics: Option<String>,
    lyrics_fetched: bool,
}

#[derive(serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LrcLibResponse {
    synced_lyrics: Option<String>,
    plain_lyrics: Option<String>,
}

#[tauri::command]
fn start_drag(window: WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_click_through(window: WebviewWindow, ignore: bool) -> Result<(), String> {
    let _ = window.set_always_on_top(true);
    window.set_ignore_cursor_events(ignore).map_err(|e| e.to_string())
}

#[tauri::command]
fn close_app(window: WebviewWindow) {
    let _ = window.destroy();
    std::process::exit(0);
}

async fn fetch_lyrics_internal(
    client: &reqwest::Client,
    title: &str,
    artist: &str,
    duration: f64,
) -> Option<(Option<String>, Option<String>)> {
    let user_agent = "RikaLyricsHUD/1.0 (contact: github.com/JatiSriPamungkas/rika)";
    println!("[Rika HUD] Fetching lyrics for: '{}' - '{}' (duration: {:.1}s)", title, artist, duration);
    
    // 1. Try GET /api/get with duration
    if duration > 0.0 {
        if let Ok(res) = client.get("https://lrclib.net/api/get")
            .header("User-Agent", user_agent)
            .query(&[
                ("track_name", title),
                ("artist_name", artist),
                ("duration", &format!("{:.0}", duration)),
            ])
            .send()
            .await
        {
            let status = res.status();
            if status.is_success() {
                if let Ok(data) = res.json::<LrcLibResponse>().await {
                    println!("[Rika HUD] Found lyrics via GET /api/get (with duration)");
                    return Some((data.synced_lyrics, data.plain_lyrics));
                }
            } else {
                println!("[Rika HUD] GET /api/get (with duration) status: {}", status);
            }
        }
    }
    
    // 2. Try GET /api/get without duration
    if let Ok(res) = client.get("https://lrclib.net/api/get")
        .header("User-Agent", user_agent)
        .query(&[
            ("track_name", title),
            ("artist_name", artist),
        ])
        .send()
        .await
    {
        let status = res.status();
        if status.is_success() {
            if let Ok(data) = res.json::<LrcLibResponse>().await {
                println!("[Rika HUD] Found lyrics via GET /api/get (without duration)");
                return Some((data.synced_lyrics, data.plain_lyrics));
            }
        } else {
            println!("[Rika HUD] GET /api/get (without duration) status: {}", status);
        }
    }
    
    // 3. Fallback to search
    if let Ok(res) = client.get("https://lrclib.net/api/search")
        .header("User-Agent", user_agent)
        .query(&[("q", &format!("{} {}", title, artist))])
        .send()
        .await
    {
        let status = res.status();
        if status.is_success() {
            if let Ok(results) = res.json::<Vec<LrcLibResponse>>().await {
                println!("[Rika HUD] Search results count: {}", results.len());
                for item in results {
                    if item.synced_lyrics.is_some() || item.plain_lyrics.is_some() {
                        println!("[Rika HUD] Found lyrics via GET /api/search fallback");
                        return Some((item.synced_lyrics, item.plain_lyrics));
                    }
                }
            }
        } else {
            println!("[Rika HUD] GET /api/search status: {}", status);
        }
    }
    
    println!("[Rika HUD] Failed to retrieve any lyrics from LrcLib.");
    None
}

fn get_spotify_state() -> TrackState {
    let mut state = TrackState {
        player_running: false,
        title: String::new(),
        artist: String::new(),
        duration: 0.0,
        position: 0.0,
        status: "Stopped".to_string(),
        synced_lyrics: None,
        plain_lyrics: None,
        lyrics_fetched: false,
    };
    
    if let Ok(finder) = mpris::PlayerFinder::new() {
        if let Ok(mut players) = finder.iter_players() {
            let spotify_player = players.find_map(|p_res| {
                if let Ok(player) = p_res {
                    let identity = player.identity();
                    let bus_name = player.bus_name_trimmed();
                    println!("[Rika HUD] Found MPRIS player: identity='{}', bus_name='{}'", identity, bus_name);
                    let identity_lower = identity.to_lowercase();
                    let bus_name_lower = bus_name.to_lowercase();
                    if identity_lower.contains("spotify") || bus_name_lower.contains("spotify") {
                        return Some(player);
                    }
                }
                None
            });
            
            if let Some(player) = spotify_player {
                state.player_running = true;
                
                if let Ok(metadata) = player.get_metadata() {
                    state.title = metadata.title().unwrap_or("").to_string();
                    state.artist = metadata.artists().unwrap_or_default().join(", ");
                    state.duration = metadata.length().map(|d| d.as_secs_f64()).unwrap_or(0.0);
                }
                
                if let Ok(status) = player.get_playback_status() {
                    state.status = format!("{:?}", status);
                }
                
                if let Ok(pos) = player.get_position() {
                    state.position = pos.as_secs_f64();
                }
            }
        }
    }
    
    state
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("GDK_BACKEND", "x11");
    }

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        let _ = app.emit("toggle-click-through", ());
                    }
                })
                .build()
        )
        .setup(|app| {
            use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
            
            let ctrl_alt_k = Shortcut::new(
                Some(Modifiers::CONTROL | Modifiers::ALT),
                Code::KeyK,
            );
            if let Err(e) = app.global_shortcut().register(ctrl_alt_k) {
                println!("[Rika HUD] Failed to register global shortcut Ctrl+Alt+K: {:?}", e);
            } else {
                println!("[Rika HUD] Successfully registered global shortcut Ctrl+Alt+K");
            }

            // Setup system tray menu and handler
            let toggle_lock = MenuItem::with_id(app, "toggle_lock", "Toggle Lock/Unlock (Ctrl+Alt+K)", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle_lock, &quit])?;

            if let Some(icon) = app.default_window_icon().cloned() {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_icon(icon.clone());
                    let _ = window.set_always_on_top(true);
                }
                let _tray = TrayIconBuilder::new()
                    .icon(icon)
                    .menu(&menu)
                    .on_menu_event(|app, event| {
                        match event.id.as_ref() {
                            "toggle_lock" => {
                                let _ = app.emit("toggle-click-through", ());
                            }
                            "quit" => {
                                std::process::exit(0);
                            }
                            _ => {}
                        }
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                            let app = tray.app_handle();
                            let _ = app.emit("toggle-click-through", ());
                        }
                    })
                    .build(app)?;
            }

            let handle = app.handle().clone();
            
            tauri::async_runtime::spawn(async move {
                let mut current_title = String::new();
                let mut current_artist = String::new();
                let mut current_synced = None;
                let mut current_plain = None;
                let mut current_lyrics_fetched = false;
                
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(3))
                    .build()
                    .unwrap_or_default();
                
                let (tx, mut rx) = tokio::sync::mpsc::channel::<(String, String, Option<(Option<String>, Option<String>)>)>(5);
                
                loop {
                    // Check for newly fetched lyrics
                    while let Ok((title, artist, result)) = rx.try_recv() {
                        if title == current_title && artist == current_artist {
                            current_lyrics_fetched = true;
                            if let Some((synced, plain)) = result {
                                current_synced = synced;
                                current_plain = plain;
                            } else {
                                current_synced = None;
                                current_plain = None;
                            }
                        }
                    }
                    
                    let mut state = get_spotify_state();
                    
                    if state.player_running {
                        let title = state.title.clone();
                        let artist = state.artist.clone();
                        let duration = state.duration;
                        
                        if title != current_title || artist != current_artist {
                            current_title = title.clone();
                            current_artist = artist.clone();
                            current_synced = None;
                            current_plain = None;
                            current_lyrics_fetched = false;
                            
                            // Spawn lyric fetcher
                            let tx_clone = tx.clone();
                            let title_clone = title.clone();
                            let artist_clone = artist.clone();
                            let duration_clone = duration;
                            let client_clone = client.clone();
                            
                            tauri::async_runtime::spawn(async move {
                                let fetched = fetch_lyrics_internal(&client_clone, &title_clone, &artist_clone, duration_clone).await;
                                let _ = tx_clone.send((title_clone, artist_clone, fetched)).await;
                            });
                        }
                        
                        state.synced_lyrics = current_synced.clone();
                        state.plain_lyrics = current_plain.clone();
                        state.lyrics_fetched = current_lyrics_fetched;
                    }
                    
                    // Emit track state to frontend
                    let _ = handle.emit("track-state", state);
                    
                    // Check for local file trigger (e.g. /tmp/rika_toggle)
                    let trigger_path = std::env::temp_dir().join("rika_toggle");
                    if trigger_path.exists() {
                        println!("[Rika Rust] Detected file trigger /tmp/rika_toggle! Emitting toggle event...");
                        if let Err(e) = std::fs::remove_file(&trigger_path) {
                            println!("[Rika Rust] Error removing trigger file: {:?}", e);
                        }
                        if let Err(e) = handle.emit("toggle-click-through", ()) {
                            println!("[Rika Rust] Error emitting toggle-click-through event: {:?}", e);
                        }
                    }
                    
                    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![start_drag, set_click_through, close_app])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}