//! Noether's theorem for agent systems.
//!
//! In classical mechanics, Noether's theorem states: every continuous symmetry
//! of the action functional yields a conserved quantity.
//!
//! For agents:
//! - Time-translation symmetry → energy conservation (token budget is constant)
//! - Spatial-translation symmetry → momentum conservation (task difficulty is neutral)
//! - Rotational symmetry → angular momentum conservation (strategy diversity is preserved)
//!
//! When a symmetry is broken, the corresponding conservation law fails — and we can
//! MEASURE the violation to detect when agents are being pushed off-conservation.

/// An agent state in phase space (position + momentum).
#[derive(Debug, Clone)]
pub struct AgentState {
    pub id: usize,
    pub position: Vec<f64>,    // Task progress
    pub momentum: Vec<f64>,    // Token allocation velocity
    pub mass: f64,             // Agent weight/budget
}

impl AgentState {
    pub fn new(id: usize, position: Vec<f64>, momentum: Vec<f64>, mass: f64) -> Self {
        Self { id, position, momentum, mass }
    }

    /// Kinetic energy: p²/2m
    pub fn kinetic_energy(&self) -> f64 {
        self.momentum.iter().map(|p| p * p).sum::<f64>() / (2.0 * self.mass)
    }

    /// Linear momentum magnitude
    pub fn momentum_magnitude(&self) -> f64 {
        self.momentum.iter().map(|p| p * p).sum::<f64>().sqrt()
    }

    /// Angular momentum L = r × p (2D: r1*p2 - r2*p1)
    pub fn angular_momentum(&self) -> f64 {
        if self.position.len() >= 2 && self.momentum.len() >= 2 {
            self.position[0] * self.momentum[1] - self.position[1] * self.momentum[0]
        } else {
            0.0
        }
    }
}

/// A Lagrangian: L = T - V (kinetic minus potential).
#[derive(Debug, Clone)]
pub struct Lagrangian {
    pub potential: fn(&[f64]) -> f64,
}

impl Lagrangian {
    pub fn new(potential: fn(&[f64]) -> f64) -> Self {
        Self { potential }
    }

    /// Evaluate the Lagrangian for a given state.
    pub fn evaluate(&self, state: &AgentState) -> f64 {
        state.kinetic_energy() - (self.potential)(&state.position)
    }

    /// Total energy E = T + V (Hamiltonian).
    pub fn energy(&self, state: &AgentState) -> f64 {
        state.kinetic_energy() + (self.potential)(&state.position)
    }
}

/// A symmetry transformation.
#[derive(Debug, Clone)]
pub enum Symmetry {
    /// Time translation: H is constant → energy conserved.
    TimeTranslation,
    /// Spatial translation along axis → momentum conserved.
    SpatialTranslation { axis: usize },
    /// Rotation in plane (i,j) → angular momentum conserved.
    Rotation { i: usize, j: usize },
    /// Scale transformation → virial conservation.
    Scale,
}

/// A conserved quantity with its associated symmetry.
#[derive(Debug, Clone)]
pub struct ConservedQuantity {
    pub name: String,
    pub symmetry: Symmetry,
    pub value: f64,
    pub agent_id: usize,
}

impl ConservedQuantity {
    pub fn new(name: &str, symmetry: Symmetry, value: f64, agent_id: usize) -> Self {
        Self { name: name.to_string(), symmetry, value, agent_id }
    }
}

