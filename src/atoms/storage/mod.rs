mod clipboard;
mod config_io;
mod draft_io;
mod external_editor;
pub mod file_watcher;

pub use clipboard::{clipboard_copy, clipboard_paste};
pub use config_io::{
    config_dir, ensure_config_dir, expand_tilde, load_config, resolve_data_dir, save_config,
};
pub use draft_io::{
    archive_draft, delete_draft, ensure_data_dirs, load_all_drafts, load_draft, restore_draft,
    save_draft,
};
pub use external_editor::{
    cleanup_temp_file, read_temp_file, resolve_editor, spawn_editor, write_temp_file,
};
