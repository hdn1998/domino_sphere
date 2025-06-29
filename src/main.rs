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
            draw_circle(self.position.x, self.position.y, self.radius, self.color);
        }
    }

    fn check_collision(
        &mut self,
        other: &mut Ball,
        color_collision_counts: &mut [u32; 10],
        color_counts: &[u32; 10],
    ) {
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

            // 新增：如果两球半径差为10倍，小球直接标记为不活跃
            if self.radius >= other.radius * 10.0 {
                other.active = false;
                return;
            } else if other.radius >= self.radius * 10.0 {
                self.active = false;
                return;
            }

            // 计算相同颜色的球的数量
            let self_color_count = color_counts[self_color_index] as f32;
            let other_color_count = color_counts[other_color_index] as f32;

            // 计算大小变化倍率（基于颜色数量的影响）
            let scale_factor = 0.5 * (self_color_count.min(other_color_count));

            // 计算原始面积
            let self_area_before = self.radius * self.radius;
            let other_area_before = other.radius * other.radius;

            // 调整球体大小（使用面积守恒）
            let area_transfer = scale_factor * scale_factor; // 使用面积单位而非半径单位

            // 确保大球增加的面积等于小球减少的面积
            if self.radius > other.radius {
                self.radius = (self_area_before + area_transfer).sqrt();
                other.radius = (other_area_before - area_transfer).sqrt();
            } else {
                self.radius = (self_area_before - area_transfer).sqrt();
                other.radius = (other_area_before + area_transfer).sqrt();
            }

            // 确保撞击球增加的面积等与被撞击球减少的面积
            // self.radius = (self_area_before + area_transfer).sqrt();
            // other.radius = (other_area_before - area_transfer).sqrt();

            // 如果小球半径变得非常小，直接设为不活跃
            if other.radius < 5.0 {
                other.active = false;
            }
            if self.radius < 5.0 {
                self.active = false;
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
            let impulse = -(1.0 + 0.8) * vn / (1.0 / 1.0 + 1.0 / 1.0);

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

    // 拖动时预览线的颜色
    let mut drag_preview_color = YELLOW;
    // 自动发射预览线的颜色
    let mut auto_fire_preview_color = BLUE;

    // 自动发射相关变量
    let mut auto_fire_enabled = true; // 是否启用自动发射
    let mut auto_fire_timer = 0.0; // 自动发射计时器
    const AUTO_FIRE_INTERVAL: f32 = 2.0; // 自动发射间隔（秒）
    const AUTO_FIRE_SPEED_MULTIPLIER: f32 = 5.0; // 自动发射速度乘数

    // 保存上一次发射的球的位置和ID
    let mut last_launched_ball_id: Option<usize> = None;
    let mut last_launched_ball_position = Vec2::new(screen_width() / 2.0, screen_height() - 100.0);

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
        last_launched_ball_id = Some(0); // 标记第一个球为最后发射的球
    }

    loop {
        clear_background(BLACK);

        let dt = get_frame_time();

        // 更新上一次发射的球的位置
        if let Some(id) = last_launched_ball_id {
            if id < balls.len() && balls[id].active {
                last_launched_ball_position = balls[id].position;
            }
        }

        // 更新自动发射计时器
        if auto_fire_enabled {
            auto_fire_timer += dt;
            if auto_fire_timer >= AUTO_FIRE_INTERVAL {
                auto_fire_timer = 0.0;

                // 找出距离最远的活跃球
                let mut farthest_distance = 0.0;
                let mut farthest_ball_position = Vec2::new(0.0, 0.0);
                let mut has_active_balls = false;

                for ball in &balls {
                    if ball.active {
                        has_active_balls = true;
                        let distance = Vec2::distance(ball.position, last_launched_ball_position);
                        if distance > farthest_distance {
                            farthest_distance = distance;
                            farthest_ball_position = ball.position;
                        }
                    }
                }

                // 如果有活跃的球，就向最远的球发射
                if has_active_balls {
                    // 找出碰撞次数最少的颜色
                    let min_count = *color_collision_counts.iter().min().unwrap_or(&0);

                    let mut min_indices = Vec::new();
                    for (i, &count) in color_collision_counts.iter().enumerate() {
                        if count == min_count {
                            min_indices.push(i);
                        }
                    }

                    let random_index = min_indices[rand::gen_range(0, min_indices.len())];
                    let hue = random_index as f32 / 10.0;
                    let color = hsl_to_rgb(hue, 0.8, 0.6);
                    auto_fire_preview_color = color; // 更新自动发射预览颜色

                    // 创建新球，位置为上一次发射的球的当前位置
                    let mut new_ball = Ball::new(
                        last_launched_ball_position.x,
                        last_launched_ball_position.y,
                        BALL_RADIUS,
                        color,
                        hue,
                    );

                    // 计算到最远球的方向和速度
                    let direction = farthest_ball_position - last_launched_ball_position;
                    let distance = direction.length();
                    let velocity = if distance > 0.0 {
                        direction.normalize() * distance * AUTO_FIRE_SPEED_MULTIPLIER
                    } else {
                        Vec2::new(0.0, -500.0) // 默认向上发射
                    };

                    new_ball.velocity = velocity;

                    // 保存新球的ID
                    let new_ball_id = balls.len();

                    // 保存位置
                    let position = new_ball.position;

                    // 添加新球到向量
                    balls.push(new_ball);

                    // 更新最后发射的球的ID和位置
                    last_launched_ball_id = Some(new_ball_id);
                    last_launched_ball_position = position;
                }
            }
        }

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
            for j in (i + 1)..balls.len() {
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

            // 计算拖动预览线的颜色
            let hue: f32;
            if !first_ball_created {
                hue = get_time() as f32 * 0.1 % 1.0;
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

            drag_preview_color = hsl_to_rgb(hue, 0.8, 0.6);
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

            let mut new_ball =
                Ball::new(drag_start_pos.x, drag_start_pos.y, BALL_RADIUS, color, hue);

            new_ball.velocity = (drag_end_pos - drag_start_pos) * 5.0;

            // 保存新球的ID
            let new_ball_id = balls.len();

            // 保存位置
            let position = new_ball.position;

            // 添加新球到向量
            balls.push(new_ball);

            // 更新最后发射的球的ID和位置
            last_launched_ball_id = Some(new_ball_id);
            last_launched_ball_position = position;
        }

        // 显示拖动预览线（使用计算好的颜色）
        if is_dragging {
            let current_pos = Vec2::new(mouse_position().0, mouse_position().1);
            draw_line(
                drag_start_pos.x,
                drag_start_pos.y,
                current_pos.x,
                current_pos.y,
                2.0,
                drag_preview_color,
            );
        }

        // 显示自动发射状态
        let status_text = if auto_fire_enabled {
            "Auto Fire: ON"
        } else {
            "Auto Fire: OFF"
        };
        draw_text(status_text, 10.0, screen_height() - 30.0, 24.0, GREEN);

        // 显示下一次自动发射倒计时
        if auto_fire_enabled {
            let countdown = AUTO_FIRE_INTERVAL - auto_fire_timer;
            draw_text(
                &format!("Next Launch: {:.1}s", countdown),
                10.0,
                screen_height() - 60.0,
                24.0,
                GREEN,
            );
        }

        // 显示发射轨迹预览（如果有活跃球）
        if auto_fire_enabled {
            let mut farthest_distance = 0.0;
            let mut farthest_ball_position = Vec2::new(0.0, 0.0);
            let mut has_active_balls = false;

            for ball in &balls {
                if ball.active {
                    has_active_balls = true;
                    let distance = Vec2::distance(ball.position, last_launched_ball_position);
                    if distance > farthest_distance {
                        farthest_distance = distance;
                        farthest_ball_position = ball.position;
                    }
                }
            }

            if has_active_balls {
                draw_circle(
                    last_launched_ball_position.x,
                    last_launched_ball_position.y,
                    BALL_RADIUS,
                    auto_fire_preview_color,
                );
                draw_line(
                    last_launched_ball_position.x,
                    last_launched_ball_position.y,
                    farthest_ball_position.x,
                    farthest_ball_position.y,
                    2.0,
                    auto_fire_preview_color,
                );
            }
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
                color,
            );
        }

        // 移除不活跃的球
        balls.retain(|ball| ball.active);

        // 按空格键切换自动发射
        if is_key_pressed(KeyCode::Space) {
            auto_fire_enabled = !auto_fire_enabled;
            auto_fire_timer = 0.0; // 重置计时器
        }

        next_frame().await
    }
}

// HSL到RGB的转换函数
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 1.0 / 6.0 {
        (c, x, 0.0)
    } else if h < 2.0 / 6.0 {
        (x, c, 0.0)
    } else if h < 3.0 / 6.0 {
        (0.0, c, x)
    } else if h < 4.0 / 6.0 {
        (0.0, x, c)
    } else if h < 5.0 / 6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Color::new(r + m, g + m, b + m, 1.0)
}
