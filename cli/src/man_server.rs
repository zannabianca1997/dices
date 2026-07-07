//! Server for the man pages

use std::{
    io,
    net::IpAddr,
    panic::resume_unwind,
    sync::Arc,
    thread::{self, JoinHandle},
};

use axum::{
    Router,
    extract::{OriginalUri, Path},
    response::{Html, Redirect},
    routing::get,
};
use dices_man::Manual;
use dices_print::{Pretty, manual::Ctx, markdown::DefaultCodeRender};
use dices_print_cli::examples::CliCodeRender;
use dices_print_html::HtmlWriter;
use itertools::Itertools;
use pretty::{Arena, Pretty as _};
use rust_embed::Embed;
use snafu::Snafu;
use tokio::{runtime::Builder, sync::oneshot};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::{
    config::{man::Links, skin::Skin},
    prompt::Prompt,
};

pub struct ManServer {
    base_url: Url,
    cancel: CancellationToken,
    thread: JoinHandle<io::Result<()>>,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(transparent)]
    Io { source: io::Error },
}

#[derive(Debug)]
struct ServerStarted {
    base_url: Url,
}

impl ManServer {
    /// Start the manual server
    pub fn spawn(Links { address, port, .. }: Links, skin: Arc<Skin>) -> Result<Self, Error> {
        let cancel = CancellationToken::new();

        let (tx, rx) = oneshot::channel();

        let thread = thread::spawn({
            let cancel = cancel.child_token();
            move || match Self::spawn_inner(address, port, cancel, skin) {
                Ok((server_started, serve)) => {
                    tx.send(Ok(server_started)).unwrap();
                    serve()
                }
                Err(err) => {
                    tx.send(Err(err)).unwrap();
                    Ok(())
                }
            }
        });

        let ServerStarted { base_url } = match rx.blocking_recv().unwrap() {
            Ok(it) => it,
            Err(err) => {
                thread
                    .join()
                    .unwrap_or_else(|err| resume_unwind(err))
                    .expect("Thread does not sen an err after having done so before");
                return Err(err);
            }
        };

        Ok(Self {
            base_url,
            cancel,
            thread,
        })
    }

    fn spawn_inner(
        addr: IpAddr,
        port: Option<u16>,
        cancel: CancellationToken,
        skin: Arc<Skin>,
    ) -> Result<(ServerStarted, impl FnOnce() -> io::Result<()>), Error> {
        let rt = Builder::new_current_thread().enable_io().build()?;

        let serve = async move {
            let listener = tokio::net::TcpListener::bind((addr, port.unwrap_or_default())).await?;

            let local_addr = listener.local_addr()?;

            let base_url = {
                let mut url = Url::parse("http://localhost/man").unwrap();
                url.set_ip_host(local_addr.ip()).unwrap();
                url.set_port(Some(local_addr.port())).unwrap();
                url
            };

            Ok::<_, Error>((
                base_url.clone(),
                axum::serve(listener, Self::router(base_url, skin))
                    .with_graceful_shutdown(cancel.cancelled_owned()),
            ))
        };

        let (base_url, serve) = rt.block_on(serve)?;

        Ok((ServerStarted { base_url }, move || {
            rt.block_on(serve.into_future())
        }))
    }

    /// Close the manual server
    pub fn join(self) -> io::Result<()> {
        // Signal the server to do a graceful shutdown
        self.cancel.cancel();
        // Wait for termination
        self.thread.join().unwrap_or_else(|err| resume_unwind(err))
    }

    /// Build the router
    fn router(base_url: Url, skin: Arc<Skin>) -> Router {
        let manual = Manual::new();
        Router::new()
            .route(
                "/man/{*path}",
                get({
                    let skin = skin.clone();
                    async move |path, uri| get_page(path, uri, &manual, &base_url, &skin).await
                }),
            )
            .route(
                "/styles/skin.css",
                get(async move || ([("Content-Type", "text/css")], skin_style(&skin))),
            )
            .route(
                "/styles/style.css",
                get(async move || ([("Content-Type", "text/css")], global_style())),
            )
    }

    /// Get the server base url
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }
}

fn global_style() -> std::borrow::Cow<'static, [u8]> {
    Assets::get("styles/style.css").unwrap().data
}

fn skin_style(skin: &Skin) -> String {
    skin.theme.to_css().to_string()
}

#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/man_server"]
struct Assets;

async fn get_page(
    Path(path): Path<String>,
    OriginalUri(uri): OriginalUri,
    manual: &Manual,
    base_url: &Url,
    skin: &Skin,
) -> Result<Html<Vec<u8>>, Redirect> {
    let mut path = path
        .rsplit_once('/')
        .map(|t| t.0)
        .unwrap_or(&path)
        .split('/')
        .map_while(|s| s.parse().ok())
        .collect_vec();

    let page = loop {
        match manual.fetch(path.clone()) {
            Some(page) => break page,
            None => {
                path.pop().expect("[] is always available");
            }
        }
    };

    // If url is misformed, send the user to the correct page
    let page_url = page.url(base_url.clone());
    if page_url.path() != uri.path() {
        return Err(Redirect::to(page_url.as_str()));
    }

    let arena = Arena::new();
    let renderer = (CliCodeRender::new(Prompt(skin)), DefaultCodeRender);
    let mut response = Vec::with_capacity(page.content().len());

    let out = &mut response;

    out.extend_from_slice(b"<!DOCTYPE html><html><head><title>");
    out.extend_from_slice(page.title().as_bytes());
    out.extend_from_slice(
        concat!(
            "</title>",
            "<link rel=\"stylesheet\" href=\"/styles/style.css\">",
            "<link rel=\"stylesheet\" href=\"/styles/skin.css\">",
            "</head>",
            "<body>",
            "<header><a href=\"",
        )
        .as_bytes(),
    );
    out.extend_from_slice(manual.root().url(base_url.clone()).as_str().as_bytes());
    out.extend_from_slice(b"\">Index</a>");
    if let Some(up) = page.parent() {
        out.extend_from_slice(b"<a href=\"");
        out.extend_from_slice(up.url(base_url.clone()).as_str().as_bytes());
        out.extend_from_slice(b"\">Up</a>");
    }
    out.extend_from_slice(b"</header><main>");

    let printing = (&page).with_ctx(Ctx::new_with_links(renderer, base_url.clone()));
    let mut writer = HtmlWriter::new(out);
    printing
        .pretty(&arena)
        .render_raw(128, &mut writer)
        .unwrap();
    let out = writer.into_inner();

    out.extend_from_slice(
        concat!(
            "</main>",
            "<footer>Served by ",
            env!("CARGO_PKG_NAME"),
            " v",
            env!("CARGO_PKG_VERSION"),
            "</footer>",
            "</body></html>"
        )
        .as_bytes(),
    );

    Ok(Html(response))
}
