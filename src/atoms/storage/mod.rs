mod clipboard;
mod config_io;
mod draft_io;
mod external_editor;

pub use clipboard::{clipboard_copy, clipboard_paste};
pub use config_io::{load_config, save_config, config_dir, ensure_config_dir};
pub use draft_io::{
    drafts_dir, ensure_drafts_dir, load_draft, load_all_drafts, save_draft, delete_draft,
};
pub use external_editor::{
    resolve_editor, write_temp_file, spawn_editor, read_temp_file, cleanup_temp_file,
};
