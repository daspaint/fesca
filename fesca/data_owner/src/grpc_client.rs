// gRPC Client Module
// ==================
// This module provides gRPC client functionality for the data owner to send
// table shares to computing nodes. It handles:
// 1. Establishing gRPC connections to computing nodes
// 2. Converting internal data structures to protobuf format
// 3. Sending share data with data owner information

use anyhow::Result;
use tonic::transport::Channel;

// Include the generated protobuf code
pub mod share_service {
    tonic::include_proto!("share_service");
}

use share_service::{
    share_service_client::ShareServiceClient,
    SendTableSharesRequest, SendTableSharesResponse,
    DataOwnerInfo as ProtoDataOwnerInfo, TableSchema as ProtoTableSchema,
    ColumnDescriptor as ProtoColumnDescriptor,
    ColumnType as ProtoColumnType,
    // Updated imports for binary format
    BinaryPartyData as ProtoBinaryPartyData,
    BinaryRow as ProtoBinaryRow,
    // Legacy imports (still needed for conversion)
    BooleanType, UnsignedIntType, FloatType, StringType,
    Charset as ProtoCharset, AsciiCharset, Utf8Charset,
};

use crate::types::{
    TableSchema, ColumnDescriptor, ColumnType, Charset,
    BinaryPartyData, BinaryRow,
};
use crate::config::DataOwnerInfo;

// Type alias for cleaner code
pub type DataOwner = DataOwnerInfo;

/// gRPC client for sending table shares to computing nodes
pub struct ShareClient {
    data_owner: DataOwner,
}

impl ShareClient {
    /// Create a new ShareClient with data owner information
    pub fn new(data_owner: DataOwner) -> Self {
        Self { data_owner }
    }

    /// Convert data owner information to protobuf format
    fn convert_data_owner_info(&self) -> ProtoDataOwnerInfo {
        ProtoDataOwnerInfo {
            owner_id: self.data_owner.owner_id.clone(),
            owner_name: self.data_owner.owner_name.clone(),
        }
    }

    /// Convert table schema to protobuf format
    fn convert_table_schema(&self, schema: &TableSchema) -> ProtoTableSchema {
        ProtoTableSchema {
            table_name: schema.table_name.clone(),
            table_id: schema.table_id,
            columns: schema.columns.iter().map(|col| self.convert_column_descriptor(col)).collect(),
            row_count: schema.row_count as u32,
        }
    }

    /// Convert column descriptor to protobuf format
    fn convert_column_descriptor(&self, col: &ColumnDescriptor) -> ProtoColumnDescriptor {
        ProtoColumnDescriptor {
            name: col.name.clone(),
            type_hint: Some(self.convert_column_type(&col.type_hint)),
        }
    }

    /// Convert column type to protobuf format
    fn convert_column_type(&self, col_type: &ColumnType) -> ProtoColumnType {
        match col_type {
            ColumnType::Boolean => ProtoColumnType {
                r#type: Some(share_service::column_type::Type::Boolean(BooleanType {})),
            },
            ColumnType::UnsignedInt => ProtoColumnType {
                r#type: Some(share_service::column_type::Type::UnsignedInt(UnsignedIntType {})),
            },
            ColumnType::Float => ProtoColumnType {
                r#type: Some(share_service::column_type::Type::Float(FloatType {})),
            },
            ColumnType::String { max_chars, charset } => ProtoColumnType {
                r#type: Some(share_service::column_type::Type::String(StringType {
                    max_chars: *max_chars as u32,
                    charset: Some(self.convert_charset(charset)),
                })),
            },
        }
    }

    /// Convert charset to protobuf format
    fn convert_charset(&self, charset: &Charset) -> ProtoCharset {
        match charset {
            Charset::Ascii => ProtoCharset {
                charset: Some(share_service::charset::Charset::Ascii(AsciiCharset {})),
            },
            Charset::Utf8 => ProtoCharset {
                charset: Some(share_service::charset::Charset::Utf8(Utf8Charset {})),
            },
        }
    }

    /// Convert binary party data to protobuf format
    pub fn convert_binary_party_data(&self, binary_data: &BinaryPartyData) -> ProtoBinaryPartyData {
        ProtoBinaryPartyData {
            party_id: binary_data.party_id,
            table_id: binary_data.table_id,
            rows: binary_data.rows.iter().map(|row| self.convert_binary_row(row)).collect(),
        }
    }

    /// Convert binary row to protobuf format
    fn convert_binary_row(&self, row: &BinaryRow) -> ProtoBinaryRow {
        ProtoBinaryRow {
            bitstring_a: row.bitstring_a.clone(),
            bitstring_b: row.bitstring_b.clone(),
            column_bit_offsets: row.column_bit_offsets.clone(),
            column_bit_lengths: row.column_bit_lengths.clone(),
        }
    }

    /// Send binary table shares to all three computing nodes
    pub async fn send_binary_table_shares(
        &self,
        schema: &TableSchema,
        binary_party_data: &[BinaryPartyData; 3],
        node_urls: &[String; 3],
    ) -> Result<Vec<SendTableSharesResponse>> {
        let mut responses = Vec::new();

        // Send binary shares to each computing node
        for (party_id, url) in node_urls.iter().enumerate() {
            let response = self.send_binary_to_node(url, schema, &binary_party_data[party_id]).await?;
            responses.push(response);
        }

        Ok(responses)
    }

    /// Send binary shares to a specific computing node
    async fn send_binary_to_node(
        &self,
        url: &str,
        schema: &TableSchema,
        binary_data: &BinaryPartyData,
    ) -> Result<SendTableSharesResponse> {
        // Establish gRPC connection
        let channel = Channel::from_shared(url.to_string())?
            .connect()
            .await?;
        
        let mut client = ShareServiceClient::new(channel);

        // Create the request with binary data
        let request = tonic::Request::new(SendTableSharesRequest {
            data_owner: Some(self.convert_data_owner_info()),
            schema: Some(self.convert_table_schema(schema)),
            party_data: Some(self.convert_binary_party_data(binary_data)),
        });

        // Send the request
        let response = client.send_table_shares(request).await?;
        Ok(response.into_inner())
    }
} 