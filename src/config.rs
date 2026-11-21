//! 1:1 port of me.truewulf.box.Config.
//!
//! The original persists itself as combat-hitboxes.json in the config dir and
//! builds its screen from these fields; on the mod API the settings page keeps
//! the values and `client_setup` syncs them into the live cell, so the file
//! read is the one seam (the storage the host owns). The field set, the
//! defaults, `normalizeColors`, `withOpaqueAlpha` and `migrateLegacy` are the
//! original's, same names, same logic.

use std::sync::{Mutex, OnceLock};

/// java: public class Config — every field, same names, same defaults.
#[derive(Clone, Debug, PartialEq)]
pub struct ModConfig {
    pub enabled: bool,
    pub hide_arrow: bool,
    pub hide_fireworks: bool,
    pub hit_box_color: u32,
    pub target_box_color: u32,
    pub eye_color: u32,
    pub look_color: u32,
    pub render_eye_height: bool,
    pub render_look_dir: bool,
    pub line_look_dir: bool,
    pub render_distance: f64,
    pub shield_color_enabled: bool,
    pub shield_color: u32,
    pub shield_only_target: bool,
    pub pearls_enabled: bool,
    pub pearl_own: bool,
    pub pearl_others: bool,
    pub pearl_unknown: bool,
    pub pearl_own_color: u32,
    pub pearl_enemy_color: u32,
    pub pearl_unknown_color: u32,
    pub pearl_line_width: f32,
    pub weapon_filter_enabled: bool,
    pub filter_off_hand: bool,
    pub filter_sword: bool,
    pub filter_axe: bool,
    pub filter_pickaxe: bool,
    pub filter_shovel: bool,
    pub filter_mace: bool,
    pub filter_trident: bool,
    pub filter_spear: bool,
    pub filter_bow: bool,
    pub filter_crossbow: bool,
    pub filter_ender_pearl: bool,
    pub filter_wind_charge: bool,
    pub filter_golden_apple: bool,
    pub filter_xp_bottle: bool,
    pub filter_totem: bool,
    pub filter_firework: bool,
    pub filter_splash_potion: bool,
    pub filter_golden_carrot: bool,
    pub filter_elytra: bool,
    pub filter_end_crystal: bool,
    pub filter_anchor: bool,
    pub filter_glowstone: bool,
    pub filter_cobweb: bool,
    pub filter_fishing_rod: bool,
    pub filter_steak: bool,
    pub filter_flint_and_steel: bool,
    pub filter_armor_stand: bool,
    pub filter_lava_bucket: bool,
    pub filter_water_bucket: bool,
    pub filter_obsidian: bool,
    pub filter_other: bool,
    pub line1: f32,
    pub dist_for2: f64,
    pub line2: f32,
    pub outline_enabled: bool,
    pub outline_color: u32,
    pub outline_multiplier: f32,
}

/// The constructor defaults, verbatim from Config.java's field initializers.
pub const DEFAULTS: ModConfig = ModConfig {
    enabled: true,
    hide_arrow: false,
    hide_fireworks: true,
    hit_box_color: 0xFFFF_FFFF,
    target_box_color: 0xFFFF_0000,
    eye_color: 0xFFFF_0000,
    look_color: 0xFF00_00FF,
    render_eye_height: true,
    render_look_dir: true,
    line_look_dir: false,
    render_distance: 72.0,
    shield_color_enabled: true,
    shield_color: 0xFFFF_CF40,
    shield_only_target: false,
    pearls_enabled: true,
    pearl_own: true,
    pearl_others: true,
    pearl_unknown: true,
    pearl_own_color: 0xFF00_55FF,
    pearl_enemy_color: 0xFFFF_3333,
    pearl_unknown_color: 0xFF33_55CC,
    pearl_line_width: 2.5,
    weapon_filter_enabled: false,
    filter_off_hand: true,
    filter_sword: true,
    filter_axe: true,
    filter_pickaxe: false,
    filter_shovel: false,
    filter_mace: true,
    filter_trident: true,
    filter_spear: true,
    filter_bow: true,
    filter_crossbow: true,
    filter_ender_pearl: true,
    filter_wind_charge: true,
    filter_golden_apple: true,
    filter_xp_bottle: false,
    filter_totem: true,
    filter_firework: false,
    filter_splash_potion: false,
    filter_golden_carrot: false,
    filter_elytra: true,
    filter_end_crystal: true,
    filter_anchor: true,
    filter_glowstone: true,
    filter_cobweb: true,
    filter_fishing_rod: false,
    filter_steak: false,
    filter_flint_and_steel: false,
    filter_armor_stand: false,
    filter_lava_bucket: true,
    filter_water_bucket: false,
    filter_obsidian: false,
    filter_other: true,
    line1: 2.5,
    dist_for2: 32.0,
    line2: 2.5,
    outline_enabled: false,
    outline_color: 0xFF00_0000,
    outline_multiplier: 2.0,
};

