use candid::Principal;
use ic_cdk::management_canister::{
    HttpRequestArgs, HttpRequestResult, TransformArgs, TransformContext, TransformFunc,
    http_request,
};

pub trait HttpOutcall {
    fn transform_context(&self) -> Option<TransformContext>;
    async fn request(&self, args: &HttpRequestArgs) -> Result<HttpRequestResult, String>;
}

pub struct DefaultHttpOutcall(Principal);

impl DefaultHttpOutcall {
    pub fn new(canister_id: Principal) -> Self {
        Self(canister_id)
    }
}

impl HttpOutcall for DefaultHttpOutcall {
    async fn request(&self, args: &HttpRequestArgs) -> Result<HttpRequestResult, String> {
        http_request(args).await.map_err(|err| format!("{err}"))
    }

    fn transform_context(&self) -> Option<TransformContext> {
        Some(TransformContext {
            function: TransformFunc::new(self.0, "inner_transform_response".to_string()),
            context: vec![],
        })
    }
}

#[ic_cdk::query(hidden = true)]
fn inner_transform_response(args: TransformArgs) -> HttpRequestResult {
    HttpRequestResult {
        status: args.response.status,
        body: args.response.body,
        // Remove headers (which may contain a timestamp) for consensus
        headers: vec![],
    }
}
