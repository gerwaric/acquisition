//! The price-notes fixture (`reference/price-notes-2026-09-04.txt`), read
//! the way its header describes it, for the tests that cite it: the
//! currency table's (C68: the dialog's words and display names) and the
//! game-side parser's (C69: every note verbatim, every priced tab name).
//! Test-only; the fixture is evidence, never shipped data.

/// One note the game wrote: the note, the item's type line, its stack.
pub(crate) struct Note {
    pub note: String,
    #[allow(dead_code)]
    pub type_line: String,
    #[allow(dead_code)]
    pub stack: u32,
}

/// The fixture's three sections.
pub(crate) struct PriceNotes {
    /// Section 1: the notes, verbatim, in file order.
    pub notes: Vec<Note>,
    /// Section 2: the priced tab names, verbatim.
    pub tab_names: Vec<String>,
    /// Section 3: the dialog's word → display name, in dialog order.
    pub dialog: Vec<(String, String)>,
}

pub(crate) fn price_notes() -> PriceNotes {
    let text = include_str!("../reference/price-notes-2026-09-04.txt");
    let mut out = PriceNotes {
        notes: Vec::new(),
        tab_names: Vec::new(),
        dialog: Vec::new(),
    };
    let mut section = 0;
    for line in text.lines() {
        if line.starts_with('#') {
            if line.contains("Tab names that parse as prices") {
                section = 1;
            } else if line.contains("currency list as the owner read it") {
                section = 2;
            }
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        match section {
            0 => {
                let mut cols = line.split('\t');
                let note = cols.next().unwrap().to_string();
                let type_line = cols.next().unwrap().to_string();
                let stack = cols.next().unwrap().parse().unwrap();
                out.notes.push(Note {
                    note,
                    type_line,
                    stack,
                });
            }
            1 => out.tab_names.push(line.to_string()),
            _ => {
                let (word, display) = line.split_once('\t').unwrap();
                out.dialog.push((word.to_string(), display.to_string()));
            }
        }
    }
    out
}
