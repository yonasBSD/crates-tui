use serde::{Deserialize, Serialize};
use strum::Display;

use crate::app::Mode;

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    KeyRefresh,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    Init { query: Option<String> },
    Refresh,
    NextTab,
    PreviousTab,
    ShowErrorPopup(String),
    ShowInfoPopup(String),
    ClosePopup,
    Help,
    GetCrates,
    SwitchMode(Mode),
    SwitchToLastMode,
    IncrementPage,
    DecrementPage,
    NextSummaryMode,
    PreviousSummaryMode,
    ToggleSortBy { reload: bool, forward: bool },
    ScrollBottom,
    ScrollTop,
    ScrollDown,
    ScrollUp,
    ScrollCrateInfoDown,
    ScrollCrateInfoUp,
    ScrollSearchResultsDown,
    ScrollSearchResultsUp,
    SubmitSearch,
    UpdateSearchTableResults,
    UpdateSummary,
    UpdateCurrentSelectionCrateInfo,
    UpdateCurrentSelectionSummary,
    ReloadData,
    ToggleShowCrateInfo,
    StoreTotalNumberOfCrates(u64),
    ClearTaskDetailsHandle(String),
    CopyCargoAddCommandToClipboard,
    OpenDocsUrlInBrowser,
    OpenCratesIOUrlInBrowser,
    ShowFullCrateInfo,
}
