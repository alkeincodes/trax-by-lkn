mod audio;
mod database;
mod import;
mod commands;
mod events;

use std::sync::Arc;
use audio::MultiTrackEngine;
use database::Database;
use commands::AppState;
use tauri::{Manager, Emitter, menu::{MenuBuilder, SubmenuBuilder, MenuItemBuilder}};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logger with default level INFO
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Initializing TraX application...");

    // Initialize database
    let database = Database::new()
        .expect("Failed to initialize database");

    log::info!("Database initialized successfully");

    // Initialize multi-track audio engine with extended capacity (32 stems)
    // Uses parallel decoding for fast load times and full pre-decode for zero dropouts
    let audio_engine = MultiTrackEngine::new_extended()
        .expect("Failed to initialize audio engine");

    log::info!("Audio engine initialized successfully");

    // Create shared application state
    let app_state = AppState::new(database, audio_engine);

    // Clone the Arc references needed for position emitter (before moving app_state)
    let (position_arc, playback_state_arc, stem_levels_arc, master_level_arc) = {
        let engine = app_state.audio_engine.lock().unwrap();
        let pos = engine.position_arc();
        let state = engine.playback_state_arc();
        let levels = engine.stem_levels_arc();
        let master = engine.master_level_arc();
        (pos, state, levels, master)
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // Build native menu
            let settings_item = MenuItemBuilder::with_id("settings", "Settings...")
                .accelerator("CmdOrCtrl+,")
                .build(app)?;

            let file_menu = SubmenuBuilder::new(app, "File")
                .item(&settings_item)
                .build()?;

            let menu = MenuBuilder::new(app)
                .item(&file_menu)
                .build()?;

            app.set_menu(menu)?;

            // Handle menu events
            app.on_menu_event(move |app, event| {
                if event.id() == "settings" {
                    // Emit event to frontend to open settings modal
                    let _ = app.emit("open-settings", ());
                }
            });

            // Start the position emitter background task
            events::start_position_emitter(app_handle, position_arc, playback_state_arc, stem_levels_arc, master_level_arc);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            // Playback commands
            commands::load_song,
            commands::play_song,
            commands::resume_playback,
            commands::pause_playback,
            commands::stop_playback,
            commands::seek_to_position,
            commands::get_playback_position,
            commands::preload_setlist,
            commands::preload_setlist_smart,
            // Stem control commands
            commands::set_stem_volume,
            commands::toggle_stem_mute,
            commands::toggle_stem_solo,
            commands::set_master_volume,
            commands::get_current_stems,
            // Library commands
            commands::import_files,
            commands::get_all_songs,
            commands::search_songs,
            commands::filter_songs,
            commands::get_song,
            commands::delete_song,
            commands::get_song_stems,
            // Setlist commands
            commands::create_setlist,
            commands::get_setlist,
            commands::update_setlist,
            commands::delete_setlist,
            commands::get_all_setlists,
            commands::add_song_to_setlist,
            commands::remove_song_from_setlist,
            commands::reorder_setlist_songs,
            // Cache commands
            commands::get_cache_stats,
            commands::set_cache_size,
            commands::clear_cache,
            // Settings commands
            commands::get_audio_devices,
            commands::get_current_audio_device,
            commands::get_audio_settings,
            commands::set_audio_device,
            commands::set_buffer_size,
            commands::set_sample_rate,
            commands::switch_audio_device,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
