use crate::{
    constants::get_quad_color_from_texture_id,
    hud::Hud,
    nadk::display::{ScreenRect, push_rect, wait_for_vblank},
    player::Player,
    renderer::{
        frustum::Frustum,
        mesh::{Quad, Triangle, Triangle2D},
        *,
    },
    world::World,
};

/// Fill a triangle in the frame buffer
#[inline]
pub fn fill_triangle(
    mut t0: Vector2<isize>,
    mut t1: Vector2<isize>,
    mut t2: Vector2<isize>,
    frame_buffer: &mut [Color565; SCREEN_TILE_WIDTH * SCREEN_TILE_HEIGHT],
    color: Color565,
) {
    if t0.y > t1.y {
        swap(&mut t0, &mut t1);
    }
    if t0.y > t2.y {
        swap(&mut t0, &mut t2);
    }
    if t1.y > t2.y {
        swap(&mut t1, &mut t2);
    }

    let triangle_height = t2.y - t0.y;
    if triangle_height == 0 {
        return;
    }
    let inv_triangle_height = 1.0 / triangle_height as f32;

    let mid_height = t1.y - t0.y;
    let inv_seg_first = if mid_height > 0 { 1.0 / mid_height as f32 } else { 0.0 };
    let inv_seg_second = if (t2.y - t1.y) > 0 { 1.0 / (t2.y - t1.y) as f32 } else { 0.0 };

    let dx20 = (t2.x - t0.x) as f32;
    let dx10 = (t1.x - t0.x) as f32;
    let dx21 = (t2.x - t1.x) as f32;

    // Clamp y range to tile bounds
    let y_start = t0.y.max(0);
    let y_end = t2.y.min(SCREEN_TILE_HEIGHT as isize - 1);

    for y in y_start..=y_end {
        let i = y - t0.y;
        let second_half = i > mid_height || mid_height == 0;

        let alpha = i as f32 * inv_triangle_height;
        let a = t0.x as f32 + dx20 * alpha;

        let b = if second_half {
            let beta = (i - mid_height) as f32 * inv_seg_second;
            t1.x as f32 + dx21 * beta
        } else {
            let beta = i as f32 * inv_seg_first;
            t0.x as f32 + dx10 * beta
        };

        let (x_start, x_end) = if a <= b {
            (a as isize, b as isize)
        } else {
            (b as isize, a as isize)
        };

        let x_start = x_start.max(0) as usize;
        let x_end = (x_end as usize).min(SCREEN_TILE_WIDTH - 1);

        if x_start > x_end {
            continue;
        }

        let row_offset = y as usize * SCREEN_TILE_WIDTH;
        frame_buffer[row_offset + x_start..=row_offset + x_end].fill(color);
    }
}

// Draw a line in the frame buffer
pub fn draw_line(
    pos1: (isize, isize),
    pos2: (isize, isize),
    frame_buffer: &mut [Color565; SCREEN_TILE_WIDTH * SCREEN_TILE_HEIGHT],
    color: Color565,
) {
    for point in bresenham::Bresenham::new(pos1, pos2) {
        if point.0 >= 0
            && point.0 < SCREEN_TILE_WIDTH as isize
            && point.1 >= 0
            && point.1 < SCREEN_TILE_HEIGHT as isize
        {
            frame_buffer[(point.0 + point.1 * SCREEN_TILE_WIDTH as isize) as usize] = color;
        }
    }
}

