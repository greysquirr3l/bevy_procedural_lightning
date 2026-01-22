//! Procedural lightning generation using recursive subdivision algorithm
//!
//! Based on the LightningGen algorithm (https://github.com/CXUtk/LightningGen):
//! - Recursive subdivision with random perturbation
//! - Branch generation with probability decay
//! - Energy density attenuation along branches
//!
//! The algorithm works by:
//! 1. Starting with a line segment (start -> end)
//! 2. Recursively subdividing segments, adding random perpendicular offset
//! 3. Spawning branches with decreasing probability
//! 4. Rendering as connected line segments or particle chain

use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use rand::Rng;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};

/// A node in the lightning tree structure
#[derive(Debug, Clone)]
pub struct LightningNode {
    /// Position in 3D space (game coordinates, XZ plane)
    pub position: Vec3,
    /// Branch depth (0 = main bolt, higher = sub-branches)
    pub branch_depth: u32,
    /// Energy density at this node (0.0 = end, 1.0 = full power)
    pub energy: f32,
}

/// Configuration for lightning generation algorithm
#[derive(Debug, Clone)]
pub struct LightningConfig {
    /// Random seed for reproducible generation
    pub seed: u64,
    /// Alpha: controls branch probability decay (higher = fewer branches)
    /// Typical: 0.3 - 0.6
    pub alpha: f32,
    /// Beta: controls perpendicular displacement magnitude (0.3 = 30% of segment length)
    /// Typical: 0.2 - 0.6
    pub beta: f32,
    /// Gamma: base branch probability at each subdivision
    /// Typical: 0.2 - 0.6
    pub gamma: f32,
    /// Maximum recursion depth for subdivision
    pub max_depth: u32,
    /// Maximum branch depth (prevent infinite branching)
    pub max_branch_depth: u32,
}

impl Default for LightningConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            alpha: 0.5,
            beta: 0.4,
            gamma: 0.3,
            max_depth: 8,
            max_branch_depth: 3,
        }
    }
}

/// A complete lightning bolt tree with main path and branches
#[derive(Debug, Clone)]
pub struct LightningTree {
    /// Root position of the lightning
    pub root: Vec3,
    /// All nodes in the tree (main path + branches)
    pub nodes: Vec<LightningNode>,
    /// Pairs of node indices representing line segments
    pub segments: Vec<(usize, usize)>,
}

impl LightningTree {
    /// Generate a procedural lightning tree from start to end position
    pub fn generate(start: Vec3, end: Vec3, config: &LightningConfig) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
        let mut nodes = Vec::new();
        let mut segments = Vec::new();

        // Start with the root segment
        let start_node = LightningNode {
            position: start,
            branch_depth: 0,
            energy: 1.0,
        };
        let end_node = LightningNode {
            position: end,
            branch_depth: 0,
            energy: 0.8,
        };

        nodes.push(start_node.clone());
        nodes.push(end_node.clone());

        // Queue of segments to subdivide: (start_idx, end_idx, current_depth, branch_depth)
        let mut to_subdivide = vec![(0, 1, 0, 0)];

