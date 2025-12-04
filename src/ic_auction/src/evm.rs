use alloy_primitives::{U256, hex::FromHex};
use alloy_rpc_types_eth::TransactionReceipt;
use ic_cdk::management_canister::{HttpHeader, HttpMethod, HttpRequestArgs};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{
    helper::APP_AGENT,
    outcall::HttpOutcall,
    types::{RPCRequest, RPCResponse},
};

pub use alloy_primitives::{Address, TxHash};

pub struct EvmClient<T: HttpOutcall> {
    pub providers: Vec<String>,
    pub max_confirmations: u64,
    pub api_token: Option<String>,
    outcall: T,
}

// https://ethereum.org/zh/developers/docs/apis/json-rpc/
impl<H: HttpOutcall> EvmClient<H> {
    pub fn new(
        providers: Vec<String>,
        max_confirmations: u64,
        api_token: Option<String>,
        outcall: H,
    ) -> Self {
        Self {
            providers,
            max_confirmations,
            api_token,
            outcall,
        }
    }
    pub async fn chain_id(&self, now_ms: u64) -> Result<u64, String> {
        let res: String = self
            .call(format!("eth_chainId-{}", now_ms), "eth_chainId", &[])
            .await?;
        hex_to_u64(&res)
    }

    pub async fn gas_price(&self, now_ms: u64) -> Result<u128, String> {
        let res: String = self
            .call(format!("eth_gasPrice-{}", now_ms), "eth_gasPrice", &[])
            .await?;
        hex_to_u128(&res)
    }

    pub async fn max_priority_fee_per_gas(&self, now_ms: u64) -> Result<u128, String> {
        let res: String = self
            .call(
                format!("eth_maxPriorityFeePerGas-{}", now_ms),
                "eth_maxPriorityFeePerGas",
                &[],
            )
            .await?;
        hex_to_u128(&res)
    }

    pub async fn block_number(&self, now_ms: u64) -> Result<u64, String> {
        let res: String = self
            .call(
                format!("eth_blockNumber-{}", now_ms),
                "eth_blockNumber",
                &[],
            )
            .await?;
        hex_to_u64(&res)
    }

    pub async fn get_transaction_count(
        &self,
        now_ms: u64,
        address: &Address,
    ) -> Result<u64, String> {
        let res: String = self
            .call(
                format!("eth_getTransactionCount-{}", now_ms),
                "eth_getTransactionCount",
                &[address.to_string().into(), "latest".into()],
            )
            .await?;
        hex_to_u64(&res)
    }

    // pub async fn get_balance(&self, now_ms: u64, address: &Address) -> Result<u128, String> {
    //     let res: String = self
    //         .call(
    //             format!("eth_getBalance-{}", now_ms),
    //             "eth_getBalance",
    //             &[address.to_string().into(), "finalized".into()],
    //         )
    //         .await?;
    //     hex_to_u128(&res)
    // }

    pub async fn get_transaction_receipt(
        &self,
        now_ms: u64,
        tx_hash: &TxHash,
    ) -> Result<Option<TransactionReceipt>, String> {
        self.call(
            format!("eth_getTransactionReceipt-{}", now_ms),
            "eth_getTransactionReceipt",
            &[tx_hash.to_string().into()],
        )
        .await
    }

    pub async fn send_raw_transaction(
        &self,
        now_ms: u64,
        signed_tx: String,
    ) -> Result<Value, String> {
        self.call(
            format!("eth_sendRawTransaction-{}", now_ms),
            "eth_sendRawTransaction",
            &[signed_tx.into()],
        )
        .await
    }

    pub async fn call_contract(
        &self,
        now_ms: u64,
        contract: &Address,
        call_data: String,
    ) -> Result<Vec<u8>, String> {
        let call_object = serde_json::json!({
            "to": contract.to_string(),
            "data": call_data,
        });

        let res: String = self
            .call(
                format!("eth_call-{}", now_ms),
                "eth_call",
                &[call_object, "latest".into()],
            )
            .await?;
        let res = res.strip_prefix("0x").unwrap_or(&res);
        <Vec<u8>>::from_hex(res).map_err(|err| err.to_string())
    }

    // pub async fn erc20_name(&self, now_ms: u64, contract: &Address) -> Result<String, String> {
    //     let res = self
    //         .call_contract(now_ms, contract, "0x06fdde03".to_string())
    //         .await?;
    //     decode_abi_string(&res)
    // }

