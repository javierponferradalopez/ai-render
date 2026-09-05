use std::sync::Mutex;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerInfo};
use rmcp::{ServerHandler, ServiceExt, tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use tokio::signal::unix::{SignalKind, signal};

use crate::flipchart::Flipchart;
use crate::lifecycle::the_session_is_over;
use crate::wire::Wire;

#[derive(Deserialize, JsonSchema)]
pub struct ShowParams {
    #[schemars(
        description = "Short human-readable name, shown to the user above the diagram - e.g. \"Current dependencies\", not \"v1\". Reusing a name replaces that view."
    )]
    view_id: String,
    #[schemars(description = "Mermaid source.")]
    diagram: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ClearParams {
    #[schemars(description = "View to remove. Omit to clear the whole flipchart.")]
    view_id: Option<String>,
}

pub struct FlipchartServer {
    flipchart: Mutex<Flipchart>,
}

#[tool_router]
impl FlipchartServer {
    pub fn new(viewer: Wire) -> Self {
        Self {
            flipchart: Mutex::new(Flipchart::new(viewer)),
        }
    }

    #[tool(
        description = "Show a diagram on the ephemeral flipchart window, as a named view. Takes Mermaid source.\n\nAny id used in a relationship must carry a label or a body when another id in the same diagram does; a bare id alongside a labelled one is rejected.\n\nShowing an existing view id replaces it and brings it to the front; several named views coexist. The flipchart dies with the session."
    )]
    async fn show(&self, Parameters(params): Parameters<ShowParams>) -> CallToolResult {
        match self
            .flipchart
            .lock()
            .expect("the flipchart lock is never held across a panic")
            .show(&params.view_id, &params.diagram)
        {
            Ok(acknowledgement) => {
                CallToolResult::success(vec![ContentBlock::text(acknowledgement)])
            }
            Err(rejection) => CallToolResult::error(vec![ContentBlock::text(rejection)]),
        }
    }

    #[tool(
        description = "Remove one view from the flipchart, or all of them. Does not close the window."
    )]
    async fn clear(&self, Parameters(params): Parameters<ClearParams>) -> CallToolResult {
        let text = self
            .flipchart
            .lock()
            .expect("the flipchart lock is never held across a panic")
            .clear(params.view_id.as_deref());
        CallToolResult::success(vec![ContentBlock::text(text)])
    }
}

#[rmcp::tool_handler]
impl ServerHandler for FlipchartServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("flipchart", env!("CARGO_PKG_VERSION")))
    }
}

pub fn serve(viewer: Wire) -> ! {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("the server thread owns its runtime");
    runtime.block_on(until_the_session_dies(viewer.clone()));
    the_session_is_over(&viewer)
}

/// The two death signals, in the order they arrive: `SIGINT` —the first thing
/// the host sends— and the EOF on stdin that the MCP-over-stdio specification
/// requires us to handle. `SIGINT` is listened for from before `initialize`,
/// because from the moment it is registered it stops killing the process on its
/// own.
async fn until_the_session_dies(viewer: Wire) {
    let mut interrupted =
        signal(SignalKind::interrupt()).expect("the server thread listens for SIGINT");
    tokio::select! {
        _ = interrupted.recv() => {}
        _ = mcp_over_stdio(viewer) => {}
    }
}

async fn mcp_over_stdio(viewer: Wire) {
    let Ok(service) = FlipchartServer::new(viewer)
        .serve(rmcp::transport::stdio())
        .await
    else {
        return;
    };
    let _ = service.waiting().await;
}