/// Apply a symmetry transformation to a state (infinitesimal).
pub fn apply_symmetry(state: &AgentState, symmetry: &Symmetry, epsilon: f64) -> AgentState {
    let mut new_state = state.clone();
    match symmetry {
        Symmetry::TimeTranslation => {
            // Time shift: evolve position by epsilon * momentum/mass
            for i in 0..new_state.position.len().min(new_state.momentum.len()) {
                new_state.position[i] += epsilon * new_state.momentum[i] / new_state.mass;
            }
        }
        Symmetry::SpatialTranslation { axis } => {
            if *axis < new_state.position.len() {
                new_state.position[*axis] += epsilon;
            }
        }
        Symmetry::Rotation { i, j } => {
            let ni = *i % new_state.position.len();
            let nj = *j % new_state.position.len();
            let xi = new_state.position[ni];
            let xj = new_state.position[nj];
            let pi = new_state.momentum[ni];
            let pj = new_state.momentum[nj];
            // Infinitesimal rotation
            new_state.position[ni] = xi * epsilon.cos() - xj * epsilon.sin();
            new_state.position[nj] = xi * epsilon.sin() + xj * epsilon.cos();
            new_state.momentum[ni] = pi * epsilon.cos() - pj * epsilon.sin();
            new_state.momentum[nj] = pi * epsilon.sin() + pj * epsilon.cos();
        }
        Symmetry::Scale => {
            for p in &mut new_state.position { *p *= 1.0 + epsilon; }
            for m in &mut new_state.momentum { *m *= 1.0 - epsilon; }
        }
    }
    new_state
}

/// Check if a Lagrangian is invariant under a symmetry (to tolerance).
pub fn check_symmetry(
    lagrangian: &Lagrangian,
    state: &AgentState,
    symmetry: &Symmetry,
    epsilon: f64,
    tolerance: f64,
) -> bool {
    let l_before = lagrangian.evaluate(state);
    let transformed = apply_symmetry(state, symmetry, epsilon);
    let l_after = lagrangian.evaluate(&transformed);
    (l_after - l_before).abs() < tolerance
}

/// Compute the Noether conserved quantity for a given symmetry.
pub fn noether_charge(state: &AgentState, symmetry: &Symmetry) -> ConservedQuantity {
    match symmetry {
        Symmetry::TimeTranslation => {
            // Conserved: total energy (Hamiltonian)
            let free_lagrangian = Lagrangian::new(|_| 0.0);
            ConservedQuantity::new("Energy", Symmetry::TimeTranslation, free_lagrangian.energy(state), state.id)
        }
        Symmetry::SpatialTranslation { axis } => {
            // Conserved: momentum along axis
            let val = if *axis < state.momentum.len() { state.momentum[*axis] } else { 0.0 };
            ConservedQuantity::new(
                &format!("Momentum[{}]", axis),
                Symmetry::SpatialTranslation { axis: *axis },
                val,
                state.id,
            )
        }
        Symmetry::Rotation { i, j } => {
            // Conserved: angular momentum in (i,j) plane
            let ni = *i % state.position.len().max(1);
            let nj = *j % state.position.len().max(1);
            let l = state.angular_momentum();
            ConservedQuantity::new(
                &format!("AngularMomentum[{},{}]", ni, nj),
                Symmetry::Rotation { i: *i, j: *j },
                l,
                state.id,
            )
        }
        Symmetry::Scale => {
            // Conserved: virial G = Σ p·r
            let virial: f64 = state.position.iter()
                .zip(state.momentum.iter())
                .map(|(r, p)| r * p)
                .sum();
            ConservedQuantity::new("Virial", Symmetry::Scale, virial, state.id)
        }
    }
}

/// Fleet-level Noether analysis.
pub struct FleetNoether {
    pub agents: Vec<AgentState>,
    pub lagrangian: Lagrangian,
}

impl FleetNoether {
    pub fn new(agents: Vec<AgentState>, lagrangian: Lagrangian) -> Self {
        Self { agents, lagrangian }
    }

    /// Fleet total energy.
    pub fn fleet_energy(&self) -> f64 {
        self.agents.iter().map(|a| self.lagrangian.energy(a)).sum()
    }

    /// Fleet total momentum.
    pub fn fleet_momentum(&self) -> Vec<f64> {
        let dim = self.agents.iter().map(|a| a.momentum.len()).max().unwrap_or(0);
        let mut total = vec![0.0; dim];
        for agent in &self.agents {
            for (i, p) in agent.momentum.iter().enumerate() {
                total[i] += p * agent.mass;
            }
        }
        total
    }

    /// Fleet total angular momentum.
    pub fn fleet_angular_momentum(&self) -> f64 {
        self.agents.iter().map(|a| a.angular_momentum() * a.mass).sum()
    }

