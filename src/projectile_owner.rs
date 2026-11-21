//! 1:1 port of me.truewulf.box.ProjectileOwner.

/// java: enum Kind { OWN, ENEMY, UNKNOWN }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Own,
    Enemy,
    Unknown,
}

/// java: ProjectileOwner.of(Projectile) — owner == null -> UNKNOWN,
/// owner == Main.mc.player -> OWN, anyone else -> ENEMY. The seam passes
/// whether the read owner is the local player, or None when there is none.
pub fn of(owner_is_player: Option<bool>) -> Kind {
    match owner_is_player {
        None => Kind::Unknown,
        Some(true) => Kind::Own,
        Some(false) => Kind::Enemy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn of_maps_the_owner_read() {
        assert_eq!(of(None), Kind::Unknown);
        assert_eq!(of(Some(true)), Kind::Own);
        assert_eq!(of(Some(false)), Kind::Enemy);
    }
}
