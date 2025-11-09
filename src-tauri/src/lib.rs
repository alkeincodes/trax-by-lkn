mod audio;
mod cache;
mod database;
mod import;
mod commands;
mod events;

use std::sync::Arc;
use audio::MultiTrackEngine;
use cache::{CacheManager, CacheSettings};
use database::Database;
use commands::AppState;

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

    // Initialize persistent cache
    let cache_settings = CacheSettings::default();
    let cache_manager = CacheManager::new(cache_settings)
        .expect("Failed to initialize cache manager");

    log::info!("Cache manager initialized successfully");

    // Initialize multi-track audio engine with standard capacity (16 stems)
    let audio_engine = MultiTrackEngine::new_standard()
        .expect("Failed to initialize audio engine");

    log::info!("Audio engine initialized successfully");

    // Create shared application state
    let app_state = AppState::new(database, audio_engine, cache_manager);

    // Clone the Arc references needed for position emitter (before moving app_state)
    // We can't pass the whole audio_engine because it contains a non-Send Stream
    let position_arc = {
        let engine = app_state.audio_engine.lock().unwrap();
        engine.position_arc()
    };
    let playback_state_arc = {
        let engine = app_state.audio_engine.lock().unwrap();
        engine.playback_state_arc()
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .setup(move |app| {
            let app_handle = app.handle().clone();
            // Start the position emitter background task
            events::start_position_emitter(app_handle, position_arc, playback_state_arc);
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
            // Stem control commands
            commands::set_stem_volume,
            commands::toggle_stem_mute,
            commands::toggle_stem_solo,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
