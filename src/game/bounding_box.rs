use threed::Vector;
use winit::dpi::PhysicalSize;

///Bounding Box defined by middle point and width and height
///The negative sides (anchor - size/2) and the positive sides (anchor + size/2) are inclusive
#[derive(Debug)]
pub struct BoundingBox {
    ///Middle point
    pub anchor: Vector<f32>,
    pub size: PhysicalSize<f32>,
}
impl BoundingBox {
    fn contains_point(&self, point: &Vector<f32>) -> bool {
        let offset = point - &self.anchor;
        let width = self.size.width / 2.0;
        let height = self.size.height / 2.0;
        offset.x >= -width && offset.x <= width && offset.y >= -height && offset.y <= height
    }

    fn contains_box(&self, other: &BoundingBox) -> bool {
        let offset = Vector::new(other.size.width, other.size.height, 0.0) / 2.0;
        let top_left = &other.anchor - &offset;
        let bottom_right = &other.anchor + &offset;
        self.contains_point(&top_left) && self.contains_point(&bottom_right)
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        let s_width = self.size.width / 2.0;
        let s_height = self.size.height / 2.0;
        let o_width = other.size.width / 2.0;
        let o_height = other.size.height / 2.0;
        self.anchor.x - s_width < other.anchor.x + o_width
            && self.anchor.x + s_width > other.anchor.x - o_width
            && self.anchor.y - s_height < other.anchor.y + o_height
            && self.anchor.y + s_height > other.anchor.y - o_height
    }

    ///Returns the nearest position for the other box to be inside self
    ///If a axis of other is bigger than self, self.anchor's value will be returned
    ///If other is already in self, None will be returned
    pub fn clamp_box_inside(&self, other: &BoundingBox) -> Option<Vector<f32>> {
        if self.contains_box(other) {
            None
        } else {
            let x = if other.size.width < self.size.width {
                let size_difference = (other.size.width - self.size.width) / 2.0;
                let left_distance = self.anchor.x - other.anchor.x;

                other.anchor.x
                    + if left_distance.abs() <= -size_difference {
                        0.0
                    } else if left_distance > 0.0 {
                        left_distance + size_difference
                    } else {
                        left_distance - size_difference
                    }
            } else {
                self.anchor.x
            };
            let y = if other.size.height < self.size.height {
                let size_difference = (other.size.height - self.size.height) / 2.0;
                let top_distance = self.anchor.y - other.anchor.y;

                other.anchor.y
                    + if top_distance.abs() <= -size_difference {
                        0.0
                    } else if top_distance > 0.0 {
                        top_distance + size_difference
                    } else {
                        top_distance - size_difference
                    }
            } else {
                self.anchor.y
            };
            let inside_anchor = Vector::new(x, y, 0.0);
            Some(inside_anchor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_box() {
        let bb = BoundingBox {
            anchor: Vector::new(0.0, 0.0, 0.0),
            size: PhysicalSize::new(800.0, 600.0),
        };
        assert!(bb.contains_point(&Vector::new(0.0, 0.0, 0.0)));
        assert!(bb.contains_point(&Vector::new(-400.0, -300.0, 0.0)));
        assert!(bb.contains_point(&Vector::new(400.0, 300.0, 0.0)));
        assert!(bb.contains_box(&bb));
    }
}
