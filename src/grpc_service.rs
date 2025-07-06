use tonic::{transport::Server, Request, Response, Status};
use rand::Rng;
use std::sync::Arc;
use tokio::sync::Mutex;

// Importiere die generierten Proto-Definitionen
pub mod correlated_randomness {
    tonic::include_proto!("correlated_randomness");
}

use correlated_randomness::correlated_randomness_service_server::CorrelatedRandomnessService;
use correlated_randomness::{
    AckMessage, ComputedValueMessage, RhoMessage, VerificationRequest, VerificationResponse,
};

// Shared state für jede Partei
#[derive(Debug)]
pub struct PartyState {
    pub rho: Option<u32>,
    pub received_rho: Option<u32>,
    pub computed_value: Option<u32>,
    pub party_id: String,
}

// gRPC Service Implementation
#[derive(Debug)]
pub struct CorrelatedRandomnessServiceImpl {
    state: Arc<Mutex<PartyState>>,
}

impl CorrelatedRandomnessServiceImpl {
    pub fn new(party_id: String) -> Self {
        Self {
            state: Arc::new(Mutex::new(PartyState {
                rho: None,
                received_rho: None,
                computed_value: None,
                party_id,
            })),
        }
    }
}

#[tonic::async_trait]
impl CorrelatedRandomnessService for CorrelatedRandomnessServiceImpl {
    async fn send_rho1(
        &self,
        request: Request<RhoMessage>,
    ) -> Result<Response<AckMessage>, Status> {
        let rho_msg = request.into_inner();
        println!("P2: Empfange ρ₁ = {} von P1", rho_msg.rho_value);
        
        let mut state = self.state.lock().await;
        state.received_rho = Some(rho_msg.rho_value);
        
        Ok(Response::new(AckMessage {
            success: true,
            message: format!("P2 hat ρ₁ = {} empfangen", rho_msg.rho_value),
        }))
    }

    async fn send_rho2(
        &self,
        request: Request<RhoMessage>,
    ) -> Result<Response<AckMessage>, Status> {
        let rho_msg = request.into_inner();
        println!("P3: Empfange ρ₂ = {} von P2", rho_msg.rho_value);
        
        let mut state = self.state.lock().await;
        state.received_rho = Some(rho_msg.rho_value);
        
        Ok(Response::new(AckMessage {
            success: true,
            message: format!("P3 hat ρ₂ = {} empfangen", rho_msg.rho_value),
        }))
    }

    async fn send_rho3(
        &self,
        request: Request<RhoMessage>,
    ) -> Result<Response<AckMessage>, Status> {
        let rho_msg = request.into_inner();
        println!("P1: Empfange ρ₃ = {} von P3", rho_msg.rho_value);
        
        let mut state = self.state.lock().await;
        state.received_rho = Some(rho_msg.rho_value);
        
        Ok(Response::new(AckMessage {
            success: true,
            message: format!("P1 hat ρ₃ = {} empfangen", rho_msg.rho_value),
        }))
    }

    async fn send_computed_value(
        &self,
        request: Request<ComputedValueMessage>,
    ) -> Result<Response<AckMessage>, Status> {
        let computed_msg = request.into_inner();
        println!(
            "{}: Empfange berechneten Wert {} von {}",
            self.state.lock().await.party_id, computed_msg.computed_value, computed_msg.party_id
        );
        
        Ok(Response::new(AckMessage {
            success: true,
            message: format!("Berechneter Wert {} empfangen", computed_msg.computed_value),
        }))
    }

    async fn verify_correlation(
        &self,
        request: Request<VerificationRequest>,
    ) -> Result<Response<VerificationResponse>, Status> {
        let verification_req = request.into_inner();
        
        if verification_req.values.len() != 3 {
            return Ok(Response::new(VerificationResponse {
                is_valid: false,
                details: "Genau 3 Werte (α, β, γ) erforderlich".to_string(),
            }));
        }
        
        let alpha = verification_req.values[0].computed_value;
        let beta = verification_req.values[1].computed_value;
        let gamma = verification_req.values[2].computed_value;
        
        let total_xor = alpha ^ beta ^ gamma;
        let is_valid = total_xor == 0;
        
        let details = format!(
            "α ⊕ β ⊕ γ = {} ⊕ {} ⊕ {} = {}",
            alpha, beta, gamma, total_xor
        );
        
        println!("Verifikation: {}", details);
        println!("Ergebnis: {}", if is_valid { "✅ Gültig" } else { "❌ Ungültig" });
        
        Ok(Response::new(VerificationResponse {
            is_valid,
            details,
        }))
    }
}

// Funktionen für die Parteien
pub async fn run_party_1_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = "[::1]:50051".parse()?;
    let service = CorrelatedRandomnessServiceImpl::new("P1".to_string());
    
    println!("P1 Server startet auf {}", addr);
    
    Server::builder()
        .add_service(correlated_randomness::correlated_randomness_service_server::CorrelatedRandomnessServiceServer::new(service))
        .serve(addr)
        .await?;
    
    Ok(())
}

pub async fn run_party_2_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = "[::1]:50052".parse()?;
    let service = CorrelatedRandomnessServiceImpl::new("P2".to_string());
    
    println!("P2 Server startet auf {}", addr);
    
    Server::builder()
        .add_service(correlated_randomness::correlated_randomness_service_server::CorrelatedRandomnessServiceServer::new(service))
        .serve(addr)
        .await?;
    
    Ok(())
}

pub async fn run_party_3_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = "[::1]:50053".parse()?;
    let service = CorrelatedRandomnessServiceImpl::new("P3".to_string());
    
    println!("P3 Server startet auf {}", addr);
    
    Server::builder()
        .add_service(correlated_randomness::correlated_randomness_service_server::CorrelatedRandomnessServiceServer::new(service))
        .serve(addr)
        .await?;
    
    Ok(())
} 