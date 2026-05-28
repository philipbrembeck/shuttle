use crate::menu_model::MenuEntry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeMenuSpec {
    Menu {
        title: String,
        children: Vec<NativeMenuSpec>,
    },
    Command {
        title: String,
        enabled: bool,
    },
    Separator,
}

pub fn build_spec(entries: &[MenuEntry]) -> Vec<NativeMenuSpec> {
    entries
        .iter()
        .map(|entry| match entry {
            MenuEntry::Menu {
                title, children, ..
            } => NativeMenuSpec::Menu {
                title: title.clone(),
                children: build_spec(children),
            },
            MenuEntry::Command { title, .. } => NativeMenuSpec::Command {
                title: title.clone(),
                enabled: true,
            },
            MenuEntry::Disabled { title } => NativeMenuSpec::Command {
                title: title.clone(),
                enabled: false,
            },
            MenuEntry::Separator => NativeMenuSpec::Separator,
        })
        .collect()
}

#[cfg(target_os = "macos")]
pub fn install_status_menu(entries: &[MenuEntry]) -> Vec<NativeMenuSpec> {
    // The objc2 AppKit bridge will consume this spec when the callback holder is
    // introduced. Keeping this conversion isolated lets the rest of the port test
    // native menu shape without depending on an active NSApplication in unit tests.
    build_spec(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::CommandHost;
    use std::collections::BTreeMap;

    #[test]
    fn converts_menu_model_to_native_spec() {
        let entries = vec![
            MenuEntry::Disabled {
                title: "Error parsing config".into(),
            },
            MenuEntry::Separator,
            MenuEntry::Command {
                title: "Prod".into(),
                command: CommandHost {
                    cmd: "ssh prod".into(),
                    name: "Prod".into(),
                    in_terminal: None,
                    theme: None,
                    title: None,
                    backend: None,
                    strategy: None,
                    extra: BTreeMap::new(),
                },
                separator_after: false,
            },
        ];
        assert_eq!(
            build_spec(&entries),
            vec![
                NativeMenuSpec::Command {
                    title: "Error parsing config".into(),
                    enabled: false
                },
                NativeMenuSpec::Separator,
                NativeMenuSpec::Command {
                    title: "Prod".into(),
                    enabled: true
                },
            ]
        );
    }
}
