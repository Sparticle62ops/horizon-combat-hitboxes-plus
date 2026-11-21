# Horizon Combat Hitboxes+

Exact 1:1 Rust translation of [Combat Hitboxes+](https://github.com/TrueWulf/combat-hitboxes-plus)
(TrueWulf, Apache-2.0) for the Horizon client. Carries the upstream license
and credit (see LICENSE).

Combat Hitboxes+ draws colored hitboxes for every mob and player in view:
the vanilla hitbox debug draw is cancelled and the mod renders its own boxes,
with a different color for the entity you are looking at and can reach, a
shield color while the entity blocks, separate colors for ender pearls by
owner (yours, enemies', unknown), the eye-height slab, the vehicle seat box,
the dragon's parts, and the look direction, plus the weapon filter (held
weapon decides whether hitboxes render) and the stuck-arrow hide.

Every Java class maps to one module: `config`, `weapon_filter`,
`projectile_owner`, `hitbox_render` (the `HitBoxRenderMixin` logic),
`stuck_arrow` (the `StuckArrowMixin`/`StuckStingerMixin` logic), `mod_menu`,
`main`, over `seams` (the Minecraft client surfaces the engine binds).

The mod API wires the original's 60 config entries as settings with
upstream's own lang keys as the labels and tooltips, and runs the original's
entry (`Main.onInitializeClient()`) on client setup.

Pinned to Horizon f00125f and horizon-mixins e59da909; exact 1:1 Rust port
tested against those revisions.

Source: https://github.com/Sparticle62ops/horizon-combat-hitboxes-plus
