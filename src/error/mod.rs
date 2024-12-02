use std::fmt;

use chrono::{DateTime, Local};
use nenyr::error::NenyrError;
use tracing::error;

/// Enum representing errors in the Galadriel system. It can be either a `CriticalError` or a `GeneralError`,
/// each wrapping a specific type of error information.
#[derive(Clone, PartialEq, Debug)]
pub enum GaladrielError {
    /// A critical error, often representing a severe issue in the system.
    CriticalError(GaladrielErrorType),
    /// A general error, usually representing non-fatal issues that can be handled more gracefully.
    GeneralError(GaladrielErrorType),
    NenyrError {
        start_time: DateTime<Local>,
        error: NenyrError,
    },
}

impl fmt::Display for GaladrielError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "TYPE: {:?}\nKIND: {:?}\nACTION: {:?}\nMESSAGE: {}",
                self.get_type(),
                self.get_kind(),
                self.get_action(),
                self.get_message()
            )
        )
    }
}

impl GaladrielError {
    /// Checks if the error is a critical error.
    ///
    /// # Returns
    /// `true` if the error is a `CriticalError`, otherwise `false`.
    pub fn is_critical(&self) -> bool {
        matches!(self, GaladrielError::CriticalError(_))
    }

    /// Checks if the error is a general error.
    ///
    /// # Returns
    /// `true` if the error is a `GeneralError`, otherwise `false`.
    pub fn is_general(&self) -> bool {
        matches!(self, GaladrielError::GeneralError(_))
    }

    /// Returns the action associated with this error.
    ///
    /// # Returns
    /// An `ErrorAction` that describes the next steps for the error.
    pub fn get_action(&self) -> ErrorAction {
        match self {
            GaladrielError::CriticalError(err) => err.get_action(),
            GaladrielError::GeneralError(err) => err.get_action(),
            GaladrielError::NenyrError { .. } => ErrorAction::Fix,
        }
    }

    /// Retrieves the error message associated with the error.
    ///
    /// # Returns
    /// A `String` representing the error message.
    pub fn get_message(&self) -> String {
        match self {
            GaladrielError::CriticalError(err) => err.get_message(),
            GaladrielError::GeneralError(err) => err.get_message(),
            GaladrielError::NenyrError { error, .. } => error.get_error_message(),
        }
    }

    /// Retrieves the kind of the error.
    ///
    /// # Returns
    /// An `ErrorKind` indicating the specific category of the error.
    pub fn get_kind(&self) -> ErrorKind {
        match self {
            GaladrielError::CriticalError(err) => err.get_kind(),
            GaladrielError::GeneralError(err) => err.get_kind(),
            GaladrielError::NenyrError { .. } => ErrorKind::NenyrError,
        }
    }

    /// Retrieves the type of the error.
    ///
    /// # Returns
    /// An `ErrorType` representing the overarching type of error.
    pub fn get_type(&self) -> ErrorType {
        match self {
            GaladrielError::CriticalError(err) => err.get_type(),
            GaladrielError::GeneralError(err) => err.get_type(),
            GaladrielError::NenyrError { .. } => ErrorType::OtherError,
        }
    }

    /// ==============================================================================================================
    ///
    /// The following `raise_*` functions may appear repetitive at first glance,
    /// but each serves a distinct purpose in enhancing the readability, interpretability,
    /// and maintainability of the Galadriel CSS codebase.
    ///
    /// ==============================================================================================================
    ///
    /// # Purpose
    /// Each function, such as `raise_critical_pipeline_error` or `raise_general_observer_error`,
    /// is designed to encapsulate a specific type of error—both in severity (critical vs. general)
    /// and in its contextual source (e.g., pipeline, observer, interface).
    /// By using these specialized functions directly in the code, developers gain an immediate
    /// understanding of both the severity and the nature of the error without needing to
    /// inspect internal error messages or additional metadata.
    ///
    /// For instance:
    ///
    /// fn some_function() -> GaladrielResult<()> {
    ///     Err(GaladrielError::raise_critical_pipeline_error(kind, message, action))
    /// }
    ///
    /// In this example, it is immediately clear that `some_function` could return
    /// a critical error related to the pipeline. This level of specificity improves
    /// both readability and the efficiency of debugging processes, making error
    /// handling more descriptive and contextually meaningful.
    ///
    /// # Rationale
    /// These functions avoid the need for additional context or inspection at the point of use.
    /// Instead, they leverage Rust's type system and function naming to provide
    /// a semantically rich way of understanding potential errors. This structured approach
    /// to error handling is especially beneficial in a complex system like Galadriel CSS,
    /// where various components (such as pipeline, observer, interface, runtime)
    /// might encounter distinct and meaningful errors.
    ///
    /// # Future-proofing
    /// Additionally, this design offers scalability. Should new error types or actions
    /// be required in the future, they can be integrated in a modular, consistent manner.
    /// Any new functions can follow this naming convention, helping to maintain clarity
    /// and organization throughout the codebase over time.
    /// ==============================================================================================================