/// java: migrateLegacy reads the parsed legacy JSON; the fields it touches,
/// present in the map or not. (The JSON parse itself is the host's storage
/// seam; the settings page keeps the live values.)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyMap {
    pub has_shield_color_enabled: bool,
    pub shield_hitbox_enabled: Option<bool>,
    pub target_box_color: Option<u32>,
    pub shield_color: Option<u32>,
}

impl ModConfig {
    /// java: save() — normalizeColors runs before the value leaves the mod.
    pub fn normalized(mut self) -> ModConfig {
        self.normalize_colors();
        self
    }

    /// java: normalizeColors() — every color gets opaque alpha.
    pub fn normalize_colors(&mut self) {
        self.hit_box_color = with_opaque_alpha(self.hit_box_color);
        self.target_box_color = with_opaque_alpha(self.target_box_color);
        self.shield_color = with_opaque_alpha(self.shield_color);
        self.eye_color = with_opaque_alpha(self.eye_color);
        self.look_color = with_opaque_alpha(self.look_color);
        self.pearl_own_color = with_opaque_alpha(self.pearl_own_color);
        self.pearl_enemy_color = with_opaque_alpha(self.pearl_enemy_color);
        self.pearl_unknown_color = with_opaque_alpha(self.pearl_unknown_color);
        self.outline_color = with_opaque_alpha(self.outline_color);
    }

    /// java: migrateLegacy(JsonObject old) — the legacy key and the retired
    /// defaults fold into the current names.
    pub fn migrate_legacy(&mut self, old: Option<&LegacyMap>) {
        let Some(old) = old else {
            return;
        };
        if old.shield_hitbox_enabled.is_some() && !old.has_shield_color_enabled {
            self.shield_color_enabled = old.shield_hitbox_enabled.unwrap();
        }
        if self.target_box_color == 0xFFFF_5555 || self.target_box_color == 0xFFFF_0000 {
            self.target_box_color = 0xFFFF_0000;
        }
        if (self.shield_color & 0xFF_FFFF) == 0x00FF_9B00
            || (self.shield_color & 0xFF_FFFF) == 0x009B_30FF
        {
            self.shield_color = 0xFFFF_CF40;
        }
        self.line_look_dir = false;
    }
}

/// java: withOpaqueAlpha(int color)
pub fn with_opaque_alpha(color: u32) -> u32 {
    if (color & 0xFF00_0000) == 0 {
        color | 0xFF00_0000
    } else {
        color
    }
}

static CELL: OnceLock<Mutex<ModConfig>> = OnceLock::new();

