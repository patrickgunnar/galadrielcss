/// Represents the type of request a client can make.
#[derive(Clone, PartialEq, Debug)]
pub enum RequestType {
    /// Request to collect the list of CSS classes.
    CollectClassList,
    /// Request to fetch the most up-to-date version of the CSS.
    FetchUpdatedCSS,
}

/// Represents different contexts in which styles are applied.
#[derive(Clone, PartialEq, Debug)]
pub enum ContextType {
    /// Central context, usually referring to global or main styles.
    Central,
    /// Layout context, typically related to layout-specific styles.
    Layout,
    /// Module context, generally referring to isolated or component-level styles.
    Module,
}

/// Represents a client request, which could be for collecting class lists or fetching updated CSS.
#[derive(Clone, PartialEq, Debug)]
pub enum Request {
    /// Request to collect the class list for a specific context.
    CollectClassList {
        /// The context in which the class list is requested.
        context_type: ContextType,
        /// Optional name of the context, used for distinguishing between instances.
        context_name: Option<String>,
        /// The name of the Nenyr class being requested.
        class_name: String,
    },
    /// Request to fetch the most updated CSS.
    FetchUpdatedCSS,
}

impl Request {
    /// Creates a new `CollectClassList` request.
    ///
    /// # Arguments
    ///
    /// * `context_type` - The context in which the class list is to be collected.
    /// * `context_name` - The optional name of the specific context.
    /// * `class_name` - The name of the Nenyr class to be collected.
    ///
    /// # Returns
    ///
    /// A new `Request::CollectClassList` variant with the provided details.
    pub fn new_class_list_request(
        context_type: ContextType,
        context_name: Option<String>,
        class_name: String,
    ) -> Self {
        Request::CollectClassList {
            context_type,
            context_name,
            class_name,
        }
    }
}

/// Represents a server-side request, including the client's name and the associated request.
#[derive(Clone, PartialEq, Debug)]
pub struct ServerRequest {
    /// The name of the client making the request.
    pub client_name: String,
    /// The request that the client is making.
    pub request: Request,
}

impl ServerRequest {
    /// Creates a new server request with a specific client and associated request.
    ///
    /// # Arguments
    ///
    /// * `client_name` - The name of the client making the request.
    /// * `request` - The request object containing the details of the request.
    ///
    /// # Returns
    ///
    /// A new `ServerRequest` instance with the provided client name and request.
    pub fn new(client_name: String, request: Request) -> Self {
        Self {
            client_name,
            request,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lothlorien::request::{ContextType, ServerRequest};

    use super::Request;

    #[test]
    fn test_new_class_list_request_with_context_name() {
        let context_type = ContextType::Central;
        let context_name = Some("test_context".to_string());
        let class_name = "class_name".to_string();

        let request =
            Request::new_class_list_request(context_type, context_name.clone(), class_name.clone());

        match request {
            Request::CollectClassList {
                context_type,
                context_name,
                class_name,
            } => {
                assert_eq!(context_type, ContextType::Central);
                assert_eq!(context_name, Some("test_context".to_string()));
                assert_eq!(class_name, "class_name".to_string());
            }
            _ => panic!("Expected CollectClassList request"),
        }
    }

    #[test]
    fn test_new_class_list_request_without_context_name() {
        let context_type = ContextType::Layout;
        let context_name: Option<String> = None;
        let class_name = "class_name".to_string();
        let request =
            Request::new_class_list_request(context_type, context_name.clone(), class_name.clone());

        match request {
            Request::CollectClassList {
                context_type,
                context_name,
                class_name,
            } => {
                assert_eq!(context_type, ContextType::Layout);
                assert_eq!(context_name, None);
                assert_eq!(class_name, "class_name".to_string());
            }
            _ => panic!("Expected CollectClassList request"),
        }
    }

    #[test]
    fn test_collect_class_list_request_variant() {
        let request = Request::CollectClassList {
            context_type: ContextType::Module,
            context_name: Some("module_context".to_string()),
            class_name: "module_class".to_string(),
        };

        match request {
            Request::CollectClassList {
                context_type,
                context_name,
                class_name,
            } => {
                assert_eq!(context_type, ContextType::Module);
                assert_eq!(context_name, Some("module_context".to_string()));
                assert_eq!(class_name, "module_class".to_string());
            }
            _ => panic!("Expected CollectClassList request"),
        }
    }

    #[test]
    fn test_fetch_updated_css_request_variant() {
        let request = Request::FetchUpdatedCSS;

        match request {
            Request::FetchUpdatedCSS => {}
            _ => panic!("Expected FetchUpdatedCSS request"),
        }
    }

    #[test]
    fn test_server_request_creation() {
        let client_name = "client_1".to_string();
        let request = Request::FetchUpdatedCSS;
        let server_request = ServerRequest::new(client_name.clone(), request.clone());

        assert_eq!(server_request.client_name, client_name);

        match server_request.request {
            Request::FetchUpdatedCSS => {}
            _ => panic!("Expected FetchUpdatedCSS request"),
        }
    }

    #[test]
    fn test_server_request_with_collect_class_list() {
        let client_name = "client_2".to_string();
        let request = Request::CollectClassList {
            context_type: ContextType::Layout,
            context_name: Some("layout_context".to_string()),
            class_name: "layout_class".to_string(),
        };

        let server_request = ServerRequest::new(client_name.clone(), request.clone());

        assert_eq!(server_request.client_name, client_name);

        match server_request.request {
            Request::CollectClassList {
                context_type,
                context_name,
                class_name,
            } => {
                assert_eq!(context_type, ContextType::Layout);
                assert_eq!(context_name, Some("layout_context".to_string()));
                assert_eq!(class_name, "layout_class".to_string());
            }
            _ => panic!("Expected CollectClassList request"),
        }
    }
}