    #[allow(dead_code)]
    pub async fn erc20_symbol(&self, now_ms: u64, contract: &Address) -> Result<String, String> {
        let res = self
            .call_contract(now_ms, contract, "0x95d89b41".to_string())
            .await?;
        decode_abi_string(&res)
    }

    pub async fn erc20_decimals(&self, now_ms: u64, contract: &Address) -> Result<u8, String> {
        let res = self
            .call_contract(now_ms, contract, "0x313ce567".to_string())
            .await?;
        let v = decode_abi_uint(&res)?;
        u8::try_from(v).map_err(|_| "decimals overflow u8".to_string())
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
            max_response_bytes: None, //optional for request
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

pub fn encode_erc20_transfer(to: &Address, value: u128) -> Vec<u8> {
    const TRANSFER_SELECTOR: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb]; // keccak256("transfer(address,uint256)")[:4]

    let mut call_data = Vec::with_capacity(4 + 32 + 32);
    call_data.extend_from_slice(&TRANSFER_SELECTOR);

    let mut padded_to = [0u8; 32];
    padded_to[12..].copy_from_slice(to.as_slice());
    call_data.extend_from_slice(&padded_to);

    let value_bytes = U256::from(value).to_be_bytes::<32>();
    call_data.extend_from_slice(&value_bytes);

    call_data
}

fn hex_to_u64(s: &str) -> Result<u64, String> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u64::from_str_radix(s, 16).map_err(|err| err.to_string())
}

fn hex_to_u128(s: &str) -> Result<u128, String> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u128::from_str_radix(s, 16).map_err(|err| err.to_string())
}

#[allow(dead_code)]
fn decode_abi_string(bytes: &[u8]) -> Result<String, String> {
    if bytes.len() < 64 {
        return Err("abi string result too short".to_string());
    }

    let offset = U256::try_from_be_slice(&bytes[0..32]).unwrap();
    let offset = usize::try_from(offset).unwrap_or(isize::MAX as usize);
    if bytes.len() < offset + 32 {
        return Err("abi string length out of bounds".to_string());
    }

    let len = U256::try_from_be_slice(&bytes[offset..offset + 32]).unwrap();
    let len = usize::try_from(len).unwrap_or(isize::MAX as usize);
    if bytes.len() < offset + 32 + len {
        return Err("abi string data out of bounds".to_string());
    }

    let data = &bytes[offset + 32..offset + 32 + len];
    String::from_utf8(data.to_vec()).map_err(|err| err.to_string())
}

fn decode_abi_uint(bytes: &[u8]) -> Result<U256, String> {
    if bytes.len() != 32 {
        return Err("abi uint result must be 32 bytes".to_string());
    }
    Ok(U256::from_be_slice(bytes))
}

