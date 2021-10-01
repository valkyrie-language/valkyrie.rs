use super::*;

pub(super) fn plan_frame_layouts(function: &mut MirFunction) {
    function.frame_layouts = function
        .suspend_points
        .iter()
        .map(|suspend_point| MirFrameLayout {
            state_id: suspend_point.state_id,
            effect: suspend_point.effect,
            resume_target: suspend_point.resume_target,
            slots: suspend_point
                .spill_candidates
                .iter()
                .enumerate()
                .map(|(slot_index, value)| MirFrameSlot { slot_index, value: *value, value_type: function.value_types.get(value).cloned() })
                .collect(),
        })
        .collect();
}
