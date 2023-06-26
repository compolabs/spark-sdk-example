use dotenv::dotenv;
use fuels::{
    accounts::{predicate::Predicate, wallet::WalletUnlocked},
    prelude::{Bech32ContractId, Provider, ViewOnlyAccount},
    types::{AssetId, Bits256, ContractId},
};
use std::str::FromStr;

use spark_sdk::{
    limit_orders_utils::{
        limit_orders_interactions::{cancel_order, create_order, fulfill_order},
        LimitOrderPredicateConfigurables,
    },
    proxy_utils::ProxySendFundsToPredicateParams,
};
use src20_sdk::{token_abi_calls, TokenContract};

const PREDICATE_BIN_PATH: &str = "artefacts/limit-order-predicate.bin";
const RPC: &str = "beta-3.fuel.network";
const PROXY_ADDRESS: &str = "0x8924a38ac11879670de1d0898c373beb1e35dca974c4cab8a70819322f6bd9c4";

const USDC: &str = "0x17e68049bb3cf21a85f00778fde367465bbd8263e8d4f8e47e533fe0df865658"; // usdc testnet assetId
const UNI: &str = "0xeaa756f320f175f0023a8c9fc2c9b7a03ce8d715f04ac49aba69d2b7d74e70b8"; // uni testnet assetId

#[tokio::test]
async fn main() {
    dotenv().ok();
    let provider = Provider::connect(RPC).await.unwrap();

    //should be a token owner, ask me a seed to allow minting
    let admin_secret = std::env::var("ADMIN").unwrap();
    let admin =
        WalletUnlocked::new_from_private_key(admin_secret.parse().unwrap(), Some(provider.clone()));
    println!("Wallets:\nAdmin address: 0x{:?}", admin.address().hash);

    let alice_secret = std::env::var("ALICE").unwrap();
    let alice =
        WalletUnlocked::new_from_private_key(alice_secret.parse().unwrap(), Some(provider.clone()));
    println!("Alice address: 0x{:?}\n", alice.address().hash);

    let bob_secret = std::env::var("BOB").unwrap();
    let bob =
        WalletUnlocked::new_from_private_key(bob_secret.parse().unwrap(), Some(provider.clone()));
    println!("Bob   address: 0x{:?}\n", bob.address().hash);

    //tokens
    let usdc_contract_id: Bech32ContractId = ContractId::from_str(USDC).unwrap().into();
    let usdc = TokenContract::new(usdc_contract_id, admin.clone());
    let usdc_asset_id = AssetId::from(*ContractId::from(usdc.contract_id()));
    let usdc_decimals = 6;

    let uni_contract_id: Bech32ContractId = ContractId::from_str(UNI).unwrap().into();
    let uni = TokenContract::new(uni_contract_id, admin.clone());
    let uni_asset_id = AssetId::from(*ContractId::from(uni.contract_id()));
    let uni_decimals = 9;

    println!("Tokens:\nUSDC address : 0x{usdc_asset_id}\nUNI  address : 0x{uni_asset_id}\n");

    //mint
    let amount0 = 1000_000_000_u64; //1000 USDC
    let amount1 = 300_000_000_000_u64; //200 UNI
    println!("USDC AssetId (asset0) = 0x{:?}", usdc_asset_id);
    println!("UNI AssetId (asset1) = 0x{:?}", uni_asset_id);
    println!("amount0 = {:?} USDC", amount0 / 1000_000);
    println!("amount1 = {:?} UNI", amount1 / 1000_000_000);

    let initial_alice_usdc_balance = alice.get_asset_balance(&usdc_asset_id).await.unwrap();
    let initial_bob_uni_balance = bob.get_asset_balance(&uni_asset_id).await.unwrap();
    if initial_alice_usdc_balance < amount0 {
        token_abi_calls::mint(&usdc, amount0, alice.address().into())
            .await
            .unwrap();
        println!("Alice minting {:?} USDC\n", amount0 / 1000_000);
    }
    if initial_bob_uni_balance < amount1 {
        token_abi_calls::mint(&uni, amount1, bob.address().into())
            .await
            .unwrap();
        println!("Bob minting {:?} UNI\n", amount1 / 1000_000_000);
    }
    let initial_alice_usdc_balance = alice.get_asset_balance(&usdc_asset_id).await.unwrap();
    let initial_alice_uni_balance = alice.get_asset_balance(&uni_asset_id).await.unwrap();
    let initial_bob_usdc_balance = bob.get_asset_balance(&usdc_asset_id).await.unwrap();

    //predicate
    let price_decimals = 9;
    let exp = (price_decimals + usdc_decimals - uni_decimals).into();
    let price = amount1 * 10u64.pow(exp) / amount0;
    println!("Order price   : {:?} UNI/USDC", price);

    let configurables = LimitOrderPredicateConfigurables::new()
        .set_ASSET0(Bits256::from_hex_str(&usdc_asset_id.to_string()).unwrap())
        .set_ASSET1(Bits256::from_hex_str(&uni_asset_id.to_string()).unwrap())
        .set_ASSET0_DECIMALS(usdc_decimals)
        .set_ASSET1_DECIMALS(uni_decimals)
        .set_MAKER(Bits256::from_hex_str(&alice.address().hash().to_string()).unwrap())
        .set_PRICE(price)
        .set_PRICE_DECIMALS(price_decimals)
        .set_MIN_FULFILL_AMOUNT0(0);

    let predicate: Predicate = Predicate::load_from(PREDICATE_BIN_PATH)
        .unwrap()
        .with_configurables(configurables)
        .with_provider(admin.provider().unwrap().to_owned());
    println!("predicateRoot : 0x{:?}", predicate.address().hash);

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

    create_order(&alice, PROXY_ADDRESS, params, amount0)
        .await
        .unwrap();

    cancel_order(&alice, &predicate, usdc_asset_id, amount0)
        .await
        .unwrap();

    assert!(alice.get_asset_balance(&usdc_asset_id).await.unwrap() == initial_alice_usdc_balance);
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

    create_order(&alice, PROXY_ADDRESS, params, amount0)
        .await
        .unwrap();

    fulfill_order(
        &bob,
        &predicate,
        alice.address(),
        usdc_asset_id,
        amount0,
        uni_asset_id,
        amount1,
    )
    .await
    .unwrap();

    assert!(
        alice.get_asset_balance(&uni_asset_id).await.unwrap()
            == initial_alice_uni_balance + amount1
    );
    assert!(
        bob.get_asset_balance(&usdc_asset_id).await.unwrap() == initial_bob_usdc_balance + amount0
    );
    println!("Fulfill order OK ");
    println!("\n\n");
}
