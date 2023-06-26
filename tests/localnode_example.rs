use fuels::{
    accounts::predicate::Predicate,
    prelude::ViewOnlyAccount,
    test_helpers::{launch_custom_provider_and_get_wallets, WalletsConfig},
    types::{AssetId, Bits256, ContractId},
};
use spark_sdk::{
    limit_orders_utils::{
        limit_orders_interactions::{cancel_order, create_order, fulfill_order},
        LimitOrderPredicateConfigurables,
    },
    proxy_utils::{deploy_proxy_contract, ProxySendFundsToPredicateParams},
};
use src20_sdk::{deploy_token_contract, token_abi_calls, DeployTokenConfig};

const FRC20_BIN_PATH: &str = "artefacts/FRC20.bin";
const PREDICATE_BIN_PATH: &str = "artefacts/limit-order-predicate.bin";
const PROXY_BIN_PATH: &str = "artefacts/proxy-contract.bin";

#[tokio::test]
async fn main() {
    let wallets_config = WalletsConfig::new(Some(5), Some(1), Some(1_000_000_000));
    let wallets = launch_custom_provider_and_get_wallets(wallets_config, None, None).await;
    let admin = &wallets[0]; //token owner
    let alice = &wallets[1]; //token mint recipient
    let bob = &wallets[2]; //token mint recipient
    println!("Wallets:\nAdmin address: 0x{:?}", admin.address().hash);
    println!("Alice address: 0x{:?}", alice.address().hash);
    println!("Bob   address: 0x{:?}\n", bob.address().hash);

    //token deploy
    let usdc_config = &DeployTokenConfig {
        name: "USD Coin".to_owned(),
        symbol: "USDC".to_owned(),
        decimals: 6,
    };
    let usdc = deploy_token_contract(admin, usdc_config, FRC20_BIN_PATH).await;
    let usdc_asset_id = AssetId::from(*ContractId::from(usdc.contract_id()));

    let uni_config = &DeployTokenConfig {
        name: "Uniswap".to_owned(),
        symbol: "UNI".to_owned(),
        decimals: 9,
    };
    let uni = deploy_token_contract(admin, uni_config, FRC20_BIN_PATH).await;
    let uni_asset_id = AssetId::from(*ContractId::from(uni.contract_id()));

    println!("Tokens:\nUSDC address : 0x{usdc_asset_id}\nUNI  address : 0x{uni_asset_id}\n");

    //mint
    let usdc_mint_amount = 1000_000_000; //1000 USDC
    token_abi_calls::mint(&usdc, usdc_mint_amount, alice.address().into())
        .await
        .unwrap();
    assert!(alice.get_asset_balance(&usdc_asset_id).await.unwrap() == usdc_mint_amount);

    let uni_mint_amount = 200_000_000_000; //200 uni
    token_abi_calls::mint(&uni, uni_mint_amount, bob.address().into())
        .await
        .unwrap();
    assert!(bob.get_asset_balance(&uni_asset_id).await.unwrap() == uni_mint_amount);

    //predicate
    let price_decimals = 9;
    let exp = (price_decimals + usdc_config.decimals - uni_config.decimals).into();
    let price = uni_mint_amount * 10u64.pow(exp) / usdc_mint_amount;
    println!("Order price   : {:?} UNI/USDC", price);

    let configurables = LimitOrderPredicateConfigurables::new()
        .set_ASSET0(Bits256::from_hex_str(&usdc_asset_id.to_string()).unwrap())
        .set_ASSET1(Bits256::from_hex_str(&uni_asset_id.to_string()).unwrap())
        .set_ASSET0_DECIMALS(usdc_config.decimals)
        .set_ASSET1_DECIMALS(uni_config.decimals)
        .set_MAKER(Bits256::from_hex_str(&alice.address().hash().to_string()).unwrap())
        .set_PRICE(price)
        .set_PRICE_DECIMALS(price_decimals)
        .set_MIN_FULFILL_AMOUNT0(0);

    let predicate: Predicate = Predicate::load_from(PREDICATE_BIN_PATH)
        .unwrap()
        .with_configurables(configurables)
        .with_provider(admin.provider().unwrap().to_owned());
    println!("predicateRoot : 0x{:?}", predicate.address().hash);

    //proxy
    let proxy = deploy_proxy_contract(alice, PROXY_BIN_PATH).await;
    let proxy_address = format!("0x{}", proxy.contract_id().hash);
    println!("proxy address : 0x{:?}\n", proxy.contract_id().hash);

    //cancel order
    let params = ProxySendFundsToPredicateParams {
        predicate_root: predicate.address().into(),
        asset_0: usdc.contract_id().into(),
        asset_1: uni.contract_id().into(),
        maker: alice.address().into(),
        min_fulfill_amount_0: 1,
        price,
        asset_0_decimals: 6,
        asset_1_decimals: 9,
        price_decimals: 9,
    };

    create_order(&alice, &proxy_address, params, usdc_mint_amount)
        .await
        .unwrap();

    cancel_order(&alice, &predicate, usdc_asset_id, usdc_mint_amount)
        .await
        .unwrap();

    assert!(alice.get_asset_balance(&usdc_asset_id).await.unwrap() == usdc_mint_amount);
    println!("Cancel order  OK ");

    //fulfill order
    let params = ProxySendFundsToPredicateParams {
        predicate_root: predicate.address().into(),
        asset_0: usdc.contract_id().into(),
        asset_1: uni.contract_id().into(),
        maker: alice.address().into(),
        min_fulfill_amount_0: 1,
        price,
        asset_0_decimals: 6,
        asset_1_decimals: 9,
        price_decimals: 9,
    };

    create_order(&alice, &proxy_address, params, usdc_mint_amount)
        .await
        .unwrap();

    fulfill_order(
        &bob,
        &predicate,
        alice.address(),
        usdc_asset_id,
        usdc_mint_amount, //amont0
        uni_asset_id,
        uni_mint_amount, //amont0
    )
    .await
    .unwrap();

    assert!(alice.get_asset_balance(&uni_asset_id).await.unwrap() == uni_mint_amount);
    assert!(bob.get_asset_balance(&usdc_asset_id).await.unwrap() == usdc_mint_amount);
    println!("Fulfill order OK ");
    println!("\n\n");
}
