use ic_auth_types::ByteBufB64;
use ic_cdk::management_canister::{HttpHeader, HttpMethod, HttpRequestArgs};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};

use super::types::*;
use crate::{
    helper::APP_AGENT,
    outcall::HttpOutcall,
    types::{RPCRequest, RPCResponse},
};

pub struct SvmClient<T: HttpOutcall> {
    pub providers: Vec<String>,
    pub commitment: Option<String>,
    pub api_token: Option<String>,
    outcall: T,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct RpcContext {
    pub slot: u64,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct RpcContextValue<T> {
    pub context: RpcContext,
    pub value: T,
}

impl<H: HttpOutcall> SvmClient<H> {
    pub fn new(
        providers: Vec<String>,
        commitment: Option<String>,
        api_token: Option<String>,
        outcall: H,
    ) -> Self {
        Self {
            providers,
            commitment: commitment.or_else(|| Some("confirmed".to_string())),
            api_token,
            outcall,
        }
    }

    pub async fn get_latest_blockhash(&self, now_ms: u64) -> Result<Hash, String> {
        let mut config = Map::new();
        self.insert_commitment(&mut config);
        let params: Vec<Value> = if config.is_empty() {
            vec![]
        } else {
            vec![Value::Object(config)]
        };

        let res: RpcContextValue<LatestBlockhash> = self
            .call(
                format!("getLatestBlockhash-{now_ms}"),
                "getLatestBlockhash",
                params.as_slice(),
            )
            .await?;

        res.value.to_hash()
    }

    #[allow(dead_code)]
    pub async fn get_block_height(&self, now_ms: u64) -> Result<u64, String> {
        let mut config = Map::new();
        self.insert_commitment(&mut config);
        let params: Vec<Value> = if config.is_empty() {
            vec![]
        } else {
            vec![Value::Object(config)]
        };

        self.call(
            format!("getBlockHeight-{now_ms}"),
            "getBlockHeight",
            params.as_slice(),
        )
        .await
    }

    pub async fn get_signature_statuses(
        &self,
        now_ms: u64,
        signature: &str,
    ) -> Result<Option<SignatureStatus>, String> {
        let params = vec![
            Value::Array(vec![signature.into()]),
            json!({
                "searchTransactionHistory": true
            }),
        ];

        let mut res: RpcContextValue<Vec<Option<SignatureStatus>>> = self
            .call(
                format!("getSignatureStatuses-{now_ms}"),
                "getSignatureStatuses",
                params.as_slice(),
            )
            .await?;
        let status = res.value.remove(0);
        Ok(status)
    }

    #[allow(dead_code)]
    pub async fn get_transaction(
        &self,
        now_ms: u64,
        signature: &str,
        encoding: Option<&str>,
        max_supported_transaction_version: Option<u8>,
    ) -> Result<Option<Value>, String> {
        let mut config = Map::new();
        self.insert_commitment(&mut config);
        config.insert(
            "encoding".to_string(),
            Value::String(encoding.unwrap_or("base64").to_string()),
        );

        if let Some(version) = max_supported_transaction_version {
            config.insert(
                "maxSupportedTransactionVersion".to_string(),
                Value::Number(version.into()),
            );
        }

        let params = vec![Value::String(signature.to_string()), Value::Object(config)];

        self.call(
            format!("getTransaction-{now_ms}-{signature}"),
            "getTransaction",
            params.as_slice(),
        )
        .await
    }

    pub async fn send_transaction(
        &self,
        now_ms: u64,
        transaction: ByteBufB64,
        skip_preflight: bool,
    ) -> Result<String, String> {
        let mut config = Map::new();
        config.insert("encoding".to_string(), "base64".into());
        self.insert_commitment(&mut config);
        if skip_preflight {
            config.insert("skipPreflight".to_string(), Value::Bool(true));
        }

        let params = if config.is_empty() {
            vec![Value::String(transaction.to_base64())]
        } else {
            vec![
                Value::String(transaction.to_base64()),
                Value::Object(config),
            ]
        };

        self.call(
            format!("sendTransaction-{now_ms}"),
            "sendTransaction",
            params.as_slice(),
        )
        .await
    }

    #[allow(dead_code)]
    pub async fn simulate_transaction(
        &self,
        now_ms: u64,
        transaction: String,
        sig_verify: bool,
    ) -> Result<Value, String> {
        let mut config = Map::new();
        self.insert_commitment(&mut config);
        config.insert("sigVerify".to_string(), Value::Bool(sig_verify));

        let params = vec![Value::String(transaction), Value::Object(config)];

        let res: RpcContextValue<Value> = self
            .call(
                format!("simulateTransaction-{now_ms}"),
                "simulateTransaction",
                params.as_slice(),
            )
            .await?;

        Ok(res.value)
    }

    pub async fn get_account_info(
        &self,
        now_ms: u64,
        pubkey: &str,
    ) -> Result<Option<UiAccount>, String> {
        let mut config = Map::new();
        self.insert_commitment(&mut config);
        config.insert("encoding".to_string(), "jsonParsed".into());

        let params = if config.is_empty() {
            vec![Value::String(pubkey.to_string())]
        } else {
            vec![Value::String(pubkey.to_string()), Value::Object(config)]
        };

        let res: RpcContextValue<Option<UiAccount>> = self
            .call(
                format!("getAccountInfo-{now_ms}-{pubkey}"),
                "getAccountInfo",
                params.as_slice(),
            )
            .await?;

        Ok(res.value)
    }

    #[allow(dead_code)]
    pub async fn get_token_account_balance(
        &self,
        now_ms: u64,
        pubkey: &str,
    ) -> Result<UiTokenAmount, String> {
        let mut config = Map::new();
        self.insert_commitment(&mut config);

        let params = if config.is_empty() {
            vec![Value::String(pubkey.to_string())]
        } else {
            vec![Value::String(pubkey.to_string()), Value::Object(config)]
        };

        let res: RpcContextValue<UiTokenAmount> = self
            .call(
                format!("getTokenAccountBalance-{now_ms}-{pubkey}"),
                "getTokenAccountBalance",
                params.as_slice(),
            )
            .await?;

        Ok(res.value)
    }

    pub async fn call<T: DeserializeOwned>(
        &self,
        idempotency_key: String,
        method: &str,
        params: &[Value],
    ) -> Result<T, String> {
        if self.providers.is_empty() {
            return Err("no available provider".to_string());
        }

        let input = RPCRequest {
            jsonrpc: "2.0",
            method,
            params,
            id: 1,
        };
        let input = serde_json::to_vec(&input).map_err(|err| err.to_string())?;
        let data = self.http_request(idempotency_key, input).await?;

        let output: RPCResponse<T> =
            serde_json::from_slice(&data).map_err(|err| err.to_string())?;

        if let Some(error) = output.error {
            return Err(serde_json::to_string(&error).map_err(|err| err.to_string())?);
        }

        match output.result {
            Some(result) => Ok(result),
            None => serde_json::from_value(Value::Null).map_err(|_| "missing result".to_string()),
        }
    }

    fn insert_commitment(&self, config: &mut Map<String, Value>) {
        if let Some(commitment) = &self.commitment {
            config
                .entry("commitment".to_string())
                .or_insert_with(|| Value::String(commitment.clone()));
        }
    }

    async fn http_request(
        &self,
        idempotency_key: String,
        body: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        let mut request_headers = vec![
            HttpHeader {
                name: "content-type".to_string(),
                value: "application/json".to_string(),
            },
            HttpHeader {
                name: "user-agent".to_string(),
                value: APP_AGENT.to_string(),
            },
            HttpHeader {
                name: "idempotency-key".to_string(),
                value: idempotency_key.clone(),
            },
        ];

        if let Some(api_token) = &self.api_token {
            request_headers.push(HttpHeader {
                name: "authorization".to_string(),
                value: api_token.clone(),
            });
        }

        let mut args = HttpRequestArgs {
            url: "".to_string(),
            max_response_bytes: None,
            method: HttpMethod::POST,
            headers: request_headers,
            body: Some(body),
            transform: self.outcall.transform_context(),
            is_replicated: Some(false),
        };

        let mut last_err = "No provider succeeded".to_string();
        for p in &self.providers {
            args.url = p.clone();
            match self.outcall.request(&args).await {
                Ok(res) => {
                    if res.status >= 200u64 && res.status < 300u64 {
                        return Ok(res.body);
                    } else {
                        last_err = format!(
                            "request provider: {}, idempotency-key: {}, status: {}, body: {}",
                            p,
                            idempotency_key,
                            res.status,
                            String::from_utf8(res.body.clone()).unwrap_or_default(),
                        );
                    }
                }
                Err(err) => {
                    last_err = format!("failed to request provider: {p}, error: {err}");
                }
            }
        }

        Err(last_err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use ic_cdk::management_canister::{HttpRequestResult, TransformContext};

    #[test]
    fn test_get_latest_blockhash() {
        let mock = MockHttpOutcall::new(vec![success_response(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "context": { "slot": 1234 },
                "value": {
                    "blockhash": "3Xdj6drp4pKAM9PH2vZ4w8NHygd8Epp7FKCvzX29VLLH",
                    "lastValidBlockHeight": 355385114
                }
            }
        }))]);

        let client = SvmClient::new(
            vec!["https://solana.rpc".to_string()],
            Some("confirmed".to_string()),
            None,
            mock.clone(),
        );
        let response = futures::executor::block_on(client.get_latest_blockhash(1000)).unwrap();

        assert_eq!(
            response.to_string(),
            "3Xdj6drp4pKAM9PH2vZ4w8NHygd8Epp7FKCvzX29VLLH"
        );
        assert_eq!(mock.urls(), vec!["https://solana.rpc".to_string()]);
    }

    #[test]
    fn test_http_request_fallbacks_between_providers() {
        let mock = MockHttpOutcall::new(vec![
            Err("timeout".to_string()),
            success_response(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": 42
            })),
        ]);

