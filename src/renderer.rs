use crate::rasterizer::Framebuffer;
use crate::vector::Vector3;
use crate::shaders::{PlanetShader, ShaderUniforms};
use nalgebra::{Matrix4, Vector4};

pub const WIDTH: usize = 1024;
pub const HEIGHT: usize = 768;

// Estructura para estrellas de fondo
pub struct Star {
    pub x: f32,
    pub y: f32,
    pub brightness: f32,
}

// Generar campo de estrellas aleatorias
pub fn generate_stars(count: usize) -> Vec<Star> {
    let mut stars = Vec::new();
    for i in 0..count {
        let seed = i as f32 * 12.9898;
        let x = ((seed.sin() * 43758.5453).fract() * WIDTH as f32) as f32;
        let y = (((seed + 1.0).sin() * 43758.5453).fract() * HEIGHT as f32) as f32;
        let brightness = ((seed * 2.0).sin() * 0.5 + 0.5) * 0.8 + 0.2;
        stars.push(Star { x, y, brightness });
    }
    stars
}

// Renderizar estrellas de fondo
pub fn render_stars(fb: &mut Framebuffer, stars: &[Star]) {
    for star in stars {
        let x = star.x as usize;
        let y = star.y as usize;
        if x < fb.width && y < fb.height {
            let intensity = (star.brightness * 255.0) as u32;
            let color = (intensity << 16) | (intensity << 8) | intensity;
            fb.color_buffer[y * fb.width + x] = color;
        }
    }
}

// Renderizar un planeta completo
pub fn render_planet(
    fb: &mut Framebuffer,
    mesh: &tobj::Mesh,
    mvp: &Matrix4<f32>,
    shader: &dyn PlanetShader,
    uniforms: &ShaderUniforms,
) {
    for tri_idx in (0..mesh.indices.len()).step_by(3) {
        let i0 = mesh.indices[tri_idx] as usize;
        let i1 = mesh.indices[tri_idx + 1] as usize;
        let i2 = mesh.indices[tri_idx + 2] as usize;

        // Extraer posiciones originales
        let p0_local = Vector3::new(
            mesh.positions[i0 * 3],
            mesh.positions[i0 * 3 + 1],
            mesh.positions[i0 * 3 + 2],
        );
        let p1_local = Vector3::new(
            mesh.positions[i1 * 3],
            mesh.positions[i1 * 3 + 1],
            mesh.positions[i1 * 3 + 2],
        );
        let p2_local = Vector3::new(
            mesh.positions[i2 * 3],
            mesh.positions[i2 * 3 + 1],
            mesh.positions[i2 * 3 + 2],
        );

        // Calcular normal del triángulo
        let edge1 = p1_local - p0_local;
        let edge2 = p2_local - p0_local;
        let tri_normal = edge1.cross(&edge2).normalize();

        // UVs esféricos
        let uv0 = calculate_spherical_uv(&p0_local);
        let uv1 = calculate_spherical_uv(&p1_local);
        let uv2 = calculate_spherical_uv(&p2_local);

        // Aplicar vertex shader
        let (v0_deformed, v0_norm) = shader.vertex_shader(p0_local, tri_normal, uv0, uniforms);
        let (v1_deformed, v1_norm) = shader.vertex_shader(p1_local, tri_normal, uv1, uniforms);
        let (v2_deformed, v2_norm) = shader.vertex_shader(p2_local, tri_normal, uv2, uniforms);

        // Transformar a clip space
        let clip0 = transform_vertex(&v0_deformed, mvp);
        let clip1 = transform_vertex(&v1_deformed, mvp);
        let clip2 = transform_vertex(&v2_deformed, mvp);

        // Frustum culling
        if clip0.3 <= 0.0 || clip1.3 <= 0.0 || clip2.3 <= 0.0 {
            continue;
        }

        // Backface culling
        let screen0 = to_screen_coords(&clip0, WIDTH as f32, HEIGHT as f32);
        let screen1 = to_screen_coords(&clip1, WIDTH as f32, HEIGHT as f32);
        let screen2 = to_screen_coords(&clip2, WIDTH as f32, HEIGHT as f32);
        
        let edge_a = (screen1.0 - screen0.0, screen1.1 - screen0.1);
        let edge_b = (screen2.0 - screen0.0, screen2.1 - screen0.1);
        let cross = edge_a.0 * edge_b.1 - edge_a.1 * edge_b.0;
        
        if cross <= 0.0 {
            continue;
        }

        // Rasterizar
        draw_triangle_with_shader(
            fb,
            &screen0,
            &screen1,
            &screen2,
            &v0_deformed,
            &v1_deformed,
            &v2_deformed,
            &v0_norm,
            &v1_norm,
            &v2_norm,
            &uv0,
            &uv1,
            &uv2,
            shader,
            uniforms,
        );
    }
}

