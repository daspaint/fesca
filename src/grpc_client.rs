use tonic::transport::Channel;
use rand::Rng;

// Importiere die generierten Proto-Definitionen
pub mod correlated_randomness {
    tonic::include_proto!("correlated_randomness");
}

use correlated_randomness::correlated_randomness_service_client::CorrelatedRandomnessServiceClient;
use correlated_randomness::{
    AckMessage, ComputedValueMessage, RhoMessage, VerificationRequest, VerificationResponse,
};

// Client für die Kommunikation zwischen Parteien
pub struct PartyClient {
    p1_client: CorrelatedRandomnessServiceClient<Channel>,
    p2_client: CorrelatedRandomnessServiceClient<Channel>,
    p3_client: CorrelatedRandomnessServiceClient<Channel>,
}

impl PartyClient {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let p1_client = CorrelatedRandomnessServiceClient::connect("http://[::1]:50051").await?;
        let p2_client = CorrelatedRandomnessServiceClient::connect("http://[::1]:50052").await?;
        let p3_client = CorrelatedRandomnessServiceClient::connect("http://[::1]:50053").await?;
        
        Ok(Self {
            p1_client,
            p2_client,
            p3_client,
        })
    }
    
    // P1 wählt ρ₁ und sendet es an P2
    pub async fn p1_send_rho1(&mut self, rho1: u32) -> Result<(), Box<dyn std::error::Error>> {
        println!("P1: Wähle ρ₁ = {}", rho1);
        println!("P1: Sende ρ₁ = {} an P2", rho1);
        
        let request = tonic::Request::new(RhoMessage {
            rho_value: rho1,
            sender_id: "P1".to_string(),
        });
        
        let response = self.p2_client.clone().send_rho1(request).await?;
        println!("P2 Antwort: {}", response.into_inner().message);
        
        Ok(())
    }
    
    // P2 wählt ρ₂ und sendet es an P3
    pub async fn p2_send_rho2(&mut self, rho2: u32) -> Result<(), Box<dyn std::error::Error>> {
        println!("P2: Wähle ρ₂ = {}", rho2);
        println!("P2: Sende ρ₂ = {} an P3", rho2);
        
        let request = tonic::Request::new(RhoMessage {
            rho_value: rho2,
            sender_id: "P2".to_string(),
        });
        
        let response = self.p3_client.clone().send_rho2(request).await?;
        println!("P3 Antwort: {}", response.into_inner().message);
        
        Ok(())
    }
    
    // P3 wählt ρ₃ und sendet es an P1
    pub async fn p3_send_rho3(&mut self, rho3: u32) -> Result<(), Box<dyn std::error::Error>> {
        println!("P3: Wähle ρ₃ = {}", rho3);
        println!("P3: Sende ρ₃ = {} an P1", rho3);
        
        let request = tonic::Request::new(RhoMessage {
            rho_value: rho3,
            sender_id: "P3".to_string(),
        });
        
        let response = self.p1_client.clone().send_rho3(request).await?;
        println!("P1 Antwort: {}", response.into_inner().message);
        
        Ok(())
    }
    
    // Sende berechnete Werte an alle Parteien
    pub async fn send_computed_values(
        &mut self,
        alpha: u32,
        beta: u32,
        gamma: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // P1 sendet α
        let p1_request = tonic::Request::new(ComputedValueMessage {
            computed_value: alpha,
            party_id: "P1".to_string(),
            rho_sent: 0, // Wird später korrekt gesetzt
            rho_received: 0, // Wird später korrekt gesetzt
        });
        
        // P2 sendet β
        let p2_request = tonic::Request::new(ComputedValueMessage {
            computed_value: beta,
            party_id: "P2".to_string(),
            rho_sent: 0, // Wird später korrekt gesetzt
            rho_received: 0, // Wird später korrekt gesetzt
        });
        
        // P3 sendet γ
        let p3_request = tonic::Request::new(ComputedValueMessage {
            computed_value: gamma,
            party_id: "P3".to_string(),
            rho_sent: 0, // Wird später korrekt gesetzt
            rho_received: 0, // Wird später korrekt gesetzt
        });
        
        // Sende an alle Parteien
        let _response1 = self.p1_client.clone().send_computed_value(p1_request).await?;
        let _response2 = self.p2_client.clone().send_computed_value(p2_request).await?;
        let _response3 = self.p3_client.clone().send_computed_value(p3_request).await?;
        
        Ok(())
    }
    
    // Verifiziere die Korrelation
    pub async fn verify_correlation(
        &mut self,
        alpha: u32,
        beta: u32,
        gamma: u32,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(VerificationRequest {
            values: vec![
                ComputedValueMessage {
                    computed_value: alpha,
                    party_id: "P1".to_string(),
                    rho_sent: 0,
                    rho_received: 0,
                },
                ComputedValueMessage {
                    computed_value: beta,
                    party_id: "P2".to_string(),
                    rho_sent: 0,
                    rho_received: 0,
                },
                ComputedValueMessage {
                    computed_value: gamma,
                    party_id: "P3".to_string(),
                    rho_sent: 0,
                    rho_received: 0,
                },
            ],
        });
        
        let response = self.p1_client.clone().verify_correlation(request).await?;
        let verification_response = response.into_inner();
        
        println!("Verifikation: {}", verification_response.details);
        println!("Ergebnis: {}", if verification_response.is_valid { "✅ Gültig" } else { "❌ Ungültig" });
        
        Ok(verification_response.is_valid)
    }
}

// Funktion für das vollständige gRPC-Protokoll
pub async fn run_grpc_protocol() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== gRPC Protokoll Simulation ===");
    
    let mut client = PartyClient::new().await?;
    let mut rng = rand::thread_rng();
    
    // Alle drei Parteien wählen ihre ρ-Werte
    let rho1 = rng.gen_range(0..=1);
    let rho2 = rng.gen_range(0..=1);
    let rho3 = rng.gen_range(0..=1);
    
    // Kommunikation: P1 → P2 → P3 → P1
    client.p1_send_rho1(rho1).await?;
    client.p2_send_rho2(rho2).await?;
    client.p3_send_rho3(rho3).await?;
    
    // Berechne α, β, γ
    let alpha = rho3 ^ rho1; // P1: α = ρ₃ ⊕ ρ₁
    let beta = rho1 ^ rho2;  // P2: β = ρ₁ ⊕ ρ₂
    let gamma = rho2 ^ rho3; // P3: γ = ρ₂ ⊕ ρ₃
    
    println!("\n=== Berechnete Werte ===");
    println!("P1 (α): {} ⊕ {} = {}", rho3, rho1, alpha);
    println!("P2 (β): {} ⊕ {} = {}", rho1, rho2, beta);
    println!("P3 (γ): {} ⊕ {} = {}", rho2, rho3, gamma);
    
    // Sende berechnete Werte
    client.send_computed_values(alpha, beta, gamma).await?;
    
    // Verifiziere Korrelation
    let is_valid = client.verify_correlation(alpha, beta, gamma).await?;
    
    if is_valid {
        println!("✅ gRPC Protokoll erfolgreich: α ⊕ β ⊕ γ = 0");
    } else {
        println!("❌ gRPC Protokoll fehlgeschlagen");
    }
    
    Ok(())
} 