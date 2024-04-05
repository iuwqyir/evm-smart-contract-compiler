use serde::{Serialize,Deserialize};
use reqwest::Client;
use foundry_compilers::{artifacts::{CompilerInput, Settings, Source, Sources}, EvmVersion};
use std::collections::BTreeMap;
use std::str::FromStr;
use foundry_compilers::{CompilerOutput, Solc};

#[derive(Serialize, Deserialize, Debug)]
struct EtherscanResponse {
    status: String,
    message: String,
    result: Vec<ContractInfo>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ContractInfo {
  SourceCode: String,
  ABI: String,
  ContractName: String,
  FileName: Option<String>,
  CompilerVersion: String,
  OptimizationUsed: String,
  Runs: String,
  ConstructorArguments: String,
  EVMVersion: String,
  Library: String,
  LicenseType: String,
  Proxy: String,
  Implementation: String,
  SwarmSource: String
}

async fn fetch_contract_source_code(address: &str) -> Result<ContractInfo, anyhow::Error> {
  let client: Client = Client::new();
  let response: EtherscanResponse = client.get("https://api.etherscan.io/api")
    .query(&[("module", "contract"), ("action", "getsourcecode"), ("address", address)])
    .send()
    .await?
    .json()
    .await?;

  match response.result.get(0) {
      Some(contract_info) => Ok(contract_info.clone()),
      None => Err(anyhow::Error::msg("Unable to fetch contract from Etherscan")),
  }
}

fn get_compiler_input(contract_info: &ContractInfo) -> CompilerInput {
  if contract_info.SourceCode.starts_with("{{") {
    return serde_json::from_str(&contract_info.SourceCode[1..contract_info.SourceCode.len()-1]).expect("Failed to parse SourceCode JSON string")
  } else {
    let file_name: String = contract_info.FileName.clone().unwrap_or_else(|| {
      format!("{}.sol", contract_info.ContractName)
    });
    let mut sources: Sources = BTreeMap::new();
    sources.insert(file_name.into(), Source { content: contract_info.SourceCode.clone().into() });
    let mut settings: Settings = Settings::default();
    if contract_info.EVMVersion.to_lowercase() == "default" {
      settings.evm_version = None;
    } else {
      settings.evm_version = Some(EvmVersion::from_str(&contract_info.EVMVersion).unwrap());
    }
    CompilerInput {
      language: "Solidity".to_string(),
      sources,
      settings
    }
  }
}

fn extract_compiler_version(contract_info: &ContractInfo) -> String {
  let compiler_ver_without_commit = contract_info.CompilerVersion.split('+').next().unwrap_or("");
  let parsed_compiler_version = if compiler_ver_without_commit.starts_with('v') {
      &compiler_ver_without_commit[1..]
  } else {
      compiler_ver_without_commit
  };
  parsed_compiler_version.to_string()
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
  // 0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789 AA EntryPoint
  // 0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984 UNI
  let contract_info: ContractInfo = fetch_contract_source_code("0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984").await?;
  let compiler_input: CompilerInput = get_compiler_input(&contract_info);
  let compiler_version: String = extract_compiler_version(&contract_info);

  let solc: Solc = Solc::find_or_install_svm_version(&compiler_version).unwrap();
  let compiler_output: CompilerOutput = solc.compile_exact(&compiler_input).expect("Failed to compile contract");
  print!("{:?}", compiler_output);
  Ok(())
}