    /// Raises a critical error related to a pipeline issue.
    ///
    /// This function creates a `CriticalError` of type `PipelineError`.
    ///
    /// # Parameters
    /// - `kind`: The specific kind of error.
    /// - `message`: A message describing the error.
    /// - `action`: The action to be taken for this error.
    ///
    /// # Returns
    /// A `GaladrielError::CriticalError` with the associated `PipelineError` details.
    pub fn raise_critical_pipeline_error(
        kind: ErrorKind,
        message: &str,
        action: ErrorAction,
    ) -> Self {
        error!(
            "Critical Pipeline Error raised. Kind: {:?}, Message: '{}', Action: {:?}",
            kind, message, action
        );

        GaladrielError::CriticalError(GaladrielErrorType::PipelineError {
            kind,
            message: message.to_string(),
            action,
        })
    }

    /// Raises a critical error related to an observer issue.
    ///
    /// This function creates a `CriticalError` of type `ObserverError`.
    pub fn raise_critical_observer_error(
        kind: ErrorKind,
        message: &str,
        action: ErrorAction,
    ) -> Self {
        error!(
            "Critical Observer Error raised. Kind: {:?}, Message: '{}', Action: {:?}",
            kind, message, action
        );

        GaladrielError::CriticalError(GaladrielErrorType::ObserverError {
            kind,
            message: message.to_string(),
            action,
        })
    }

    /// Raises a critical error related to an interface issue.
    ///
    /// This function creates a `CriticalError` of type `InterfaceError`.
    pub fn raise_critical_interface_error(
        kind: ErrorKind,
        message: &str,
        action: ErrorAction,
    ) -> Self {
        error!(
            "Critical Interface Error raised. Kind: {:?}, Message: '{}', Action: {:?}",
            kind, message, action
        );

        GaladrielError::CriticalError(GaladrielErrorType::InterfaceError {
            kind,
            message: message.to_string(),
            action,
        })
    }

    /// Raises a critical error related to a runtime issue.
    ///
    /// This function creates a `CriticalError` of type `RuntimeError`.
    pub fn raise_critical_runtime_error(
        kind: ErrorKind,
        message: &str,
        action: ErrorAction,
    ) -> Self {
        error!(
            "Critical Galadriel CSS Runtime Error raised. Kind: {:?}, Message: '{}', Action: {:?}",
            kind, message, action
        );

        GaladrielError::CriticalError(GaladrielErrorType::RuntimeError {
            kind,
            message: message.to_string(),
            action,
        })
    }

    /// Raises a critical error related to an unspecified issue.
    ///
    /// This function creates a `CriticalError` of type `OtherError`.
    pub fn raise_critical_other_error(kind: ErrorKind, message: &str, action: ErrorAction) -> Self {
        error!(
            "Critical Error raised. Kind: {:?}, Message: '{}', Action: {:?}",
            kind, message, action
        );

        GaladrielError::CriticalError(GaladrielErrorType::OtherError {
            kind,
            message: message.to_string(),
            action,
        })
    }

    /// Raises a general error related to a pipeline issue.
    ///
    /// This function creates a `GeneralError` of type `PipelineError`.
    pub fn raise_general_pipeline_error(
        kind: ErrorKind,
        message: &str,
        action: ErrorAction,
    ) -> Self {
        error!(
            "General Pipeline Error raised. Kind: {:?}, Message: '{}', Action: {:?}",
            kind, message, action
        );

        GaladrielError::GeneralError(GaladrielErrorType::PipelineError {
            kind,
            message: message.to_string(),
            action,
        })
    }

