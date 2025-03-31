use cosmic::iced::{advanced::layout::flex::Axis, Length};

use super::{
    axis_toplevel_layout::{AxisPoint, AxisRectangle, AxisSize, AxisToplevelLayout},
    row_col_toplevel_layout::RowColToplevelLayout,
    LayoutToplevel,
};

// TODO should impl `Copy` in iced
fn copy_axis(axis: &Axis) -> Axis {
    match axis {
        Axis::Horizontal => Axis::Horizontal,
        Axis::Vertical => Axis::Vertical,
    }
}

pub(crate) struct TwoRowColToplevelLayout(RowColToplevelLayout);

impl TwoRowColToplevelLayout {
    pub fn new(axis: Axis, spacing: u32) -> Self {
        Self(RowColToplevelLayout::new(axis, spacing))
    }
}

impl AxisToplevelLayout for TwoRowColToplevelLayout {
    fn axis(&self) -> &Axis {
        &self.0.axis
    }

    fn size(&self) -> AxisSize<Length> {
        AxisSize {
            main: Length::Fill,
            cross: Length::Shrink,
        }
    }

    fn layout(
        &self,
        max_limit: AxisSize,
        toplevels: &[LayoutToplevel<'_, AxisSize>],
    ) -> impl Iterator<Item = AxisRectangle> {
        let requested_main_total = self.0.requested_main_total(&toplevels);
        let scale_factor = (max_limit.main / requested_main_total).min(1.0);

        // Add padding to center if total requested size doesn't fill available space
        let padding = (max_limit.main - requested_main_total).max(0.) / 2.;

        // See if two row layout is better
        // TODO not a good fix if there is a large window and many smaller ones?
        if requested_main_total > max_limit.main && toplevels.len() > 1 {
            let max_requested_cross = toplevels
                .iter()
                .map(|t| t.preferred_size.cross)
                .reduce(f32::max)
                .unwrap();
            let cross_scale_factor = (max_limit.cross / 2.) / max_requested_cross;
            // decide best way to partition list
            let (split_point, two_row_scale_factor) = (1..toplevels.len())
                .map(|i| {
                    let (top_row, bottom_row) = toplevels.split_at(i);
                    let top_total = self.0.requested_main_total(top_row);
                    let bottom_total = self.0.requested_main_total(bottom_row);
                    let max = top_total.max(bottom_total);
                    let scale_factor = (max_limit.main / max).min(1.0);
                    (i, scale_factor)
                })
                .max_by(|(_, scale1), (_, scale2)| scale1.total_cmp(scale2))
                .unwrap();
            let two_row_scale_factor = two_row_scale_factor.min(cross_scale_factor);
            // Better layout
            if two_row_scale_factor > scale_factor {
                // TODO padding
                // TODO cross axis spacing
                let half_max_limit = AxisSize {
                    main: max_limit.main,
                    cross: max_limit.cross / 2.,
                };
                let row1 = self.0.layout(half_max_limit, &toplevels[..split_point]);
                let row2 = self.0.layout(half_max_limit, &toplevels[split_point..]).map(move |mut rect| {
                    rect.origin.cross += max_limit.cross / 2.;
                    rect
                });
                return itertools::Either::Left(row1.chain(row2));
            }
        }

        itertools::Either::Right(self.0.layout(max_limit, toplevels))
    }
}
