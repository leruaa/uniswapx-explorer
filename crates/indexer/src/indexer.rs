use std::sync::Arc;

use anyhow::Result;
use defillama::{Coin, CoinsClient};
use erc20::{
    mainnet::{ETH, WETH},
    TokenId, TokenStore,
};
use ethers::{
    providers::{Http, Provider},
    types::Bytes,
};
use futures::StreamExt;
use stapifaction::json::ToJsonIterable;
use tokio::{select, signal, sync::mpsc};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use types::{Order, OrderAsset, OrderDetails};
use uniswapx::{decode_order, orders_stream, types::OrdersRequest};

pub async fn start(eth_http_rpc: String) -> Result<()> {
    let (shutdown_handle, mut shutdown_complete) = mpsc::channel(1);
    let token = CancellationToken::new();
    let cloned_token = token.clone();
    let mut orders = Vec::<Order>::new();
    let coins_client = CoinsClient::default();
    let http_provider = Arc::new(Provider::<Http>::try_from(&eth_http_rpc)?);
    let token_store = TokenStore::new(1, http_provider);

    tokio::spawn(async move {
        let request = OrdersRequest {
            chain_id: Some(1),
            cursor: Some(String::from("eyJjaGFpbklkIjoxLCJjcmVhdGVkQXQiOjE2OTA0NDM2NDcsIm9yZGVySGFzaCI6IjB4MDUzQUNGMjkxNjFENDQwRjgzNDM0Mzg0QTQ0NzIwRUNFNzg1MzExMzgyQzI3MEJERjExMjkwRTM1ODFFMDQ1QyJ9")),
            ..Default::default()
        };

        let mut orders_stream =
            orders_stream(String::from("https://api.uniswap.org/v2/orders"), request).boxed();

        loop {
            select! {
                Some(order) = orders_stream.next() => {
                    match order {
                        Ok(order) => {
                            info!("Adding order {}", order.order_hash);
                            match build_order(order, &coins_client, &token_store).await {
                                Ok(order) => orders.push(order),
                                Err(err) => error!("Failed to build order: {err:#}"),
                            }
                        },
                        Err(err) => error!("Failed to fetch order: {err}"),
                    }
                },

                _ = cloned_token.cancelled() => {
                    info!("Persisting orders...");

                    orders.to_json("../web/public/data").unwrap();

                    shutdown_handle.send(()).await.unwrap();
                    return;
                }
            }
        }
    });

    wait_shutdown_signal().await;

    token.cancel();

    // When every sender has gone out of scope, the recv call
    // will return with an error. We ignore the error.
    let _ = shutdown_complete.recv().await;

    Ok(())
}

async fn build_order(
    uniswapx_order: uniswapx::types::Order,
    coins_client: &CoinsClient,
    token_store: &TokenStore,
) -> Result<Order> {
    let encoded_order = uniswapx_order.encoded_order.to_string();
    let decoded_order = decode_order(&encoded_order).ok();
    let (input_token, input_coin) = if uniswapx_order.input.token.is_zero() {
        (
            Arc::new(ETH.clone()),
            Coin::Address(1_u64.try_into().unwrap(), WETH.address.0.into()),
        )
    } else {
        let input_token = token_store
            .get(TokenId::Address(uniswapx_order.input.token))
            .await?;
        (
            input_token.clone(),
            Coin::Address(1_u64.try_into().unwrap(), input_token.address.0.into()),
        )
    };

    let (fee, outputs) = uniswapx_order
        .outputs
        .clone()
        .into_iter()
        .partition::<Vec<_>, _>(|o| {
            o.recipient.to_string() == "0x37a8f295612602f2774d331e562be9e61B83a327"
        });

    let output = outputs.first().unwrap();
    let fee = fee.first();
    let (output_token, output_coin) = if output.token.is_zero() {
        (
            Arc::new(ETH.clone()),
            Coin::Address(1_u64.try_into().unwrap(), WETH.address.0.into()),
        )
    } else {
        let output_token = token_store.get(TokenId::Address(output.token)).await?;
        (
            output_token.clone(),
            Coin::Address(1_u64.try_into().unwrap(), output_token.address.0.into()),
        )
    };

    let output_token_price = coins_client
        .current_prices(&[output_coin.clone()])
        .await?
        .get(&output_coin)
        .unwrap()
        .price;

    let order = Order {
        hash: uniswapx_order.order_hash.clone(),
        chain_id: uniswapx_order.chain_id,
        ty: uniswapx_order.order_type.clone(),
        created_at: uniswapx_order.created_at,
        status: uniswapx_order.order_status.clone(),
        input: OrderAsset::new(
            &input_token,
            uniswapx_order.input.start_amount,
            uniswapx_order.input.end_amount,
            coins_client
                .current_prices(&[input_coin.clone()])
                .await?
                .get(&input_coin)
                .unwrap()
                .price,
        ),
        output: OrderAsset::new(
            &output_token,
            output.start_amount,
            output.end_amount,
            output_token_price,
        ),
        fee: fee.map(|f| {
            OrderAsset::new(
                &output_token,
                f.start_amount,
                f.end_amount,
                output_token_price,
            )
        }),
        recipient: format!("{:?}", output.recipient),
        signature: uniswapx_order.signature.to_string(),
        tx: uniswapx_order.tx_hash,
        details: decoded_order.map(|decoded_order| OrderDetails {
            decay_start_time: decoded_order.decayStartTime.try_into().unwrap(),
            decay_end_time: decoded_order.decayEndTime.try_into().unwrap(),
            exclusive_filler: format!("{:?}", decoded_order.exclusiveFiller),
            exclusivity_override_bps: decoded_order.exclusivityOverrideBps.try_into().unwrap(),
            reactor: format!("{:?}", decoded_order.info.reactor),
            swapper: format!("{:?}", decoded_order.info.swapper),
            nonce: format!("{}", decoded_order.info.nonce),
            deadline: decoded_order.info.deadline.try_into().unwrap(),
            additional_validation_contract: format!(
                "{:?}",
                decoded_order.info.additionalValidationContract
            ),
            additional_validation_data: Bytes::from(decoded_order.info.additionalValidationData)
                .to_string(),
        }),
    };

    Ok(order)
}

async fn wait_shutdown_signal() {
    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }
}