// fn decode_abi_address(bytes: &[u8]) -> Result<Address, String> {
//     if bytes.len() != 32 {
//         return Err("abi address result must be 32 bytes".to_string());
//     }
//     Address::try_from(&bytes[12..32]).map_err(|err| err.to_string())
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use ic_cdk::management_canister::{HttpRequestResult, TransformContext};

    #[test]
    fn test_encode_erc20_transfer() {
        let addr = Address::from_hex("0x00112233445566778899aabbccddeeff00112233").unwrap();
        let encoded = encode_erc20_transfer(&addr, 12345);

        let mut expected = vec![0xa9, 0x05, 0x9c, 0xbb];
        expected.extend(vec![0u8; 12]);
        expected.extend_from_slice(addr.as_ref());
        expected.extend(U256::from(12345u128).to_be_bytes::<32>());

        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_hex_to_u64_and_u128() {
        assert_eq!(hex_to_u64("0x2a").unwrap(), 42);
        assert_eq!(hex_to_u128("0xff").unwrap(), 255);
        assert!(hex_to_u64("g1").is_err());
        assert!(hex_to_u128("xyz").is_err());
    }

    #[test]
    fn test_decode_abi_string() {
        let mut payload = Vec::new();
        payload.extend(U256::from(32u8).to_be_bytes::<32>());
        payload.extend(U256::from(11u8).to_be_bytes::<32>());
        payload.extend_from_slice(b"hello world");
        payload.extend(vec![0u8; 21]);

        assert_eq!(decode_abi_string(&payload).unwrap(), "hello world");

        assert!(decode_abi_string(&payload[..60]).is_err());
    }

    #[test]
    fn test_decode_abi_uint() {
        let value = U256::from(999u64).to_be_bytes::<32>();
        assert_eq!(decode_abi_uint(&value).unwrap(), U256::from(999u64));
        assert!(decode_abi_uint(&value[..31]).is_err());
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

    #[test]
    fn test_chain_id_uses_mock_outcall() {
        let mock = MockHttpOutcall::new(vec![success_response(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x2a"
        }))]);

        let client = EvmClient::new(vec!["https://rpc.one".to_string()], 5, None, mock.clone());
        let value = futures::executor::block_on(client.chain_id(1_000)).unwrap();

        assert_eq!(value, 42);
        assert_eq!(mock.urls(), vec!["https://rpc.one".to_string()]);
    }

    #[test]
    fn test_http_request_fallbacks_between_providers() {
        let mock = MockHttpOutcall::new(vec![
            Err("network down".to_string()),
            success_response(serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": "0xa"
            })),
        ]);

        let client = EvmClient::new(
            vec!["https://first".to_string(), "https://second".to_string()],
            5,
            None,
            mock.clone(),
        );

        let block = futures::executor::block_on(client.block_number(2_000)).unwrap();

        assert_eq!(block, 10);
        assert_eq!(
            mock.urls(),
            vec!["https://first".to_string(), "https://second".to_string()]
        );
    }

    #[test]
    fn test_call_handles_error_payload() {
        let error_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {"code": -32000, "message": "execution reverted"}
        });
        let mock = MockHttpOutcall::new(vec![success_response(error_body)]);
        let client = EvmClient::new(vec!["https://rpc".to_string()], 5, None, mock);

        let result: Result<u64, _> =
            futures::executor::block_on(client.call("id-key".to_string(), "method", &[]));
        assert!(result.unwrap_err().contains("execution reverted"));
    }

    #[test]
    fn test_get_transaction_receipt() {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": null
        });
        let mock = MockHttpOutcall::new(vec![success_response(body)]);
        let client = EvmClient::new(vec!["https://rpc".to_string()], 5, None, mock);

        let tx_hash = TxHash::from([0u8; 32]);
        let result = futures::executor::block_on(client.get_transaction_receipt(1000, &tx_hash));
        assert!(result.unwrap().is_none());

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "type": "0x2",
                "from": "0x9ac6b9ffbb4269fc51cf0ef7bcd322cefb3e5e14",
                "to": "0xe74583edaff618d88463554b84bc675196b36990",
                "status": "0x1",
                "cumulativeGasUsed": "0x3e45f",
                "logsBloom": "0x0000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000210800000000000000000000000000000000000000000000000000100000000000000000000004000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000800000000000000000000000000020000000000000000000000000000000000000000800000000000000",
                "logs": [
                    {
                        "address": "0xe74583edaff618d88463554b84bc675196b36990",
                        "topics": [
                            "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
                            "0x0000000000000000000000009ac6b9ffbb4269fc51cf0ef7bcd322cefb3e5e14",
                            "0x0000000000000000000000009792cc010fe26155c676d0cc0057a3c66564fbcd"
                        ],
                        "data": "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000",
                        "blockNumber": "0x418f472",
                        "transactionHash": "0xbbded599a5f088cb82d9b439043ff691857ebff4f480225d5d563aed4ef11aaa",
                        "transactionIndex": "0x3",
                        "blockHash": "0xc2259f320a755bb1f21ab3cd3590f6838a48c8167268088dcf648acee2362b15",
                        "logIndex": "0x5",
                        "removed": false
                    }
                ],
                "transactionHash": "0xbbded599a5f088cb82d9b439043ff691857ebff4f480225d5d563aed4ef11aaa",
                "contractAddress": null,
                "gasUsed": "0xcbdb",
                "blockHash": "0xc2259f320a755bb1f21ab3cd3590f6838a48c8167268088dcf648acee2362b15",
                "blockNumber": "0x418f472",
                "transactionIndex": "0x3",
                "effectiveGasPrice": "0x7270e00"
            }
        });
        let mock = MockHttpOutcall::new(vec![success_response(body)]);
        let client = EvmClient::new(vec!["https://rpc".to_string()], 5, None, mock);

        let tx_hash =
            TxHash::from_hex("0xbbded599a5f088cb82d9b439043ff691857ebff4f480225d5d563aed4ef11aaa")
                .unwrap();
        let result =
            futures::executor::block_on(client.get_transaction_receipt(1000, &tx_hash)).unwrap();
        assert!(result.is_some());
        println!("{:?}", result);
    }
}