    /// Fleet virial.
    pub fn fleet_virial(&self) -> f64 {
        self.agents.iter().map(|a| {
            a.position.iter().zip(a.momentum.iter()).map(|(r, p)| r * p * a.mass).sum::<f64>()
        }).sum()
    }

    /// All Noether charges for the fleet.
    pub fn all_charges(&self) -> Vec<ConservedQuantity> {
        let mut charges = Vec::new();
        charges.push(ConservedQuantity::new("FleetEnergy", Symmetry::TimeTranslation, self.fleet_energy(), 0));
        let mom = self.fleet_momentum();
        for (i, m) in mom.iter().enumerate() {
            charges.push(ConservedQuantity::new(
                &format!("FleetMomentum[{}]", i),
                Symmetry::SpatialTranslation { axis: i },
                *m,
                0,
            ));
        }
        charges.push(ConservedQuantity::new("FleetAngularMomentum", Symmetry::Rotation { i: 0, j: 1 }, self.fleet_angular_momentum(), 0));
        charges.push(ConservedQuantity::new("FleetVirial", Symmetry::Scale, self.fleet_virial(), 0));
        charges
    }

    /// Measure symmetry violation: compare charges before and after evolution.
    pub fn symmetry_violation(&self, evolved: &FleetNoether) -> Vec<(String, f64)> {
        let before = self.all_charges();
        let after = evolved.all_charges();
        before.iter().zip(after.iter())
            .map(|(b, a)| (b.name.clone(), (a.value - b.value).abs()))
            .collect()
    }
}

/// Euler-Lagrange evolution step (symplectic Euler).
pub fn euler_lagrange_step(
    state: &AgentState,
    lagrangian: &Lagrangian,
    potential_grad: fn(&[f64]) -> Vec<f64>,
    dt: f64,
) -> AgentState {
    let grad = potential_grad(&state.position);
    let mut new_momentum = state.momentum.clone();
    let mut new_position = state.position.clone();

    // Update momentum: dp/dt = -∇V
    for (p, g) in new_momentum.iter_mut().zip(grad.iter()) {
        *p -= g * dt;
    }

    // Update position: dr/dt = p/m
    for (r, p) in new_position.iter_mut().zip(new_momentum.iter()) {
        *r += p * dt / state.mass;
    }

    AgentState::new(state.id, new_position, new_momentum, state.mass)
}

