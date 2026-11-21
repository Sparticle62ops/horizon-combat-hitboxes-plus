//! 1:1 port of me.truewulf.box.ModMenu (the ModMenu config screen).
//!
//! The screen's four categories (general, colors, projectiles, filter) and
//! their sections map onto the settings page the mod API draws: every entry
//! registers with the category's lang key as its label, in the screen's order
//! (states, rendering / colors, outline, width / pearls / melee, ranged, items,
//! misc), and the entries' tooltips carry their `.tooltip` lang keys. The
//! screen's field defaults are Config's; the one mismatch in the original
//! (look_as_line's toggle shows true on the screen while Config.java defaults
//! the field false) stays as the field default.

/// java: ModMenu's screen title lang key.
pub const CONFIG_SCREEN: &str = "combat_hitboxes.title";

/// The category lang keys, screen order.
pub const CATEGORIES: [&str; 4] = [
    "combat_hitboxes.category.general",
    "combat_hitboxes.category.colors",
    "combat_hitboxes.category.projectiles",
    "combat_hitboxes.category.filter",
];

/// The section lang keys, in the order each category draws them.
pub const SECTIONS: [&str; 10] = [
    "combat_hitboxes.section.states",
    "combat_hitboxes.section.rendering",
    "combat_hitboxes.section.colors",
    "combat_hitboxes.section.outline",
    "combat_hitboxes.section.width",
    "combat_hitboxes.section.pearls",
    "combat_hitboxes.section.melee",
    "combat_hitboxes.section.ranged",
    "combat_hitboxes.section.items",
    "combat_hitboxes.section.misc",
];
