//! 1:1 port of me.truewulf.box.mixin.StuckArrowMixin and
//! me.truewulf.box.mixin.StuckStingerMixin: both are the same RETURN hook on
//! the layer's numStuck count.

/// java: numStuck(CallbackInfoReturnable<Integer>) — when hideArrow is on the
/// count is 0, otherwise the original's count.
pub fn num_stuck(original: i32, hide_arrow: bool) -> i32 {
    if hide_arrow {
        0
    } else {
        original
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hide_arrow_zeroes_the_count() {
        assert_eq!(num_stuck(3, true), 0);
        assert_eq!(num_stuck(3, false), 3);
    }
}
