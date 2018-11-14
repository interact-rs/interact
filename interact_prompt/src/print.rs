use ansi_term::Color;
use interact::NodeTree;
use std::collections::HashMap;
use std::sync::atomic::Ordering;

pub struct NodePrinterSettings {
    pub max_line_length: u16,
    pub indent_step: u16,
}

struct Printer<'a> {
    settings: &'a NodePrinterSettings,
    indent: usize,
    indent_string: String,
    line_used: usize,
    item_linebreak: bool,
    seen: HashMap<usize, usize>,
    seen_idx: usize,
}

impl<'a> Printer<'a> {
    fn write(&mut self, s: &str) {
        if self.line_used == 0 {
            print!("{}", self.indent_string);
        }
        print!("{}", s);
        self.line_used += s.len();
    }

    fn end_line(&mut self) {
        println!();
        self.line_used = 0;
    }

    fn up_indent(&mut self) {
        self.indent += self.settings.indent_step as usize;
        self.indent_string = " ".repeat(self.indent);
    }

    fn down_indent(&mut self) {
        self.indent -= self.settings.indent_step as usize;
        self.indent_string = " ".repeat(self.indent);
    }

    fn inner_pretty_print(&mut self, elem: &NodeTree) {
        use interact::NodeInfo::*;

        let mut repeated_idx = None;
        if let Some(ptr_meta) = &elem.meta {
            use std::collections::hash_map::Entry;

            use std::borrow::Borrow;
            let b = (*ptr_meta.0).borrow();
            let arc_ptr = (b as *const _) as usize;

            if let Repeated = &elem.info {
                repeated_idx = Some(match self.seen.entry(arc_ptr) {
                    Entry::Occupied(entry) => *entry.get(),
                    Entry::Vacant(entry) => {
                        let idx = self.seen_idx;
                        entry.insert(idx);
                        self.seen_idx += 1;
                        idx
                    }
                });
            } else {
                let nr_refs = ptr_meta.0.load(Ordering::Relaxed);

                if nr_refs >= 2 {
                    let seen_idx = match self.seen.entry(arc_ptr) {
                        Entry::Occupied(entry) => *entry.get(),
                        Entry::Vacant(entry) => {
                            let idx = self.seen_idx;
                            entry.insert(idx);
                            self.seen_idx += 1;
                            idx
                        }
                    };

                    self.write(&format!(
                        "{}",
                        Color::Green.paint(format!("[#{}] ", seen_idx))
                    ));
                }
            }
        }

        match &elem.info {
            Grouped(prefix, sub, end) => {
                self.write(&format!("{}", Color::Cyan.paint(format!("{}", prefix))));
                let item_linebreak = self.item_linebreak;
                let mut indented = false;

                let has_space = if let Delimited(_, ref v) = &sub.info {
                    !v.is_empty() && prefix != &'('
                } else {
                    true
                };

                if self.indent + elem.size > self.settings.max_line_length as usize {
                    self.item_linebreak = true;
                    self.up_indent();
                    self.end_line();
                    indented = true;
                } else {
                    self.item_linebreak = false;
                    if has_space {
                        self.write(&" ");
                    }
                }

                self.inner_pretty_print(&sub);

                self.item_linebreak = item_linebreak;

                if indented {
                    self.down_indent();
                    self.end_line();
                } else if has_space {
                    self.write(&" ");
                }

                self.write(&format!("{}", Color::Cyan.paint(&format!("{}", end))));
            }
            Delimited(delimiter, v) => {
                for (idx, i) in v.iter().enumerate() {
                    if idx > 0 {
                        if self.item_linebreak {
                            self.write(&format!("{}", delimiter));
                            self.end_line();
                        } else {
                            self.write(&format!("{} ", delimiter));
                        }
                    }

                    self.inner_pretty_print(&i);
                }
            }
            Tuple(key, sep, value) => {
                self.inner_pretty_print(key);
                self.write(&format!("{} ", Color::Cyan.paint(*sep)));
                self.inner_pretty_print(value);
            }
            Named(item, next) => {
                self.inner_pretty_print(item);
                self.write(&" ");
                self.inner_pretty_print(next);
            }
            Limited => {
                self.write(&format!("{}", Color::Yellow.bold().paint("...<<<>>>...")));
            }
            Hole(_) => {
                self.write(&format!(
                    "{}",
                    Color::Red.bold().paint(format!("< - hole - >"))
                ));
            }
            Repeated => {
                self.write(&format!(
                    "{}",
                    Color::Green.paint(format!("[#{}]", repeated_idx.unwrap_or(0)))
                ));
            }
            BorrowedMut => {
                self.write(&format!(
                    "{}",
                    Color::Red.bold().paint(format!("< borrowed-mut >"))
                ));
            }
            Locked => {
                self.write(&format!(
                    "{}",
                    Color::Red.bold().paint(format!("< locked >"))
                ));
            }
            Leaf(s) => {
                self.write(s);
            }
        };
    }

    fn inner_pretty_end(&mut self, elem: &NodeTree) {
        self.inner_pretty_print(elem);

        if self.line_used > 0 {
            self.end_line();
        }
    }
}

pub fn pretty_format(elem: &NodeTree, settings: &NodePrinterSettings) {
    let mut state = Printer {
        settings,
        indent: 0,
        line_used: 0,
        item_linebreak: true,
        indent_string: String::from(""),
        seen: HashMap::new(),
        seen_idx: 1,
    };

    state.inner_pretty_end(elem);
}
