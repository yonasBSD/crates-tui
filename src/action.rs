use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    KeyRefresh,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    Refresh,
    Error(String),
    Info(String),
    ClosePopup,
    Help,
    GetCrates,
    EnterSearchInsertMode,
    EnterFilterInsertMode,
    IncrementPage,
    DecrementPage,
    EnterNormal,
    ScrollBottom,
    ScrollTop,
    ScrollDown,
    ScrollUp,
    SubmitSearch,
    UpdateCurrentSelectionCrateInfo,
    ReloadData,
    ToggleShowHelp,
    ToggleShowCrateInfo,
    StoreTotalNumberOfCrates(u64),
}
