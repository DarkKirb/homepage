use std::io::Cursor;

use anyhow::Result;
use async_trait::async_trait;

use crate::Context;
use crate::Response;
use crate::Route;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct OwORoute;

#[async_trait]
impl Route for OwORoute {
    #[tracing::instrument]
    async fn handle(&self, _: Context<'async_trait>) -> Result<Response> {
        Ok(Response::Success(
            "text/gemini".to_string(),
            Box::new(Cursor::new(
                b"```\n*notices client* OwO what's this\n```".to_vec(),
            )),
        ))
    }
}
