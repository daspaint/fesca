use rand::Rng;

// Neue Implementierung basierend auf dem Paper-Protokoll
pub struct PartyState {
    pub rho: u8,  // Das zufällige Bit, das diese Partei wählt
    pub received: u8,  // Das empfangene Bit von der vorherigen Partei
    pub computed_value: u8,  // α, β oder γ
}

pub fn party_1_generate() -> PartyState {
    let mut rng = rand::thread_rng();
    let rho_1 = rng.gen_range(0..=1);
    
    // P1 sendet rho_1 an P2 (simuliert)
    println!("P1: Wähle ρ₁ = {}", rho_1);
    println!("P1: Sende ρ₁ = {} an P2", rho_1);
    
    // P1 empfängt ρ₃ von P3 (wird später simuliert)
    // Für jetzt nehmen wir an, dass P3 bereits ρ₃ gewählt hat
    let rho_3 = rng.gen_range(0..=1); // Simuliert ρ₃ von P3
    
    // P1 berechnet α = ρ₃ ⊕ ρ₁
    let alpha = rho_3 ^ rho_1;
    
    println!("P1: Empfange ρ₃ = {} von P3", rho_3);
    println!("P1: Berechne α = ρ₃ ⊕ ρ₁ = {} ⊕ {} = {}", rho_3, rho_1, alpha);
    
    PartyState {
        rho: rho_1,
        received: rho_3,
        computed_value: alpha,
    }
}

pub fn party_2_generate() -> PartyState {
    let mut rng = rand::thread_rng();
    let rho_2 = rng.gen_range(0..=1);
    
    // P2 sendet rho_2 an P3 (simuliert)
    println!("P2: Wähle ρ₂ = {}", rho_2);
    println!("P2: Sende ρ₂ = {} an P3", rho_2);
    
    // P2 empfängt ρ₁ von P1 (wird später simuliert)
    let rho_1 = rng.gen_range(0..=1); // Simuliert ρ₁ von P1
    
    // P2 berechnet β = ρ₁ ⊕ ρ₂
    let beta = rho_1 ^ rho_2;
    
    println!("P2: Empfange ρ₁ = {} von P1", rho_1);
    println!("P2: Berechne β = ρ₁ ⊕ ρ₂ = {} ⊕ {} = {}", rho_1, rho_2, beta);
    
    PartyState {
        rho: rho_2,
        received: rho_1,
        computed_value: beta,
    }
}

pub fn party_3_generate() -> PartyState {
    let mut rng = rand::thread_rng();
    let rho_3 = rng.gen_range(0..=1);
    
    // P3 sendet rho_3 an P1 (simuliert)
    println!("P3: Wähle ρ₃ = {}", rho_3);
    println!("P3: Sende ρ₃ = {} an P1", rho_3);
    
    // P3 empfängt ρ₂ von P2 (wird später simuliert)
    let rho_2 = rng.gen_range(0..=1); // Simuliert ρ₂ von P2
    
    // P3 berechnet γ = ρ₂ ⊕ ρ₃
    let gamma = rho_2 ^ rho_3;
    
    println!("P3: Empfange ρ₂ = {} von P2", rho_2);
    println!("P3: Berechne γ = ρ₂ ⊕ ρ₃ = {} ⊕ {} = {}", rho_2, rho_3, gamma);
    
    PartyState {
        rho: rho_3,
        received: rho_2,
        computed_value: gamma,
    }
}

// Funktion, die das vollständige Protokoll simuliert
pub fn simulate_paper_protocol() {
    println!("\n=== Paper-Protokoll Simulation ===");
    
    // Alle drei Parteien führen ihre Berechnungen durch
    let p1_state = party_1_generate();
    let p2_state = party_2_generate();
    let p3_state = party_3_generate();
    
    // Überprüfe, ob α ⊕ β ⊕ γ = 0
    let total_xor = p1_state.computed_value ^ p2_state.computed_value ^ p3_state.computed_value;
    
    println!("\n=== Ergebnisse ===");
    println!("P1 (α): {} ⊕ {} = {}", p1_state.received, p1_state.rho, p1_state.computed_value);
    println!("P2 (β): {} ⊕ {} = {}", p2_state.received, p2_state.rho, p2_state.computed_value);
    println!("P3 (γ): {} ⊕ {} = {}", p3_state.received, p3_state.rho, p3_state.computed_value);
    println!("α ⊕ β ⊕ γ = {} ⊕ {} ⊕ {} = {}", 
             p1_state.computed_value, p2_state.computed_value, p3_state.computed_value, total_xor);
    
    if total_xor == 0 {
        println!("✅ Erfolgreich: α ⊕ β ⊕ γ = 0");
    } else {
        println!("❌ Fehler: α ⊕ β ⊕ γ ≠ 0");
    }
    
    // Sicherheitsanalyse: Was jede Partei weiß
    println!("\n=== Sicherheitsanalyse ===");
    println!("P1 kennt: ρ₁ = {}, ρ₃ = {}, α = {}", p1_state.rho, p1_state.received, p1_state.computed_value);
    println!("P1 kennt NICHT: ρ₂ (da ρ₂ in β und γ enthalten ist)");
    println!("P2 kennt: ρ₂ = {}, ρ₁ = {}, β = {}", p2_state.rho, p2_state.received, p2_state.computed_value);
    println!("P2 kennt NICHT: ρ₃ (da ρ₃ in α und γ enthalten ist)");
    println!("P3 kennt: ρ₃ = {}, ρ₂ = {}, γ = {}", p3_state.rho, p3_state.received, p3_state.computed_value);
    println!("P3 kennt NICHT: ρ₁ (da ρ₁ in α und β enthalten ist)");
}
