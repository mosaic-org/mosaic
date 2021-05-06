//! Mapping of inputs to sequences of actions.
use std::collections::HashMap;

use super::actions::Action;
use super::config;

use serde::Deserialize;
use strum::IntoEnumIterator;
use zellij_tile::data::*;

/// Used in the config struct
#[derive(Clone, Debug, PartialEq)]
pub struct Keybinds(HashMap<InputMode, ModeKeybinds>);
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModeKeybinds(HashMap<Key, Vec<Action>>);

/// Intermediate struct used for deserialisation
/// Used in the config file.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct KeybindsFromYaml {
    #[serde(flatten)]
    keybinds: HashMap<InputMode, Vec<KeyActionUnbind>>,
    #[serde(default)]
    unbind: Unbind,
}

/// Intermediate enum used for deserialisation
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
enum KeyActionUnbind {
    KeyAction(KeyActionFromYaml),
    // TODO: use the enum
    //Unbind(UnbindFromYaml),
}

/// Intermediate struct used for deserialisation
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct KeyActionFromYaml {
    action: Vec<Action>,
    key: Vec<Key>,
}

/// Intermediate struct used for deserialisation
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
struct UnbindFromYaml {
    unbind: Unbind,
}

/// List of keys, for which to disable their respective default actions
/// `All` is a catch all, and will disable the default actions for all keys.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(untagged)]
enum Unbind {
    All(bool),
    // TODO@a-kenji: use the enum
    //Keys(Vec<Key>),
}

impl Default for Keybinds {
    fn default() -> Keybinds {
        config::Config::from_default_assets()
            .expect("Keybinds from default assets Error")
            .keybinds
    }
}

impl Keybinds {
    pub fn new() -> Keybinds {
        Keybinds(HashMap::<InputMode, ModeKeybinds>::new())
    }

    pub fn get_default_keybinds_with_config(from_yaml: Option<KeybindsFromYaml>) -> Keybinds {
        let default_keybinds = match from_yaml.clone() {
            Some(keybinds) => match keybinds.unbind {
                Unbind::All(true) => Keybinds::new(),
                Unbind::All(false) => Keybinds::default(),
            },
            None => Keybinds::default(),
        };

        if let Some(keybinds) = from_yaml {
            default_keybinds.merge_keybinds(Keybinds::from(keybinds))
        } else {
            default_keybinds
        }
    }

    /// Merges two Keybinds structs into one Keybinds struct
    /// `other` overrides the ModeKeybinds of `self`.
    fn merge_keybinds(&self, other: Keybinds) -> Keybinds {
        let mut keybinds = Keybinds::new();

        for mode in InputMode::iter() {
            let mut mode_keybinds = ModeKeybinds::new();
            if let Some(keybind) = self.0.get(&mode) {
                mode_keybinds.0.extend(keybind.0.clone());
            };
            if let Some(keybind) = other.0.get(&mode) {
                mode_keybinds.0.extend(keybind.0.clone());
            }
            if !mode_keybinds.0.is_empty() {
                keybinds.0.insert(mode, mode_keybinds);
            }
        }
        keybinds
    }


    /// Converts a [`Key`] terminal event to a sequence of [`Action`]s according to the current
    /// [`InputMode`] and [`Keybinds`].
    pub fn key_to_actions(
        key: &Key,
        input: Vec<u8>,
        mode: &InputMode,
        keybinds: &Keybinds,
    ) -> Vec<Action> {
        let mode_keybind_or_action = |action: Action| {
            keybinds
                .0
                .get(mode)
                .unwrap_or_else(|| unreachable!("Unrecognized mode: {:?}", mode))
                .0
                .get(key)
                .cloned()
                .unwrap_or_else(|| vec![action])
        };
        match *mode {
            InputMode::Normal | InputMode::Locked => mode_keybind_or_action(Action::Write(input)),
            InputMode::RenameTab => mode_keybind_or_action(Action::TabNameInput(input)),
            InputMode::SearchTab => mode_keybind_or_action(Action::SearchTabInput(input)),
            _ => mode_keybind_or_action(Action::NoOp),
        }
    }
}

impl ModeKeybinds {
    fn new() -> ModeKeybinds {
        ModeKeybinds(HashMap::<Key, Vec<Action>>::new())
    }

    /// Merges `self` with `other`, if keys are the same, `other` overwrites.
    fn merge(self, other: ModeKeybinds) -> ModeKeybinds {
        let mut merged = self;
        merged.0.extend(other.0);
        merged
    }
}

impl From<KeybindsFromYaml> for Keybinds {
    fn from(keybinds_from_yaml: KeybindsFromYaml) -> Keybinds {
        let mut keybinds = Keybinds::new();

        for mode in InputMode::iter() {
            let mut mode_keybinds = ModeKeybinds::new();
            for key_action in keybinds_from_yaml.keybinds.get(&mode).iter() {
                for keybind in key_action.iter() {
                    mode_keybinds = mode_keybinds.merge(ModeKeybinds::from(keybind.clone()));
                }
            }
            keybinds.0.insert(mode, mode_keybinds);
        }
        keybinds
    }
}

/// For each `Key` assigned to `Action`s,
/// map the `Action`s to the key
impl From<KeyActionFromYaml> for ModeKeybinds {
    fn from(key_action: KeyActionFromYaml) -> ModeKeybinds {
        let keys = key_action.key;
        let actions = key_action.action;

        ModeKeybinds(
            keys.into_iter()
                .map(|k| (k, actions.clone()))
                .collect::<HashMap<Key, Vec<Action>>>(),
        )
    }
}

// Currently an enum for future use
impl From<KeyActionUnbind> for ModeKeybinds {
    fn from(key_action_unbind: KeyActionUnbind) -> ModeKeybinds {
        match key_action_unbind {
            KeyActionUnbind::KeyAction(key_action) => ModeKeybinds::from(key_action),
        }
    }
}

impl Default for Unbind {
    fn default() -> Unbind {
        Unbind::All(false)
    }
}

// The unit test location.
#[cfg(test)]
#[path = "./unit/keybinds_test.rs"]
mod keybinds_test;