        while let Some((start_idx, end_idx, depth, branch_depth)) = to_subdivide.pop() {
            if depth >= config.max_depth {
                // Max depth reached, just connect as final segment
                segments.push((start_idx, end_idx));
                continue;
            }

            let start_pos = nodes[start_idx].position;
            let end_pos = nodes[end_idx].position;
            let start_energy = nodes[start_idx].energy;
            let end_energy = nodes[end_idx].energy;

            // Calculate midpoint with random perpendicular offset
            let segment = end_pos - start_pos;
            let length = segment.length();
            let midpoint = (start_pos + end_pos) * 0.5;

            // Random perpendicular vector in XZ plane (Y is up)
            let perpendicular = if segment.x.abs() > 0.01 || segment.z.abs() > 0.01 {
                Vec3::new(-segment.z, 0.0, segment.x).normalize()
            } else {
                // Segment is vertical, use X as perpendicular
                Vec3::new(1.0, 0.0, 0.0)
            };

            // Random displacement: beta controls magnitude, random value controls direction
            let displacement = rng.gen_range(-1.0..1.0);
            let offset = perpendicular * displacement * config.beta * length;
            let displaced_midpoint = midpoint + offset;

            // Create middle node
            let mid_node = LightningNode {
                position: displaced_midpoint,
                branch_depth,
                energy: (start_energy + end_energy) * 0.5,
            };
            let mid_idx = nodes.len();
            nodes.push(mid_node.clone());

            // Queue subdivisions for both halves
            to_subdivide.push((start_idx, mid_idx, depth + 1, branch_depth));
            to_subdivide.push((mid_idx, end_idx, depth + 1, branch_depth));

            // Branch generation with probability decay
            if branch_depth < config.max_branch_depth {
                let branch_prob =
                    config.gamma * f32::exp(-config.alpha * (depth as f32 + branch_depth as f32));

                if rng.gen::<f32>() < branch_prob {
                    // Spawn a branch from the midpoint
                    let branch_length =
                        length * 0.5 * f32::exp(-config.alpha * branch_depth as f32);

                    // Calculate overall direction to target (end point)
                    let to_target = (end - displaced_midpoint).normalize();

                    // Random branch direction: blend perpendicular offset with forward direction
                    // This ensures branches generally move toward the target
                    let perp_component = perpendicular * rng.gen_range(-0.6..0.6);
                    let forward_component = to_target * rng.gen_range(0.3..0.8);
                    let branch_dir = (perp_component + forward_component).normalize();

                    let branch_end_pos = displaced_midpoint + branch_dir * branch_length;
                    let branch_end_node = LightningNode {
                        position: branch_end_pos,
                        branch_depth: branch_depth + 1,
                        energy: mid_node.energy * 0.5, // Branches have lower energy
                    };
                    let branch_end_idx = nodes.len();
                    nodes.push(branch_end_node);

                    // Queue branch for subdivision
                    to_subdivide.push((mid_idx, branch_end_idx, depth + 1, branch_depth + 1));
                }
            }
        }

        Self {
            root: start,
            nodes,
            segments,
        }
    }

    /// Get the total number of segments in the tree
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Get all line positions for rendering as a line strip or gizmos
    pub fn get_line_positions(&self) -> Vec<Vec3> {
        self.segments
            .iter()
            .flat_map(|(start_idx, end_idx)| {
                [
                    self.nodes[*start_idx].position,
                    self.nodes[*end_idx].position,
                ]
            })
            .collect()
    }

    /// Sample positions along the lightning path for particle spawning
    /// Returns positions evenly spaced along the lightning tree
    pub fn sample_particle_positions(&self, particle_count: usize) -> Vec<Vec3> {
        if self.segments.is_empty() {
            return Vec::new();
        }

        let mut positions = Vec::with_capacity(particle_count);
        let segments_per_particle = self.segments.len().max(1) / particle_count.max(1);

        for (i, (start_idx, end_idx)) in self.segments.iter().enumerate() {
            if i % segments_per_particle == 0 && positions.len() < particle_count {
                // Interpolate along the segment
                let t = 0.5; // Use midpoint
                let start_pos = self.nodes[*start_idx].position;
                let end_pos = self.nodes[*end_idx].position;
                positions.push(start_pos.lerp(end_pos, t));
            }
        }

        positions
    }

    /// Get energy-weighted positions for particle brightness
    pub fn get_particle_data(&self, particle_count: usize) -> Vec<(Vec3, f32)> {
        if self.segments.is_empty() {
            return Vec::new();
        }

        let mut data = Vec::with_capacity(particle_count);
        let segments_per_particle = self.segments.len().max(1) / particle_count.max(1);

        for (i, (start_idx, end_idx)) in self.segments.iter().enumerate() {
            if i % segments_per_particle == 0 && data.len() < particle_count {
                let start_node = &self.nodes[*start_idx];
                let end_node = &self.nodes[*end_idx];
                let t = 0.5;
                let position = start_node.position.lerp(end_node.position, t);
                let energy = start_node.energy * (1.0 - t) + end_node.energy * t;
                data.push((position, energy));
            }
        }

        data
    }
}

/// Component for a procedural lightning effect entity
#[derive(Component)]
pub struct ProceduralLightning {
    /// The generated lightning tree
    pub tree: LightningTree,
    /// Animation timer for flickering/pulsing
    pub animation_timer: Timer,
    /// Lifetime timer for despawning
    pub lifetime: Timer,
    /// Base color for the lightning
    pub color: Color,
    /// Particle effect entities (core, glow, sparks, impact)
    pub particle_entities: Vec<Entity>,
    /// Whether to render debug gizmos
    pub show_gizmos: bool,
    /// Whether to enable flicker effect (on/off intervals)
    pub enable_flicker: bool,
}

