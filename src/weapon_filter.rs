//! 1:1 port of me.truewulf.box.WeaponFilter.

use crate::config::ModConfig;

/// java: WeaponFilter.playerAllowed() — the filter off passes everything; a
/// missing player passes; then the main hand, then the off hand only when
/// `filterOffHand` is on. The seam passes the held stacks' item ids, present
/// only when a local player exists.
pub fn player_allowed(config: &ModConfig, main_hand: Option<&str>, off_hand: Option<&str>) -> bool {
    if !config.weapon_filter_enabled {
        return true;
    }
    // java: if (player == null) return true;
    let (Some(main_hand), Some(off_hand)) = (main_hand, off_hand) else {
        return true;
    };
    if matches(config, Some(main_hand)) {
        return true;
    }
    config.filter_off_hand && matches(config, Some(off_hand))
}

/// java: WeaponFilter.matches(ItemStack, Config) — the same chain over the
/// item id.
fn matches(config: &ModConfig, stack: Option<&str>) -> bool {
    let Some(id) = stack else {
        return false;
    };
    if config.filter_sword && id.contains("_sword") {
        return true;
    }
    if config.filter_axe && id.contains("_axe") {
        return true;
    }
    if config.filter_pickaxe && id.contains("_pickaxe") {
        return true;
    }
    if config.filter_shovel && id.contains("_shovel") {
        return true;
    }
    if config.filter_mace && id.contains("mace") {
        return true;
    }
    if config.filter_spear && id.contains("_spear") {
        return true;
    }
    if config.filter_trident && id == "minecraft:trident" {
        return true;
    }
    if config.filter_bow && id == "minecraft:bow" {
        return true;
    }
    if config.filter_crossbow && id == "minecraft:crossbow" {
        return true;
    }
    if config.filter_ender_pearl && id == "minecraft:ender_pearl" {
        return true;
    }
    if config.filter_wind_charge && id.contains("wind_charge") {
        return true;
    }
    if config.filter_golden_apple && id.contains("golden_apple") {
        return true;
    }
    if config.filter_xp_bottle && id == "minecraft:experience_bottle" {
        return true;
    }
    if config.filter_totem && id == "minecraft:totem_of_undying" {
        return true;
    }
    if config.filter_firework && id == "minecraft:firework_rocket" {
        return true;
    }
    if config.filter_splash_potion && id == "minecraft:splash_potion" {
        return true;
    }
    if config.filter_golden_carrot && id == "minecraft:golden_carrot" {
        return true;
    }
    if config.filter_elytra && id == "minecraft:elytra" {
        return true;
    }
    if config.filter_end_crystal && id == "minecraft:end_crystal" {
        return true;
    }
    if config.filter_anchor && id == "minecraft:respawn_anchor" {
        return true;
    }
    if config.filter_glowstone && id == "minecraft:glowstone" {
        return true;
    }
    if config.filter_cobweb && id == "minecraft:cobweb" {
        return true;
    }
    if config.filter_fishing_rod && id == "minecraft:fishing_rod" {
        return true;
    }
    if config.filter_steak && id == "minecraft:cooked_beef" {
        return true;
    }
    if config.filter_flint_and_steel && id == "minecraft:flint_and_steel" {
        return true;
    }
    if config.filter_armor_stand && id == "minecraft:armor_stand" {
        return true;
    }
    if config.filter_lava_bucket && id == "minecraft:lava_bucket" {
        return true;
    }
    if config.filter_water_bucket && id == "minecraft:water_bucket" {
        return true;
    }
    if config.filter_obsidian && id == "minecraft:obsidian" {
        return true;
    }
    if config.filter_other
        && (id.ends_with("_halberd")
            || id.ends_with("_katana")
            || id.ends_with("_rapier")
            || id.ends_with("_cutlass")
            || id.ends_with("_scythe")
            || id.ends_with("_dagger")
            || id.ends_with("_hammer")
            || id.ends_with("_glaive"))
    {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> ModConfig {
        let mut c = crate::config::DEFAULTS.clone();
        c.weapon_filter_enabled = true;
        c
    }

    #[test]
    fn the_filter_off_passes_everything() {
        let mut c = crate::config::DEFAULTS.clone();
        assert!(player_allowed(&c, Some("minecraft:nothing"), Some("minecraft:nothing")));
        c.weapon_filter_enabled = true;
        c.filter_sword = false;
        assert!(!player_allowed(&c, Some("minecraft:stone_sword"), Some("minecraft:dirt")));
    }

    #[test]
    fn a_missing_player_passes() {
        let c = cfg();
        assert!(player_allowed(&c, None, None));
    }

    #[test]
    fn the_main_hand_then_the_off_hand() {
        let mut c = cfg();
        assert!(player_allowed(&c, Some("minecraft:iron_sword"), Some("minecraft:dirt")));
        c.filter_off_hand = false;
        assert!(!player_allowed(&c, Some("minecraft:dirt"), Some("minecraft:bow")));
        c.filter_off_hand = true;
        assert!(player_allowed(&c, Some("minecraft:dirt"), Some("minecraft:bow")));
    }

    #[test]
    fn the_id_chain_matches_the_java_rules() {
        let c = cfg();
        assert!(matches(&c, Some("minecraft:netherite_sword")));
        assert!(matches(&c, Some("minecraft:diamond_axe")));
        assert!(!matches(&c, Some("minecraft:pickaxe_disabled")));
        let mut c2 = c.clone();
        c2.filter_pickaxe = true;
        assert!(matches(&c2, Some("minecraft:iron_pickaxe")));
        assert!(matches(&c, Some("minecraft:mace"))); // contains "mace"
        assert!(matches(&c, Some("minecraft:trident")));
        assert!(matches(&c, Some("minecraft:bow")));
        assert!(matches(&c, Some("minecraft:wind_charge")));
        assert!(matches(&c, Some("minecraft:golden_apple")));
        assert!(!matches(&c, Some("minecraft:golden_carrot"))); // its own flag, off by default
        let mut c3 = c.clone();
        c3.filter_golden_carrot = true;
        assert!(matches(&c3, Some("minecraft:golden_carrot")));
        assert!(matches(&c, Some("minecraft:elytra")));
        assert!(matches(&c, Some("custommod:steel_dagger"))); // filter_other suffix
        assert!(!matches(&c, Some("minecraft:dirt")));
        assert!(!matches(&c, None));
    }
}
