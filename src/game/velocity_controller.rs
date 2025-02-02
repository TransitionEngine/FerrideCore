use threed::Vector;

pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}
/// 8 directional VelocityController
pub struct VelocityController {
    speed: f32,
    up: bool,
    right: bool,
    down: bool,
    left: bool,
}
impl VelocityController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            up: false,
            right: false,
            down: false,
            left: false,
        }
    }

    pub fn stop_movement(&mut self) {
        self.up = false;
        self.down = false;
        self.left = false;
        self.right = false;
    }

    pub fn set_direction(&mut self, direction: Direction, value: bool) {
        match direction {
            Direction::Up => {
                self.up = value;
            }
            Direction::Right => {
                self.right = value;
            }
            Direction::Down => {
                self.down = value;
            }
            Direction::Left => {
                self.left = value;
            }
        }
    }

    pub fn get_velocity(&self) -> Vector<f32> {
        let mut velocity = Vector::new(0.0, 0.0, 0.0);
        if self.up {
            velocity.y += 1.0;
        }
        if self.right {
            velocity.x += 1.0;
        }
        if self.down {
            velocity.y -= 1.0;
        }
        if self.left {
            velocity.x -= 1.0;
        }
        let magnitude: f32 = velocity.magnitude_squared();
        if magnitude >= 1.0 {
            velocity *= 1.0 / magnitude.sqrt();
        }
        velocity * self.speed
    }
}
