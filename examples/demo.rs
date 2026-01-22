//! Procedural Lightning VFX Demo
//!
//! Demonstrates the procedural lightning generation system inspired by LightningGen.
//! Shows different configurations, branch patterns, and rendering techniques.

#![allow(clippy::multiple_crate_versions)]

use procedural_lightning::{
    spawn_procedural_lightning, LightningConfig, LightningTree,
    ProceduralLightning, ProceduralLightningPlugin,
};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bevy_hanabi::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Procedural Lightning Demo".to_string(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins(HanabiPlugin)
        .add_plugins(ProceduralLightningPlugin)
        .init_resource::<DemoState>()
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, demo_ui)
        .add_systems(
            Update,
            (
                move_target,
                spawn_lightning_on_click,
                apply_flicker_setting,
                update_config_preview,
                cleanup_old_lightning,
            ),
        )
        .run();
}

#[derive(Resource)]
struct DemoState {
    // Algorithm parameters
    alpha: f32,
    beta: f32,
    gamma: f32,
    max_depth: u32,
    max_branch_depth: u32,

    // Visual settings
    color: [f32; 3],
    lifetime: f32,
    auto_spawn: bool,
    spawn_timer: Timer,
    
    // Rendering options
    show_gizmos: bool,
    enable_flicker: bool,
    flicker_speed: f32, // seconds per flicker cycle
    show_particles: bool,

    // Presets
    selected_preset: LightningPreset,

    // Preview
    preview_tree: Option<LightningTree>,
    preview_stats: TreeStats,

    // Spawn trigger from UI
    spawn_requested: bool,
}

#[derive(Component)]
struct LightningTarget;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum LightningPreset {
    Classic,
    Dense,
    Sparse,
    Chaotic,
    Smooth,
    Branchy,
}

#[derive(Default, Debug)]
struct TreeStats {
    node_count: usize,
    segment_count: usize,
    max_branch_depth: u32,
}

impl Default for DemoState {
    fn default() -> Self {
        Self {
            alpha: 0.5,
            beta: 0.4,
            gamma: 0.3,
            max_depth: 8,
            max_branch_depth: 3,
            color: [0.3, 0.7, 1.0], // Electric blue
            lifetime: 0.5,
            auto_spawn: false,
            spawn_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            show_gizmos: true, // Show gizmos enhanced with particles
            enable_flicker: false,
            flicker_speed: 0.05,
            show_particles: true,
            selected_preset: LightningPreset::Classic,
            preview_tree: None,
            preview_stats: TreeStats::default(),
            spawn_requested: false,
        }
    }
}

impl LightningPreset {
    const fn config(&self) -> (f32, f32, f32, u32, u32) {
        // Returns: (alpha, beta, gamma, max_depth, max_branch_depth)
        match self {
            Self::Classic => (0.5, 0.4, 0.3, 8, 3),
            Self::Dense => (0.3, 0.3, 0.5, 10, 4),
            Self::Sparse => (0.7, 0.5, 0.2, 6, 2),
            Self::Chaotic => (0.4, 0.6, 0.6, 12, 5),
            Self::Smooth => (0.6, 0.2, 0.15, 7, 2),
            Self::Branchy => (0.35, 0.45, 0.55, 9, 4),
        }
    }

    const fn description(&self) -> &'static str {
        match self {
            Self::Classic => "Balanced natural lightning\nModerate branches, medium density",
            Self::Dense => "Dense with many segments\nHeavy branching, detailed",
            Self::Sparse => "Clean minimal bolt\nFew branches, simple",
            Self::Chaotic => "Wild, erratic paths\nHeavy displacement, many branches",
            Self::Smooth => "Smooth controlled arcs\nMinimal noise, graceful",
            Self::Branchy => "Maximum branching\nComplex tree structure",
        }
    }
}

#[derive(Component)]
struct DemoLightning {
    spawn_time: f32,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn(Camera3d::default()).insert(
        Transform::from_xyz(0.0, 200.0, 400.0).looking_at(Vec3::new(0.0, 100.0, 0.0), Vec3::Y),
    );

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.3, 0.0)),
    ));

    // Load duck model
    let duck_model = asset_server.load("models/characters/duck/gltf/character_duck.gltf#Scene0");

    // Spawn target duck at origin (scaled up for visibility)
    commands.spawn((
        SceneRoot(duck_model),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(20.0)),
        LightningTarget,
    ));

    info!("Procedural Lightning Demo Started");
    info!("Use WASD/Arrows to move target, SPACEBAR to spawn lightning");
}

