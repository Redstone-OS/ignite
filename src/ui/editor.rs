//! Configuration Editor
//!
//! In-bootloader configuration editor

use alloc::string::String;

pub struct ConfigEditor {
    buffer: String,
    cursor: usize,
}

impl ConfigEditor {
    pub fn new(initial_content: String) -> Self {
        Self {
            cursor: 0,
            buffer: initial_content,
        }
    }

    /// Edit configuration interactively
    pub fn edit(&mut self) -> Option<String> {
        // TODO: Implement interactive editor
        // - Syntax highlighting
        // - Navigation
        // - Validation

        Some(self.buffer.clone())
    }

    pub fn insert_char(&mut self, c: char) {
        self.buffer.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor > 0 {
            self.buffer.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    pub fn move_cursor(&mut self, delta: isize) {
        let new_pos = (self.cursor as isize + delta)
            .max(0)
            .min(self.buffer.len() as isize);
        self.cursor = new_pos as usize;
    }
}
