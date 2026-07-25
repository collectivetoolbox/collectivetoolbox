use crate::anyhow::{Result, anyhow, ensure};
use serde::Serialize;

use crate::util::{
    array::{print_arr, set_element, str_print_arr},
    string::{
        int_arr_from_str_printed_arr, str_join_esc_no_trailing,
        str_split_escaped,
    },
};

/// Environment / shared state.
/// In JS this lived on the global object with many independent keys;
/// here we consolidate into a struct.
#[derive(Default, Serialize)]
pub struct EiteState {
    // Preferences / environment
    pub debug_level: i32,
    pub env_preferred_format: String,
    pub env_char_encoding: String,
    pub env_terminal_type: String,
    pub env_language: String,
    pub env_locale_config: String,
    pub env_code_language: String,
    pub env_resolution_w: i32,
    pub env_resolution_h: i32,

    // Execution-related state (present in original snippet but not yet used here)
    pub document_exec_data: Vec<String>,
    pub document_exec_symbol_index: Vec<String>,
    pub document_exec_ptrs: Vec<String>,
    pub document_exec_frames: Vec<String>,
    pub document_exec_events: Vec<String>,
    pub document_exec_logs: Vec<String>,
    pub document_exec_settings: Vec<String>,

    // Test counters
    pub tests_passed: u32,
    pub tests_failed: u32,
    pub tests_total: u32,

    // Import / export configuration
    pub import_settings: Vec<String>,
    pub export_settings: Vec<String>,
    pub import_deferred_settings_stack: Vec<String>,
    pub export_deferred_settings_stack: Vec<String>,

    // Storage config
    pub storage_cfg: Vec<String>,
}

impl EiteState {
    pub fn new() -> Self {
        Self {
            debug_level: 0,
            env_preferred_format: String::new(),
            env_char_encoding: "asciiSafeSubset".into(),
            env_terminal_type: "vt100".into(),
            env_language: "en-US".into(),
            env_locale_config: "inherit:usa,".into(),
            env_code_language: "javascript".into(),
            env_resolution_w: 0,
            env_resolution_h: 0,
            ..Default::default()
        }
    }

    /* -------------------- Document Exec State Management ---------------------- */
    /// Prepare a new document execution context. Returns exec ID.
    pub fn prepare_document_exec(&mut self, contents: &[u32]) -> usize {
        let exec_id = self.document_exec_ptrs.len();
        self.document_exec_data.push(str_print_arr(contents));
        self.document_exec_symbol_index.push(String::new());
        // Pointer stack stored as escaped join; initial current pointer = 0 (trailing comma).
        self.document_exec_ptrs.push("0,".to_string());
        self.document_exec_frames.push(String::new());
        self.document_exec_events.push(String::new());
        self.document_exec_logs.push(String::new());
        self.document_exec_settings.push(",".to_string()); // empty kv join
        exec_id
    }

    pub fn is_exec_id(&self, exec_id: usize) -> bool {
        exec_id < self.document_exec_ptrs.len()
    }

    /// NOTE: Minimal kv split: split on ',' dropping empty trailing segments.
    pub fn get_exec_settings(&self, exec_id: usize) -> Result<Vec<String>> {
        if !self.is_exec_id(exec_id) {
            return Err(anyhow!("Exec id out of range"));
        }
        let settings = self
            .document_exec_settings
            .get(exec_id)
            .ok_or_else(|| anyhow!("Exec id not found"))?;
        Ok(settings
            .split(',')
            .filter(|s| !s.is_empty())
            .map(std::string::ToString::to_string)
            .collect())
    }

    /// Replace full settings (joined with trailing comma for symmetry with original usage).
    pub fn set_exec_settings(
        &mut self,
        exec_id: usize,
        kv: &[String],
    ) -> Result<()> {
        if !self.is_exec_id(exec_id) {
            return Err(anyhow!("Exec id out of range"));
        }
        let mut joined = String::new();
        for k in kv {
            joined.push_str(k);
            joined.push(',');
        }
        if let Some(item) = self.document_exec_settings.get_mut(exec_id) {
            *item = joined;
        }
        Ok(())
    }

    pub fn get_exec_ptrs(&self, exec_id: usize) -> Result<Vec<String>> {
        if !self.is_exec_id(exec_id) {
            return Err(anyhow!("Exec id out of range"));
        }
        let ptr_str = self.document_exec_ptrs.get(exec_id).ok_or_else(|| anyhow!("Exec id out of range"))?;
        Ok(str_split_escaped(ptr_str, ","))
    }