fn demo_ui(mut contexts: EguiContexts, mut demo_state: ResMut<DemoState>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("⚡ Procedural Lightning Generator")
        .default_width(350.0)
        .show(ctx, |ui| {
            ui.heading("Lightning Algorithm Parameters");
            ui.separator();

            // Presets
            ui.label("Presets:");
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(
                        demo_state.selected_preset == LightningPreset::Classic,
                        "Classic",
                    )
                    .clicked()
                {
                    apply_preset(&mut demo_state, LightningPreset::Classic);
                }
                if ui
                    .selectable_label(
                        demo_state.selected_preset == LightningPreset::Dense,
                        "Dense",
                    )
                    .clicked()
                {
                    apply_preset(&mut demo_state, LightningPreset::Dense);
                }
                if ui
                    .selectable_label(
                        demo_state.selected_preset == LightningPreset::Sparse,
                        "Sparse",
                    )
                    .clicked()
                {
                    apply_preset(&mut demo_state, LightningPreset::Sparse);
                }
            });
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(
                        demo_state.selected_preset == LightningPreset::Chaotic,
                        "Chaotic",
                    )
                    .clicked()
                {
                    apply_preset(&mut demo_state, LightningPreset::Chaotic);
                }
                if ui
                    .selectable_label(
                        demo_state.selected_preset == LightningPreset::Smooth,
                        "Smooth",
                    )
                    .clicked()
                {
                    apply_preset(&mut demo_state, LightningPreset::Smooth);
                }
                if ui
                    .selectable_label(
                        demo_state.selected_preset == LightningPreset::Branchy,
                        "Branchy",
                    )
                    .clicked()
                {
                    apply_preset(&mut demo_state, LightningPreset::Branchy);
                }
            });

            ui.label(demo_state.selected_preset.description());
            ui.separator();

            // Parameters
            ui.label("Alpha (branch decay):");
            ui.add(egui::Slider::new(&mut demo_state.alpha, 0.1..=1.0));
            ui.label("Higher = fewer branches, faster decay");

            ui.label("Beta (displacement):");
            ui.add(egui::Slider::new(&mut demo_state.beta, 0.0..=1.0));
            ui.label("Higher = more jagged, chaotic paths");

            ui.label("Gamma (branch probability):");
            ui.add(egui::Slider::new(&mut demo_state.gamma, 0.0..=1.0));
            ui.label("Higher = more frequent branching");

            ui.label("Max Depth:");
            ui.add(egui::Slider::new(&mut demo_state.max_depth, 4..=15));
            ui.label("Subdivision iterations (detail level)");

            ui.label("Max Branch Depth:");
            ui.add(egui::Slider::new(&mut demo_state.max_branch_depth, 0..=6));
            ui.label("Maximum nested branch levels");

            ui.separator();

            // Visual settings
            ui.heading("Visual Settings");
            ui.label("Lightning Color:");
            ui.color_edit_button_rgb(&mut demo_state.color);

            ui.label("Lifetime (seconds):");
            ui.add(egui::Slider::new(&mut demo_state.lifetime, 0.1..=2.0));

            ui.checkbox(&mut demo_state.auto_spawn, "Auto-spawn");
            if demo_state.auto_spawn {
                let mut interval = demo_state.spawn_timer.duration().as_secs_f32();
                ui.label("Spawn interval:");
                if ui
                    .add(egui::Slider::new(&mut interval, 0.2..=3.0))
                    .changed()
                {
                    demo_state
                        .spawn_timer
                        .set_duration(std::time::Duration::from_secs_f32(interval));
                }
            }

            ui.separator();

            // Rendering options
            ui.heading("🎨 Rendering");
            ui.checkbox(&mut demo_state.show_gizmos, "Show Debug Gizmos");
            ui.label("Toggle debug line visualization");
            
            ui.checkbox(&mut demo_state.enable_flicker, "Enable Flicker Effect");
            if demo_state.enable_flicker {
                ui.label("Flicker Speed:");
                ui.add(egui::Slider::new(&mut demo_state.flicker_speed, 0.02..=0.2).suffix(" sec"));
            }
            
            ui.checkbox(&mut demo_state.show_particles, "Show Particle Effects");
            ui.label("Ionized particle trails");

            ui.separator();

            // Spawn controls
            ui.heading("⚡ Spawn Controls");
            if ui.button("🎯 Spawn Lightning to Target").clicked() {
                demo_state.spawn_requested = true;
            }
            ui.label("Keyboard: SPACEBAR to spawn");
            ui.label("Move Target: WASD or Arrow Keys");

            ui.separator();

            // Stats
            ui.heading("Preview Statistics");
            ui.label(format!("Nodes: {}", demo_state.preview_stats.node_count));
            ui.label(format!(
                "Segments: {}",
                demo_state.preview_stats.segment_count
            ));
            ui.label(format!(
                "Max Branch Depth: {}",
                demo_state.preview_stats.max_branch_depth
            ));
        });
}

