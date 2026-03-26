// FormAction and SelectionResult — shared event vocabulary for form components.
//
// Design: These enums carry zero renderer-specific types so they can cross
// the boundary between fs-core (abstract) and any backend (fs-tui, fs-gui …).
// All form nodes return FormAction from handle_key / handle_mouse.
// All selection popups return SelectionResult from their input handlers.

// ── FormAction ────────────────────────────────────────────────────────────────

/// What a form node returns after handling a keyboard or mouse event.
///
/// The outer handler (events.rs in the host app) reacts to these without
/// knowing field internals.
#[derive(Debug, Clone, PartialEq)]
pub enum FormAction {
    /// Event consumed internally; no outer action needed.
    Consumed,
    /// Value was modified (triggers the form's `on_change` hook).
    ValueChanged,
    /// Move focus to the next node in the current tab.
    FocusNext,
    /// Move focus to the previous node in the current tab.
    FocusPrev,
    /// Value was modified AND focus should advance to the next field.
    ///
    /// Used by SelectInput / MultiSelectInput after popup confirmation so that
    /// `on_change` fires and the cursor advances in one step.
    AcceptAndNext,
    /// Advance to the next form tab (Ctrl+Right).
    TabNext,
    /// Go back to the previous form tab (Ctrl+Left).
    TabPrev,
    /// Attempt to submit the form (Ctrl+S).
    Submit,
    /// Close the form / pop the current screen (Esc).
    Cancel,
    /// Toggle the UI language (L/l key outside text input).
    LangToggle,
    /// Quit the application (Ctrl+C — handled before node dispatch).
    Quit,
    /// Event not handled by this node; fall through to the outer handler.
    Unhandled,
}

// ── SelectionResult ───────────────────────────────────────────────────────────

/// What a selection popup returns after handling a key or mouse event.
///
/// Returned by `SelectionPopup::handle_key` and `SelectionPopup::handle_mouse`.
/// The owning node (SelectInputNode / MultiSelectInputNode) maps this to a
/// `FormAction` for the outer handler.
#[derive(Debug, PartialEq)]
pub enum SelectionResult {
    /// Key handled internally — no value change.
    Consumed,
    /// Confirmed: single value selected (for SingleMode).
    Accepted(String),
    /// Confirmed: multiple values selected (for MultiMode).
    AcceptedMulti(Vec<String>),
    /// Popup closed without confirming (Esc / click outside).
    Rejected,
    /// Key not for the popup — fall through.
    Unhandled,
}
