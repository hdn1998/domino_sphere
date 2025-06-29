use macroquad::prelude::*;

struct Ball {
    position: Vec2,
    velocity: Vec2,
    radius: f32,
    color: Color,
    hue: f32,
    active: bool, // 新增：标记球是否活跃（未消失）
}

impl Ball {
    fn new(x: f32, y: f32, radius: f32, color: Color, hue: f32) -> Self {
        Ball {
            position: Vec2::new(x, y),
            velocity: Vec2::ZERO,
            radius,
            color,
            hue,
            active: true,
        }
    }

    fn update(&mut self, dt: f32) {
        if !self.active {
            return;
        }

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
        if self.active {
            draw_circle(
                self.position.x,
                self.position.y,
                self.radius,
                self.color
            );
        }
    }

    fn check_collision(&mut self, other: &mut Ball, color_collision_counts: &mut [u32; 10], color_counts: &[u32; 10]) {
        if !self.active || !other.active {
            return;
        }

        let dx = other.position.x - self.position.x;
        let dy = other.position.y - self.position.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < self.radius + other.radius {
            // 计算两个球的颜色索引
            let self_color_index = (self.hue * 10.0) as usize;
            let other_color_index = (other.hue * 10.0) as usize;

            // 增加对应颜色的碰撞次数
            color_collision_counts[self_color_index] += 1;
            color_collision_counts[other_color_index] += 1;

            // 计算相同颜色的球的数量
            let self_color_index = (self.hue * 10.0) as usize;
            let other_color_index = (other.hue * 10.0) as usize;
            let self_color_count = color_counts[self_color_index] as f32;
            let other_color_count = color_counts[other_color_index] as f32;

            // 计算大小变化倍率
            let scale_factor = 0.05 * (self_color_count.min(other_color_count));

            // 调整球体大小
            self.radius += scale_factor;
            other.radius -= scale_factor;

            // 检查大小差距是否超过10倍
            let size_ratio = self.radius / other.radius;
            if size_ratio >= 10.0 || size_ratio <= 0.1 {
                // 小球直接消失
                if self.radius < other.radius {
                    self.active = false;
                } else {
                    other.active = false;
                }
            }

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
    let mut color_collision_counts = [0u32; 10];

    let mut is_dragging = false;
    let mut drag_start_pos = Vec2::new(0.0, 0.0);
    let mut first_ball_created = false;

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

            balls.push(Ball::new(x, y, BALL_RADIUS, color, hue));
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
        // 先计算每种颜色的球的数量
        let mut color_counts = [0u32; 10];
        for ball in &balls {
            if ball.active {
                let color_index = (ball.hue * 10.0) as usize;
                color_counts[color_index] += 1;
            }
        }
        
        for i in 0..balls.len() {
            for j in (i+1)..balls.len() {
                let (left, right) = balls.split_at_mut(j);
                left[i].check_collision(&mut right[0], &mut color_collision_counts, &color_counts);
            }
        }

        // 绘制所有球
        for ball in &balls {
            ball.draw();
        }

        // 鼠标交互逻辑
        if is_mouse_button_pressed(MouseButton::Left) {
            is_dragging = true;
            drag_start_pos = Vec2::new(mouse_position().0, mouse_position().1);
        }

        if is_dragging && is_mouse_button_released(MouseButton::Left) {
            is_dragging = false;
            let drag_end_pos = Vec2::new(mouse_position().0, mouse_position().1);

            let hue: f32;

            if !first_ball_created {
                hue = get_time() as f32 * 0.1 % 1.0;
                first_ball_created = true;
            } else {
                // 找出碰撞次数最少的颜色
                let min_count = *color_collision_counts.iter().min().unwrap_or(&0);

                let mut min_indices = Vec::new();
                for (i, &count) in color_collision_counts.iter().enumerate() {
                    if count == min_count {
                        min_indices.push(i);
                    }
                }

                let random_index = min_indices[rand::gen_range(0, min_indices.len())];
                hue = random_index as f32 / 10.0;
            }

            let color = hsl_to_rgb(hue, 0.8, 0.6);

            let mut new_ball = Ball::new(
                drag_start_pos.x,
                drag_start_pos.y,
                BALL_RADIUS,
                color,
                hue
            );

            new_ball.velocity = (drag_end_pos - drag_start_pos) * 5.0;

            balls.push(new_ball);
        }

        // 显示拖动预览线
        if is_dragging {
            let current_pos = Vec2::new(mouse_position().0, mouse_position().1);
            draw_line(
                drag_start_pos.x, drag_start_pos.y,
                current_pos.x, current_pos.y,
                2.0, YELLOW
            );
        }

        // 生成排序后的颜色碰撞次数列表
        let mut sorted_colors: Vec<(usize, u32)> = color_collision_counts
            .iter()
            .enumerate()
            .map(|(i, &count)| (i, count))
            .collect();

        // 按碰撞次数降序排序
        sorted_colors.sort_by_key(|&(_, count)| std::cmp::Reverse(count));

        // 在屏幕上绘制排行榜标题
        draw_text("Collision Leaderboard:", 10.0, 30.0, 24.0, WHITE);

        // 绘制排序后的颜色碰撞次数
        for (rank, &(color_index, count)) in sorted_colors.iter().enumerate() {
            let color = hsl_to_rgb(color_index as f32 / 10.0, 0.8, 0.6);
            draw_text(
                &format!("{}. {:.1}: {}", rank + 1, color_index as f32 / 10.0, count),
                10.0,
                60.0 + rank as f32 * 30.0,
                20.0,
                color
            );
        }

        // 移除不活跃的球
        balls.retain(|ball| ball.active);

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
