//! Routing Homepage Requests

use std::{collections::HashMap, net::IpAddr, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use regex::bytes::{Captures, Regex, RegexSet};
use tokio::{io::AsyncRead, sync::RwLock};
use tracing::info;
use url::Url;

pub mod routes;

pub enum Response {
    Success(String, Box<dyn AsyncRead + Unpin + Send + Sync + 'static>),
    NotFound,
    ServerError,
}

#[derive(Debug)]
pub struct Context<'a> {
    pub url: &'a Url,
    pub remote_ip: IpAddr,
    pub captures: Option<Captures<'a>>,
}

#[async_trait]
pub trait Route: std::fmt::Debug {
    async fn handle(&self, ctx: Context<'async_trait>) -> Result<Response>;
}

#[derive(Debug, Clone)]
struct RouterInner {
    routes: HashMap<String, (Arc<Box<dyn Route + Send + Sync>>, Option<Regex>)>,
    route_list: Vec<String>,
    set: Option<RegexSet>,
}

impl RouterInner {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            route_list: Vec::new(),
            set: None,
        }
    }

    pub fn add_route(
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
                (Arc::new(Box::new(route)), Some(regex)),
            );
        } else {
            self.routes
                .insert(regex_str.to_string(), (Arc::new(Box::new(route)), None));
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

#[derive(Debug)]
pub struct Router {
    inner: RwLock<RouterInner>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(RouterInner::new()),
        }
    }

    pub async fn add_route(
        &self,
        regex: impl AsRef<str>,
        with_captures: bool,
        route: impl Route + 'static + Send + Sync,
    ) -> Result<()> {
        self.inner
            .write()
            .await
            .add_route(regex, with_captures, route)
    }

    pub async fn add_default_routes(&self) -> Result<()> {
        self.add_route("^/owo.gmi", false, routes::OwORoute).await?;
        Ok(())
    }

    async fn rebuild_routes(&self) -> Result<()> {
        self.inner.write().await.rebuild_routes()
    }

    #[tracing::instrument]
    pub async fn handle(&self, url: Url, remote_ip: IpAddr) -> Result<Response> {
        if self.inner.read().await.set.is_none() {
            self.rebuild_routes().await?;
        }

        let inner = self.inner.read().await;
        let path = url.path().as_bytes();
        let regex_set = inner.set.as_ref().unwrap();
        if let Some(v) = regex_set.matches(path).iter().next() {
            let (route, regex) = &inner.routes[&regex_set.patterns()[v]];
            let mut captures = None;
            if let Some(regex) = regex {
                captures = regex.captures(path);
            }
            let context = Context {
                url: &url,
                remote_ip,
                captures,
            };
            return route.handle(context).await;
        }

        Ok(Response::NotFound)
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
