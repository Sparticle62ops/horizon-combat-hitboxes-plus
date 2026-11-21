//! Host seams: every Minecraft client surface the port touches sits here as a
//! thin view the engine fills, in the shape the original's calls take. The
//! port logic stays pure over them, so the tests drive the whole path.
//!
//! The per-view draw surface (the original cancels the vanilla hitbox debug
//! renderer's draw and draws its own gizmos in its place) is the mod API gap:
//! `register_debug_renderers`/`debug_renderer_draw` push new renderers but
//! there is no surface to cancel an existing renderer's draw, and no client
//! read for the entity set a view draws (bounding box, interpolated position,
//! eye height, view vector, vehicle, dragon parts, blocking state, hurt time)
//! or for the local player's reads (distance, targeted hit result, the 1.1
//! interaction range, held item ids). Filed against Horizon; until it lands
//! the draw path stays on these seams.

use std::sync::OnceLock;

/// java: net.minecraft.world.phys.AABB, the two corners the draws need.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

impl Aabb {
    /// java: AABB.move(double, double, double)
    pub fn moved(&self, d: [f64; 3]) -> Aabb {
        Aabb {
            min: [self.min[0] + d[0], self.min[1] + d[1], self.min[2] + d[2]],
            max: [self.max[0] + d[0], self.max[1] + d[1], self.max[2] + d[2]],
        }
    }
}

/// java: net.minecraft.gizmos.GizmoStyle.stroke — the color and the line
/// width; `None` is the no-width overload (the engine's own default).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stroke {
    pub color: u32,
    pub width: Option<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Draw {
    /// java: Gizmos.cuboid(aabb, style)
    Cuboid(Aabb, Stroke),
    /// java: Gizmos.point(pos, color, size)
    Point([f64; 3], u32, f32),
    /// java: Gizmos.line(a, b, color, width)
    Line([f64; 3], [f64; 3], u32, f32),
    /// java: Gizmos.arrow(a, b, color, width)
    Arrow([f64; 3], [f64; 3], u32, f32),
}

/// java: net.minecraft.gizmos.Gizmos — the world-space gizmo surface the
/// mixin's draws go through.
pub trait Gizmos {
    fn cuboid(&mut self, aabb: Aabb, stroke: Stroke);
    fn point(&mut self, pos: [f64; 3], color: u32, size: f32);
    fn line(&mut self, a: [f64; 3], b: [f64; 3], color: u32, width: f32);
    fn arrow(&mut self, a: [f64; 3], b: [f64; 3], color: u32, width: f32);
}

/// The recording gizmo surface the tests drive.
#[derive(Default)]
pub struct RecordingGizmos {
    pub draws: Vec<Draw>,
}

impl Gizmos for RecordingGizmos {
    fn cuboid(&mut self, aabb: Aabb, stroke: Stroke) {
        self.draws.push(Draw::Cuboid(aabb, stroke));
    }
    fn point(&mut self, pos: [f64; 3], color: u32, size: f32) {
        self.draws.push(Draw::Point(pos, color, size));
    }
    fn line(&mut self, a: [f64; 3], b: [f64; 3], color: u32, width: f32) {
        self.draws.push(Draw::Line(a, b, color, width));
    }
    fn arrow(&mut self, a: [f64; 3], b: [f64; 3], color: u32, width: f32) {
        self.draws.push(Draw::Arrow(a, b, color, width));
    }
}

/// java: one EnderDragonPart — position, interpolated position, box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragonPart {
    pub position: [f64; 3],
    pub position_at: [f64; 3],
    pub bounding_box: Aabb,
}

/// java: entity.getVehicle() reduced to the two reads the seat box makes:
/// the vehicle's width and getPassengerRidingPosition(entity).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleView {
    pub bb_width: f32,
    pub riding_position: [f64; 3],
}

/// java: net.minecraft.world.entity.Entity, reduced to the reads the hitbox
/// draw makes on the entity of the view.
pub struct EntityView {
    pub position: [f64; 3],
    /// java: entity.getPosition(tickProgress)
    pub position_at: [f64; 3],
    pub bounding_box: Aabb,
    pub eye_height: f32,
    /// java: entity.getViewVector(tickProgress)
    pub view_vector: [f64; 3],
    pub is_alive: bool,
    pub is_spectator: bool,
    pub hurt_time: i32,
    pub bb_width: f32,
    pub is_living: bool,
    /// java: living.isBlocking() && getItemBlockingWith().is(Items.SHIELD)
    pub is_blocking: bool,
    pub blocking_with_shield: bool,
    pub is_projectile: bool,
    pub is_ender_pearl: bool,
    /// java: ProjectileOwner.of(projectile) — owner == null -> None, the
    /// local player -> Some(true), anyone else -> Some(false).
    pub pearl_owner_is_own: Option<bool>,
    pub is_firework_rocket: bool,
    pub is_ender_dragon: bool,
    pub dragon_parts: Vec<DragonPart>,
    pub vehicle: Option<VehicleView>,
}

impl EntityView {
    /// A blank entity the tests shape.
    pub fn blank() -> EntityView {
        EntityView {
            position: [0.0; 3],
            position_at: [0.0; 3],
            bounding_box: Aabb { min: [0.0; 3], max: [0.0; 3] },
            eye_height: 1.8,
            view_vector: [0.0; 3],
            is_alive: true,
            is_spectator: false,
            hurt_time: 0,
            bb_width: 0.6,
            is_living: true,
            is_blocking: false,
            blocking_with_shield: false,
            is_projectile: false,
            is_ender_pearl: false,
            pearl_owner_is_own: None,
            is_firework_rocket: false,
            is_ender_dragon: false,
            dragon_parts: Vec::new(),
            vehicle: None,
        }
    }

    /// java: entity.getPosition(tickProgress).subtract(entity.position())
    pub fn delta(&self) -> [f64; 3] {
        [
            self.position_at[0] - self.position[0],
            self.position_at[1] - self.position[1],
            self.position_at[2] - self.position[2],
        ]
    }
}

/// java: the local player's reads (Main.mc.player, Main.mc.hitResult,
/// player.distanceTo, player.isWithinEntityInteractionRange) and the engine's
/// inLocalServer flag the mixin checks first.
pub struct PlayerEnv<'a> {
    pub in_local_server: bool,
    /// Main.mc.player == null
    pub present: bool,
    pub distance_to_entity: f64,
    /// java: hitResult instanceof EntityHitResult && hit.getEntity() == entity
    pub hit_targets_entity: bool,
    /// java: player.isWithinEntityInteractionRange(entity, 1.1)
    pub within_interaction_range_1_1: bool,
    pub main_hand_id: Option<&'a str>,
    pub off_hand_id: Option<&'a str>,
}

impl PlayerEnv<'static> {
    /// A player present with no hands, the tests' base.
    pub fn blank() -> PlayerEnv<'static> {
        PlayerEnv {
            in_local_server: false,
            present: true,
            distance_to_entity: 0.0,
            hit_targets_entity: false,
            within_interaction_range_1_1: false,
            main_hand_id: None,
            off_hand_id: None,
        }
    }
}

// ---------------------------------------------------------------- logging

pub struct Logger;

impl Logger {
    pub fn info(&self, msg: &str) {
        let _ = msg;
    }
}

pub fn logger() -> &'static Logger {
    static LOGGER: OnceLock<Logger> = OnceLock::new();
    LOGGER.get_or_init(|| Logger)
}
