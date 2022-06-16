use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionMetadata {
    pub update_authority: String,
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Option<Vec<Creator>>,
    pub is_mutable: bool,
    pub primary_sale_happened: bool,
    pub token_standard: Option<TokenStandard>,
    pub uses: Option<Uses>,
    pub collection: Option<Collection>,
    pub pubkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Creator {
    address: String,
    share: u8,
    verified: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TokenStandard {
    NonFungible,
    Fungible,
    FungibleAsset,
    NonfungibleEdition,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Uses {
    use_method: UseMethod,
    remaining: u64,
    total: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UseMethod {
    Burn,
    Multiple,
    Single,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    key: String,
    verified: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionNft {
    pub metadata: CollectionMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JRPCRequest {
    method: String,
    jsonrpc: String,
    params: Value,
    id: u8,
}

impl JRPCRequest {
    pub fn new(method: &str, params: Value) -> Self {
        Self {
            method: method.to_string(),
            jsonrpc: "2.0".to_string(),
            params,
            id: 1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GPAResponse {
    pub jsonrpc: String,
    pub id: u8,
    pub result: Vec<GPAResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GPAResult {
    pub pubkey: String,
    pub account: IndexIoAccount,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TLAResult {
    pub context: Context,
    pub value: Vec<LargestAccount>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Context {
    pub slot: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LargestAccount {
    pub address: String,
    pub amount: String,
    pub decimals: u8,
    #[serde(rename = "uiAmount")]
    pub ui_amount: f32,
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexIoAccount {
    pub data: Value,
    pub executable: bool,
    pub lamports: u64,
    pub owner: String,
    #[serde(rename = "rentEpoch")]
    pub rent_epoch: u64,
}

pub const THE_INDEX_MAINNET: &str = "https://rpc.theindex.io/mainnet-beta";