// Takes a Triangle2D and draw it as a filled triangle or lines depending of the texture_id
pub fn draw_2d_triangle(
    tri: &Triangle2D,
    frame_buffer: &mut [Color565; SCREEN_TILE_WIDTH * SCREEN_TILE_HEIGHT],
) {
    if tri.texture_id == 255 {
        // Block marker
        draw_line(
            (tri.p1.x as isize, tri.p1.y as isize),
            (tri.p2.x as isize, tri.p2.y as isize),
            frame_buffer,
            Color565::new(0b11111, 0b0, 0b0),
        );
        draw_line(
            (tri.p2.x as isize, tri.p2.y as isize),
            (tri.p3.x as isize, tri.p3.y as isize),
            frame_buffer,
            Color565::new(0b11111, 0b0, 0b0),
        );
    } else {
        // Normal Triangle
        fill_triangle(
            Vector2::new(tri.p1.x as isize, tri.p1.y as isize),
            Vector2::new(tri.p2.x as isize, tri.p2.y as isize),
            Vector2::new(tri.p3.x as isize, tri.p3.y as isize),
            frame_buffer,
            get_quad_color_from_texture_id(tri.texture_id).apply_light(tri.light * 17),
        );
    }
}

pub fn matrix_point_at(
    pos: &Vector3<f32>,
    target: &Vector3<f32>,
    up: &Vector3<f32>,
) -> Matrix4<f32> {
    let new_forward = (target - pos).normalize();

    let new_up = (up - new_forward * up.dot(&new_forward)).normalize();
    let new_right = new_up.cross(&new_forward);

    Matrix4::new(
        new_right.x,
        new_up.x,
        new_forward.x,
        pos.x,
        new_right.y,
        new_up.y,
        new_forward.y,
        pos.y,
        new_right.z,
        new_up.z,
        new_forward.z,
        pos.z,
        0.0,
        0.0,
        0.0,
        1.0,
    )
}

pub fn vector_intersect_plane(
    plane_p: &Vector3<f32>,
    plane_n: &Vector3<f32>,
    line_start: &Vector3<f32>,
    line_end: &Vector3<f32>,
) -> Vector3<f32> {
    let plane_n = plane_n.normalize();
    let plane_d = -plane_n.dot(plane_p);
    let ad = line_start.dot(&plane_n);
    let bd = line_end.dot(&plane_n);
    let t = (-plane_d - ad) / (bd - ad);
    let line_start_to_end = line_end - line_start;
    let line_to_intersect = line_start_to_end * t;
    line_start + line_to_intersect
}

pub fn vector_intersect_line(
    line_p: &Vector2<f32>,
    line_n: &Vector2<f32>,
    line_start: &Vector2<f32>,
    line_end: &Vector2<f32>,
) -> Vector2<i16> {
    let line_n = line_n.normalize();
    let line_d = -line_n.dot(line_p);
    let ad = line_start.dot(&line_n);
    let bd = line_end.dot(&line_n);
    let t = (-line_d - ad) / (bd - ad);
    let line_start_to_end = line_end - line_start;
    let line_to_intersect = line_start_to_end * t;
    let coords = line_start + line_to_intersect;
    coords.map(|x| x as i16)
}

pub fn triangle_clip_against_line(
    line_p: &Vector2<f32>,
    line_n: &Vector2<f32>,
    in_tri: &Triangle2D,
) -> (Option<Triangle2D>, Option<Triangle2D>) {
    let line_n = line_n.normalize();

    let dist = |p: Vector2<f32>| line_n.x * p.x + line_n.y * p.y - line_n.dot(line_p);

    let p1 = in_tri.p1.map(|x| x as f32);
    let p2 = in_tri.p2.map(|x| x as f32);
    let p3 = in_tri.p3.map(|x| x as f32);

    let d0 = dist(p1);
    let d1 = dist(p2);
    let d2 = dist(p3);

    let pts = [(&p1, d0), (&p2, d1), (&p3, d2)];
    let mut inside: [&Vector2<f32>; 3] = [&p1; 3];
    let mut outside: [&Vector2<f32>; 3] = [&p1; 3];
    let mut ni = 0usize;
    let mut no = 0usize;
    for (p, d) in &pts {
        if *d >= 0.0 { inside[ni] = p; ni += 1; }
        else         { outside[no] = p; no += 1; }
    }

    if ni == 0 { return (None, None); }
    if ni == 3 { return (Some(*in_tri), None); }

    if ni == 1 && no == 2 {
        return (Some(Triangle2D {
            p1: inside[0].map(|x| x as i16),
            p2: vector_intersect_line(line_p, &line_n, inside[0], outside[0]),
            p3: vector_intersect_line(line_p, &line_n, inside[0], outside[1]),
            texture_id: in_tri.texture_id,
            light: in_tri.light,
        }), None);
    }

    // ni == 2 && no == 1
    let t1 = Triangle2D {
        p1: inside[0].map(|x| x as i16),
        p2: inside[1].map(|x| x as i16),
        p3: vector_intersect_line(line_p, &line_n, inside[0], outside[0]),
        texture_id: in_tri.texture_id,
        light: in_tri.light,
    };
    let t2 = Triangle2D {
        p1: inside[1].map(|x| x as i16),
        p2: t1.p3,
        p3: vector_intersect_line(line_p, &line_n, inside[1], outside[0]),
        texture_id: in_tri.texture_id,
        light: in_tri.light,
    };
    (Some(t1), Some(t2))
}

