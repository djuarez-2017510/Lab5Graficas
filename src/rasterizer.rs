// Legacy code no usado en pipeline actual (renderer.rs hace rasterización directa)
#[allow(unused_imports)]
use nalgebra::{Vector3, Vector4};

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub color_buffer: Vec<u32>,
    pub depth_buffer: Vec<f32>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            color_buffer: vec![0; width * height],
            depth_buffer: vec![f32::INFINITY; width * height],
        }
    }

    pub fn clear(&mut self, color: u32) {
        self.color_buffer.fill(color);
        self.depth_buffer.fill(f32::INFINITY);
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, depth: f32, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        
        let index = y * self.width + x;
        if depth < self.depth_buffer[index] {
            self.depth_buffer[index] = depth;
            self.color_buffer[index] = color;
        }
    }
}

// Legacy structures/functions no usados, mantenidos por compatibilidad
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct VertexShaderOutput {
    pub position: Vector4<f32>,
    pub world_pos: Vector3<f32>,
    pub v_pos: Vector3<f32>,
}

#[allow(dead_code)]
pub fn ndc_to_screen(ndc: &Vector4<f32>, width: f32, height: f32) -> (f32, f32, f32) {
    let x = (ndc.x / ndc.w + 1.0) * 0.5 * width;
    let y = (1.0 - (ndc.y / ndc.w + 1.0) * 0.5) * height;
    let z = ndc.z / ndc.w;
    (x, y, z)
}

#[allow(dead_code)]
pub fn draw_triangle<F>(
    fb: &mut Framebuffer,
    v0: &VertexShaderOutput,
    v1: &VertexShaderOutput,
    v2: &VertexShaderOutput,
    fragment_shader: F,
)
where
    F: Fn(&Vector3<f32>, &Vector3<f32>) -> u32,
{
    let width = fb.width as f32;
    let height = fb.height as f32;

    let (x0, y0, z0) = ndc_to_screen(&v0.position, width, height);
    let (x1, y1, z1) = ndc_to_screen(&v1.position, width, height);
    let (x2, y2, z2) = ndc_to_screen(&v2.position, width, height);

    let min_x = x0.min(x1).min(x2).floor().max(0.0) as usize;
    let max_x = x0.max(x1).max(x2).ceil().min(width - 1.0) as usize;
    let min_y = y0.min(y1).min(y2).floor().max(0.0) as usize;
    let max_y = y0.max(y1).max(y2).ceil().min(height - 1.0) as usize;

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let p_x = px as f32 + 0.5;
            let p_y = py as f32 + 0.5;

            let (w0, w1, w2) = barycentric(x0, y0, x1, y1, x2, y2, p_x, p_y);

            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                let depth = w0 * z0 + w1 * z1 + w2 * z2;

                let world_pos = Vector3::new(
                    w0 * v0.world_pos.x + w1 * v1.world_pos.x + w2 * v2.world_pos.x,
                    w0 * v0.world_pos.y + w1 * v1.world_pos.y + w2 * v2.world_pos.y,
                    w0 * v0.world_pos.z + w1 * v1.world_pos.z + w2 * v2.world_pos.z,
                );

                let v_pos = Vector3::new(
                    w0 * v0.v_pos.x + w1 * v1.v_pos.x + w2 * v2.v_pos.x,
                    w0 * v0.v_pos.y + w1 * v1.v_pos.y + w2 * v2.v_pos.y,
                    w0 * v0.v_pos.z + w1 * v1.v_pos.z + w2 * v2.v_pos.z,
                );

                let color = fragment_shader(&world_pos, &v_pos);
                fb.set_pixel(px, py, depth, color);
            }
        }
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn rgb_to_u32(r: f32, g: f32, b: f32) -> u32 {
    let r = (r.clamp(0.0, 1.0) * 255.0) as u32;
    let g = (g.clamp(0.0, 1.0) * 255.0) as u32;
    let b = (b.clamp(0.0, 1.0) * 255.0) as u32;
    (r << 16) | (g << 8) | b
}