fn apply_preset(demo_state: &mut DemoState, preset: LightningPreset) {
    let (alpha, beta, gamma, max_depth, max_branch_depth) = preset.config();
    demo_state.alpha = alpha;
    demo_state.beta = beta;
    demo_state.gamma = gamma;
    demo_state.max_depth = max_depth;
    demo_state.max_branch_depth = max_branch_depth;
    demo_state.selected_preset = preset;
}

fn move_target(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut target_query: Query<&mut Transform, With<LightningTarget>>,
) {
    let Ok(mut transform) = target_query.single_mut() else {
        return;
    };

    let speed = 100.0 * time.delta_secs();
    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.z += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction.length_squared() > 0.0 {
        transform.translation += direction.normalize() * speed;
        transform.translation.x = transform.translation.x.clamp(-200.0, 200.0);
        transform.translation.z = transform.translation.z.clamp(-150.0, 150.0);
    }
}

fn update_config_preview(mut demo_state: ResMut<DemoState>) {
    let config = LightningConfig {
        seed: 42,
        alpha: demo_state.alpha,
        beta: demo_state.beta,
        gamma: demo_state.gamma,
        max_depth: demo_state.max_depth,
        max_branch_depth: demo_state.max_branch_depth,
    };

    let tree = LightningTree::generate(
        Vec3::new(0.0, 200.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        &config,
    );

    let max_branch = tree.nodes.iter().map(|n| n.branch_depth).max().unwrap_or(0);

    demo_state.preview_stats = TreeStats {
        node_count: tree.nodes.len(),
        segment_count: tree.segments.len(),
        max_branch_depth: max_branch,
    };

    demo_state.preview_tree = Some(tree);
}

fn spawn_lightning_on_click(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut demo_state: ResMut<DemoState>,
    target_query: Query<&Transform, With<LightningTarget>>,
    time: Res<Time>,
    mut spawn_timer: Local<Option<Timer>>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    if spawn_timer.is_none() {
        *spawn_timer = Some(demo_state.spawn_timer.clone());
    }

    let should_spawn = if demo_state.auto_spawn {
        if let Some(timer) = spawn_timer.as_mut() {
            timer.tick(time.delta());
            timer.just_finished()
        } else {
            false
        }
    } else {
        let spacebar = keyboard.just_pressed(KeyCode::Space);
        let button = demo_state.spawn_requested;
        demo_state.spawn_requested = false;
        spacebar || button
    };

    if !should_spawn {
        return;
    }

    let target_pos = if let Ok(target_transform) = target_query.single() {
        target_transform.translation
    } else {
        Vec3::ZERO
    };

    let seed = (time.elapsed_secs() * 1000.0) as u64;

    let config = LightningConfig {
        seed,
        alpha: demo_state.alpha,
        beta: demo_state.beta,
        gamma: demo_state.gamma,
        max_depth: demo_state.max_depth,
        max_branch_depth: demo_state.max_branch_depth,
    };

    let [r, g, b] = demo_state.color;
    let color = Color::srgb(r, g, b);

    let start = target_pos + Vec3::new(0.0, 250.0, 0.0);
    let end = target_pos;

    let lightning_entity = spawn_procedural_lightning(
        &mut commands,
        &mut effects,
        start,
        end,
        &config,
        demo_state.lifetime,
        color,
        demo_state.show_gizmos,
        demo_state.show_particles,
    );

    commands.entity(lightning_entity).insert(DemoLightning {
        spawn_time: time.elapsed_secs(),
    });
}

// System to apply flicker setting to newly spawned lightning
fn apply_flicker_setting(
    demo_state: Res<DemoState>,
    mut query: Query<&mut ProceduralLightning, Added<ProceduralLightning>>,
) {
    for mut lightning in &mut query {
        lightning.enable_flicker = demo_state.enable_flicker;
        lightning.animation_timer.set_duration(std::time::Duration::from_secs_f32(demo_state.flicker_speed));
    }
}

fn cleanup_old_lightning(
    mut commands: Commands,
    query: Query<(Entity, &DemoLightning)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    for (entity, demo_lightning) in &query {
        if current_time - demo_lightning.spawn_time > 2.0 {
            commands.entity(entity).despawn();
        }
    }
}
