use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
    time::{SystemTime, UNIX_EPOCH},
};

use itertools::Itertools;
use near_units::{near::parse, parse_near};
use serde_json::json;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use utils::get_dir_path;
use workspaces::{result::ExecutionFinalResult, Account, AccountId, InMemorySigner};
use xlsxwriter::Workbook;

use crate::utils::convert_oct_u128_from_string;

pub mod utils;

const OCT_TOKEN_CONTRACT_ID: &str = "f5cfbc74057c610c8ef151a439252680ac68c6dc.factory.bridge.near";
#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    init_log();
    let args: Vec<String> = env::args().collect();

    let signer_id = args[1].clone();
    let filename = args[2].clone();

    let worker = workspaces::mainnet()
        .rpc_addr("https://1rpc.io/near")
        // .rpc_addr("https://near-testnet.infura.io/v3/4f80a04e6eb2437a9ed20cb874e10d55")
        .await?;

    let signer = Account::from_file(get_dir_path(&signer_id), &worker)?;

    let receiver_list = read_receiver_list(&filename);

    send_oct_by_receiver_list(&signer, receiver_list).await?;

    anyhow::Ok(())
}

pub async fn send_oct_by_receiver_list(
    signer: &Account,
    receiver_list: Vec<(AccountId, u128)>,
) -> anyhow::Result<()> {
    let oct_token_contract_id = OCT_TOKEN_CONTRACT_ID.parse()?;

    let filename_by_timestamp = format!(
        "result_{}.xlsx",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
    );
    let workbook = Workbook::new(&filename_by_timestamp)?;
    let mut sheet = workbook.add_worksheet(None)?;

    let mut i = 0;
    for (receiver, amount) in receiver_list {
        let result = ft_transfer(signer, &oct_token_contract_id, receiver.clone(), amount)
            .await
            .into_result();

        match result {
            Ok(v) => {
                // v.outcome().receipt_ids
                let receipts = v
                    .outcome()
                    .receipt_ids
                    .iter()
                    .map(|e| e.to_string())
                    .join(",");
                sheet.write_string(i, 0, &receiver.to_string(), None)?;
                sheet.write_string(i, 1, &amount.to_string(), None)?;
                sheet.write_string(i, 2, &receipts, None)?;
            }
            Err(error) => {
                tracing::error!(
                    "Failed to send {} oct to {},error: {} ",
                    receiver,
                    amount,
                    error
                );
                break;
            }
        };
        i += 1;
    }
    workbook.close()?;

    anyhow::Ok(())
}

pub fn read_receiver_list(filename: &str) -> Vec<(AccountId, u128)> {
    let mut receiver_list = vec![];

    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.unwrap();
        let mut split = line.split(',');
        let col1 = split.next().unwrap();
        let col2 = split.next().unwrap();
        receiver_list.push((col1.parse().unwrap(), convert_oct_u128_from_string(col2)));
    }

    receiver_list
}

pub async fn ft_transfer(
    signer: &Account,
    contract_id: &AccountId,
    receiver_id: AccountId,
    amount: u128,
) -> ExecutionFinalResult {
    let result = signer
        .call(contract_id, "ft_transfer")
        .args_json(json!({
            "receiver_id": receiver_id,
            "amount": amount.to_string(),
        }))
        .deposit(1)
        .transact()
        .await
        .unwrap();
    tracing::info!("{:?}", &result);
    result
}

pub fn init_log() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // 输出到控制台中
    let formatting_layer = fmt::layer().pretty().with_writer(std::io::stderr);

    Registry::default()
        .with(env_filter)
        .with(formatting_layer)
        .init();
}