/// The live config the settings page syncs into (java: Config.getInstance()).
pub fn client() -> &'static Mutex<ModConfig> {
    CELL.get_or_init(|| Mutex::new(DEFAULTS))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_java_field_initializers() {
        let d = &DEFAULTS;
        assert!(d.enabled);
        assert!(!d.hide_arrow);
        assert!(d.hide_fireworks);
        assert_eq!(d.hit_box_color, 0xFFFF_FFFF);
        assert_eq!(d.target_box_color, 0xFFFF_0000);
        assert_eq!(d.eye_color, 0xFFFF_0000);
        assert_eq!(d.look_color, 0xFF00_00FF);
        assert!(d.render_eye_height);
        assert!(d.render_look_dir);
        assert!(!d.line_look_dir);
        assert_eq!(d.render_distance, 72.0);
        assert!(d.shield_color_enabled);
        assert_eq!(d.shield_color, 0xFFFF_CF40);
        assert!(!d.shield_only_target);
        assert!(d.pearls_enabled && d.pearl_own && d.pearl_others && d.pearl_unknown);
        assert_eq!(d.pearl_own_color, 0xFF00_55FF);
        assert_eq!(d.pearl_enemy_color, 0xFFFF_3333);
        assert_eq!(d.pearl_unknown_color, 0xFF33_55CC);
        assert_eq!(d.pearl_line_width, 2.5);
        assert!(!d.weapon_filter_enabled);
        assert!(d.filter_off_hand && d.filter_sword && d.filter_axe);
        assert!(!d.filter_pickaxe && !d.filter_shovel);
        assert!(d.filter_mace && d.filter_trident && d.filter_spear);
        assert!(d.filter_bow && d.filter_crossbow && d.filter_ender_pearl);
        assert!(d.filter_wind_charge && d.filter_golden_apple);
        assert!(!d.filter_xp_bottle && !d.filter_firework && !d.filter_splash_potion);
        assert!(d.filter_totem && !d.filter_golden_carrot);
        assert!(d.filter_elytra && d.filter_end_crystal && d.filter_anchor);
        assert!(d.filter_glowstone && d.filter_cobweb);
        assert!(!d.filter_fishing_rod && !d.filter_steak && !d.filter_flint_and_steel);
        assert!(!d.filter_armor_stand);
        assert!(d.filter_lava_bucket && !d.filter_water_bucket && !d.filter_obsidian);
        assert!(d.filter_other);
        assert_eq!(d.line1, 2.5);
        assert_eq!(d.dist_for2, 32.0);
        assert_eq!(d.line2, 2.5);
        assert!(!d.outline_enabled);
        assert_eq!(d.outline_color, 0xFF00_0000);
        assert_eq!(d.outline_multiplier, 2.0);
    }

    #[test]
    fn normalize_colors_forces_opaque_alpha() {
        let mut c = DEFAULTS.clone();
        c.hit_box_color = 0x00FF_FF00;
        c.pearl_enemy_color = 0x1011_2233;
        c.normalize_colors();
        assert_eq!(c.hit_box_color, 0xFFFF_FF00);
        assert_eq!(c.pearl_enemy_color, 0x1011_2233);
        assert_eq!(with_opaque_alpha(0x0012_3456), 0xFF12_3456);
        assert_eq!(with_opaque_alpha(0xFF12_3456), 0xFF12_3456);
    }

    #[test]
    fn migrate_legacy_folds_the_retired_names() {
        let mut c = DEFAULTS.clone();
        c.migrate_legacy(Some(&LegacyMap {
            has_shield_color_enabled: false,
            shield_hitbox_enabled: Some(false),
            target_box_color: Some(0xFFFF_5555),
            shield_color: Some(0x00FF_9B00 | 0xFF00_0000),
        }));
        assert!(!c.shield_color_enabled);
        assert_eq!(c.target_box_color, 0xFFFF_0000);
        assert_eq!(c.shield_color, 0xFFFF_CF40);
        assert!(!c.line_look_dir);

        let mut untouched = DEFAULTS.clone();
        untouched.migrate_legacy(None);
        assert_eq!(untouched, DEFAULTS);

        let mut modern = DEFAULTS.clone();
        modern.migrate_legacy(Some(&LegacyMap {
            has_shield_color_enabled: true,
            shield_hitbox_enabled: Some(false),
            target_box_color: None,
            shield_color: None,
        }));
        assert!(modern.shield_color_enabled);
    }
}
