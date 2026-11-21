//! 1:1 Horizon Rust port of Combat Hitboxes+ (TrueWulf, Apache-2.0).
//! Upstream: https://github.com/TrueWulf/combat-hitboxes-plus
//!
//! Every Java class maps to one module below, same names, same logic, same
//! defaults, same control flow. Host seams (seams.rs) carry the Minecraft
//! client render surfaces the engine binds; where an upstream type is
//! engine-bound, the seam mirrors its Java calls one to one.
//!
//! This root carries the mod API wiring on the pinned SDK (Horizon f00125f61f,
//! mixins e59da909, mod api 0.2):
//! - the original's 60 config entries as settings with upstream's own lang
//!   keys as the labels and tooltips, in the ModMenu screen's order (the
//!   colors register as 0xRRGGBB; the original stores 0xAARRGGBB and
//!   normalizeColors forces opaque alpha on read), the numeric bounds the
//!   original's own screen sets (renderDistance floor 4.0, line1/line2/
//!   pearlLineWidth 0..25, distFor2 0..256, outlineMultiplier 1.0000001..10);
//!   the settings page's slider needs an upper bound for renderDistance, so it
//!   caps at 1024 without binding gameplay;
//! - `client_setup` syncs the settings into the live config cell
//!   (Config.getInstance()'s role) and runs the original's entry
//!   (`Main.onInitializeClient()`).
//!
//! Gap, tracked: the mod API has no surface to cancel an existing debug
//! renderer's draw (the mixin cancels the vanilla hitbox debug renderer's
//! draw and draws its own gizmos in its place) and no client read for the
//! per-view entity set (bounding box, interpolated position, eye height, view
//! vector, vehicle, dragon parts, blocking state, hurt time) or the local
//! player's reads (distance, targeted hit result, the 1.1 interaction range,
//! held item ids); the draw path runs on the seams until one lands.
//!
//! Upstream licence: Apache-2.0 (LICENSE carries the upstream text verbatim,
//! §4(a) full text, and the Rust translation is distributed under the same
//! licence). Source: https://github.com/Sparticle62ops/horizon-combat-hitboxes-plus

use mod_api::{ClientSetupEvent, ClientHost, RegistrationContext, Registrar, SettingSpec};

pub mod config;
pub mod hitbox_render;
pub mod mod_menu;
pub mod projectile_owner;
pub mod seams;
pub mod stuck_arrow;
pub mod weapon_filter;

/// java: me.truewulf.box.Main — the bin target would eat a `src/main.rs`,
/// so the class stays an inline module (same content).
pub mod main {
    use crate::seams;

    /// java: Main.LOGGER — "CombatHitboxes".
    pub const LOGGER_NAME: &str = "CombatHitboxes";

    /// java: Main.onInitializeClient() — the log line is the whole entry.
    pub fn on_initialize_client() {
        seams::logger().info("Combat Hitboxes+ loaded");
    }
}

