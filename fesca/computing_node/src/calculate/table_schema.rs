/*
Deserializable schema of a table, used to extract bitstrings.
 */
use serde::Deserialize;


#[derive(Debug, Deserialize, Clone)]
pub struct Column {
    pub name: String,
    pub type_hint: Option<String>,
}


#[derive(Debug, Deserialize, Clone)]
pub struct DataOwner {
    pub owner_id: String,
    pub owner_name: String,
}


#[derive(Debug, Deserialize, Clone)]
pub struct Schema {
    pub columns: Vec<Column>,
    pub data_owner: DataOwner,
    pub row_count: u64,
    pub table_id: u64,
    pub table_name: String,
}