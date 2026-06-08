# si-noether-agent

> **Proof of Concept:** Noether's theorem for agent systems — every symmetry of the action functional yields a conserved quantity.

## The Insight

Emmy Noether proved (1918) that every continuous symmetry of a Lagrangian system corresponds to a conserved quantity:

| Symmetry | Conserved Quantity | Agent Meaning |
|----------|-------------------|---------------|
| Time translation | Energy | Token budget is constant |
| Spatial translation | Momentum | Task difficulty is neutral |
| Rotation | Angular momentum | Strategy diversity preserved |
| Scale | Virial | Work distribution invariant |

When a symmetry is **broken**, the conservation law **fails**. We measure the violation to detect when agents are being pushed off-conservation.

## What This Proves

1. **Energy conservation** = time-translation symmetry → budget stays constant
2. **Momentum conservation** = spatial symmetry → fair task allocation
3. **Angular momentum** = rotational symmetry → balanced strategy mix
4. **Symmetry breaking detection** = measure conservation violation to find drift

## Usage

```rust
use si_noether_agent::*;

// Define agent state
let agent = AgentState::new(0, vec![1.0, 0.0], vec![0.0, 1.0], 1.0);

// Compute Noether charges
let energy = noether_charge(&agent, &Symmetry::TimeTranslation);
let momentum = noether_charge(&agent, &Symmetry::SpatialTranslation { axis: 0 });
let angular = noether_charge(&agent, &Symmetry::Rotation { i: 0, j: 1 });

// Track conservation along trajectory
let lagrangian = Lagrangian::new(|q: &[f64]| q.iter().map(|x| x * x).sum::<f64>() * 0.5);
let history = track_conservation(&agent, &lagrangian, |q| q.iter().copied().collect(), 0.001, 1000);

// Fleet-level analysis
let fleet = FleetNoether::new(vec![agent], lagrangian);
let charges = fleet.all_charges();
```

## Modules

- `AgentState` — agent in phase space (position + momentum + mass)
- `Lagrangian` — action functional (kinetic - potential)
- `Symmetry` — symmetry transformations (time/space/rotation/scale)
- `noether_charge()` — compute the conserved quantity for each symmetry
- `FleetNoether` — fleet-level conservation analysis
- `track_conservation()` — monitor conservation law violations along trajectories

## Connection to Conservation Law

Noether's theorem IS the deep reason behind γ + η = C:
- γ (durable budget) = conserved under time symmetry
- η (ephemeral budget) = the "action" being extremized
- Conservation violation = symmetry breaking = budget leak

## Tests: 23

Covers: kinetic energy, momentum, angular momentum, Lagrangian evaluation, Noether charges, symmetry checking, Euler-Lagrange evolution, fleet conservation, symmetry violation measurement.

## License

MIT