        let client = SvmClient::new(
            vec!["https://first".to_string(), "https://second".to_string()],
            Some("confirmed".to_string()),
            None,
            mock.clone(),
        );

        let slot = futures::executor::block_on(client.get_block_height(2_000)).unwrap();

        assert_eq!(slot, 42);
        assert_eq!(
            mock.urls(),
            vec!["https://first".to_string(), "https://second".to_string()]
        );
    }

    #[test]
    fn test_call_handles_error_payload() {
        let mock = MockHttpOutcall::new(vec![success_response(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {"code": -32000, "message": "custom error"}
        }))]);
        let client = SvmClient::new(vec!["https://sol".to_string()], None, None, mock);

        let result: Result<Value, _> =
            futures::executor::block_on(client.call("key".to_string(), "method", &[]));
        assert!(result.unwrap_err().contains("custom error"));
    }

    #[test]
    fn test_send_transaction_returns_signature() {
        let mock = MockHttpOutcall::new(vec![success_response(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "5N7signature"
        }))]);

        let client = SvmClient::new(vec!["https://sol".to_string()], None, None, mock);
        let signature =
            futures::executor::block_on(client.send_transaction(1_234, [1, 2, 3, 4].into(), true))
                .unwrap();

        assert_eq!(signature, "5N7signature");
    }

    #[test]
    fn test_get_transaction_returns_none() {
        let mock = MockHttpOutcall::new(vec![success_response(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": null
        }))]);

        let client = SvmClient::new(vec!["https://sol".to_string()], None, None, mock);
        let tx =
            futures::executor::block_on(client.get_transaction(1_000, "sig", None, None)).unwrap();

        assert!(tx.is_none());
    }

    #[test]
    fn test_get_token_account_balance() {
        let mock = MockHttpOutcall::new(vec![success_response(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "context": {"slot": 321},
                "value": {
                    "amount": "12345",
                    "decimals": 6,
                    "uiAmount": 0.012345,
                    "uiAmountString": "0.012345"
                }
            }
        }))]);

        let client = SvmClient::new(vec!["https://sol".to_string()], None, None, mock);
        let balance =
            futures::executor::block_on(client.get_token_account_balance(1_111, "TokenPubkey"))
                .unwrap();

        assert_eq!(balance.amount, "12345");
        assert_eq!(balance.decimals, 6);
        assert_eq!(balance.ui_amount.unwrap(), 0.012345);
        assert_eq!(balance.ui_amount_string, "0.012345");
    }

    #[derive(Clone, Default)]
    struct MockHttpOutcall {
        responses: Arc<Mutex<VecDeque<Result<HttpRequestResult, String>>>>,
        urls: Arc<Mutex<Vec<String>>>,
    }

    impl MockHttpOutcall {
        fn new(responses: Vec<Result<HttpRequestResult, String>>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(responses.into_iter().collect())),
                urls: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn urls(&self) -> Vec<String> {
            self.urls.lock().unwrap().clone()
        }
    }

    impl HttpOutcall for MockHttpOutcall {
        async fn request(
            &self,
            args: &ic_cdk::management_canister::HttpRequestArgs,
        ) -> Result<HttpRequestResult, String> {
            self.urls.lock().unwrap().push(args.url.clone());
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| Err("no mock response".to_string()))
        }

        fn transform_context(&self) -> Option<TransformContext> {
            None
        }
    }

    fn success_response(body: serde_json::Value) -> Result<HttpRequestResult, String> {
        Ok(HttpRequestResult {
            status: 200u64.into(),
            body: serde_json::to_vec(&body).unwrap(),
            headers: vec![],
        })
    }
}
