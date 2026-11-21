//! 1:1 port of me.truewulf.box.mixin.HitBoxRenderMixin (the 1.21.11+ path,
//! the only one active in the build it ships: the legacy main-hand widths
//! `lineWidthLiving`/`lineWidthEntity` stay in Config as dead fields, unused
//! by any active mixin, and are not carried).

use crate::config::ModConfig;
use crate::projectile_owner::{self, Kind};
use crate::seams::{Aabb, EntityView, Gizmos, PlayerEnv, Stroke};
use crate::weapon_filter;

/// What the mixin's showHitboxes callback did: let the vanilla draw through,
/// or cancel it and whether this port drew in its place.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome {
    Vanilla,
    Cancelled { drew: bool },
}

/// java: HitBoxRenderMixin#showHitboxes(EntityRenderState, CallbackInfo) —
/// the whole gate chain and the draw, same order.
pub fn on_show_hitboxes(
    gizmos: &mut dyn Gizmos,
    entity: &EntityView,
    player: &PlayerEnv,
    config: &ModConfig,
) -> Outcome {
    // java: if (inLocalServer) return;
    if player.in_local_server {
        return Outcome::Vanilla;
    }
    // java: if (!config.enabled || Main.mc.player == null) return;
    if !config.enabled || !player.present {
        return Outcome::Vanilla;
    }
    // java: ci.cancel() — everything after runs in place of the vanilla draw.
    if player.distance_to_entity > config.render_distance {
        return Outcome::Cancelled { drew: false };
    }
    if !weapon_filter::player_allowed(config, player.main_hand_id, player.off_hand_id) {
        return Outcome::Cancelled { drew: false };
    }
    if config.hide_fireworks && entity.is_firework_rocket {
        return Outcome::Cancelled { drew: false };
    }
    if entity.is_projectile {
        let drew = render_projectile(gizmos, entity, config);
        return Outcome::Cancelled { drew };
    }
    let targeted = player.hit_targets_entity;
    let in_range = entity.is_alive && !entity.is_spectator && player.within_interaction_range_1_1;
    if !entity.is_living {
        let color = if targeted && in_range {
            config.target_box_color
        } else {
            config.hit_box_color
        };
        draw_box(gizmos, entity, color, config.line1);
        return Outcome::Cancelled { drew: true };
    }
    let blocking = config.shield_color_enabled
        && (!config.shield_only_target || targeted)
        && entity.is_blocking
        && entity.blocking_with_shield;
    let line_width = if player.distance_to_entity > config.dist_for2 {
        config.line2
    } else {
        config.line1
    };
    let main_color = if blocking {
        config.shield_color
    } else if targeted && in_range && entity.hurt_time == 0 {
        config.target_box_color
    } else {
        config.hit_box_color
    };
    render_entity_box(gizmos, entity, line_width, main_color, config);
    Outcome::Cancelled { drew: true }
}

/// java: the mixin's projectile branch — ender pearls get their kind's color
/// and gate, everything else the base hitbox color.
/// Returns whether anything drew (the gates stop the draw after the cancel).
fn render_projectile(gizmos: &mut dyn Gizmos, entity: &EntityView, config: &ModConfig) -> bool {
    if entity.is_ender_pearl {
        if !config.pearls_enabled {
            return false;
        }
        let kind = projectile_owner::of(entity.pearl_owner_is_own);
        let allowed = match kind {
            Kind::Own => config.pearl_own,
            Kind::Enemy => config.pearl_others,
            Kind::Unknown => config.pearl_unknown,
        };
        if !allowed {
            return false;
        }
        let color = match kind {
            Kind::Own => config.pearl_own_color,
            Kind::Enemy => config.pearl_enemy_color,
            Kind::Unknown => config.pearl_unknown_color,
        };
        draw_box(gizmos, entity, color, config.pearl_line_width);
        return true;
    }
    draw_box(gizmos, entity, config.hit_box_color, config.line1);
    true
}

/// java: the mixin's drawBox — the bounding box moved by the interpolated
/// position delta.
fn draw_box(gizmos: &mut dyn Gizmos, entity: &EntityView, color: u32, width: f32) {
    let delta = entity.delta();
    gizmos.cuboid(
        entity.bounding_box.moved(delta),
        Stroke {
            color,
            width: Some(width),
        },
    );
}