// Renderizar anillos (simplificado como disco plano) - Legacy, no usado
#[allow(dead_code)]
pub fn render_rings(
    fb: &mut Framebuffer,
    projection: &Matrix4<f32>,
    view: &Matrix4<f32>,
    model: &Matrix4<f32>,
    _time: f32,
) {
    let mvp = projection * view * model;
    let ring_color = 0xCCAA88; // Color beige/dorado
    
    // Crear 3 anillos concéntricos
    for ring_radius_step in 14..17 {
        let ring_radius = ring_radius_step as f32 * 0.1;
        let segments = 64;
        
        for i in 0..segments {
            let angle1 = (i as f32 / segments as f32) * std::f32::consts::PI * 2.0;
            let angle2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::PI * 2.0;
            
            let x1 = angle1.cos() * ring_radius;
            let z1 = angle1.sin() * ring_radius;
            let x2 = angle2.cos() * ring_radius;
            let z2 = angle2.sin() * ring_radius;
            
            let p1 = Vector3::new(x1, 0.0, z1);
            let p2 = Vector3::new(x2, 0.0, z2);
            
            let clip1 = transform_vertex(&p1, &mvp);
            let clip2 = transform_vertex(&p2, &mvp);
            
            if clip1.3 > 0.0 && clip2.3 > 0.0 {
                let screen1 = to_screen_coords(&clip1, WIDTH as f32, HEIGHT as f32);
                let screen2 = to_screen_coords(&clip2, WIDTH as f32, HEIGHT as f32);
                
                draw_line(fb, screen1, screen2, ring_color);
            }
        }
    }
}