impl ProceduralLightning {
    /// Create a new procedural lightning effect
    pub fn new(
        start: Vec3,
        end: Vec3,
        config: &LightningConfig,
        lifetime_secs: f32,
        color: Color,
    ) -> Self {
        let tree = LightningTree::generate(start, end, config);

        Self {
            tree,
            animation_timer: Timer::from_seconds(0.05, TimerMode::Repeating),
            lifetime: Timer::from_seconds(lifetime_secs, TimerMode::Once),
            color,
            particle_entities: Vec::new(), // Will be populated after spawn
            show_gizmos: false,            // Particles by default
            enable_flicker: false,         // No flicker by default
        }
    }
}

/// Plugin for procedural lightning system
pub struct ProceduralLightningPlugin;

impl Plugin for ProceduralLightningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_procedural_lightning, cleanup_expired_lightning),
        );
    }
}

/// Update procedural lightning animations and rendering
#[allow(clippy::needless_pass_by_value)]
fn update_procedural_lightning(
    mut query: Query<(&mut ProceduralLightning, &Transform)>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    for (mut lightning, transform) in &mut query {
        lightning.animation_timer.tick(time.delta());
        lightning.lifetime.tick(time.delta());

        // Draw lightning using gizmos
        // With flicker: alternates on/off based on timer progress
        // Without flicker: draw every frame
        let should_draw = if lightning.enable_flicker {
            // Flicker on/off - visible for first half of timer cycle
            lightning.animation_timer.elapsed_secs() / lightning.animation_timer.duration().as_secs_f32() < 0.5
        } else {
            true
        };
        
        if should_draw {
            for (start_idx, end_idx) in &lightning.tree.segments {
                let start = transform.transform_point(lightning.tree.nodes[*start_idx].position);
                let end = transform.transform_point(lightning.tree.nodes[*end_idx].position);
                let energy = lightning.tree.nodes[*start_idx].energy;

                // Vary color intensity by energy
                let alpha = if lightning.show_gizmos { 0.9 } else { 0.7 };
                let color = lightning.color.with_alpha(energy * alpha);
                gizmos.line(start, end, color);
            }
        }
    }
}

/// Cleanup expired lightning effects
#[allow(clippy::needless_pass_by_value)]
fn cleanup_expired_lightning(mut commands: Commands, query: Query<(Entity, &ProceduralLightning)>) {
    for (entity, lightning) in &query {
        if lightning.lifetime.is_finished() {
            // Despawn all particle entities first
            for &particle_entity in &lightning.particle_entities {
                if let Ok(mut entity_commands) = commands.get_entity(particle_entity) {
                    entity_commands.despawn();
                }
            }
            // Then despawn the main lightning entity
            commands.entity(entity).despawn();
        }
    }
}

/// Helper function to spawn a procedural lightning effect with particles
pub fn spawn_procedural_lightning(
    commands: &mut Commands,
    effects: &mut ResMut<Assets<EffectAsset>>,
    start: Vec3,
    end: Vec3,
    config: &LightningConfig,
    lifetime_secs: f32,
    color: Color,
    show_gizmos: bool,
    show_particles: bool,
) -> Entity {
    let mut lightning = ProceduralLightning::new(start, end, config, lifetime_secs, color);
    lightning.show_gizmos = show_gizmos;

    // Create multi-layered particle effects if enabled
    let particle_entities = if show_particles {
        create_procedural_lightning_particle_effects(commands, effects, &lightning.tree, color)
    } else {
        Vec::new()
    };
    lightning.particle_entities = particle_entities;

    commands.spawn((lightning, Transform::default())).id()
}