/// java: the mixin's renderEntityBox — the box, its outline, the point at the
/// interpolated position, the vehicle seat, the eye-height slab, the dragon
/// parts, and the look direction.
pub fn render_entity_box(
    gizmos: &mut dyn Gizmos,
    entity: &EntityView,
    line_width: f32,
    main_color: u32,
    config: &ModConfig,
) {
    let delta = entity.delta();
    let bb = entity.bounding_box.moved(delta);
    if config.outline_enabled {
        gizmos.cuboid(
            bb,
            Stroke {
                color: config.outline_color,
                width: Some(line_width * config.outline_multiplier),
            },
        );
    }
    gizmos.cuboid(bb, Stroke { color: main_color, width: Some(line_width) });
    gizmos.point(entity.position_at, main_color, 2.0);
    if let Some(vehicle) = &entity.vehicle {
        // java: Math.min(vehicle.getBbWidth(), entity.getBbWidth()) / 2.0F
        let half = (vehicle.bb_width.min(entity.bb_width) / 2.0) as f64;
        let seat = [
            vehicle.riding_position[0] + delta[0],
            vehicle.riding_position[1] + delta[1],
            vehicle.riding_position[2] + delta[2],
        ];
        gizmos.cuboid(
            Aabb {
                min: [seat[0] - half, seat[1], seat[2] - half],
                max: [seat[0] + half, seat[1] + 0.0625, seat[2] + half],
            },
            // java: -256, the constant color for the seat box.
            Stroke {
                color: 0xFFFF_FF00,
                width: Some(line_width),
            },
        );
    }
    if config.render_eye_height {
        let eye = entity.eye_height as f64;
        gizmos.cuboid(
            Aabb {
                min: [bb.min[0], bb.min[1] + eye - 0.01, bb.min[2]],
                max: [bb.max[0], bb.min[1] + eye + 0.01, bb.max[2]],
            },
            Stroke {
                color: config.eye_color,
                width: Some(line_width),
            },
        );
    }
    if entity.is_ender_dragon {
        for part in &entity.dragon_parts {
            let pd = [
                part.position_at[0] - part.position[0],
                part.position_at[1] - part.position[1],
                part.position_at[2] - part.position[2],
            ];
            // java: ARGB.colorFromFloat(1.0F, 0.25F, 1.0F, 0.0F)
            gizmos.cuboid(
                part.bounding_box.moved(pd),
                Stroke {
                    color: 0xFF40_FF00,
                    width: None,
                },
            );
        }
    }
    if config.render_look_dir {
        let eye = [
            entity.position_at[0],
            entity.position_at[1] + entity.eye_height as f64,
            entity.position_at[2],
        ];
        if config.line_look_dir {
            gizmos.line(eye, add(eye, scale(entity.view_vector, 2.0)), config.look_color, line_width);
        } else {
            gizmos.arrow(
                eye,
                add(eye, scale(entity.view_vector, 2.5)),
                config.look_color,
                line_width * 1.5,
            );
        }
    }
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(v: [f64; 3], s: f64) -> [f64; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seams::{Aabb, RecordingGizmos};

    fn entity() -> EntityView {
        EntityView {
            position: [10.0, 64.0, 10.0],
            position_at: [10.5, 64.0, 10.5],
            bounding_box: Aabb {
                min: [9.3, 64.0, 9.3],
                max: [10.7, 65.8, 10.7],
            },
            ..EntityView::blank()
        }
    }

    #[test]
    fn the_vanilla_draw_passes_when_disabled() {
        let mut g = RecordingGizmos::default();
        let e = entity();
        let mut p = PlayerEnv::blank();
        let mut c = crate::config::DEFAULTS.clone();
        c.enabled = false;
        assert_eq!(on_show_hitboxes(&mut g, &e, &p, &c), Outcome::Vanilla);
        p.in_local_server = true;
        c.enabled = true;
        assert_eq!(on_show_hitboxes(&mut g, &e, &p, &c), Outcome::Vanilla);
        p.in_local_server = false;
        p.present = false;
        assert_eq!(on_show_hitboxes(&mut g, &e, &p, &c), Outcome::Vanilla);
        assert!(g.draws.is_empty());
    }

    #[test]
    fn beyond_render_distance_cancels_and_draws_nothing() {
        let mut g = RecordingGizmos::default();
        let e = entity();
        let mut p = PlayerEnv::blank();
        p.distance_to_entity = 73.0;
        assert_eq!(
            on_show_hitboxes(&mut g, &e, &p, &crate::config::DEFAULTS),
            Outcome::Cancelled { drew: false }
        );
        assert!(g.draws.is_empty());
    }

    #[test]
    fn the_weapon_filter_gates_the_draw() {
        let mut g = RecordingGizmos::default();
        let e = entity();
        let mut p = PlayerEnv::blank();
        p.main_hand_id = Some("minecraft:dirt");
        p.off_hand_id = Some("minecraft:dirt");
        let mut c = crate::config::DEFAULTS.clone();
        c.weapon_filter_enabled = true;
        assert_eq!(
            on_show_hitboxes(&mut g, &e, &p, &c),
            Outcome::Cancelled { drew: false }
        );
        assert!(g.draws.is_empty());
    }

    #[test]
    fn a_hidden_firework_cancels_and_draws_nothing() {
        let mut g = RecordingGizmos::default();
        let e = EntityView {
            is_firework_rocket: true,
            ..entity()
        };
        let p = PlayerEnv::blank();
        assert_eq!(
            on_show_hitboxes(&mut g, &e, &p, &crate::config::DEFAULTS),
            Outcome::Cancelled { drew: false }
        );
        assert!(g.draws.is_empty());
    }

    #[test]
    fn a_pearl_draws_its_kind_color_and_gates() {
        let mut g = RecordingGizmos::default();
        let e = EntityView {
            is_projectile: true,
            is_ender_pearl: true,
            pearl_owner_is_own: Some(true),
            ..entity()
        };
        let p = PlayerEnv::blank();
        let c = crate::config::DEFAULTS;
        assert_eq!(
            on_show_hitboxes(&mut g, &e, &p, &c),
            Outcome::Cancelled { drew: true }
        );
        assert_eq!(
            g.draws[0],
            crate::seams::Draw::Cuboid(
                e.bounding_box.moved(e.delta()),
                Stroke {
                    color: c.pearl_own_color,
                    width: Some(c.pearl_line_width)
                }
            )
        );
        let mut g2 = RecordingGizmos::default();
        let e2 = EntityView {
            is_projectile: true,
            is_ender_pearl: true,
            pearl_owner_is_own: Some(true),
            ..entity()
        };
        let mut c2 = crate::config::DEFAULTS;
        c2.pearl_own = false;
        // the pearl gate returns after the cancel: nothing draws
        assert_eq!(
            on_show_hitboxes(&mut g2, &e2, &p, &c2),
            Outcome::Cancelled { drew: false }
        );
        assert!(g2.draws.is_empty());
    }

    #[test]
    fn a_projectile_that_is_not_a_pearl_draws_the_base_color() {
        let mut g = RecordingGizmos::default();
        let e = EntityView {
            is_projectile: true,
            ..entity()
        };
        let p = PlayerEnv::blank();
        on_show_hitboxes(&mut g, &e, &p, &crate::config::DEFAULTS);
        assert!(matches!(
            &g.draws[0],
            crate::seams::Draw::Cuboid(_, Stroke { color: 0xFFFF_FFFF, .. })
        ));
    }

    #[test]
    fn a_non_living_draws_target_or_base_color_with_line1() {
        let mut g = RecordingGizmos::default();
        let e = EntityView {
            is_living: false,
            ..entity()
        };
        let mut p = PlayerEnv::blank();
        p.hit_targets_entity = true;
        p.within_interaction_range_1_1 = true;
        p.distance_to_entity = 3.0;
        on_show_hitboxes(&mut g, &e, &p, &crate::config::DEFAULTS);
        assert!(matches!(
            &g.draws[0],
            crate::seams::Draw::Cuboid(_, Stroke { color: 0xFFFF_0000, width: Some(2.5) })
        ));
        let mut g2 = RecordingGizmos::default();
        p.within_interaction_range_1_1 = false;
        on_show_hitboxes(&mut g2, &e, &p, &crate::config::DEFAULTS);
        assert!(matches!(
            &g2.draws[0],
            crate::seams::Draw::Cuboid(_, Stroke { color: 0xFFFF_FFFF, width: Some(2.5) })
        ));
    }

    #[test]
    fn blocking_and_distance_pick_the_color_and_width() {
        let mut g = RecordingGizmos::default();
        let e = EntityView {
            is_blocking: true,
            blocking_with_shield: true,
            ..entity()
        };
        let mut p = PlayerEnv::blank();
        p.distance_to_entity = 40.0;
        on_show_hitboxes(&mut g, &e, &p, &crate::config::DEFAULTS);
        // within distFor2: line1, shield color while blocking
        assert!(matches!(
            &g.draws[0],
            crate::seams::Draw::Cuboid(_, Stroke { color: 0xFFFF_CF40, width: Some(2.5) })
        ));
        let mut g2 = RecordingGizmos::default();
        p.distance_to_entity = 50.0;
        on_show_hitboxes(&mut g2, &e, &p, &crate::config::DEFAULTS);
        // beyond distFor2: line2 (2.5 here too; the branch still reads)
        assert!(matches!(
            &g2.draws[0],
            crate::seams::Draw::Cuboid(_, Stroke { color: 0xFFFF_CF40, width: Some(2.5) })
        ));
    }

    #[test]
    fn the_living_draw_records_its_parts() {
        let mut g = RecordingGizmos::default();
        let e = EntityView {
            vehicle: Some(crate::seams::VehicleView {
                bb_width: 1.0,
                riding_position: [10.0, 66.0, 10.0],
            }),
            dragon_parts: vec![crate::seams::DragonPart {
                position: [0.0, 0.0, 0.0],
                position_at: [1.0, 0.0, 0.0],
                bounding_box: Aabb {
                    min: [-1.0, 0.0, -1.0],
                    max: [1.0, 1.0, 1.0],
                },
            }],
            ..entity()
        };
        let p = PlayerEnv::blank();
        on_show_hitboxes(&mut g, &e, &p, &crate::config::DEFAULTS);
        let draws = &g.draws;
        // cuboid, point at the interpolated position, seat box, eye slab, look arrow
        assert!(matches!(draws[0], crate::seams::Draw::Cuboid(_, _)));
        assert!(matches!(draws[1], crate::seams::Draw::Point(_, 0xFFFF_FFFF, 2.0)));
        assert!(matches!(
            draws[2],
            crate::seams::Draw::Cuboid(_, Stroke { color: 0xFFFF_FF00, .. })
        ));
        // the eye-height slab in the entity color
        assert!(matches!(
            draws[3],
            crate::seams::Draw::Cuboid(_, Stroke { color: 0xFFFF_0000, .. })
        ));
        // the look direction arrow, width line1 * 1.5
        assert!(matches!(
            draws[4],
            crate::seams::Draw::Arrow(_, _, 0xFF00_00FF, 3.75)
        ));
        // outline off by default: no extra draw
        assert_eq!(draws.len(), 5);
    }

    #[test]
    fn outline_and_look_line_flip_the_draws() {
        let mut g = RecordingGizmos::default();
        let e = entity();
        let p = PlayerEnv::blank();
        let mut c = crate::config::DEFAULTS.clone();
        c.outline_enabled = true;
        c.line_look_dir = true;
        on_show_hitboxes(&mut g, &e, &p, &c);
        assert!(matches!(g.draws[0], crate::seams::Draw::Cuboid(_, Stroke { color: 0xFF00_0000, width: Some(5.0) })));
        assert!(matches!(g.draws[1], crate::seams::Draw::Cuboid(_, Stroke { color: 0xFFFF_FFFF, width: Some(2.5) })));
        assert!(matches!(g.draws[4], crate::seams::Draw::Line(_, _, 0xFF00_00FF, 2.5)));
    }

    #[test]
    fn dragon_parts_draw_green() {
        let mut g = RecordingGizmos::default();
        let e = EntityView {
            is_ender_dragon: true,
            dragon_parts: vec![crate::seams::DragonPart {
                position: [0.0, 0.0, 0.0],
                position_at: [1.0, 0.0, 0.0],
                bounding_box: Aabb {
                    min: [-1.0, 0.0, -1.0],
                    max: [1.0, 1.0, 1.0],
                },
            }],
            ..entity()
        };
        let p = PlayerEnv::blank();
        on_show_hitboxes(&mut g, &e, &p, &crate::config::DEFAULTS);
        let part_draw = g
            .draws
            .iter()
            .find(|d| matches!(d, crate::seams::Draw::Cuboid(_, Stroke { color: 0xFF40_FF00, width: None })));
        assert!(part_draw.is_some());
    }
}