const P: &str = "combat_hitboxes.";

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl mod_api::Hooks for Mod {
    /// 1:1 of the original's 60 config entries the settings page can hold,
    /// the field defaults verbatim, upstream's own lang keys as the labels and
    /// tooltips, in the ModMenu screen's order.
    fn register(&self, registrar: &mut dyn Registrar, _context: &RegistrationContext) {
        // general — states
        let _ = registrar.register_setting(SettingSpec::toggle("enabled", P.to_string() + "general.enabled", config::DEFAULTS.enabled));
        let _ = registrar.register_setting(SettingSpec::toggle("shield_color_enabled", P.to_string() + "general.shield_color", config::DEFAULTS.shield_color_enabled).tooltip(P.to_string() + "general.shield_color.tooltip"));
        let _ = registrar.register_setting(SettingSpec::toggle("shield_only_target", P.to_string() + "general.shield_only_target", config::DEFAULTS.shield_only_target));
        let _ = registrar.register_setting(SettingSpec::toggle("hide_fireworks", P.to_string() + "general.hide_fireworks", config::DEFAULTS.hide_fireworks));
        let _ = registrar.register_setting(SettingSpec::toggle("hide_arrow", P.to_string() + "general.hide_stuck_arrows", config::DEFAULTS.hide_arrow).tooltip(P.to_string() + "general.hide_stuck_arrows.tooltip"));
        // general — rendering
        let _ = registrar.register_setting(SettingSpec::toggle("render_eye_height", P.to_string() + "general.render_eye_height", config::DEFAULTS.render_eye_height));
        let _ = registrar.register_setting(SettingSpec::toggle("render_look_dir", P.to_string() + "general.render_look_dir", config::DEFAULTS.render_look_dir));
        let _ = registrar.register_setting(SettingSpec::toggle("line_look_dir", P.to_string() + "general.look_as_line", config::DEFAULTS.line_look_dir).tooltip(P.to_string() + "general.look_as_line.tooltip"));
        let _ = registrar.register_setting(SettingSpec::range(
            // java: the ModMenu entry clamps the floor at 4.0; the original's
            // field itself is unbounded, the slider caps at 1024.
            "render_distance",
            P.to_string() + "general.render_distance",
            4.0,
            1024.0,
            1.0,
            config::DEFAULTS.render_distance,
        ).tooltip(P.to_string() + "general.render_distance.tooltip"));
        // colors
        let _ = registrar.register_setting(SettingSpec::color("hit_box_color", P.to_string() + "colors.base", 0xFF_FFFF));
        let _ = registrar.register_setting(SettingSpec::color("target_box_color", P.to_string() + "colors.target_box", 0xFF_0000).tooltip(P.to_string() + "colors.target_box.tooltip"));
        let _ = registrar.register_setting(SettingSpec::color("shield_color", P.to_string() + "colors.shield", 0xFF_CF40));
        let _ = registrar.register_setting(SettingSpec::color("eye_color", P.to_string() + "colors.eye", 0xFF_0000));
        let _ = registrar.register_setting(SettingSpec::color("look_color", P.to_string() + "colors.look", 0x00_00FF));
        let _ = registrar.register_setting(SettingSpec::toggle("outline_enabled", P.to_string() + "colors.outline_enabled", config::DEFAULTS.outline_enabled));
        let _ = registrar.register_setting(SettingSpec::color("outline_color", P.to_string() + "colors.outline_color", 0x00_0000));
        let _ = registrar.register_setting(SettingSpec::range(
            // java: setMin(Math.nextUp(1.0F)).setMax(10.0F)
            "outline_multiplier",
            P.to_string() + "colors.outline_multiplier",
            1.0000001,
            10.0,
            0.1,
            config::DEFAULTS.outline_multiplier as f64,
        ));
        let _ = registrar.register_setting(SettingSpec::range(
            // java: setMin(0.0F).setMax(25.0F)
            "line1",
            P.to_string() + "colors.line1",
            0.0,
            25.0,
            0.1,
            config::DEFAULTS.line1 as f64,
        ));
        let _ = registrar.register_setting(SettingSpec::range(
            // java: setMin(0).setMax(256)
            "dist_for2",
            P.to_string() + "colors.dist_for2",
            0.0,
            256.0,
            1.0,
            config::DEFAULTS.dist_for2,
        ).tooltip(P.to_string() + "colors.dist_for2.tooltip"));
        let _ = registrar.register_setting(SettingSpec::range(
            "line2",
            P.to_string() + "colors.line2",
            0.0,
            25.0,
            0.1,
            config::DEFAULTS.line2 as f64,
        ));
        // projectiles — pearls
        let _ = registrar.register_setting(SettingSpec::toggle("pearls_enabled", P.to_string() + "projectiles.pearls", config::DEFAULTS.pearls_enabled));
        let _ = registrar.register_setting(SettingSpec::toggle("pearl_own", P.to_string() + "projectiles.pearl_own", config::DEFAULTS.pearl_own));
        let _ = registrar.register_setting(SettingSpec::toggle("pearl_others", P.to_string() + "projectiles.pearl_others", config::DEFAULTS.pearl_others));
        let _ = registrar.register_setting(SettingSpec::toggle("pearl_unknown", P.to_string() + "projectiles.pearl_unknown", config::DEFAULTS.pearl_unknown));
        let _ = registrar.register_setting(SettingSpec::color("pearl_own_color", P.to_string() + "projectiles.pearl_own_color", 0x00_55FF));
        let _ = registrar.register_setting(SettingSpec::color("pearl_enemy_color", P.to_string() + "projectiles.pearl_enemy_color", 0xFF_3333));
        let _ = registrar.register_setting(SettingSpec::color("pearl_unknown_color", P.to_string() + "projectiles.pearl_unknown_color", 0x33_55CC));
        let _ = registrar.register_setting(SettingSpec::range(
            "pearl_line_width",
            P.to_string() + "projectiles.pearl_width",
            0.0,
            25.0,
            0.1,
            config::DEFAULTS.pearl_line_width as f64,
        ));
        // filter — melee
        let _ = registrar.register_setting(SettingSpec::toggle("weapon_filter_enabled", P.to_string() + "filter.enabled", config::DEFAULTS.weapon_filter_enabled).tooltip(P.to_string() + "filter.enabled.tooltip"));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_off_hand", P.to_string() + "filter.offhand", config::DEFAULTS.filter_off_hand));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_sword", P.to_string() + "filter.sword", config::DEFAULTS.filter_sword));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_axe", P.to_string() + "filter.axe", config::DEFAULTS.filter_axe));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_pickaxe", P.to_string() + "filter.pickaxe", config::DEFAULTS.filter_pickaxe));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_shovel", P.to_string() + "filter.shovel", config::DEFAULTS.filter_shovel));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_mace", P.to_string() + "filter.mace", config::DEFAULTS.filter_mace));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_trident", P.to_string() + "filter.trident", config::DEFAULTS.filter_trident));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_spear", P.to_string() + "filter.spear", config::DEFAULTS.filter_spear));
        // filter — ranged
        let _ = registrar.register_setting(SettingSpec::toggle("filter_bow", P.to_string() + "filter.bow", config::DEFAULTS.filter_bow));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_crossbow", P.to_string() + "filter.crossbow", config::DEFAULTS.filter_crossbow));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_ender_pearl", P.to_string() + "filter.ender_pearl", config::DEFAULTS.filter_ender_pearl));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_wind_charge", P.to_string() + "filter.wind_charge", config::DEFAULTS.filter_wind_charge));
        // filter — items
        let _ = registrar.register_setting(SettingSpec::toggle("filter_golden_apple", P.to_string() + "filter.golden_apple", config::DEFAULTS.filter_golden_apple));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_golden_carrot", P.to_string() + "filter.golden_carrot", config::DEFAULTS.filter_golden_carrot));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_xp_bottle", P.to_string() + "filter.xp_bottle", config::DEFAULTS.filter_xp_bottle));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_totem", P.to_string() + "filter.totem", config::DEFAULTS.filter_totem));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_firework", P.to_string() + "filter.firework", config::DEFAULTS.filter_firework));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_splash_potion", P.to_string() + "filter.splash_potion", config::DEFAULTS.filter_splash_potion));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_elytra", P.to_string() + "filter.elytra", config::DEFAULTS.filter_elytra));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_end_crystal", P.to_string() + "filter.end_crystal", config::DEFAULTS.filter_end_crystal));
        // filter — misc
        let _ = registrar.register_setting(SettingSpec::toggle("filter_anchor", P.to_string() + "filter.anchor", config::DEFAULTS.filter_anchor));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_glowstone", P.to_string() + "filter.glowstone", config::DEFAULTS.filter_glowstone));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_cobweb", P.to_string() + "filter.cobweb", config::DEFAULTS.filter_cobweb));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_fishing_rod", P.to_string() + "filter.fishing_rod", config::DEFAULTS.filter_fishing_rod));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_steak", P.to_string() + "filter.steak", config::DEFAULTS.filter_steak));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_flint_and_steel", P.to_string() + "filter.flint_and_steel", config::DEFAULTS.filter_flint_and_steel));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_armor_stand", P.to_string() + "filter.armor_stand", config::DEFAULTS.filter_armor_stand));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_lava_bucket", P.to_string() + "filter.lava_bucket", config::DEFAULTS.filter_lava_bucket));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_water_bucket", P.to_string() + "filter.water_bucket", config::DEFAULTS.filter_water_bucket));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_obsidian", P.to_string() + "filter.obsidian", config::DEFAULTS.filter_obsidian));
        let _ = registrar.register_setting(SettingSpec::toggle("filter_other", P.to_string() + "filter.filter_other", config::DEFAULTS.filter_other));
    }

    /// 1:1 of the original's entry (Config.getInstance()'s sync plus
    /// Main.onInitializeClient): the settings sync into the live config cell,
    /// colors re-opaque through normalizeColors, then the entry logs.
    fn client_setup(&self, host: &mut dyn ClientHost, _event: &mut ClientSetupEvent) {
        let d = &config::DEFAULTS;
        let bool_at = |key: &str, v: bool| host.setting_bool(key).unwrap_or(v);
        let num_at = |key: &str, v: f64| host.setting_f64(key).unwrap_or(v);
        let color_at = |key: &str, v: u32| host.setting_color(key).unwrap_or(v >> 24 & 0xFF_FFFF);
        let mut cfg = config::ModConfig {
            enabled: bool_at("enabled", d.enabled),
            hide_arrow: bool_at("hide_arrow", d.hide_arrow),
            hide_fireworks: bool_at("hide_fireworks", d.hide_fireworks),
            hit_box_color: color_at("hit_box_color", d.hit_box_color),
            target_box_color: color_at("target_box_color", d.target_box_color),
            eye_color: color_at("eye_color", d.eye_color),
            look_color: color_at("look_color", d.look_color),
            render_eye_height: bool_at("render_eye_height", d.render_eye_height),
            render_look_dir: bool_at("render_look_dir", d.render_look_dir),
            line_look_dir: bool_at("line_look_dir", d.line_look_dir),
            render_distance: num_at("render_distance", d.render_distance),
            shield_color_enabled: bool_at("shield_color_enabled", d.shield_color_enabled),
            shield_color: color_at("shield_color", d.shield_color),
            shield_only_target: bool_at("shield_only_target", d.shield_only_target),
            pearls_enabled: bool_at("pearls_enabled", d.pearls_enabled),
            pearl_own: bool_at("pearl_own", d.pearl_own),
            pearl_others: bool_at("pearl_others", d.pearl_others),
            pearl_unknown: bool_at("pearl_unknown", d.pearl_unknown),
            pearl_own_color: color_at("pearl_own_color", d.pearl_own_color),
            pearl_enemy_color: color_at("pearl_enemy_color", d.pearl_enemy_color),
            pearl_unknown_color: color_at("pearl_unknown_color", d.pearl_unknown_color),
            pearl_line_width: num_at("pearl_line_width", d.pearl_line_width as f64) as f32,
            weapon_filter_enabled: bool_at("weapon_filter_enabled", d.weapon_filter_enabled),
            filter_off_hand: bool_at("filter_off_hand", d.filter_off_hand),
            filter_sword: bool_at("filter_sword", d.filter_sword),
            filter_axe: bool_at("filter_axe", d.filter_axe),
            filter_pickaxe: bool_at("filter_pickaxe", d.filter_pickaxe),
            filter_shovel: bool_at("filter_shovel", d.filter_shovel),
            filter_mace: bool_at("filter_mace", d.filter_mace),
            filter_trident: bool_at("filter_trident", d.filter_trident),
            filter_spear: bool_at("filter_spear", d.filter_spear),
            filter_bow: bool_at("filter_bow", d.filter_bow),
            filter_crossbow: bool_at("filter_crossbow", d.filter_crossbow),
            filter_ender_pearl: bool_at("filter_ender_pearl", d.filter_ender_pearl),
            filter_wind_charge: bool_at("filter_wind_charge", d.filter_wind_charge),
            filter_golden_apple: bool_at("filter_golden_apple", d.filter_golden_apple),
            filter_xp_bottle: bool_at("filter_xp_bottle", d.filter_xp_bottle),
            filter_totem: bool_at("filter_totem", d.filter_totem),
            filter_firework: bool_at("filter_firework", d.filter_firework),
            filter_splash_potion: bool_at("filter_splash_potion", d.filter_splash_potion),
            filter_golden_carrot: bool_at("filter_golden_carrot", d.filter_golden_carrot),
            filter_elytra: bool_at("filter_elytra", d.filter_elytra),
            filter_end_crystal: bool_at("filter_end_crystal", d.filter_end_crystal),
            filter_anchor: bool_at("filter_anchor", d.filter_anchor),
            filter_glowstone: bool_at("filter_glowstone", d.filter_glowstone),
            filter_cobweb: bool_at("filter_cobweb", d.filter_cobweb),
            filter_fishing_rod: bool_at("filter_fishing_rod", d.filter_fishing_rod),
            filter_steak: bool_at("filter_steak", d.filter_steak),
            filter_flint_and_steel: bool_at("filter_flint_and_steel", d.filter_flint_and_steel),
            filter_armor_stand: bool_at("filter_armor_stand", d.filter_armor_stand),
            filter_lava_bucket: bool_at("filter_lava_bucket", d.filter_lava_bucket),
            filter_water_bucket: bool_at("filter_water_bucket", d.filter_water_bucket),
            filter_obsidian: bool_at("filter_obsidian", d.filter_obsidian),
            filter_other: bool_at("filter_other", d.filter_other),
            line1: num_at("line1", d.line1 as f64) as f32,
            dist_for2: num_at("dist_for2", d.dist_for2),
            line2: num_at("line2", d.line2 as f64) as f32,
            outline_enabled: bool_at("outline_enabled", d.outline_enabled),
            outline_color: color_at("outline_color", d.outline_color),
            outline_multiplier: num_at("outline_multiplier", d.outline_multiplier as f64) as f32,
        };
        cfg.normalize_colors();
        *config::client().lock().unwrap() = cfg;
        main::on_initialize_client();
    }
}
