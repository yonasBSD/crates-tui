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
    ShowErrorPopup(String),
    ShowInfoPopup(String),
    ClosePopup,
    Help,
    GetCrates,
    EnterSearchInsertMode,
    EnterFilterInsertMode,
    HandleFilterPromptChange,
    IncrementPage,
    DecrementPage,
    ToggleSortBy,
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
