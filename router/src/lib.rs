//! Router used by the darkkirb.de homepage

use std::{collections::HashMap, net::IpAddr, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use regex::bytes::{Captures, Regex, RegexSet};
use tokio::{io::AsyncRead, sync::RwLock};
use tracing::info;
use url::Url;

pub mod routes;

/// Simplified multi-protocol response enum
///
/// This enum only covers return codes that are actually used  by the crate currently
#[non_exhaustive]
pub enum Response {
    /// Success Response (HTTP Code 200, Gemini Code 20)
    Success(
        #[doc = "MIME-Type or Content-Type"] String,
        #[doc = "Body of Response"] Box<dyn AsyncRead + Unpin + Send + Sync + 'static>,
    ),
    /// File Not Found Error (HTTP Code 404, Gemini Code 51)
    NotFound,
    /// Server Error (HTTP Code 500, Gemini Code 42)
    ServerError,
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success(arg0, _) => f
                .debug_tuple("Success")
                .field(arg0)
                .field(&"{opaque}")
                .finish(),
            Self::NotFound => write!(f, "NotFound"),
            Self::ServerError => write!(f, "ServerError"),
        }
    }
}

/// Context given to a [`Route`]
#[derive(Debug)]
pub struct Context<'a> {
    /// URL that is attempted to be accessed
    pub url: &'a Url,
    /// IP Address of Peer
    pub remote_ip: IpAddr,
    /// If enabled, captures of the route regex match
    pub captures: Option<Captures<'a>>,
}

/// A Single Route for the Router
#[async_trait]
pub trait Route: std::fmt::Debug {
    /// Handles a router requet
    ///
    /// # Arguments
    /// - `ctx` - The [`Context`] to the structure
    ///
    /// # Errors
    /// The router function may return an error if handling the request was unsuccessful. The callee has to handle the error and return a ServerError response to the clent.
    async fn handle(&self, ctx: Context<'async_trait>) -> Result<Response>;
}

#[derive(Clone, Debug)]
struct RouteMeta {
    route: Arc<Box<dyn Route + Send + Sync>>,
    regex: Option<Regex>,
}

#[derive(Debug, Clone)]
struct RouterInner {
    routes: HashMap<String, RouteMeta>,
    route_list: Vec<String>,
    set: Option<RegexSet>,
}

impl RouterInner {
    fn new() -> Self {
        Self {
            routes: HashMap::new(),
            route_list: Vec::new(),
            set: None,
        }
    }

    fn add_route(
        &mut self,
        regex: impl AsRef<str>,
        with_captures: bool,
        route: impl Route + 'static + Send + Sync,
    ) -> Result<()> {
        let regex_str = regex.as_ref();
        let regex = Regex::new(regex_str)?;
        if with_captures {
            self.routes.insert(
                regex_str.to_string(),
                RouteMeta {
                    route: Arc::new(Box::new(route)),
                    regex: Some(regex),
                },
            );
        } else {
            self.routes.insert(
                regex_str.to_string(),
                RouteMeta {
                    route: Arc::new(Box::new(route)),
                    regex: None,
                },
            );
        }
        self.route_list.push(regex_str.to_string());
        self.set = None;

        Ok(())
    }

    fn rebuild_routes(&mut self) -> Result<()> {
        if self.set.is_some() {
            return Ok(());
        }
        info!("Regenerating the routes");
        self.set = Some(RegexSet::new(self.route_list.iter())?);
        Ok(())
    }
}

impl Default for RouterInner {
    fn default() -> Self {
        Self::new()
    }
}

/// Router Struct
///
/// The router manages multiple [`Route`]s and their [`Regex`] matches. Paths are matched with a binary regex engine.
#[derive(Debug)]
pub struct Router {
    inner: RwLock<RouterInner>,
}

impl Router {
    /// Create a new `Router`. The default router has no routes.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(RouterInner::new()),
        }
    }

    /// Asynchronously adds a route
    ///
    /// # Arguments
    /// - `regex` contains the regex that needs to be matched for the route to execute
    /// - `with_captures` states whether or not the captures are required for the route to function. This option only makes sense if the regex contains captures, which is not validated. There is a performance penalty to enabling captures.
    /// - `route` The route implementation
    ///
    /// # Errors
    /// The function returns an error if the supplied regex is invalid.
    pub async fn add_route(
        &self,
        regex: impl AsRef<str> + Send,
        with_captures: bool,
        route: impl Route + 'static + Send + Sync,
    ) -> Result<()> {
        self.inner
            .write()
            .await
            .add_route(regex, with_captures, route)
    }

    /// Adds the default Routes
    ///
    /// This function will add the routes found in the `routes` module.
    #[allow(clippy::missing_panics_doc)] // false positive
    pub async fn add_default_routes(&self) {
        self.add_route("^/owo.gmi", false, routes::OwORoute)
            .await
            .unwrap();
    }

    async fn rebuild_routes(&self) -> Result<()> {
        self.inner.write().await.rebuild_routes()
    }

    /// Handle a Request
    ///
    /// # Arguments
    /// - `url` - Url of the request
    /// - `remote_ip` - IP Address of peer
    ///
    /// # Errors
    /// This function will return an error if the route handler fails.
    #[tracing::instrument]
    pub async fn handle(&self, url: Url, remote_ip: IpAddr) -> Result<Response> {
        if self.inner.read().await.set.is_none() {
            self.rebuild_routes().await?;
        }

        let inner = self.inner.read().await;
        let path = url.path().as_bytes();
        let regex_set = inner.set.as_ref().unwrap();
        if let Some(v) = regex_set.matches(path).iter().next() {
            let meta = &inner.routes[&regex_set.patterns()[v]];
            let mut captures = None;
            if let Some(regex) = &meta.regex {
                captures = regex.captures(path);
            }
            let context = Context {
                url: &url,
                remote_ip,
                captures,
            };
            return meta.route.handle(context).await;
        }

        Ok(Response::NotFound)
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
