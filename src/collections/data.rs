use super::common::*;

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
    NonFungibleEdition,
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
    params: Vec<String>,
    id: u8,
}

impl JRPCRequest {
    pub fn new(method: &str, params: Vec<String>) -> Self {
        Self {
            method: method.to_string(),
            jsonrpc: "2.0".to_string(),
            params,
            id: 1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub id: u8,
    pub result: Vec<CollectionNft>,
}

#[derive(Debug)]
pub enum GetCollectionItemsMethods {
    TheIndexIO,
}

impl FromStr for GetCollectionItemsMethods {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "the_index_io" => Ok(GetCollectionItemsMethods::TheIndexIO),
            _ => Err(format!("Invalid method: {}", s)),
        }
    }
}

pub const PARALLEL_LIMIT: usize = 50;
