use std::sync::Arc;

use alloy::{
    network::{Ethereum, Network},
    providers::ProviderBuilder,
    rpc::client::RpcClient,
    transports::Transport,
};
use anyhow::{anyhow, Result};
use defillama::{Chain, Coin, CoinsClient};
use erc20::{
    clients::{CachableTokenClient, TokenClient},
    mainnet::{ETH, WETH},
    stores::BasicTokenStore,
    TokenId,
};
use futures::StreamExt;
use stapifaction::json::ToJsonIterable;
use tokio::{select, signal, sync::mpsc};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use types::{Order, OrderAsset, OrderDetails};
use uniswapx::{
    decode_order, orders_stream,
    types::{OrderStatus, OrdersRequest},
};

pub async fn start(eth_http_rpc: String) -> Result<()> {
    let (shutdown_handle, mut shutdown_complete) = mpsc::channel(1);
    let token = CancellationToken::new();
    let cloned_token = token.clone();
    let mut orders = Vec::<Order>::new();
    let coins_client = CoinsClient::default();
    let http_provider = ProviderBuilder::<_, Ethereum>::new().on_client(
        RpcClient::builder()
            .reqwest_http(eth_http_rpc.parse().unwrap())
            .boxed(),
    );
    let token_client = CachableTokenClient::new(
        TokenClient::new(Arc::new(http_provider)),
        1,
        BasicTokenStore::new(),
    );

    tokio::spawn(async move {
        /*
        let request = OrdersRequest {
            chain_id: Some(1),
            cursor: Some(String::from("eyJjaGFpbklkIjoxLCJjcmVhdGVkQXQiOjE2OTA0NDM2NDcsIm9yZGVySGFzaCI6IjB4MDUzQUNGMjkxNjFENDQwRjgzNDM0Mzg0QTQ0NzIwRUNFNzg1MzExMzgyQzI3MEJERjExMjkwRTM1ODFFMDQ1QyJ9")),
            ..Default::default()
        };*/

        let request = OrdersRequest {
            chain_id: Some(137),
            order_status: Some(OrderStatus::Filled),
            ..Default::default()
        };

        let mut timestamp = 0;

        let mut orders_stream =
            orders_stream(String::from("https://api.uniswap.org/v2/orders"), request).boxed();

        loop {
            select! {
                Some(order) = orders_stream.next() => {
                    match order {
                        Ok(order) => {
                            if order.created_at < timestamp {
                                panic!("STOP");
                            }

                            timestamp = order.created_at;

                            info!("Adding order {}, {timestamp}", order.order_hash);
                            match build_order(order, &coins_client, &token_client, Chain::Polygon).await {
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

async fn build_order<N, T>(
    uniswapx_order: uniswapx::types::Order,
    coins_client: &CoinsClient,
    token_client: &CachableTokenClient<N, T>,
    chain: Chain,
) -> Result<Order>
where
    N: Network + Clone,
    T: Transport + Clone,
{
    let encoded_order = uniswapx_order.encoded_order.to_string();
    let decoded_order = decode_order(&encoded_order).ok();
    let (input_token, input_coin) = if uniswapx_order.input.token.is_zero() {
        (
            Arc::new(ETH.clone()),
            Coin::Address(chain.clone(), WETH.address.0.into()),
        )
    } else {
        let input_token = token_client
            .retrieve_token(TokenId::Address(uniswapx_order.input.token))
            .await?;
        (
            input_token.clone(),
            Coin::Address(chain.clone(), input_token.address.0.into()),
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
            Coin::Address(chain.clone(), WETH.address.0.into()),
        )
    } else {
        let output_token = token_client
            .retrieve_token(TokenId::Address(output.token))
            .await?;
        (
            output_token.clone(),
            Coin::Address(chain, output_token.address.0.into()),
        )
    };

    let output_token_price = coins_client
        .historical_prices(uniswapx_order.created_at, &[output_coin.clone()])
        .await?
        .get(&output_coin)
        .ok_or(anyhow!(
            "Failed to get historical price for output: {}",
            &output_coin
        ))?
        .price;

    let settled = uniswapx_order
        .settled_amounts
        .and_then(|amounts| amounts.first().cloned());

    let order = Order {
        hash: format!("{:?}", uniswapx_order.order_hash),
        chain_id: uniswapx_order.chain_id,
        ty: uniswapx_order.order_type.clone(),
        created_at: uniswapx_order.created_at,
        status: uniswapx_order.order_status.clone(),
        input: OrderAsset::new(
            &input_token,
            uniswapx_order.input.start_amount,
            uniswapx_order.input.end_amount,
            settled.clone().and_then(|s| s.amount_in),
            coins_client
                .historical_prices(uniswapx_order.created_at, &[input_coin.clone()])
                .await?
                .get(&input_coin)
                .ok_or(anyhow!(
                    "Failed to get historical price for input: {}",
                    &input_coin
                ))?
                .price,
        ),
        output: OrderAsset::new(
            &output_token,
            output.start_amount,
            output.end_amount,
            settled.map(|s| s.amount_out),
            output_token_price,
        ),
        fee: fee.map(|f| {
            OrderAsset::new(
                &output_token,
                f.start_amount,
                f.end_amount,
                None,
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
            additional_validation_data: decoded_order.info.additionalValidationData.to_string(),
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