pub fn triangle_clip_against_plane(
    plane_p: &Vector3<f32>,
    plane_n: &Vector3<f32>,
    in_tri: &Triangle,
) -> (Option<Triangle>, Option<Triangle>) {
    let plane_n = plane_n.normalize();

    let dist =
        |p: Vector3<f32>| plane_n.x * p.x + plane_n.y * p.y + plane_n.z * p.z - plane_n.dot(plane_p);

    let d0 = dist(in_tri.p1);
    let d1 = dist(in_tri.p2);
    let d2 = dist(in_tri.p3);

    let pts = [(&in_tri.p1, d0), (&in_tri.p2, d1), (&in_tri.p3, d2)];
    let mut inside: [&Vector3<f32>; 3] = [&in_tri.p1; 3];
    let mut outside: [&Vector3<f32>; 3] = [&in_tri.p1; 3];
    let mut ni = 0usize;
    let mut no = 0usize;
    for (p, d) in &pts {
        if *d >= 0.0 { inside[ni] = p; ni += 1; }
        else         { outside[no] = p; no += 1; }
    }

    if ni == 0 { return (None, None); }
    if ni == 3 { return (Some(*in_tri), None); }

    if ni == 1 && no == 2 {
        return (Some(Triangle {
            p1: *inside[0],
            p2: vector_intersect_plane(plane_p, &plane_n, inside[0], outside[0]),
            p3: vector_intersect_plane(plane_p, &plane_n, inside[0], outside[1]),
            texture_id: in_tri.texture_id,
            light: in_tri.light,
        }), None);
    }

    // ni == 2 && no == 1
    let t1 = Triangle {
        p1: *inside[0],
        p2: *inside[1],
        p3: vector_intersect_plane(plane_p, &plane_n, inside[0], outside[0]),
        texture_id: in_tri.texture_id,
        light: in_tri.light,
    };
    let t2 = Triangle {
        p1: *inside[1],
        p2: t1.p3,
        p3: vector_intersect_plane(plane_p, &plane_n, inside[1], outside[0]),
        texture_id: in_tri.texture_id,
        light: in_tri.light,
    };
    (Some(t1), Some(t2))
}

impl Renderer {
    pub fn update_fov(&mut self, new_fov: f32) {
        self.camera.set_fov(new_fov);
        self.projection_matrix =
            Perspective3::new(ASPECT_RATIO, self.camera.get_fov(), ZNEAR, ZFAR);
    }

    pub fn project_point(&self, point: Vector3<f32>) -> Vector2<f32> {
        self.projection_matrix.project_vector(&point).xy()
    }

    pub fn clear_screen(&mut self, color: Color565) {
        self.tile_frame_buffer.fill(color);
    }

    fn get_mat_view(&self) -> Matrix4<f32> {
        let up: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);
        let target: Vector3<f32> = Vector3::new(0.0, 0.0, 1.0);
        let look_dir = self.camera.get_rotation_matrix() * target.to_homogeneous();
        let target = self.camera.get_pos() + look_dir.xyz();

