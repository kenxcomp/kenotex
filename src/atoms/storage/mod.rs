mod clipboard;
mod config_io;
mod draft_io;
mod external_editor;
pub mod file_watcher;

pub use clipboard::{clipboard_copy, clipboard_paste};
pub use config_io::{load_config, save_config, config_dir, ensure_config_dir, expand_tilde, resolve_data_dir};
pub use draft_io::{
    ensure_data_dirs, load_draft, load_all_drafts, save_draft, delete_draft,
    archive_draft, restore_draft,
};
pub use external_editor::{
    resolve_editor, write_temp_file, spawn_editor, read_temp_file, cleanup_temp_file,
};