// Dibujar línea simple (Bresenham) - Legacy, no usado
#[allow(dead_code)]
fn draw_line(fb: &mut Framebuffer, p1: (f32, f32, f32), p2: (f32, f32, f32), color: u32) {
    let (mut x0, mut y0, z0) = (p1.0 as i32, p1.1 as i32, p1.2);
    let (x1, y1, z1) = (p2.0 as i32, p2.1 as i32, p2.2);
    
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    
    loop {
        if x0 >= 0 && x0 < fb.width as i32 && y0 >= 0 && y0 < fb.height as i32 {
            let depth = (z0 + z1) * 0.5;
            fb.set_pixel(x0 as usize, y0 as usize, depth, color);
        }
        
        if x0 == x1 && y0 == y1 {
            break;
        }
        
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

// Calcular UVs esféricos
pub fn calculate_spherical_uv(pos: &Vector3) -> (f32, f32) {
    let normalized = pos.normalize();
    let u = 0.5 + (normalized.z.atan2(normalized.x) / (2.0 * std::f32::consts::PI));
    let v = 0.5 - (normalized.y.asin() / std::f32::consts::PI);
    (u, v)
}

// Transformar vértice con matriz MVP
pub fn transform_vertex(pos: &Vector3, mvp: &Matrix4<f32>) -> (f32, f32, f32, f32) {
    let v = Vector4::new(pos.x, pos.y, pos.z, 1.0);
    let transformed = mvp * v;
    (transformed.x, transformed.y, transformed.z, transformed.w)
}

// Convertir de clip space a screen space
pub fn to_screen_coords(clip: &(f32, f32, f32, f32), width: f32, height: f32) -> (f32, f32, f32) {
    let ndc_x = clip.0 / clip.3;
    let ndc_y = clip.1 / clip.3;
    let ndc_z = clip.2 / clip.3;
    
    let screen_x = (ndc_x + 1.0) * 0.5 * width;
    let screen_y = (1.0 - ndc_y) * 0.5 * height;
    
    (screen_x, screen_y, ndc_z)
}

// Rasterizar triángulo
fn draw_triangle_with_shader(
    fb: &mut Framebuffer,
    screen0: &(f32, f32, f32),
    screen1: &(f32, f32, f32),
    screen2: &(f32, f32, f32),
    pos0: &Vector3,
    pos1: &Vector3,
    pos2: &Vector3,
    norm0: &Vector3,
    norm1: &Vector3,
    norm2: &Vector3,
    uv0: &(f32, f32),
    uv1: &(f32, f32),
    uv2: &(f32, f32),
    shader: &dyn PlanetShader,
    uniforms: &ShaderUniforms,
) {
    let (x0, y0, z0) = *screen0;
    let (x1, y1, z1) = *screen1;
    let (x2, y2, z2) = *screen2;

    // Bounding box
    let min_x = x0.min(x1).min(x2).floor().max(0.0) as usize;
    let max_x = x0.max(x1).max(x2).ceil().min(fb.width as f32 - 1.0) as usize;
    let min_y = y0.min(y1).min(y2).floor().max(0.0) as usize;
    let max_y = y0.max(y1).max(y2).ceil().min(fb.height as f32 - 1.0) as usize;

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let p_x = px as f32 + 0.5;
            let p_y = py as f32 + 0.5;

            // Coordenadas baricéntricas
            let (w0, w1, w2) = barycentric(x0, y0, x1, y1, x2, y2, p_x, p_y);

            // Usar un pequeño bias negativo para cubrir gaps entre triángulos
            let edge_bias = -0.001;
            if w0 >= edge_bias && w1 >= edge_bias && w2 >= edge_bias {
                // Interpolar depth
                let depth = w0 * z0 + w1 * z1 + w2 * z2;

                if depth >= -1.0 && depth <= 1.0 && depth < fb.depth_buffer[py * fb.width + px] {
                    // Interpolar atributos
                    let interp_pos = Vector3::new(
                        w0 * pos0.x + w1 * pos1.x + w2 * pos2.x,
                        w0 * pos0.y + w1 * pos1.y + w2 * pos2.y,
                        w0 * pos0.z + w1 * pos1.z + w2 * pos2.z,
                    );

                    let interp_norm = Vector3::new(
                        w0 * norm0.x + w1 * norm1.x + w2 * norm2.x,
                        w0 * norm0.y + w1 * norm1.y + w2 * norm2.y,
                        w0 * norm0.z + w1 * norm1.z + w2 * norm2.z,
                    ).normalize();

                    let interp_uv = (
                        w0 * uv0.0 + w1 * uv1.0 + w2 * uv2.0,
                        w0 * uv0.1 + w1 * uv1.1 + w2 * uv2.1,
                    );

                    // Fragment shader
                    let color = shader.fragment_shader(interp_pos, interp_norm, interp_uv, uniforms);

                    // Convertir a u32
                    let r = (color.r.clamp(0.0, 1.0) * 255.0) as u32;
                    let g = (color.g.clamp(0.0, 1.0) * 255.0) as u32;
                    let b = (color.b.clamp(0.0, 1.0) * 255.0) as u32;
                    let pixel_color = (r << 16) | (g << 8) | b;

                    fb.set_pixel(px, py, depth, pixel_color);
                }
            }
        }
    }
}

fn barycentric(
    x0: f32, y0: f32,
    x1: f32, y1: f32,
    x2: f32, y2: f32,
    px: f32, py: f32,
) -> (f32, f32, f32) {
    let denom = (y1 - y2) * (x0 - x2) + (x2 - x1) * (y0 - y2);
    if denom.abs() < 1e-10 {
        return (-1.0, -1.0, -1.0);
    }

    let w0 = ((y1 - y2) * (px - x2) + (x2 - x1) * (py - y2)) / denom;
    let w1 = ((y2 - y0) * (px - x2) + (x0 - x2) * (py - y2)) / denom;
    let w2 = 1.0 - w0 - w1;

    (w0, w1, w2)
}