        let mat_camera = matrix_point_at(self.camera.get_pos(), &target, &up);

        mat_camera.try_inverse().unwrap()
    }

    fn add_3d_triangle_to_render(&mut self, tri: Triangle, mat_view: &Matrix4<f32>) {
        let mut tri = tri;

        let camera_ray = tri.p1 - self.camera.get_pos();

        if tri.get_normal().dot(&camera_ray) < 0.0 {
            tri.p1 = (mat_view * Vector4::new(tri.p1.x, tri.p1.y, tri.p1.z, 1.0)).xyz(); // try to_homogenous here
            tri.p2 = (mat_view * Vector4::new(tri.p2.x, tri.p2.y, tri.p2.z, 1.0)).xyz();
            tri.p3 = (mat_view * Vector4::new(tri.p3.x, tri.p3.y, tri.p3.z, 1.0)).xyz();

            let clipped_triangles: (Option<Triangle>, Option<Triangle>) = if tri.texture_id != 255 {
                triangle_clip_against_plane(
                    &Vector3::new(0.0, 0.0, 0.1),
                    &Vector3::new(0.0, 0.0, 1.0),
                    &tri,
                )
            } else {
                (Some(tri), None)
            };

            let mut project_and_add = |to_project: Triangle| {
                let projected_triangle = Triangle2D {
                    p1: ((self.project_point(to_project.p1) + Vector2::new(1., 1.))
                        .component_mul(&HALF_SCREEN))
                    .map(|x| x as i16),
                    p2: ((self.project_point(to_project.p2) + Vector2::new(1., 1.))
                        .component_mul(&HALF_SCREEN))
                    .map(|x| x as i16),
                    p3: ((self.project_point(to_project.p3) + Vector2::new(1., 1.))
                        .component_mul(&HALF_SCREEN))
                    .map(|x| x as i16),
                    texture_id: to_project.texture_id,
                    light: to_project.light,
                };

                let mut clip_buffer: heapless::Deque<Triangle2D, 16> = heapless::Deque::new(); // 2^4

                clip_buffer.push_back(projected_triangle).unwrap();
                let mut new_tris = 1;

                let mut clip_triangle = |line_p, line_n| {
                    while new_tris > 0 {
                        let test = clip_buffer.pop_front().unwrap();
                        new_tris -= 1;

                        let clipped = triangle_clip_against_line(&line_p, &line_n, &test);

                        if let Some(clipped_tri) = clipped.0 {
                            clip_buffer.push_back(clipped_tri).unwrap();
                        }
                        if let Some(clipped_tri) = clipped.1 {
                            clip_buffer.push_back(clipped_tri).unwrap();
                        }
                    }
                    new_tris = clip_buffer.len();
                };

                if tri.texture_id != 255 {
                    clip_triangle(Vector2::new(0.0, 0.0), Vector2::new(0.0, 1.0));
                    clip_triangle(Vector2::new(0.0, SCREEN_HEIGHTF), Vector2::new(0.0, -1.0));
                    clip_triangle(Vector2::new(0.0, 0.0), Vector2::new(1.0, 0.0));
                    clip_triangle(
                        Vector2::new(SCREEN_WIDTHF - 1.0, 0.0),
                        Vector2::new(-1.0, 0.0),
                    );
                }

                for tri in clip_buffer {
                    if self.triangles_to_render.len() >= MAX_TRIANGLES {
                        // TODO : Find a proper fix for this
                        break;
                    }
                    self.triangles_to_render.push(tri.to_small()); // Do nothing if overflow
                }
            };

            if let Some(clipped) = clipped_triangles.0 {
                project_and_add(clipped)
            }
            if let Some(clipped) = clipped_triangles.1 {
                project_and_add(clipped)
            }
        }
    }

    fn draw_triangles(&mut self, tile_x: usize, tile_y: usize) {
        let tile_offset = Vector2::new(
            -((SCREEN_TILE_WIDTH * tile_x) as i16),
            -((SCREEN_TILE_HEIGHT * tile_y) as i16),
        );
        for tri in self.triangles_to_render.iter_mut().rev() {
            let mut tri_copy = tri.to_tri_2d();
            tri_copy.p1 += tile_offset;

            tri_copy.p2 += tile_offset;

            tri_copy.p3 += tile_offset;

            draw_2d_triangle(&tri_copy, &mut self.tile_frame_buffer);
        }
    }

    fn add_quad_to_render(
        &mut self,
        quad: &Quad,
        mat_view: &Matrix4<f32>,
        chunk_block_pos: Vector3<isize>,
    ) {
        let quad_triangles = quad.get_triangles(chunk_block_pos);
        self.add_3d_triangle_to_render(quad_triangles.0, mat_view);
        self.add_3d_triangle_to_render(quad_triangles.1, mat_view);
    }

    pub fn draw_game(
        &mut self,
        world: &mut World,
        player: &Player,
        frame_time: u64,
        hud: &Hud,
        draw_hud: bool,
    ) {
        let mat_view = self.get_mat_view();

        let frustum = Frustum::new(
            &self.camera,
            ASPECT_RATIO,
            self.camera.get_fov(),
            ZNEAR,
            ZFAR,
        );

        // Add the player block marker
        let mut block_marker = player.get_block_marker();
        for quad in block_marker.0.get_reference_vec() {
            self.add_quad_to_render(quad, &mat_view, block_marker.1);
        }

        for chunk in world
            .chunks_manager
            .get_chunks_sorted_by_distance(*self.camera.get_pos())
        {
            let chunk_blocks_pos = chunk.get_pos() * CHUNK_SIZE_I;
            let chunk_blocks_posf = chunk_blocks_pos.map(|x| x as f32);
            let chunk_blocks_pos_maxf =
                (chunk_blocks_pos + Vector3::repeat(CHUNK_SIZE_I)).map(|x| x as f32);

            if !(frustum.is_aabb_in_frustum(chunk_blocks_posf, chunk_blocks_pos_maxf)) {
                continue;
            }

            let need_sorting = chunk.need_sorting || self.camera.get_has_moved();

            let quads = chunk.get_mesh().get_reference_vec();

            if need_sorting {
                let cam = *self.camera.get_pos();
                quads.sort_by(|a, b| -> Ordering {
                    let a_pos = a.get_pos().map(|x| x as isize) + chunk_blocks_pos;
                    let b_pos = b.get_pos().map(|x| x as isize) + chunk_blocks_pos;
                    let avec = Vector3::new(a_pos.x as f32 + 0.5, a_pos.y as f32 + 0.5, a_pos.z as f32 + 0.5);
                    let bvec = Vector3::new(b_pos.x as f32 + 0.5, b_pos.y as f32 + 0.5, b_pos.z as f32 + 0.5);
                    (avec - cam).norm_squared()
                        .total_cmp(&(bvec - cam).norm_squared())
                });
            }
            for quad in quads {
                self.add_quad_to_render(quad, &mat_view, chunk_blocks_pos);
            }
        }

        for x in 0..SCREEN_TILE_SUBDIVISION {
            for y in 0..SCREEN_TILE_SUBDIVISION {
                self.clear_screen(Color565::new(0b01110, 0b110110, 0b11111));
                self.draw_triangles(x, y);
                self.draw_flat_model_entities(world, &mat_view, x, y, &frustum);

                if draw_hud {
                    self.draw_hud(hud, frame_time, x, y);
                }

                push_rect(
                    ScreenRect {
                        x: (SCREEN_TILE_WIDTH * x) as u16,
                        y: (SCREEN_TILE_HEIGHT * y) as u16,
                        width: SCREEN_TILE_WIDTH as u16,
                        height: SCREEN_TILE_HEIGHT as u16,
                    },
                    &self.tile_frame_buffer,
                );
            }
        }
        if self.enable_vsync {
            wait_for_vblank();
        }
        self.triangles_to_render.clear();
    }
}