    /// Raises a general error related to an observer issue.
    ///
    /// This function creates a `GeneralError` of type `ObserverError`.
    pub fn raise_general_observer_error(
        kind: ErrorKind,
        message: &str,
        action: ErrorAction,
    ) -> Self {
        error!(
            "General Observer Error raised. Kind: {:?}, Message: '{}', Action: {:?}",
            kind, message, action
        );

        GaladrielError::GeneralError(GaladrielErrorType::ObserverError {
            kind,
            message: message.to_string(),
            action,
        })
    }

    /// Raises a general error related to an interface issue.
    ///
    /// This function creates a `GeneralError` of type `InterfaceError`.
    pub fn raise_general_interface_error(
        kind: ErrorKind,
        message: &str,
        action: ErrorAction,
    ) -> Self {
        error!(
            "General Interface Error raised. Kind: {:?}, Message: '{}', Action: {:?}",
            kind, message, action
        );

        GaladrielError::GeneralError(GaladrielErrorType::InterfaceError {
            kind,
            message: message.to_string(),
            action,
        })
    }

    /// Raises a general error related to a runtime issue.
    ///
    /// This function creates a `GeneralError` of type `RuntimeError`.
    pub fn raise_general_runtime_error(
        kind: ErrorKind,
        message: &str,
        action: ErrorAction,
    ) -> Self {
        error!(
            "General Galadriel CSS Runtime Error raised. Kind: {:?}, Message: '{}', Action: {:?}",
            kind, message, action
        );

        GaladrielError::GeneralError(GaladrielErrorType::RuntimeError {
            kind,
            message: message.to_string(),
            action,
        })
    }

    /// Raises a general error related to an unspecified issue.
    ///
    /// This function creates a `GeneralError` of type `OtherError`.
    pub fn raise_general_other_error(kind: ErrorKind, message: &str, action: ErrorAction) -> Self {
        error!(
            "General Error raised. Kind: {:?}, Message: '{}', Action: {:?}",
            kind, message, action
        );

        GaladrielError::GeneralError(GaladrielErrorType::OtherError {
            kind,
            message: message.to_string(),
            action,
        })
    }

    pub fn raise_nenyr_error(start_time: DateTime<Local>, error: NenyrError) -> Self {
        GaladrielError::NenyrError { start_time, error }
    }
}

/// The `GaladrielErrorType` enum represents various types of errors that can occur in the application.
/// Each variant contains information about the error kind, message, and action to take upon encountering the error.
#[derive(Clone, PartialEq, Debug)]
pub enum GaladrielErrorType {
    /// Represents an error that occurred in the server pipeline process.
    /// It includes the error kind, message, and the corresponding action.
    PipelineError {
        kind: ErrorKind,
        message: String,
        action: ErrorAction,
    },
    /// Represents an error that occurred in the observer component.
    /// It includes the error kind, message, and the corresponding action.
    ObserverError {
        kind: ErrorKind,
        message: String,
        action: ErrorAction,
    },
    /// Represents an error that occurred in the interface layer.
    /// It includes the error kind, message, and the corresponding action.
    InterfaceError {
        kind: ErrorKind,
        message: String,
        action: ErrorAction,
    },
    /// Represents an error that occurred during runtime.
    /// It includes the error kind, message, and the corresponding action.
    RuntimeError {
        kind: ErrorKind,
        message: String,
        action: ErrorAction,
    },
    /// Represents any other type of error that doesn't fit the above categories.
    /// It includes the error kind, message, and the corresponding action.
    OtherError {
        kind: ErrorKind,
        message: String,
        action: ErrorAction,
    },
}

impl GaladrielErrorType {
    /// Retrieves the action to take for this error.
    /// This is useful to determine whether to restart, log, ignore, or notify on the error.
    ///
    /// # Returns
    /// Returns the `ErrorAction` associated with this error type.
    pub fn get_action(&self) -> ErrorAction {
        match self {
            GaladrielErrorType::PipelineError { action, .. } => action.clone(),
            GaladrielErrorType::ObserverError { action, .. } => action.clone(),
            GaladrielErrorType::InterfaceError { action, .. } => action.clone(),
            GaladrielErrorType::RuntimeError { action, .. } => action.clone(),
            GaladrielErrorType::OtherError { action, .. } => action.clone(),
        }
    }

