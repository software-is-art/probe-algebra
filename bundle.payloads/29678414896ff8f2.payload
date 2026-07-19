//! desk — the suit's workbench, the editor loop's file half (the payload-shuttle
//! retirement): `open` materializes an item's exact bytes onto a SHEET, the sheet
//! is edited in place by whatever editor the author holds, `desk` preflights every
//! changed sheet against the judgment `land` would reach, and `offer` closes the
//! loop as one envelope. What the desk removes is the shuttle itself — no path
//! bookkeeping between `show` and `edit`, no offer argv retyped from memory: the
//! binding is remembered here, the judgment is standing, and the offer is atomic.
//!
//! The bindings file speaks the journal grammar (`open <module> — <address> @<orig>`),
//! and the materialized bytes live in the payload store, so "changed" is a
//! comparison against remembered content, never a guess — the desk's memory is the
//! memory everything else already trusts. A sheet file deleted from the desk is a
//! sheet taken OFF it (the desk is the file world, deliberately); `desk` and
//! `offer` read absence as withdrawal, not error.

use crate::discover::envelope::split_address;

/// One sheet on the desk: a module, the address whose bytes were materialized
/// (`+` marks a fresh add sheet, born empty), and the payload-store address of
/// those bytes as materialized.
#[derive(Debug, Clone)]
pub struct Sheet {
    /// The module path exactly as `open` was given it.
    pub module: String,
    /// The item address the sheet holds — or `+` for an add sheet.
    pub address: String,
    /// The payload-store address of the bytes as materialized.
    pub original: String,
}

/// The desk: sheets in opening order. Order is the offer order.
#[derive(Debug, Clone)]
pub struct Desk {
    /// The sheets, oldest first.
    pub sheets: Vec<Sheet>,
}

#[crate::mutate("desk")]
impl Desk {
    /// Parse the bindings file — journal grammar, every row an `open`, every
    /// detail carrying its original's payload address. The file is
    /// machine-written: a line the grammar cannot read means a hand touched it.
    pub fn parse(text: &str) -> Result<Desk, String> {
        let mut sheets = Vec::new();
        for (n, line) in text.lines().enumerate() {
            let parsed = line
                .split_once(' ')
                .and_then(|(verb, rest)| rest.split_once(" — ").map(|(m, d)| (verb, m, d)));
            let Some(("open", module, detail)) = parsed else {
                return Err(format!(
                    "bundle desk: bindings line {} is not `open <module> — <address> @<orig>`: \
                     `{line}`",
                    n + 1
                ));
            };
            let (address, original) = split_address(detail);
            let Some(original) = original else {
                return Err(format!(
                    "bundle desk: bindings line {} carries no original address: `{line}`",
                    n + 1
                ));
            };
            sheets.push(Sheet {
                module: module.to_string(),
                address: address.to_string(),
                original: original.to_string(),
            });
        }
        Ok(Desk { sheets })
    }

    /// Render the bindings back to the file's text — `parse`'s inverse.
    pub fn render(&self) -> String {
        self.sheets
            .iter()
            .map(|sheet| {
                format!(
                    "open {} — {} @{}\n",
                    sheet.module, sheet.address, sheet.original
                )
            })
            .collect()
    }

    /// The sheet's file name on the desk — 1-based, stable across edits, never
    /// renamed by later openings or withdrawals.
    pub fn sheet_file(index: usize) -> String {
        format!("sheet-{}.rs", index + 1)
    }

    /// The sheet already holding an address, if any — `open` refuses a duplicate
    /// by pointing at the standing sheet instead of materializing a twin.
    pub fn holding(&self, module: &str, address: &str) -> Option<usize> {
        self.sheets
            .iter()
            .position(|sheet| sheet.module == module && sheet.address == address)
    }
}

#[cfg(test)]
mod probes {
    use super::Desk;

    /// The bindings grammar round-trips; a strange line and a missing original
    /// refuse by number — the file is machine-written, so either means a hand.
    #[test]
    fn the_bindings_round_trip_and_strange_lines_refuse() {
        let text = "open src/m.rs — Sort @0123456789abcdef\n\
                    open src/m.rs — + @fedcba9876543210\n";
        let desk = Desk::parse(text).expect("parses");
        assert_eq!(desk.sheets.len(), 2);
        assert_eq!(desk.sheets[1].address, "+");
        assert_eq!(desk.render(), text, "parse∘render is the identity");
        let strange = Desk::parse("edit src/m.rs — x @0123456789abcdef\n").unwrap_err();
        assert!(strange.contains("bindings line 1"), "{strange}");
        let bare = Desk::parse("open src/m.rs — Sort\n").unwrap_err();
        assert!(bare.contains("no original address"), "{bare}");
    }

    /// Sheet names are 1-based and stable, and `holding` finds the standing sheet
    /// for a module + address pair — the duplicate-open refusal's lookup.
    #[test]
    fn sheet_names_are_stable_and_holding_finds_the_standing_sheet() {
        assert_eq!(Desk::sheet_file(0), "sheet-1.rs");
        assert_eq!(Desk::sheet_file(2), "sheet-3.rs");
        let desk = Desk::parse(
            "open src/m.rs — Sort @0123456789abcdef\n\
             open src/other.rs — Sort @fedcba9876543210\n",
        )
        .unwrap();
        assert_eq!(desk.holding("src/m.rs", "Sort"), Some(0));
        assert_eq!(desk.holding("src/other.rs", "Sort"), Some(1));
        assert_eq!(desk.holding("src/m.rs", "Mode"), None);
    }
}