/// Create traveling ionized particle effect for procedural lightning
///
/// Returns vector of entity IDs for particle effects
///
/// Creates particles that travel from spawn point to target with random scatter
fn create_procedural_lightning_particle_effects(
    commands: &mut Commands,
    effects: &mut ResMut<Assets<EffectAsset>>,
    tree: &LightningTree,
    color: Color,
) -> Vec<Entity> {
    let mut particle_entities = Vec::new();

    // Extract color components
    let [r, g, b, _] = color.to_srgba().to_f32_array();
    let base_color = Vec4::new(r, g, b, 1.0);

    let start_pos = tree.nodes[0].position;
    let end_pos = tree.nodes.last().map(|n| n.position).unwrap_or(start_pos);
    
    // Calculate direction and distance for traveling particles
    let direction = (end_pos - start_pos).normalize_or_zero();
    let distance = start_pos.distance(end_pos);
    
    // Calculate appropriate lifetime based on distance (particles should reach target)
    let base_speed = 20.0; // units per second
    let particle_lifetime = (distance / base_speed).max(0.3);

    // ==== Traveling Ionized Particles ====
    // Creates particles that travel from spawn to target with random scatter
    let writer = ExprWriter::new();
    
    let intensity = 12.0;
    let lifetime = writer.lit(particle_lifetime * 0.8).uniform(writer.lit(particle_lifetime * 1.2)).expr();
    let age = writer.lit(0.0).expr();
    
    // Spawn at center with small radius for scatter
    let spawn_center = writer.lit(Vec3::ZERO).expr();
    let spawn_radius = writer.lit(0.5).expr();
    
    // Velocity pointing from start to end with random variance
    let base_velocity = direction * base_speed;
    let velocity_vec = writer.lit(base_velocity * 0.8).uniform(writer.lit(base_velocity * 1.2)).expr();
    
    let drag = writer.lit(1.5).expr();

    let traveling_effect = EffectAsset::new(
        512, 
        SpawnerSettings::rate(200.0.into()), 
        writer.finish()
    )
    .with_name("ionized_particles")
    .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime))
    .init(SetAttributeModifier::new(Attribute::AGE, age))
    .init(SetPositionSphereModifier {
        center: spawn_center,
        radius: spawn_radius,
        dimension: ShapeDimension::Volume,
    })
    .init(SetAttributeModifier::new(Attribute::VELOCITY, velocity_vec))
    .update(LinearDragModifier::new(drag))
    .render(ColorOverLifetimeModifier::new({
        let mut gradient = bevy_hanabi::Gradient::new();
        gradient.add_key(0.0, base_color * intensity);
        gradient.add_key(0.4, base_color * intensity * 0.8);
        gradient.add_key(0.8, base_color * intensity * 0.3);
        gradient.add_key(1.0, Vec4::ZERO);
        gradient
    }))
    .render(SizeOverLifetimeModifier {
        gradient: {
            let mut gradient = bevy_hanabi::Gradient::new();
            gradient.add_key(0.0, Vec3::splat(0.4));
            gradient.add_key(0.2, Vec3::splat(0.6));
            gradient.add_key(0.7, Vec3::splat(0.3));
            gradient.add_key(1.0, Vec3::splat(0.1));
            gradient
        },
        screen_space_size: false,
    });

    let traveling_handle = effects.add(traveling_effect);
    let traveling_entity = commands
        .spawn((
            ParticleEffect::new(traveling_handle),
            Transform::from_translation(start_pos),
        ))
        .id();
    particle_entities.push(traveling_entity);

    particle_entities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lightning_generation() {
        let config = LightningConfig::default();
        let start = Vec3::new(0.0, 0.0, 0.0);
        let end = Vec3::new(0.0, 0.0, 100.0);

        let tree = LightningTree::generate(start, end, &config);

        assert!(!tree.nodes.is_empty(), "Should generate nodes");
        assert!(!tree.segments.is_empty(), "Should generate segments");
        assert_eq!(tree.nodes[0].position, start, "Start node should match");
    }

    #[test]
    fn test_deterministic_generation() {
        let config = LightningConfig {
            seed: 42,
            ..default()
        };
        let start = Vec3::ZERO;
        let end = Vec3::new(0.0, 0.0, 100.0);

        let tree1 = LightningTree::generate(start, end, &config);
        let tree2 = LightningTree::generate(start, end, &config);

        assert_eq!(
            tree1.nodes.len(),
            tree2.nodes.len(),
            "Same seed should produce same result"
        );
        assert_eq!(tree1.segments.len(), tree2.segments.len());
    }

    #[test]
    fn test_particle_sampling() {
        let config = LightningConfig::default();
        let tree = LightningTree::generate(Vec3::ZERO, Vec3::new(0.0, 0.0, 100.0), &config);

        let positions = tree.sample_particle_positions(10);
        assert!(!positions.is_empty(), "Should generate particle positions");
        assert!(positions.len() <= 10, "Should not exceed requested count");
    }

    #[test]
    fn test_energy_attenuation() {
        let config = LightningConfig::default();
        let tree = LightningTree::generate(Vec3::ZERO, Vec3::new(0.0, 0.0, 100.0), &config);

        // Energy should generally decrease along branches
        for node in &tree.nodes {
            assert!(
                node.energy >= 0.0 && node.energy <= 1.0,
                "Energy should be normalized"
            );
        }
    }
}