    /// Retrieves the message describing the error.
    ///
    /// # Returns
    /// Returns the error message as a `String`.
    pub fn get_message(&self) -> String {
        match self {
            GaladrielErrorType::PipelineError { message, .. } => message.clone(),
            GaladrielErrorType::ObserverError { message, .. } => message.clone(),
            GaladrielErrorType::InterfaceError { message, .. } => message.clone(),
            GaladrielErrorType::RuntimeError { message, .. } => message.clone(),
            GaladrielErrorType::OtherError { message, .. } => message.clone(),
        }
    }

    /// Retrieves the kind of error that occurred.
    ///
    /// # Returns
    /// Returns the `ErrorKind` associated with this error type.
    pub fn get_kind(&self) -> ErrorKind {
        match self {
            GaladrielErrorType::PipelineError { kind, .. } => kind.clone(),
            GaladrielErrorType::ObserverError { kind, .. } => kind.clone(),
            GaladrielErrorType::InterfaceError { kind, .. } => kind.clone(),
            GaladrielErrorType::RuntimeError { kind, .. } => kind.clone(),
            GaladrielErrorType::OtherError { kind, .. } => kind.clone(),
        }
    }

    /// Retrieves the type of error (e.g., PipelineError, ObserverError, etc.).
    ///
    /// # Returns
    /// Returns the `ErrorType` variant representing the error type.
    pub fn get_type(&self) -> ErrorType {
        match self {
            GaladrielErrorType::PipelineError { .. } => ErrorType::PipelineError,
            GaladrielErrorType::ObserverError { .. } => ErrorType::ObserverError,
            GaladrielErrorType::InterfaceError { .. } => ErrorType::InterfaceError,
            GaladrielErrorType::RuntimeError { .. } => ErrorType::RuntimeError,
            GaladrielErrorType::OtherError { .. } => ErrorType::OtherError,
        }
    }
}

/// The `ErrorAction` enum defines possible actions to take when an error occurs.
/// This determines how the error should be handled.
#[derive(Clone, PartialEq, Debug)]
pub enum ErrorAction {
    /// Restart the operation that caused the error.
    Restart,
    /// Ignore the error and continue without intervention.
    Ignore,
    /// Notify the user or system administrator of the error.
    Notify,
    Exit,
    Fix,
}

/// The `ErrorType` enum categorizes errors into distinct types for easier identification.
/// This helps to understand what layer of the system the error originated from.
#[derive(Clone, PartialEq, Debug)]
pub enum ErrorType {
    PipelineError,
    ObserverError,
    InterfaceError,
    RuntimeError,
    OtherError,
}

/// The `ErrorKind` enum defines the specific kind of error that occurred.
/// It helps to provide more granular details about the nature of the error.
#[derive(Clone, PartialEq, Debug)]
pub enum ErrorKind {
    CurrentWorkingDirRetrievalFailed,
    MissingGaladrielModeError,
    InvalidGaladrielModeError,
    TracingSubscriberInitializationFailed,
    ProcessInitializationFailed,
    ConfigFileReadError,
    ConfigFileParsingError,
    ExcludeMatcherCreationError,
    ExcludeMatcherBuildFailed,
    TerminalRawModeActivationFailed,
    TerminalRawModeDeactivationFailed,
    EnterTerminalAltScreenMouseCaptureFailed,
    LeaveTerminalAltScreenMouseCaptureFailed,
    TerminalCursorHideFailed,
    TerminalCursorUnhideFailed,
    TerminalClearScreenFailed,
    TerminalWidgetRenderingError,
    TerminalInitializationFailed,
    TerminalEventReceiveFailed,
    SocketAddressBindingError,
    ServerEventReceiveFailed,
    ServerPortRegistrationFailed,
    ServerPortWriteError,
    ServerPortRemovalFailed,
    ServerLocalAddrFetchFailed,
    NotificationSendError,
    ServerSyncAcceptFailed,
    ObserverEventReceiveFailed,
    AsyncDebouncerCreationFailed,
    DebouncerWatchFailed,
    AsyncDebouncerWatchError,
    RequestTokenInvalid,
    MissingRequestTokens,
    UnsupportedRequestToken,
    ClientResponseError,
    ConnectionInitializationError,
    ConnectionTerminationError,
    DebouncedEventError,
    AsyncWatcherInitializationFailed,
    NenyrSyntaxIntegrationFailed,
    NenyrSyntaxMissing,
    NenyrSyntaxHighlightingError,
    GaladrielConfigOpenFileError,
    GaladrielConfigSerdeSerializationError,
    GaladrielConfigFileWriteError,
    NenyrError,
    FileReadMaxRetriesExceeded,
    FileReadFailed,
    TaskFailure,
    AccessDeniedToStylitronAST,
    AccessDeniedToClassinatorAST,
    ContextNameConflict,
    Other,
}

