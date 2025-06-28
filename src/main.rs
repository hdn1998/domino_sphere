use macroquad::prelude::*;

struct Ball {
    position: Vec2,
    velocity: Vec2,
    radius: f32,
    color: Color,
}

impl Ball {
    fn new(x: f32, y: f32, radius: f32, color: Color) -> Self {
        Ball {
            position: Vec2::new(x, y),
            velocity: Vec2::ZERO,
            radius,
            color,
        }
    }

    fn update(&mut self, dt: f32) {
        self.position += self.velocity * dt;

        // 边界检测
        if self.position.x - self.radius < 0.0 {
            self.position.x = self.radius;
            self.velocity.x = -self.velocity.x * 0.8;
        } else if self.position.x + self.radius > screen_width() {
            self.position.x = screen_width() - self.radius;
            self.velocity.x = -self.velocity.x * 0.8;
        }

        if self.position.y - self.radius < 0.0 {
            self.position.y = self.radius;
            self.velocity.y = -self.velocity.y * 0.8;
        } else if self.position.y + self.radius > screen_height() {
            self.position.y = screen_height() - self.radius;
            self.velocity.y = -self.velocity.y * 0.8;
        }

        // 摩擦力
        self.velocity *= 0.99;
    }

    fn draw(&self) {
        draw_circle(
            self.position.x,
            self.position.y,
            self.radius,
            self.color
        );
    }

    fn check_collision(&mut self, other: &mut Ball, collision_count: &mut u32) {
        let dx = other.position.x - self.position.x;
        let dy = other.position.y - self.position.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < self.radius + other.radius {
            // 增加碰撞次数
            *collision_count += 1;

            // 计算碰撞后的速度变化
            let nx = dx / distance;
            let ny = dy / distance;

            let delta = self.radius + other.radius - distance;

            // 分离球体
            self.position.x -= nx * delta * 0.5;
            self.position.y -= ny * delta * 0.5;
            other.position.x += nx * delta * 0.5;
            other.position.y += ny * delta * 0.5;

            // 计算相对速度
            let dvx = other.velocity.x - self.velocity.x;
            let dvy = other.velocity.y - self.velocity.y;

            // 计算相对速度在碰撞法线方向的分量
            let vn = dvx * nx + dvy * ny;

            // 如果球正在分离，则不进行碰撞处理
            if vn > 0.0 {
                return;
            }

            // 计算冲量
            let impulse = -(1.0 + 0.8) * vn / (1.0/1.0 + 1.0/1.0);

            // 应用冲量
            self.velocity.x -= impulse * nx / 1.0;
            self.velocity.y -= impulse * ny / 1.0;
            other.velocity.x += impulse * nx / 1.0;
            other.velocity.y += impulse * ny / 1.0;
        }
    }
}

#[macroquad::main("Ball Domino Effect")]
async fn main() {
    let mut balls = Vec::new();
    let mut collision_count: u32 = 0;

    // 新增：跟踪鼠标拖动状态
    let mut is_dragging = false;
    let mut drag_start_pos = Vec2::ZERO;

    // 创建多米诺骨牌式排列的球
    const BALL_RADIUS: f32 = 20.0;
    const ROWS: i32 = 5;
    const COLS: i32 = 10;
    const SPACING: f32 = BALL_RADIUS * 2.5;

    for row in 0..ROWS {
        for col in 0..COLS {
            let x = 100.0 + col as f32 * SPACING + (row % 2) as f32 * SPACING * 0.5;
            let y = 100.0 + row as f32 * SPACING;

            let hue = (row * COLS + col) as f32 / (ROWS * COLS) as f32;
            let color = hsl_to_rgb(hue, 0.8, 0.6);

            balls.push(Ball::new(x, y, BALL_RADIUS, color));
        }
    }

    if let Some(first_ball) = balls.first_mut() {
        first_ball.velocity.x = 200.0;
    }

    loop {
        clear_background(BLACK);

        let dt = get_frame_time();

        // 更新所有球的位置
        for ball in &mut balls {
            ball.update(dt);
        }

        // 检测球之间的碰撞
        for i in 0..balls.len() {
            for j in (i+1)..balls.len() {
                let (left, right) = balls.split_at_mut(j);
                left[i].check_collision(&mut right[0], &mut collision_count);
            }
        }

        // 绘制所有球
        for ball in &balls {
            ball.draw();
        }

        // 改进的鼠标交互逻辑
        if is_mouse_button_pressed(MouseButton::Left) {
            is_dragging = true;
            drag_start_pos = Vec2::new(mouse_position().0, mouse_position().1);
        }

        if is_dragging && is_mouse_button_released(MouseButton::Left) {
            is_dragging = false;
            let drag_end_pos = Vec2::new(mouse_position().0, mouse_position().1);

            // 创建新球并根据拖动距离设置速度
            let mut new_ball = Ball::new(
                drag_start_pos.x,
                drag_start_pos.y,
                BALL_RADIUS,
                hsl_to_rgb(get_time() as f32 * 0.1 % 1.0, 0.8, 0.6)
            );

            // 计算速度向量（拖动方向和距离）
            new_ball.velocity = (drag_end_pos - drag_start_pos) * 5.0;

            balls.push(new_ball);
        }

        // 在屏幕上绘制碰撞次数
        draw_text(
            &format!("Collision Count: {}", collision_count),
            10.0,
            30.0,
            30.0,
            WHITE
        );

        // 显示拖动预览线
        if is_dragging {
            let current_pos = Vec2::new(mouse_position().0, mouse_position().1);
            draw_line(
                drag_start_pos.x, drag_start_pos.y,
                current_pos.x, current_pos.y,
                2.0, YELLOW
            );
        }

        next_frame().await
    }
}

// HSL到RGB的转换函数
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 1.0/6.0 {
        (c, x, 0.0)
    } else if h < 2.0/6.0 {
        (x, c, 0.0)
    } else if h < 3.0/6.0 {
        (0.0, c, x)
    } else if h < 4.0/6.0 {
        (0.0, x, c)
    } else if h < 5.0/6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Color::new(r + m, g + m, b + m, 1.0)
}