    pub fn set_exec_ptrs(
        &mut self,
        exec_id: usize,
        ptrs: &[String],
    ) -> Result<()> {
        if !self.is_exec_id(exec_id) {
            return Err(anyhow!("Exec id out of range"));
        }
        if let Some(item) = self.document_exec_ptrs.get_mut(exec_id) {
            *item = str_join_esc_no_trailing(ptrs, ",");
        }
        Ok(())
    }

    fn parse_ptr(val: &str) -> u32 {
        if val.is_empty() {
            0
        } else {
            val.parse::<u32>().unwrap_or(0)
        }
    }

    pub fn get_current_exec_ptr_pos(&self, exec_id: usize) -> Result<u32> {
        let ptrs = self.get_exec_ptrs(exec_id)?;
        if let Some(last) = ptrs.last() {
            Ok(Self::parse_ptr(last))
        } else {
            Ok(0)
        }
    }

    pub fn set_exec_ptr_pos(
        &mut self,
        exec_id: usize,
        new_pos: u32,
    ) -> Result<()> {
        let mut ptrs = self.get_exec_ptrs(exec_id)?;
        set_element(&mut ptrs, -1, new_pos.to_string());
        self.set_exec_ptrs(exec_id, &ptrs)?;
        Ok(())
    }

    pub fn incr_exec_ptr_pos(&mut self, exec_id: usize) -> Result<()> {
        let cur = self.get_current_exec_ptr_pos(exec_id)?;
        self.set_exec_ptr_pos(exec_id, cur.saturating_add(1))
    }

    pub fn get_next_level_exec_ptr_pos(&self, exec_id: usize) -> Result<u32> {
        let ptrs = self.get_exec_ptrs(exec_id)?;
        if let Some(val) = ptrs.len().checked_sub(2).and_then(|idx| ptrs.get(idx)) {
            Ok(Self::parse_ptr(val))
        } else {
            Ok(0)
        }
    }

    pub fn get_current_exec_data(&self, exec_id: usize) -> Result<Vec<u32>> {
        if !self.is_exec_id(exec_id) {
            return Err(anyhow!("Exec id out of range"));
        }
        let data_str = self.document_exec_data.get(exec_id).ok_or_else(|| anyhow!("Exec id out of range"))?;
        int_arr_from_str_printed_arr(data_str)
    }

    pub fn get_current_exec_frame(&self, exec_id: usize) -> Result<Vec<u32>> {
        if !self.is_exec_id(exec_id) {
            return Err(anyhow!("Exec id out of range"));
        }
        let frames_str = self.document_exec_frames.get(exec_id).ok_or_else(|| anyhow!("Exec id out of range"))?;
        int_arr_from_str_printed_arr(frames_str)
    }

    /// Retrieve a single exec option (key -> value) from settings KV.
    pub fn get_exec_option(
        &self,
        exec_id: usize,
        key: &str,
    ) -> Result<Option<String>> {
        let kv = self.get_exec_settings(exec_id)?;
        for pair in kv {
            if let Some((k, v)) = pair.split_once('=') {
                if k == key {
                    return Ok(Some(v.to_string()));
                }
            }
        }
        Ok(None)
    }

    /// Set (or insert) an exec option key=value pair.
    pub fn set_exec_option(
        &mut self,
        exec_id: usize,
        key: &str,
        val: &str,
    ) -> Result<()> {
        let mut kv = self.get_exec_settings(exec_id)?;
        let mut found = false;
        for item in &mut kv {
            if let Some((k, _)) = item.split_once('=') {
                if k == key {
                    *item = format!("{key}={val}");
                    found = true;
                    break;
                }
            }
        }
        if !found {
            kv.push(format!("{key}={val}"));
        }
        self.set_exec_settings(exec_id, &kv)
    }

    /// Overwrite current frame output.
    pub fn set_exec_frame(
        &mut self,
        exec_id: usize,
        frame: &[u32],
    ) -> Result<()> {
        ensure!(self.is_exec_id(exec_id), "Invalid exec id {exec_id}");
        if exec_id >= self.document_exec_frames.len() {
            self.document_exec_frames.resize(exec_id.saturating_add(1), String::new());
        }
        if let Some(item) = self.document_exec_frames.get_mut(exec_id) {
            *item = print_arr(frame);
        }
        Ok(())
    }

    pub fn get_env_preferred_format(&self) -> &str {
        if self.env_preferred_format.is_empty() {
            "utf8"
        } else {
            &self.env_preferred_format
        }
    }
}
