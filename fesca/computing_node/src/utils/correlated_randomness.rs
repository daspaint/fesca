use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use log::{info, warn, error};
use tonic::Request;

// Include the generated protobuf code for key exchange
pub mod key_exchange_service {
    tonic::include_proto!("key_exchange_service");
}

use key_exchange_service::{
    key_exchange_service_client::KeyExchangeServiceClient,
    SendKeyRequest, RequestKeyRequest,
};

/// Configuration structure for computing node
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComputingNodeConfig {
    pub computation_urls: ComputationUrls,
    pub node_id: String,
    pub storage_path: String,
    pub key_1: String,
    pub key_2: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComputationUrls {
    pub url1: String,
    pub url2: String,
}

impl ComputingNodeConfig {
    /// Load configuration from config_computing_node.json
    pub fn load() -> Result<Self> {
        let config_path = "computing_node/config_computing_node.json";
        let config_content = fs::read_to_string(config_path)?;
        let config: ComputingNodeConfig = serde_json::from_str(&config_content)?;
        Ok(config)
    }

    /// Save configuration back to config_computing_node.json
    pub fn save(&self) -> Result<()> {
        let config_path = "computing_node/config_computing_node.json";
        let config_content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, config_content)?;
        Ok(())
    }

    /// Check if keys are already set
    pub fn has_keys(&self) -> bool {
        !self.key_1.is_empty() && !self.key_2.is_empty()
    }
}

/// Generate a 256-bit key as a hex string
fn generate_256bit_key() -> String {
    let mut rng = rand::rng();
    let mut key_bytes = [0u8; 32]; // 32 bytes = 256 bits
    rng.fill(&mut key_bytes);
    
    // Convert to hex string
    key_bytes.iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>()
}

/// Main function to generate keys for correlated randomness
/// 1. If config already has keys set, do nothing
/// 2. Otherwise generate key_1 and write to config
/// 3. Send key_1 to url1 and receive key_2 from url2
pub async fn generate_keys() -> Result<()> {
    info!("Starting correlated randomness key generation...");
    
    // Load current configuration
    let mut config = ComputingNodeConfig::load()?;
    
    // Check if keys are already set
    if config.has_keys() {
        info!("Keys already set in configuration. Skipping key generation.");
        return Ok(());
    }
    
    // Generate key_1 if not set
    if config.key_1.is_empty() {
        info!("Generating new 256-bit key for key_1...");
        config.key_1 = generate_256bit_key();
        info!("Generated key_1: {}...", &config.key_1[..16]); // Show first 16 chars only
        
        // Save the updated configuration
        config.save()?;
        info!("Saved key_1 to configuration file.");
    }
    
    // Perform key exchange if URLs are configured
    if !config.computation_urls.url1.is_empty() && !config.computation_urls.url2.is_empty() {
        info!("Starting key exchange...");
        
        let mut send_success = false;
        let mut receive_success = false;
        
        // Send key_1 to url1
        info!("Sending key_1 to: {}", config.computation_urls.url1);
        match send_key_to_url(&config.key_1, &config.computation_urls.url1).await {
            Ok(()) => {
                info!("Successfully sent key_1 to url1");
                send_success = true;
            }
            Err(e) => {
                error!("Failed to send key_1 to url1: {}", e);
            }
        }
        
        // Receive key_2 from url2
        info!("Attempting to receive key_2 from: {}", config.computation_urls.url2);
        match receive_key_from_url(&config.computation_urls.url2).await {
            Ok(received_key) => {
                config.key_2 = received_key;
                config.save()?;
                info!("Successfully received and saved key_2");
                receive_success = true;
            }
            Err(e) => {
                warn!("Failed to receive key_2 from url2: {}", e);
            }
        }
        
        // Return error if key exchange failed - this enables retries
        if !send_success || !receive_success {
            return Err(anyhow::anyhow!("Key exchange incomplete: send={}, receive={}", send_success, receive_success));
        }
        
        info!("Key exchange completed successfully");
    } else {
        warn!("URLs not configured for key exchange. Skipping network operations.");
        warn!("Configure url1 and url2 in config to enable key exchange.");
        return Err(anyhow::anyhow!("URLs not configured for key exchange"));
    }
    
    Ok(())
}

/// Send a key to another computing node via gRPC
async fn send_key_to_url(key: &str, url: &str) -> Result<()> {
    // Format the URL properly for gRPC
    let target_url = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("http://{}", url)
    };
    
    info!("Connecting to gRPC service at: {}", target_url);
    
    // Create gRPC client
    let mut client = match KeyExchangeServiceClient::connect(target_url.clone()).await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to connect to {}: {}", target_url, e);
            return Err(anyhow::anyhow!("gRPC connection failed: {}", e));
        }
    };
    
    // Get our node ID from config
    let config = ComputingNodeConfig::load()?;
    
    // Create the request
    let request = Request::new(SendKeyRequest {
        sender_node_id: config.node_id.clone(),
        key: key.to_string(),
        key_type: "key_1".to_string(),
    });
    
    // Send the key
    match client.send_key(request).await {
        Ok(response) => {
            let resp = response.into_inner();
            if resp.success {
                info!("Successfully sent key to {}: {}", url, resp.message);
                Ok(())
            } else {
                error!("Failed to send key to {}: {}", url, resp.message);
                Err(anyhow::anyhow!("Key sending failed: {}", resp.message))
            }
        }
        Err(e) => {
            error!("gRPC call failed when sending key to {}: {}", url, e);
            Err(anyhow::anyhow!("gRPC send_key failed: {}", e))
        }
    }
}

/// Receive a key from another computing node via gRPC
async fn receive_key_from_url(url: &str) -> Result<String> {
    // Format the URL properly for gRPC
    let target_url = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("http://{}", url)
    };
    
    info!("Connecting to gRPC service at: {} to request key", target_url);
    
    // Create gRPC client
    let mut client = match KeyExchangeServiceClient::connect(target_url.clone()).await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to connect to {}: {}", target_url, e);
            return Err(anyhow::anyhow!("gRPC connection failed: {}", e));
        }
    };
    
    // Get our node ID from config
    let config = ComputingNodeConfig::load()?;
    
    // Try to request the key (single attempt - retries handled at higher level)
    info!("Requesting key_2 from {}", url);
    
    let request = Request::new(RequestKeyRequest {
        requester_node_id: config.node_id.clone(),
        key_type: "key_2".to_string(),
    });
    
    match client.request_key(request).await {
        Ok(response) => {
            let resp = response.into_inner();
            if resp.success && !resp.key.is_empty() {
                info!("Successfully received key_2 from {}", url);
                Ok(resp.key)
            } else if resp.success && resp.key.is_empty() {
                Err(anyhow::anyhow!("Key not ready yet at {}", url))
            } else {
                Err(anyhow::anyhow!("Failed to get key from {}: {}", url, resp.message))
            }
        }
        Err(e) => {
            Err(anyhow::anyhow!("gRPC call failed when requesting key from {}: {}", url, e))
        }
    }
}