#[cfg(test)]
mod tests {
    use crate::error::{ErrorAction, ErrorKind, ErrorType, GaladrielError};

    #[test]
    fn test_assert_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<GaladrielError>();
        assert_sync::<GaladrielError>();
    }

    #[test]
    fn test_is_critical() {
        let critical_error = GaladrielError::raise_critical_pipeline_error(
            ErrorKind::Other,
            "Critical error occurred",
            ErrorAction::Restart,
        );
        assert!(critical_error.is_critical());

        let general_error = GaladrielError::raise_general_pipeline_error(
            ErrorKind::Other,
            "General error occurred",
            ErrorAction::Notify,
        );
        assert!(!general_error.is_critical());
    }

    #[test]
    fn test_is_general() {
        let critical_error = GaladrielError::raise_critical_pipeline_error(
            ErrorKind::Other,
            "Critical error occurred",
            ErrorAction::Restart,
        );
        assert!(!critical_error.is_general());

        let general_error = GaladrielError::raise_general_pipeline_error(
            ErrorKind::Other,
            "General error occurred",
            ErrorAction::Notify,
        );
        assert!(general_error.is_general());
    }

    #[test]
    fn test_get_message() {
        let critical_error = GaladrielError::raise_critical_pipeline_error(
            ErrorKind::Other,
            "Critical error occurred",
            ErrorAction::Restart,
        );
        assert_eq!(critical_error.get_message(), "Critical error occurred");

        let general_error = GaladrielError::raise_general_pipeline_error(
            ErrorKind::Other,
            "General error occurred",
            ErrorAction::Notify,
        );
        assert_eq!(general_error.get_message(), "General error occurred");
    }

    #[test]
    fn test_get_kind() {
        let critical_error = GaladrielError::raise_critical_pipeline_error(
            ErrorKind::Other,
            "Critical error occurred",
            ErrorAction::Restart,
        );
        assert_eq!(critical_error.get_kind(), ErrorKind::Other);

        let general_error = GaladrielError::raise_general_pipeline_error(
            ErrorKind::Other,
            "General error occurred",
            ErrorAction::Notify,
        );
        assert_eq!(general_error.get_kind(), ErrorKind::Other);
    }

    #[test]
    fn test_get_type() {
        let critical_error = GaladrielError::raise_critical_pipeline_error(
            ErrorKind::Other,
            "Critical error occurred",
            ErrorAction::Restart,
        );
        assert_eq!(critical_error.get_type(), ErrorType::PipelineError);

        let general_error = GaladrielError::raise_general_pipeline_error(
            ErrorKind::Other,
            "General error occurred",
            ErrorAction::Notify,
        );
        assert_eq!(general_error.get_type(), ErrorType::PipelineError);
    }

    #[test]
    fn test_get_action() {
        let critical_error = GaladrielError::raise_critical_pipeline_error(
            ErrorKind::Other,
            "Critical error occurred",
            ErrorAction::Restart,
        );
        assert_eq!(critical_error.get_action(), ErrorAction::Restart);

        let general_error = GaladrielError::raise_general_pipeline_error(
            ErrorKind::Other,
            "General error occurred",
            ErrorAction::Notify,
        );
        assert_eq!(general_error.get_action(), ErrorAction::Notify);
    }

    #[test]
    fn test_display() {
        let critical_error = GaladrielError::raise_critical_pipeline_error(
            ErrorKind::Other,
            "Critical error occurred",
            ErrorAction::Restart,
        );
        assert_eq!(
            format!("{}", critical_error),
            "TYPE: PipelineError\nKIND: Other\nACTION: Restart\nMESSAGE: Critical error occurred"
        );

        let general_error = GaladrielError::raise_general_pipeline_error(
            ErrorKind::Other,
            "General error occurred",
            ErrorAction::Notify,
        );
        assert_eq!(
            format!("{}", general_error),
            "TYPE: PipelineError\nKIND: Other\nACTION: Notify\nMESSAGE: General error occurred"
        );
    }
}