/// Run a trajectory and track conservation law violations.
pub fn track_conservation(
    initial: &AgentState,
    lagrangian: &Lagrangian,
    potential_grad: fn(&[f64]) -> Vec<f64>,
    dt: f64,
    n_steps: usize,
) -> Vec<(f64, f64, f64, f64)> {
    let mut state = initial.clone();
    let mut history = Vec::new();
    let e0 = lagrangian.energy(&state);
    let p0 = state.momentum_magnitude();
    let l0 = state.angular_momentum();
    let v0: f64 = state.position.iter().zip(state.momentum.iter()).map(|(r, p)| r * p).sum();

    for _ in 0..n_steps {
        state = euler_lagrange_step(&state, lagrangian, potential_grad, dt);
        let e = lagrangian.energy(&state);
        let p = state.momentum_magnitude();
        let l = state.angular_momentum();
        let v: f64 = state.position.iter().zip(state.momentum.iter()).map(|(r, p)| r * p).sum();
        history.push((
            (e - e0).abs(), // energy drift
            (p - p0).abs(), // momentum drift
            (l - l0).abs(), // angular momentum drift
            (v - v0).abs(), // virial drift
        ));
    }
    history
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero_potential(_: &[f64]) -> f64 { 0.0 }
    fn zero_grad(_: &[f64]) -> Vec<f64> { vec![0.0, 0.0] }
    fn harmonic_potential(q: &[f64]) -> f64 { q.iter().map(|x| x * x).sum::<f64>() * 0.5 }
    fn harmonic_grad(q: &[f64]) -> Vec<f64> { q.iter().copied().collect() }

    fn make_agent(id: usize) -> AgentState {
        AgentState::new(id, vec![1.0, 0.0], vec![0.0, 1.0], 1.0)
    }

    #[test]
    fn test_kinetic_energy() {
        let a = AgentState::new(0, vec![0.0], vec![3.0, 4.0], 1.0);
        assert!((a.kinetic_energy() - 12.5).abs() < 1e-10);
    }

    #[test]
    fn test_momentum_magnitude() {
        let a = AgentState::new(0, vec![], vec![3.0, 4.0], 1.0);
        assert!((a.momentum_magnitude() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_angular_momentum() {
        let a = AgentState::new(0, vec![1.0, 0.0], vec![0.0, 1.0], 1.0);
        assert!((a.angular_momentum() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_lagrangian_free_particle() {
        let lagrangian = Lagrangian::new(zero_potential);
        let a = AgentState::new(0, vec![0.0], vec![2.0], 1.0);
        // L = T - V = 2.0 - 0 = 2.0
        assert!((lagrangian.evaluate(&a) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_energy_conservation_free_particle() {
        let lagrangian = Lagrangian::new(zero_potential);
        let a = make_agent(0);
        let trajectory = track_conservation(&a, &lagrangian, zero_grad, 0.01, 100);
        let max_energy_drift = trajectory.iter().map(|(e, _, _, _)| *e).fold(0.0_f64, f64::max);
        assert!(max_energy_drift < 0.01, "Energy should be conserved for free particle, drift = {}", max_energy_drift);
    }

    #[test]
    fn test_momentum_conservation_free_particle() {
        let lagrangian = Lagrangian::new(zero_potential);
        let a = make_agent(0);
        let trajectory = track_conservation(&a, &lagrangian, zero_grad, 0.01, 100);
        let max_mom_drift = trajectory.iter().map(|(_, p, _, _)| *p).fold(0.0_f64, f64::max);
        assert!(max_mom_drift < 0.1, "Momentum magnitude should be roughly conserved");
    }

    #[test]
    fn test_noether_charge_energy() {
        let a = make_agent(0);
        let charge = noether_charge(&a, &Symmetry::TimeTranslation);
        assert!(charge.value > 0.0);
        assert_eq!(charge.name, "Energy");
    }

    #[test]
    fn test_noether_charge_momentum() {
        let a = make_agent(0);
        let charge = noether_charge(&a, &Symmetry::SpatialTranslation { axis: 1 });
        assert!((charge.value - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_noether_charge_angular() {
        let a = make_agent(0);
        let charge = noether_charge(&a, &Symmetry::Rotation { i: 0, j: 1 });
        assert!((charge.value - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_noether_charge_virial() {
        // Virial = Σ r_i * p_i. For make_agent: [1,0]·[0,1] = 0
        // Use a non-orthogonal agent instead
        let a = AgentState::new(0, vec![1.0, 1.0], vec![1.0, 0.0], 1.0);
        let charge = noether_charge(&a, &Symmetry::Scale);
        assert!(charge.value > 0.0, "Virial should be nonzero");
    }

    #[test]
    fn test_check_symmetry_free_particle_time() {
        let lagrangian = Lagrangian::new(zero_potential);
        let a = make_agent(0);
        assert!(check_symmetry(&lagrangian, &a, &Symmetry::TimeTranslation, 0.01, 0.1));
    }

    #[test]
    fn test_check_symmetry_harmonic_not_time() {
        let lagrangian = Lagrangian::new(harmonic_potential);
        let a = make_agent(0);
        // Harmonic potential breaks time translation for individual state
        // (energy is conserved but L changes)
        assert!(!check_symmetry(&lagrangian, &a, &Symmetry::TimeTranslation, 0.5, 0.01));
    }

    #[test]
    fn test_fleet_energy() {
        let fleet = FleetNoether::new(
            vec![make_agent(0), make_agent(1)],
            Lagrangian::new(zero_potential),
        );
        assert!(fleet.fleet_energy() > 0.0);
    }

    #[test]
    fn test_fleet_momentum() {
        let fleet = FleetNoether::new(
            vec![make_agent(0), make_agent(1)],
            Lagrangian::new(zero_potential),
        );
        let mom = fleet.fleet_momentum();
        // Both agents have momentum [0, 1], masses 1.0
        assert!((mom[0] - 0.0).abs() < 1e-10);
        assert!((mom[1] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_fleet_angular_momentum() {
        let fleet = FleetNoether::new(
            vec![make_agent(0), make_agent(1)],
            Lagrangian::new(zero_potential),
        );
        assert!((fleet.fleet_angular_momentum() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_fleet_all_charges() {
        let fleet = FleetNoether::new(
            vec![make_agent(0)],
            Lagrangian::new(zero_potential),
        );
        let charges = fleet.all_charges();
        assert!(charges.len() >= 4); // Energy + 2 momenta + angular + virial
    }

    #[test]
    fn test_symmetry_violation() {
        let fleet1 = FleetNoether::new(
            vec![make_agent(0)],
            Lagrangian::new(zero_potential),
        );
        let fleet2 = FleetNoether::new(
            vec![AgentState::new(0, vec![2.0, 0.0], vec![0.0, 2.0], 1.0)],
            Lagrangian::new(zero_potential),
        );
        let violations = fleet1.symmetry_violation(&fleet2);
        assert!(!violations.is_empty());
        // Energy should differ
        assert!(violations[0].1 > 0.0);
    }

    #[test]
    fn test_euler_lagrange_step() {
        let a = make_agent(0);
        let lagrangian = Lagrangian::new(zero_potential);
        let evolved = euler_lagrange_step(&a, &lagrangian, zero_grad, 0.1);
        // p=[0,1], m=1, dt=0.1 → dr = p*dt/m = [0, 0.1]
        // r=[1,0] → [1, 0.1]
        assert!((evolved.position[0] - 1.0).abs() < 1e-10);
        assert!((evolved.position[1] - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_harmonic_energy_nearly_conserved() {
        let lagrangian = Lagrangian::new(harmonic_potential);
        let a = make_agent(0);
        let trajectory = track_conservation(&a, &lagrangian, harmonic_grad, 0.001, 100);
        let max_energy_drift = trajectory.iter().map(|(e, _, _, _)| *e).fold(0.0_f64, f64::max);
        // Symplectic Euler should keep energy roughly conserved for small dt
        assert!(max_energy_drift < 0.1, "Energy drift too large: {}", max_energy_drift);
    }

    #[test]
    fn test_apply_spatial_translation() {
        let a = make_agent(0);
        let translated = apply_symmetry(&a, &Symmetry::SpatialTranslation { axis: 0 }, 5.0);
        assert!((translated.position[0] - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_apply_rotation() {
        let a = AgentState::new(0, vec![1.0, 0.0], vec![0.0, 1.0], 1.0);
        let rotated = apply_symmetry(&a, &Symmetry::Rotation { i: 0, j: 1 }, std::f64::consts::FRAC_PI_2);
        assert!((rotated.position[0]).abs() < 1e-10); // cos(90) = 0
        assert!((rotated.position[1] - 1.0).abs() < 1e-10); // sin(90) = 1
    }

    #[test]
    fn test_five_agent_fleet_conservation() {
        let agents: Vec<AgentState> = (0..5).map(|i| {
            let angle = i as f64 * std::f64::consts::TAU / 5.0;
            AgentState::new(i, vec![angle.cos(), angle.sin()], vec![-angle.sin(), angle.cos()], 1.0)
        }).collect();
        let fleet = FleetNoether::new(agents, Lagrangian::new(zero_potential));
        let charges = fleet.all_charges();

        // With symmetric setup, fleet momentum should be ~0
        let mom_charge = charges.iter().find(|c| c.name.contains("Momentum")).unwrap();
        assert!(mom_charge.value.abs() < 1.0, "Symmetric fleet should have ~0 net momentum");
    }

    #[test]
    fn test_virial_for_harmonic() {
        // For harmonic oscillator, virial theorem: <T> = <V> = E/2
        let a = AgentState::new(0, vec![1.0], vec![0.0], 1.0);
        let charge = noether_charge(&a, &Symmetry::Scale);
        // Virial = r·p = 1*0 = 0 at turning point
        assert!(charge.value.abs() < 1e-10);
    }
}
